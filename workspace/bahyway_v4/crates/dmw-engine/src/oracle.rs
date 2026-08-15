//! The Oracle — generates ranked, actionable recommendations for a query plan.
//!
//! Each recommendation has a RotationType (the action to apply) and an
//! `alignment_gain_pct` — how many alignment percentage points the fix is
//! expected to recover.  "APPLY ROTATION" in the UI maps to these types.

use crate::bottleneck::{Bottleneck, BottleneckKind, Severity};
use crate::journey::SevenLevelJourney;
use crate::plan::QueryPlan;

/// The class of fix to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotationType {
    /// Convert NestedLoop join to HashMatch via query hint or join rewrite.
    LoopToHashConversion,
    /// Rewrite predicate to allow index seek (remove function wrap).
    SargableTunnel,
    /// REBUILD fragmented index (> 30 %).
    RebuildIndex,
    /// REORGANIZE mildly fragmented index (10–30 %).
    ReorganizeIndex,
    /// UPDATE STATISTICS on the affected table.
    UpdateStatistics,
    /// Force hash join via OPTION(HASH JOIN) query hint.
    ForceHashJoin,
    /// Add INCLUDE columns to an existing non-clustered index.
    AddCoveringColumns,
    /// Create a new composite index that eliminates the key lookup + sort.
    CreateCoveringIndex,
    /// Rewrite query to avoid EagerSpool (e.g. materialise CTE or use OPTION(RECOMPILE)).
    EliminateSpool,
    /// Break chained outer joins into staged CTEs to reduce deadlock surface.
    StageOuterJoins,
}

impl RotationType {
    pub fn label(&self) -> &'static str {
        match self {
            RotationType::LoopToHashConversion  => "Loop-to-Hash Conversion",
            RotationType::SargableTunnel        => "Sargable Tunneling",
            RotationType::RebuildIndex          => "Rebuild Index",
            RotationType::ReorganizeIndex       => "Reorganize Index",
            RotationType::UpdateStatistics      => "Update Statistics",
            RotationType::ForceHashJoin         => "Force Hash Join",
            RotationType::AddCoveringColumns    => "Add Covering Columns",
            RotationType::CreateCoveringIndex   => "Create Covering Index",
            RotationType::EliminateSpool        => "Eliminate Spool",
            RotationType::StageOuterJoins       => "Stage Outer Joins",
        }
    }
}

/// One actionable recommendation from The Oracle.
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub rotation:           RotationType,
    pub title:              String,
    pub detail:             String,
    /// Estimated alignment gain in percentage points if this fix is applied.
    pub alignment_gain_pct: f32,
    /// Priority: 1 = apply first (highest gain / lowest risk).
    pub priority:           u8,
}

impl Recommendation {
    fn new(rotation: RotationType, detail: impl Into<String>, gain: f32, priority: u8) -> Self {
        let title = rotation.label().to_string();
        Recommendation { rotation, title, detail: detail.into(), alignment_gain_pct: gain, priority }
    }
}

/// The Oracle — stateless advisor that converts bottlenecks into recommendations.
pub struct TheOracle;

impl TheOracle {
    pub fn advise(
        _plan:       &QueryPlan,
        bottlenecks: &[Bottleneck],
        journey:     &SevenLevelJourney,
    ) -> Vec<Recommendation> {
        let _ = journey; // used implicitly via bottleneck kinds
        let mut out: Vec<Recommendation> = Vec::new();
        let mut priority: u8 = 1;

        for bn in bottlenecks {
            match &bn.kind {
                BottleneckKind::NestedLoopOnLargeSet { inner_rows } => {
                    out.push(Recommendation::new(
                        RotationType::LoopToHashConversion,
                        format!(
                            "Nested loop join with inner set of {} rows causes O(n²) execution. \
                             Force hash join via OPTION(HASH JOIN) or rewrite the join order. \
                             Alignment gain: +{:.0}%", inner_rows,
                            Self::gain_for_severity(bn.severity, 25.0)
                        ),
                        Self::gain_for_severity(bn.severity, 25.0),
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::UnsargablePredicate { table } => {
                    out.push(Recommendation::new(
                        RotationType::SargableTunnel,
                        format!(
                            "Function wrapping a column on '{table}' prevents index use. \
                             Rewrite predicate to compare directly on the column. \
                             Alignment gain: +{:.0}%",
                            Self::gain_for_severity(bn.severity, 20.0)
                        ),
                        Self::gain_for_severity(bn.severity, 20.0),
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::KeyLookupBottleneck { lookup_count } => {
                    let gain = ((*lookup_count as f32) * 5.0).min(25.0);
                    out.push(Recommendation::new(
                        RotationType::AddCoveringColumns,
                        format!(
                            "{lookup_count} KeyLookup(s): include projected columns in the non-clustered index. \
                             Example: CREATE INDEX IX_T_Col ON T (key_col) INCLUDE (col1, col2). \
                             Alignment gain: +{gain:.0}%"
                        ),
                        gain,
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::StaleStatistics { estimated, actual, ratio } => {
                    out.push(Recommendation::new(
                        RotationType::UpdateStatistics,
                        format!(
                            "Estimated {estimated} rows vs actual {actual} ({ratio:.0}× off). \
                             Run UPDATE STATISTICS WITH FULLSCAN on affected tables. \
                             Alignment gain: +{:.0}%",
                            Self::gain_for_severity(bn.severity, 15.0)
                        ),
                        Self::gain_for_severity(bn.severity, 15.0),
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::ExcessiveSort => {
                    out.push(Recommendation::new(
                        RotationType::CreateCoveringIndex,
                        format!(
                            "Sort operator without supporting index adds O(n log n) overhead. \
                             Create a composite index matching the ORDER BY columns. \
                             Alignment gain: +{:.0}%",
                            Self::gain_for_severity(bn.severity, 10.0)
                        ),
                        Self::gain_for_severity(bn.severity, 10.0),
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::SpoolRecompute => {
                    out.push(Recommendation::new(
                        RotationType::EliminateSpool,
                        "EagerSpool materialises the inner set on every outer row. \
                         Materialise the inner query as a CTE or add OPTION(RECOMPILE) \
                         to force parameter-specific plan. Alignment gain: +8%".to_string(),
                        8.0,
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::DeadlockRisk { join_count } => {
                    out.push(Recommendation::new(
                        RotationType::StageOuterJoins,
                        format!(
                            "{join_count} chained LeftOuter joins under concurrent write load risk deadlock. \
                             Break into staged CTEs with row-level lock hints or use SET TRANSACTION ISOLATION LEVEL \
                             READ COMMITTED SNAPSHOT. Alignment gain: +{:.0}%",
                            Self::gain_for_severity(bn.severity, 15.0)
                        ),
                        Self::gain_for_severity(bn.severity, 15.0),
                        priority,
                    ));
                    priority += 1;
                }
                BottleneckKind::CartesianProduct => {
                    out.push(Recommendation::new(
                        RotationType::ForceHashJoin,
                        "Missing join condition creates row explosion. Add explicit ON predicate \
                         and verify the join intent. Alignment gain: +35%".to_string(),
                        35.0,
                        priority,
                    ));
                    priority += 1;
                }
            }
        }

        // Sort: highest gain first, then lowest priority number
        out.sort_by(|a, b| b.alignment_gain_pct
            .partial_cmp(&a.alignment_gain_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.priority.cmp(&b.priority)));

        // Re-number priority after sort
        for (i, rec) in out.iter_mut().enumerate() { rec.priority = (i + 1) as u8; }

        out
    }

    /// Projected alignment after applying all recommendations.
    pub fn projected_alignment(current: f32, recommendations: &[Recommendation]) -> f32 {
        let total_gain: f32 = recommendations.iter().map(|r| r.alignment_gain_pct).sum();
        (current + total_gain).min(100.0)
    }

    pub fn top_recommendation(recommendations: &[Recommendation]) -> Option<&Recommendation> {
        recommendations.first()
    }

    fn gain_for_severity(severity: Severity, base: f32) -> f32 {
        match severity {
            Severity::Critical => base * 1.4,
            Severity::High     => base * 1.2,
            Severity::Medium   => base,
            Severity::Low      => base * 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bottleneck::BottleneckDetector;
    use crate::journey::SevenLevelJourney;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan};

    fn full_analysis(plan: &QueryPlan) -> (Vec<Bottleneck>, SevenLevelJourney, Vec<Recommendation>) {
        let bns  = BottleneckDetector::detect(plan);
        let j    = SevenLevelJourney::analyze(plan, &bns);
        let recs = TheOracle::advise(plan, &bns, &j);
        (bns, j, recs)
    }

    #[test] fn sales_order_has_recommendations() {
        let plan = sales_order_plan();
        let (_, _, recs) = full_analysis(&plan);
        assert!(!recs.is_empty(), "Oracle should produce at least one recommendation");
    }
    #[test] fn recommendations_sorted_by_gain_desc() {
        let plan = simple_nested_loop_plan();
        let (_, _, recs) = full_analysis(&plan);
        for w in recs.windows(2) {
            assert!(w[0].alignment_gain_pct >= w[1].alignment_gain_pct,
                "Recs must be sorted by gain desc");
        }
    }
    #[test] fn priority_renumbered_from_one() {
        let plan = sales_order_plan();
        let (_, _, recs) = full_analysis(&plan);
        if !recs.is_empty() { assert_eq!(recs[0].priority, 1); }
        for (i, r) in recs.iter().enumerate() { assert_eq!(r.priority as usize, i + 1); }
    }
    #[test] fn projected_alignment_gte_current() {
        let plan = sales_order_plan();
        let (bns, j, recs) = full_analysis(&plan);
        let _ = &bns;
        let current = j.alignment_score();
        let proj    = TheOracle::projected_alignment(current, &recs);
        assert!(proj >= current);
        assert!(proj <= 100.0);
    }
    #[test] fn top_recommendation_matches_first() {
        let plan = simple_nested_loop_plan();
        let (_, _, recs) = full_analysis(&plan);
        if let Some(top) = TheOracle::top_recommendation(&recs) {
            assert_eq!(top.priority, 1);
        }
    }
    #[test] fn rotation_labels_non_empty() {
        let rots = [
            RotationType::LoopToHashConversion,
            RotationType::SargableTunnel,
            RotationType::RebuildIndex,
            RotationType::ReorganizeIndex,
            RotationType::UpdateStatistics,
            RotationType::ForceHashJoin,
            RotationType::AddCoveringColumns,
            RotationType::CreateCoveringIndex,
            RotationType::EliminateSpool,
            RotationType::StageOuterJoins,
        ];
        for r in &rots { assert!(!r.label().is_empty()); }
    }
    #[test] fn detail_non_empty_for_each_rec() {
        let plan = simple_nested_loop_plan();
        let (_, _, recs) = full_analysis(&plan);
        for r in &recs { assert!(!r.detail.is_empty(), "empty detail for {:?}", r.rotation); }
    }
    #[test] fn stage_outer_joins_recommended_for_deadlock() {
        let plan = sales_order_plan();
        let (_, _, recs) = full_analysis(&plan);
        assert!(recs.iter().any(|r| r.rotation == RotationType::StageOuterJoins),
            "Two chained LeftOuter joins should trigger StageOuterJoins");
    }
    #[test] fn loop_to_hash_recommended_for_nested_loop() {
        let plan = simple_nested_loop_plan();
        let (_, _, recs) = full_analysis(&plan);
        assert!(recs.iter().any(|r| r.rotation == RotationType::LoopToHashConversion));
    }
}
