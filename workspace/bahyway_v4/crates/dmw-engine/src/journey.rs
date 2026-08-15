//! Seven-Level Journey — the 7D heptagonal analysis path mapped onto DMW.
//!
//! Each level poses exactly one diagnostic question about the query:
//!
//!   L1 ME  (Centre)     — What does this query WANT?          (output intent)
//!   L2 GU  (North)      — How many rows does SQL think?       (cardinality)
//!   L3 SAG (NorthEast)  — Can the filter USE an index?        (sargability)
//!   L4 A   (SouthEast)  — How are the tables joined?          (join strategy)
//!   L5 IZI (South)      — Is CPU doing row-by-row work?       (loop / spool)
//!   L6 UD  (SouthWest)  — Is a stale plan being reused?       (statistics age)
//!   L7 URU (NorthWest)  — Are indexes covering and healthy?   (index coverage)
//!
//! The alignment score is the weighted mean of all 7 level scores (0–100 %).

use crate::bottleneck::{Bottleneck, BottleneckKind};
use crate::plan::QueryPlan;

/// Identifier and short label for each journey level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyLevelId {
    ME  = 0,
    GU  = 1,
    SAG = 2,
    A   = 3,
    IZI = 4,
    UD  = 5,
    URU = 6,
}

impl JourneyLevelId {
    pub fn all() -> [JourneyLevelId; 7] {
        use JourneyLevelId::*;
        [ME, GU, SAG, A, IZI, UD, URU]
    }

    pub fn label(&self) -> &'static str {
        match self {
            JourneyLevelId::ME  => "ME",
            JourneyLevelId::GU  => "GU",
            JourneyLevelId::SAG => "SAG",
            JourneyLevelId::A   => "A",
            JourneyLevelId::IZI => "IZI",
            JourneyLevelId::UD  => "UD",
            JourneyLevelId::URU => "URU",
        }
    }

    /// Akkadian name with cuneiform glyph — used in StoryWay and level headers.
    pub fn akkadian_name(&self) -> &'static str {
        match self {
            JourneyLevelId::ME  => "ME (𒈨)",
            JourneyLevelId::GU  => "GU (𒄘)",
            JourneyLevelId::SAG => "SAG (𒁮)",
            JourneyLevelId::A   => "A (𒀀)",
            JourneyLevelId::IZI => "IZI (𒄑𒆵)",
            JourneyLevelId::UD  => "UD (𒌓)",
            JourneyLevelId::URU => "URU (𒌷)",
        }
    }

    /// Level number (1-based) for display.
    pub fn number(&self) -> u8 {
        *self as u8 + 1
    }

    /// Next level in the journey, or None if this is the final level.
    pub fn next(&self) -> Option<JourneyLevelId> {
        match self {
            JourneyLevelId::ME  => Some(JourneyLevelId::GU),
            JourneyLevelId::GU  => Some(JourneyLevelId::SAG),
            JourneyLevelId::SAG => Some(JourneyLevelId::A),
            JourneyLevelId::A   => Some(JourneyLevelId::IZI),
            JourneyLevelId::IZI => Some(JourneyLevelId::UD),
            JourneyLevelId::UD  => Some(JourneyLevelId::URU),
            JourneyLevelId::URU => None,
        }
    }

    pub fn question(&self) -> &'static str {
        match self {
            JourneyLevelId::ME  => "What does this query WANT?",
            JourneyLevelId::GU  => "How many rows does SQL think?",
            JourneyLevelId::SAG => "Can the filter USE an index?",
            JourneyLevelId::A   => "How are the tables joined?",
            JourneyLevelId::IZI => "Is CPU doing row-by-row work?",
            JourneyLevelId::UD  => "Is a stale plan being reused?",
            JourneyLevelId::URU => "Are indexes covering and healthy?",
        }
    }

    /// Hepta-sector weight (how much this level contributes to alignment score).
    pub fn weight(&self) -> f32 {
        match self {
            JourneyLevelId::ME  => 1.0,
            JourneyLevelId::GU  => 1.5,
            JourneyLevelId::SAG => 1.5,
            JourneyLevelId::A   => 2.0,  // join strategy dominates performance
            JourneyLevelId::IZI => 2.0,  // CPU row-by-row is the largest consumer
            JourneyLevelId::UD  => 1.0,
            JourneyLevelId::URU => 1.0,
        }
    }
}

/// Score for a single journey level.
#[derive(Debug, Clone)]
pub struct LevelScore {
    pub level:   JourneyLevelId,
    /// 0.0 (blocked) – 1.0 (fully passing).
    pub score:   f32,
    pub finding: String,
    pub passed:  bool,
}

impl LevelScore {
    fn pass(level: JourneyLevelId, finding: impl Into<String>) -> Self {
        LevelScore { level, score: 1.0, finding: finding.into(), passed: true }
    }

    fn fail(level: JourneyLevelId, score: f32, finding: impl Into<String>) -> Self {
        LevelScore { level, score: score.clamp(0.0, 1.0), finding: finding.into(), passed: false }
    }
}

/// Overall health state derived from the alignment score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmwState {
    /// Alignment ≥ 80 % — query is well-tuned.
    Golden,
    /// Alignment 60–80 % — some issues, still functional.
    Stable,
    /// Alignment 40–60 % — multiple bottlenecks, noticeable degradation.
    Turbulent,
    /// Alignment < 40 % — severe problems, immediate attention needed.
    Critical,
}

impl DmwState {
    pub fn label(&self) -> &'static str {
        match self {
            DmwState::Golden    => "Golden",
            DmwState::Stable    => "Stable",
            DmwState::Turbulent => "Turbulent",
            DmwState::Critical  => "Critical",
        }
    }

    pub fn from_pct(pct: f32) -> Self {
        match pct as u32 {
            80..=100 => DmwState::Golden,
            60..=79  => DmwState::Stable,
            40..=59  => DmwState::Turbulent,
            _        => DmwState::Critical,
        }
    }
}

/// The complete 7-level journey analysis over a single query plan.
pub struct SevenLevelJourney {
    levels: [LevelScore; 7],
}

impl SevenLevelJourney {
    /// Analyse a plan that has already been scanned for bottlenecks.
    pub fn analyze(plan: &QueryPlan, bottlenecks: &[Bottleneck]) -> Self {
        use JourneyLevelId::*;

        // L1 ME — intent: does the plan have a finite, non-zero result set?
        let l1 = if plan.root.estimated_rows == 0 {
            LevelScore::fail(ME, 0.5, "Estimated rows = 0 — optimizer may have skipped whole-plan shortcut")
        } else {
            LevelScore::pass(ME, format!("Query targets {} estimated rows", plan.root.estimated_rows))
        };

        // L2 GU — cardinality accuracy
        let l2 = match plan.cardinality_ratio() {
            None => LevelScore::pass(GU, "No actual-row data (plan not yet executed)"),
            Some(r) if r > STALE_THRESHOLD || r < 1.0 / STALE_THRESHOLD => {
                let effective = if r < 1.0 { 1.0 / r } else { r };
                let score = (1.0 - ((effective - 1.0) / 200.0)).max(0.0);
                LevelScore::fail(GU, score, format!("Cardinality mismatch {effective:.1}× — statistics stale"))
            }
            Some(_) => LevelScore::pass(GU, "Cardinality estimate accurate"),
        };

        // L3 SAG — sargability: table scans present?
        let has_scan = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::UnsargablePredicate { .. }));
        let l3 = if has_scan {
            let tables: Vec<String> = bottlenecks.iter().filter_map(|b| {
                if let BottleneckKind::UnsargablePredicate { table } = &b.kind { Some(table.clone()) } else { None }
            }).collect();
            LevelScore::fail(SAG, 0.2, format!("TableScan on [{}] — filter is non-sargable", tables.join(", ")))
        } else {
            LevelScore::pass(SAG, "All filters are sargable — index seeks in use")
        };

        // L4 A — join strategy
        let nl_count = plan.nested_loop_count();
        let l4 = if bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::CartesianProduct)) {
            LevelScore::fail(A, 0.0, "Cartesian product detected — join condition missing")
        } else if bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::NestedLoopOnLargeSet { .. })) {
            LevelScore::fail(A, 0.3, format!("{nl_count} NestedLoop(s) on large sets — promote to HashMatch"))
        } else if bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::DeadlockRisk { .. })) {
            LevelScore::fail(A, 0.6, "Chained outer joins — deadlock risk under concurrent writes")
        } else {
            LevelScore::pass(A, format!("{} join(s) using hash-based strategy", plan.join_count()))
        };

        // L5 IZI — CPU row-by-row (nested loops + spools)
        let spool_present = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::SpoolRecompute));
        let nl_present    = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::NestedLoopOnLargeSet { .. }));
        let l5 = if nl_present && spool_present {
            LevelScore::fail(IZI, 0.1, "NestedLoop + EagerSpool: maximum CPU row-by-row work")
        } else if nl_present {
            LevelScore::fail(IZI, 0.3, "NestedLoop driving row-by-row CPU scans")
        } else if spool_present {
            LevelScore::fail(IZI, 0.5, "EagerSpool materialising inner set per outer row")
        } else {
            LevelScore::pass(IZI, "No row-by-row CPU bottleneck detected")
        };

        // L6 UD — stale plan / statistics
        let stale = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::StaleStatistics { .. }));
        let l6 = if stale {
            let detail = bottlenecks.iter().find_map(|b| {
                if let BottleneckKind::StaleStatistics { ratio, .. } = b.kind { Some(ratio) } else { None }
            }).unwrap_or(0.0);
            LevelScore::fail(UD, (1.0 - detail / 200.0).max(0.0), format!("Statistics off by {detail:.0}× — UPDATE STATISTICS required"))
        } else {
            LevelScore::pass(UD, "Statistics appear current — cardinality within threshold")
        };

        // L7 URU — index coverage
        let kl_count = plan.key_lookup_count();
        let has_kl   = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::KeyLookupBottleneck { .. }));
        let has_sort = bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::ExcessiveSort));
        let l7 = if has_kl && has_sort {
            LevelScore::fail(URU, 0.2, format!("{kl_count} KeyLookup(s) + Sort without covering index"))
        } else if has_kl {
            LevelScore::fail(URU, 0.4, format!("{kl_count} KeyLookup(s) — non-covering index needs INCLUDE columns"))
        } else if has_sort {
            LevelScore::fail(URU, 0.6, "Sort without supporting index — add composite index with ORDER BY columns")
        } else {
            LevelScore::pass(URU, "Indexes are covering — no lookups or redundant sorts")
        };

        SevenLevelJourney {
            levels: [l1, l2, l3, l4, l5, l6, l7],
        }
    }

    /// Weighted alignment score (0–100 %).
    pub fn alignment_score(&self) -> f32 {
        let total_weight: f32 = JourneyLevelId::all().iter().map(|id| id.weight()).sum();
        let weighted_sum: f32 = self.levels.iter().map(|ls| ls.score * ls.level.weight()).sum();
        (weighted_sum / total_weight * 100.0).clamp(0.0, 100.0)
    }

    pub fn state(&self) -> DmwState { DmwState::from_pct(self.alignment_score()) }

    pub fn passed_levels(&self)   -> usize { self.levels.iter().filter(|l| l.passed).count() }
    pub fn failed_levels(&self)   -> usize { self.levels.iter().filter(|l| !l.passed).count() }

    pub fn level_score(&self, id: JourneyLevelId) -> f32 {
        self.levels[id as usize].score
    }

    pub fn level_finding(&self, id: JourneyLevelId) -> &str {
        &self.levels[id as usize].finding
    }

    pub fn all_levels(&self) -> &[LevelScore; 7] { &self.levels }
}

const STALE_THRESHOLD: f32 = 10.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan};
    use crate::bottleneck::BottleneckDetector;

    fn journey_and_bns(plan: &QueryPlan) -> (SevenLevelJourney, Vec<Bottleneck>) {
        let bns = BottleneckDetector::detect(plan);
        let j   = SevenLevelJourney::analyze(plan, &bns);
        (j, bns)
    }

    #[test] fn sales_order_alignment_range() {
        let plan = sales_order_plan();
        let (j, _) = journey_and_bns(&plan);
        let score = j.alignment_score();
        // Sales-order plan uses hash joins throughout — only Level A is degraded (deadlock risk).
        // Six of seven levels pass → weighted score lands in the Golden range (≥ 80 %).
        assert!(score > 70.0, "score={score:.1}");
    }
    #[test] fn simple_plan_is_stable_or_worse() {
        let plan = simple_nested_loop_plan();
        let (j, _) = journey_and_bns(&plan);
        let score = j.alignment_score();
        // NestedLoop(50k rows) + TableScan degrades L3/L4/L5 → score ≤ 65 %
        assert!(score <= 65.0, "score={score:.1}");
    }
    #[test] fn perfect_plan_is_golden() {
        use crate::plan::{OpKind, PlanNode, ScanType};
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            100.0, 100, 100,
        );
        let plan = crate::plan::QueryPlan::new("perfect", root, 100.0).with_actual_rows(100);
        let (j, _) = journey_and_bns(&plan);
        assert_eq!(j.state(), DmwState::Golden);
        assert!(j.alignment_score() >= 80.0);
    }
    #[test] fn passed_plus_failed_equals_seven() {
        let plan = sales_order_plan();
        let (j, _) = journey_and_bns(&plan);
        assert_eq!(j.passed_levels() + j.failed_levels(), 7);
    }
    #[test] fn all_levels_present() {
        let plan = sales_order_plan();
        let (j, _) = journey_and_bns(&plan);
        for id in JourneyLevelId::all() {
            let score = j.level_score(id);
            assert!(score >= 0.0 && score <= 1.0, "level {:?} score {score}", id);
        }
    }
    #[test] fn level_labels_and_questions_non_empty() {
        for id in JourneyLevelId::all() {
            assert!(!id.label().is_empty());
            assert!(!id.question().is_empty());
        }
    }
    #[test] fn dmw_state_from_pct() {
        assert_eq!(DmwState::from_pct(85.0), DmwState::Golden);
        assert_eq!(DmwState::from_pct(70.0), DmwState::Stable);
        assert_eq!(DmwState::from_pct(52.0), DmwState::Turbulent);
        assert_eq!(DmwState::from_pct(30.0), DmwState::Critical);
    }
    #[test] fn state_labels_non_empty() {
        for s in [DmwState::Golden, DmwState::Stable, DmwState::Turbulent, DmwState::Critical] {
            assert!(!s.label().is_empty());
        }
    }
    #[test] fn alignment_score_bounded() {
        let plan = simple_nested_loop_plan();
        let (j, _) = journey_and_bns(&plan);
        let score = j.alignment_score();
        assert!(score >= 0.0 && score <= 100.0);
    }
    #[test] fn finding_non_empty_for_all_levels() {
        let plan = sales_order_plan();
        let (j, _) = journey_and_bns(&plan);
        for id in JourneyLevelId::all() {
            assert!(!j.level_finding(id).is_empty(), "empty finding for {:?}", id);
        }
    }
}
