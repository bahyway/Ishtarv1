//! Bottleneck detection — identifies performance anti-patterns in a QueryPlan.
//!
//! Each bottleneck kind maps to a concrete observable problem:
//!   NestedLoopOnLargeSet → O(n²) row-by-row join when the inner set is large
//!   CartesianProduct     → missing join condition produces row explosion
//!   KeyLookupBottleneck  → non-covering index forces one row-lookup per seek
//!   UnsargablePredicate  → function-wrapped column blocks index seek
//!   StaleStatistics      → cardinality estimate diverges >10× from actual rows
//!   ExcessiveSort        → blocking Sort operator without a supporting index
//!   SpoolRecompute       → EagerSpool signals parameter-sniffing / plan instability
//!   DeadlockRisk         → multiple LeftOuter + nested exclusive-lock chain

use crate::plan::{OpKind, PlanNode, QueryPlan};

/// Row-count threshold above which a NestedLoop inner is considered "large".
pub const NESTED_LOOP_LARGE_ROWS: u32 = 1_000;

/// Cardinality ratio beyond which statistics are flagged as stale (10× off).
pub const STALE_STATS_RATIO: f32 = 10.0;

/// Severity of a detected bottleneck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Low      => "Low",
            Severity::Medium   => "Medium",
            Severity::High     => "High",
            Severity::Critical => "Critical",
        }
    }
}

/// The kind of performance problem found.
#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckKind {
    /// NestedLoop join with an inner set larger than NESTED_LOOP_LARGE_ROWS — O(n²).
    NestedLoopOnLargeSet { inner_rows: u32 },
    /// Cross-join detected (no join predicate, row count explodes).
    CartesianProduct,
    /// Non-covering index forces one KeyLookup per row returned.
    KeyLookupBottleneck { lookup_count: usize },
    /// Full table scan with no index path.
    UnsargablePredicate { table: String },
    /// Optimizer's row estimate diverges >10× from actual rows returned.
    StaleStatistics { estimated: u32, actual: u32, ratio: f32 },
    /// Blocking Sort operator without an ORDER BY–supporting index.
    ExcessiveSort,
    /// EagerSpool materialises the entire inner set — plan instability signal.
    SpoolRecompute,
    /// Two or more chained LeftOuter joins over large sets — deadlock risk under concurrent writes.
    DeadlockRisk { join_count: usize },
}

impl BottleneckKind {
    pub fn title(&self) -> String {
        match self {
            BottleneckKind::NestedLoopOnLargeSet { .. } => "Nested Loop on Large Set".into(),
            BottleneckKind::CartesianProduct             => "Cartesian Product".into(),
            BottleneckKind::KeyLookupBottleneck { .. }   => "Key Lookup Bottleneck".into(),
            BottleneckKind::UnsargablePredicate { .. }   => "Unsargable Predicate / Table Scan".into(),
            BottleneckKind::StaleStatistics { .. }       => "Stale Statistics".into(),
            BottleneckKind::ExcessiveSort                => "Excessive Sort".into(),
            BottleneckKind::SpoolRecompute               => "Spool Recompute".into(),
            BottleneckKind::DeadlockRisk { .. }          => "Deadlock Risk — Chained Outer Joins".into(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            BottleneckKind::NestedLoopOnLargeSet { inner_rows } =>
                format!("NestedLoop with inner set of {inner_rows} rows causes O(n²) execution. Promote to HashMatch."),
            BottleneckKind::CartesianProduct =>
                "Missing join condition creates a row explosion. Add an explicit ON predicate.".into(),
            BottleneckKind::KeyLookupBottleneck { lookup_count } =>
                format!("{lookup_count} KeyLookup(s) detected. Widen the non-clustered index to cover projected columns."),
            BottleneckKind::UnsargablePredicate { table } =>
                format!("Full TableScan on '{table}' — predicate is non-sargable. Rewrite filter to allow index seek."),
            BottleneckKind::StaleStatistics { estimated, actual, ratio } =>
                format!("Row estimate {estimated} vs actual {actual} (ratio {ratio:.1}×). Run UPDATE STATISTICS."),
            BottleneckKind::ExcessiveSort =>
                "Blocking Sort operator without a supporting index. Add an index covering ORDER BY columns.".into(),
            BottleneckKind::SpoolRecompute =>
                "EagerSpool materialises inner set on every outer row. Investigate parameter sniffing; consider OPTION(RECOMPILE).".into(),
            BottleneckKind::DeadlockRisk { join_count } =>
                format!("{join_count} chained LeftOuter joins under concurrent write load risk deadlock. Isolate or batch."),
        }
    }

    pub fn default_severity(&self) -> Severity {
        match self {
            BottleneckKind::CartesianProduct                     => Severity::Critical,
            BottleneckKind::DeadlockRisk { .. }                  => Severity::Critical,
            BottleneckKind::NestedLoopOnLargeSet { inner_rows }  =>
                if *inner_rows > 10_000 { Severity::Critical } else { Severity::High },
            BottleneckKind::StaleStatistics { ratio, .. }        =>
                if *ratio > 100.0 { Severity::Critical } else { Severity::High },
            BottleneckKind::KeyLookupBottleneck { .. }           => Severity::High,
            BottleneckKind::UnsargablePredicate { .. }           => Severity::High,
            BottleneckKind::ExcessiveSort                        => Severity::Medium,
            BottleneckKind::SpoolRecompute                       => Severity::Medium,
        }
    }
}

/// A single detected bottleneck with its location in the plan tree.
#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub kind:            BottleneckKind,
    pub severity:        Severity,
    /// Cost share attributed to this bottleneck (0–100 %).
    pub cost_impact_pct: f32,
    /// Breadcrumb path in the plan tree (e.g. "root/HashMatch/NestedLoop").
    pub operator_path:   String,
}

impl Bottleneck {
    pub fn new(kind: BottleneckKind, cost_impact_pct: f32, path: impl Into<String>) -> Self {
        let severity = kind.default_severity();
        Bottleneck { kind, severity, cost_impact_pct, operator_path: path.into() }
    }

    pub fn title(&self)       -> String { self.kind.title() }
    pub fn description(&self) -> String { self.kind.description() }
    pub fn is_critical(&self) -> bool   { self.severity == Severity::Critical }
}

/// Stateless detector — call `detect(plan)` to get all bottlenecks.
pub struct BottleneckDetector;

impl BottleneckDetector {
    pub fn detect(plan: &QueryPlan) -> Vec<Bottleneck> {
        let mut out = Vec::new();

        // 1. Nested Loop on large inner set
        let mut nl_nodes: Vec<&PlanNode> = Vec::new();
        plan.root.collect_where(&|n| n.op.is_nested_loop(), &mut nl_nodes);
        for node in &nl_nodes {
            // The inner (second) child is the repeated side
            let inner_rows = node.children.get(1).map(|c| c.estimated_rows).unwrap_or(node.estimated_rows);
            if inner_rows >= NESTED_LOOP_LARGE_ROWS {
                out.push(Bottleneck::new(
                    BottleneckKind::NestedLoopOnLargeSet { inner_rows },
                    node.cost_pct,
                    format!("root/{}", node.op.label()),
                ));
            }
        }

        // 2. Key Lookups
        let kl_count = plan.key_lookup_count();
        if kl_count > 0 {
            let cost: f32 = {
                let mut kls = Vec::new();
                plan.root.collect_where(&|n| n.op.is_key_lookup(), &mut kls);
                kls.iter().map(|n| n.cost_pct).sum()
            };
            out.push(Bottleneck::new(
                BottleneckKind::KeyLookupBottleneck { lookup_count: kl_count },
                cost,
                "root/KeyLookup",
            ));
        }

        // 3. Table scans (unsargable predicate proxy)
        let mut ts_nodes: Vec<&PlanNode> = Vec::new();
        plan.root.collect_where(&|n| n.op.is_table_scan(), &mut ts_nodes);
        for node in &ts_nodes {
            if let OpKind::TableScan { name } = &node.op {
                out.push(Bottleneck::new(
                    BottleneckKind::UnsargablePredicate { table: name.clone() },
                    node.cost_pct,
                    format!("root/TableScan[{name}]"),
                ));
            }
        }

        // 4. Stale statistics (cardinality mismatch)
        if let Some(ratio) = plan.cardinality_ratio() {
            if ratio > STALE_STATS_RATIO || ratio < 1.0 / STALE_STATS_RATIO {
                let eff_ratio = if ratio < 1.0 { 1.0 / ratio } else { ratio };
                out.push(Bottleneck::new(
                    BottleneckKind::StaleStatistics {
                        estimated: plan.root.estimated_rows,
                        actual:    plan.actual_rows,
                        ratio:     eff_ratio,
                    },
                    5.0,
                    "root",
                ));
            }
        }

        // 5. Excessive sort
        if plan.has_sorts() {
            let sort_cost: f32 = {
                let mut sorts = Vec::new();
                plan.root.collect_where(&|n| matches!(n.op, OpKind::Sort { .. }), &mut sorts);
                sorts.iter().map(|n| n.cost_pct).sum()
            };
            out.push(Bottleneck::new(BottleneckKind::ExcessiveSort, sort_cost, "root/Sort"));
        }

        // 6. Spool recompute
        if plan.has_spools() {
            let spool_cost: f32 = {
                let mut spools = Vec::new();
                plan.root.collect_where(&|n| matches!(n.op, OpKind::Spool { is_eager: true }), &mut spools);
                spools.iter().map(|n| n.cost_pct).sum()
            };
            out.push(Bottleneck::new(BottleneckKind::SpoolRecompute, spool_cost, "root/EagerSpool"));
        }

        // 7. Deadlock risk — ≥3 chained LeftOuter joins (concurrent-write hazard)
        let left_outer_count = {
            let mut v = Vec::new();
            plan.root.collect_where(&|n| matches!(&n.op, OpKind::HashMatch(jt) | OpKind::NestedLoopJoin(jt)
                if matches!(jt, crate::plan::JoinType::LeftOuter | crate::plan::JoinType::OuterApply)), &mut v);
            v.len()
        };
        if left_outer_count >= 2 {
            out.push(Bottleneck::new(
                BottleneckKind::DeadlockRisk { join_count: left_outer_count },
                left_outer_count as f32 * 5.0,
                "root/ChainedOuterJoins",
            ));
        }

        // Sort by severity descending, then cost_impact descending
        out.sort_by(|a, b| b.severity.cmp(&a.severity)
            .then(b.cost_impact_pct.partial_cmp(&a.cost_impact_pct).unwrap_or(std::cmp::Ordering::Equal)));
        out
    }

    pub fn critical_count(bottlenecks: &[Bottleneck]) -> usize {
        bottlenecks.iter().filter(|b| b.is_critical()).count()
    }

    pub fn high_or_above_count(bottlenecks: &[Bottleneck]) -> usize {
        bottlenecks.iter().filter(|b| b.severity >= Severity::High).count()
    }

    pub fn total_cost_impact(bottlenecks: &[Bottleneck]) -> f32 {
        bottlenecks.iter().map(|b| b.cost_impact_pct).sum::<f32>().min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan};

    #[test] fn sales_order_detects_deadlock_risk() {
        let plan = sales_order_plan();
        let bns  = BottleneckDetector::detect(&plan);
        assert!(bns.iter().any(|b| matches!(b.kind, BottleneckKind::DeadlockRisk { .. })),
            "Two chained LeftOuter joins must trigger DeadlockRisk");
    }
    #[test] fn sales_order_no_nested_loop_bottleneck() {
        let plan = sales_order_plan();
        let bns  = BottleneckDetector::detect(&plan);
        assert!(!bns.iter().any(|b| matches!(b.kind, BottleneckKind::NestedLoopOnLargeSet { .. })));
    }
    #[test] fn simple_plan_nested_loop_detected() {
        let plan = simple_nested_loop_plan();
        let bns  = BottleneckDetector::detect(&plan);
        assert!(bns.iter().any(|b| matches!(b.kind, BottleneckKind::NestedLoopOnLargeSet { .. })));
    }
    #[test] fn simple_plan_table_scan_detected() {
        let plan = simple_nested_loop_plan();
        let bns  = BottleneckDetector::detect(&plan);
        assert!(bns.iter().any(|b| matches!(b.kind, BottleneckKind::UnsargablePredicate { .. })));
    }
    #[test] fn bottleneck_list_sorted_critical_first() {
        let plan = simple_nested_loop_plan();
        let bns  = BottleneckDetector::detect(&plan);
        if bns.len() > 1 {
            assert!(bns[0].severity >= bns[1].severity);
        }
    }
    #[test] fn severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High     > Severity::Medium);
        assert!(Severity::Medium   > Severity::Low);
    }
    #[test] fn bottleneck_title_non_empty() {
        let kinds = vec![
            BottleneckKind::NestedLoopOnLargeSet { inner_rows: 5000 },
            BottleneckKind::CartesianProduct,
            BottleneckKind::KeyLookupBottleneck { lookup_count: 3 },
            BottleneckKind::UnsargablePredicate { table: "T".into() },
            BottleneckKind::StaleStatistics { estimated: 100, actual: 5000, ratio: 50.0 },
            BottleneckKind::ExcessiveSort,
            BottleneckKind::SpoolRecompute,
            BottleneckKind::DeadlockRisk { join_count: 3 },
        ];
        for k in &kinds {
            assert!(!k.title().is_empty());
            assert!(!k.description().is_empty());
        }
    }
    #[test] fn critical_count_counts_correctly() {
        let plan = simple_nested_loop_plan();
        let bns  = BottleneckDetector::detect(&plan);
        let cc   = BottleneckDetector::critical_count(&bns);
        let actual_critical = bns.iter().filter(|b| b.is_critical()).count();
        assert_eq!(cc, actual_critical);
    }
    #[test] fn total_cost_impact_capped_at_100() {
        let bns: Vec<Bottleneck> = (0..10).map(|i| Bottleneck::new(
            BottleneckKind::ExcessiveSort, 15.0, format!("root/Sort{i}"),
        )).collect();
        assert!(BottleneckDetector::total_cost_impact(&bns) <= 100.0);
    }
    #[test] fn deadlock_risk_severity_is_critical() {
        let k = BottleneckKind::DeadlockRisk { join_count: 3 };
        assert_eq!(k.default_severity(), Severity::Critical);
    }
    #[test] fn cartesian_product_severity_is_critical() {
        assert_eq!(BottleneckKind::CartesianProduct.default_severity(), Severity::Critical);
    }
    #[test] fn nested_loop_very_large_is_critical() {
        let k = BottleneckKind::NestedLoopOnLargeSet { inner_rows: 50_000 };
        assert_eq!(k.default_severity(), Severity::Critical);
    }
}
