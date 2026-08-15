//! QueryPlan — pure Rust representation of an RDBMS query execution plan.
//!
//! No live database connection is required.  Plans are described as data
//! structures that mirror what a query optimizer would produce, allowing the
//! DMW Engine to analyse bottlenecks, join strategies, and cost distribution
//! entirely in memory.

/// How two row-sets are combined.
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    CrossApply,
    OuterApply,
    SemiJoin,
    AntiSemiJoin,
}

/// Granularity of an index or table access.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanType {
    Clustered,
    NonClustered,
    FullTable,
    Covering,
}

/// A single node in the query execution plan tree.
#[derive(Debug, Clone, PartialEq)]
pub enum OpKind {
    HashMatch(JoinType),
    NestedLoopJoin(JoinType),
    MergeJoin(JoinType),
    IndexScan { name: String, scan_type: ScanType },
    IndexSeek { name: String, scan_type: ScanType },
    TableScan { name: String },
    KeyLookup { table: String },
    ComputeScalar,
    Sort { is_distinct: bool },
    StreamAggregate,
    HashAggregate,
    Spool { is_eager: bool },
    Filter,
    Top { with_ties: bool },
    Concatenation,
    Parallelism,
}

impl OpKind {
    /// True when this operator is a join that can exhibit O(n²) behaviour.
    pub fn is_nested_loop(&self) -> bool {
        matches!(self, OpKind::NestedLoopJoin(_))
    }

    /// True when this operator is a hash-based join.
    pub fn is_hash_join(&self) -> bool {
        matches!(self, OpKind::HashMatch(_))
    }

    /// True when this is a full table scan (no index).
    pub fn is_table_scan(&self) -> bool {
        matches!(self, OpKind::TableScan { .. })
    }

    /// True when this operator forces a row-by-row lookup back to the base table.
    pub fn is_key_lookup(&self) -> bool {
        matches!(self, OpKind::KeyLookup { .. })
    }

    /// True when this is a spool (materialises intermediate results — often a sign of parameter sniffing).
    pub fn is_spool(&self) -> bool {
        matches!(self, OpKind::Spool { .. })
    }

    /// Human-readable label for display and Oracle messages.
    pub fn label(&self) -> String {
        match self {
            OpKind::HashMatch(jt)       => format!("HashMatch({jt:?})"),
            OpKind::NestedLoopJoin(jt)  => format!("NestedLoop({jt:?})"),
            OpKind::MergeJoin(jt)       => format!("MergeJoin({jt:?})"),
            OpKind::IndexScan { name, .. } => format!("IndexScan[{name}]"),
            OpKind::IndexSeek { name, .. } => format!("IndexSeek[{name}]"),
            OpKind::TableScan { name }  => format!("TableScan[{name}]"),
            OpKind::KeyLookup { table } => format!("KeyLookup[{table}]"),
            OpKind::ComputeScalar       => "ComputeScalar".into(),
            OpKind::Sort { .. }         => "Sort".into(),
            OpKind::StreamAggregate     => "StreamAggregate".into(),
            OpKind::HashAggregate       => "HashAggregate".into(),
            OpKind::Spool { is_eager }  => if *is_eager { "EagerSpool".into() } else { "LazyIndexSpool".into() },
            OpKind::Filter              => "Filter".into(),
            OpKind::Top { .. }          => "Top".into(),
            OpKind::Concatenation       => "Concatenation".into(),
            OpKind::Parallelism         => "Parallelism".into(),
        }
    }
}

/// One node in the query plan tree, with its children.
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub op: OpKind,
    /// Cost of this subtree as a percentage of the total query cost (0–100).
    pub cost_pct: f32,
    /// Rows the optimiser estimates this node will produce.
    pub estimated_rows: u32,
    /// Logical reads (I/O cost proxy).
    pub logical_reads: u32,
    pub children: Vec<PlanNode>,
}

impl PlanNode {
    pub fn new(op: OpKind, cost_pct: f32, estimated_rows: u32, logical_reads: u32) -> Self {
        Self { op, cost_pct, estimated_rows, logical_reads, children: Vec::new() }
    }

    pub fn with_children(mut self, children: Vec<PlanNode>) -> Self {
        self.children = children;
        self
    }

    /// Recursively count all nodes that satisfy a predicate.
    pub fn count_where<F: Fn(&PlanNode) -> bool>(&self, f: &F) -> usize {
        let self_count = if f(self) { 1 } else { 0 };
        self_count + self.children.iter().map(|c| c.count_where(f)).sum::<usize>()
    }

    /// Walk the tree and collect all nodes matching a predicate (depth-first).
    pub fn collect_where<'a, F: Fn(&PlanNode) -> bool>(&'a self, f: &F, out: &mut Vec<&'a PlanNode>) {
        if f(self) { out.push(self); }
        for child in &self.children { child.collect_where(f, out); }
    }

    /// Node with the highest cost_pct in this subtree.
    pub fn highest_cost_node(&self) -> &PlanNode {
        let mut best = self;
        for child in &self.children {
            let cand = child.highest_cost_node();
            if cand.cost_pct > best.cost_pct { best = cand; }
        }
        best
    }
}

/// Complete description of a query execution plan.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Unique identifier for this plan (e.g. "SalesOrder Correlated Subquery").
    pub query_id:    String,
    pub root:        PlanNode,
    /// Estimated total cost in abstract optimizer units.
    pub total_cost:  f32,
    /// Whether the plan uses intra-query parallelism.
    pub parallelism: bool,
    /// Actual rows returned (0 if not yet executed).
    pub actual_rows: u32,
}

impl QueryPlan {
    pub fn new(query_id: impl Into<String>, root: PlanNode, total_cost: f32) -> Self {
        Self { query_id: query_id.into(), root, total_cost, parallelism: false, actual_rows: 0 }
    }

    pub fn with_parallelism(mut self, p: bool) -> Self { self.parallelism = p; self }
    pub fn with_actual_rows(mut self, r: u32) -> Self { self.actual_rows = r; self }

    pub fn join_count(&self) -> usize {
        self.root.count_where(&|n| {
            matches!(n.op, OpKind::HashMatch(_) | OpKind::NestedLoopJoin(_) | OpKind::MergeJoin(_))
        })
    }

    pub fn nested_loop_count(&self) -> usize {
        self.root.count_where(&|n| matches!(n.op, OpKind::NestedLoopJoin(_)))
    }

    pub fn hash_match_count(&self) -> usize {
        self.root.count_where(&|n| matches!(n.op, OpKind::HashMatch(_)))
    }

    pub fn key_lookup_count(&self) -> usize {
        self.root.count_where(&|n| n.op.is_key_lookup())
    }

    pub fn table_scan_count(&self) -> usize {
        self.root.count_where(&|n| n.op.is_table_scan())
    }

    pub fn spool_count(&self) -> usize {
        self.root.count_where(&|n| n.op.is_spool())
    }

    pub fn sort_count(&self) -> usize {
        self.root.count_where(&|n| matches!(n.op, OpKind::Sort { .. }))
    }

    pub fn highest_cost_node(&self) -> &PlanNode {
        self.root.highest_cost_node()
    }

    pub fn has_key_lookups(&self) -> bool { self.key_lookup_count() > 0 }
    pub fn has_table_scans(&self) -> bool { self.table_scan_count() > 0 }
    pub fn has_sorts(&self) -> bool       { self.sort_count() > 0 }
    pub fn has_spools(&self) -> bool      { self.spool_count() > 0 }

    /// Cardinality mismatch ratio: actual_rows / estimated_rows of root node.
    /// > 10× or < 0.1× signals stale statistics.
    pub fn cardinality_ratio(&self) -> Option<f32> {
        if self.root.estimated_rows == 0 || self.actual_rows == 0 { return None; }
        Some(self.actual_rows as f32 / self.root.estimated_rows as f32)
    }
}

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// Builds the SalesOrder correlated-subquery plan from the DeepMotionWay v1.1.0 screenshot.
/// Hash chain: two HashMatch joins feeding through ComputeScalar operators,
/// backed by one clustered + two non-clustered index scans.
pub fn sales_order_plan() -> QueryPlan {
    use OpKind::*;
    use ScanType::*;
    use JoinType::*;

    let idx_phone = PlanNode::new(IndexScan { name: "IX_PersonPhone_PhoneNumber".into(), scan_type: NonClustered }, 3.0, 19972, 291);
    let cs1       = PlanNode::new(ComputeScalar, 0.0, 19972, 0);
    let cs2       = PlanNode::new(ComputeScalar, 0.0, 291, 0);
    let _idx_cl   = PlanNode::new(IndexScan { name: "IX_Person_LastName".into(), scan_type: Clustered }, 1.0, 291, 808);
    let cs3       = PlanNode::new(ComputeScalar, 0.0, 291, 0);

    let hm_inner  = PlanNode::new(HashMatch(Inner), 15.0, 291, 808)
        .with_children(vec![cs2, cs3.clone()]);

    let cs4       = PlanNode::new(ComputeScalar, 0.0, 808, 0);
    let idx_country = PlanNode::new(
        IndexScan { name: "IX_PersonPhone_CountryRegion".into(), scan_type: NonClustered }, 0.0, 1, 1,
    );

    let hm_outer  = PlanNode::new(HashMatch(LeftOuter), 18.0, 808, 808)
        .with_children(vec![cs4, PlanNode::new(ComputeScalar, 0.0, 1, 0).with_children(vec![idx_country])]);

    let root      = PlanNode::new(HashMatch(LeftOuter), 63.0, 6069, 19972)
        .with_children(vec![
            PlanNode::new(ComputeScalar, 0.0, 6069, 0).with_children(vec![
                PlanNode::new(IndexSeek { name: "PK_BusinessEntity".into(), scan_type: Clustered }, 0.0, 6069, 290),
            ]),
            hm_outer.with_children(vec![hm_inner, cs1.with_children(vec![idx_phone])]),
        ]);

    QueryPlan::new("SalesOrder Correlated Subquery", root, 100.0)
        .with_actual_rows(6069)
}

/// A simple single-join plan used in unit tests.
pub fn simple_nested_loop_plan() -> QueryPlan {
    use OpKind::*; use JoinType::*; use ScanType::*;
    let inner = PlanNode::new(TableScan { name: "Orders".into() }, 80.0, 50_000, 50_000);
    let outer = PlanNode::new(IndexSeek { name: "PK_Customer".into(), scan_type: Clustered }, 5.0, 100, 100);
    let root  = PlanNode::new(NestedLoopJoin(Inner), 95.0, 100, 50_000)
        .with_children(vec![outer, inner]);
    QueryPlan::new("Simple NestedLoop", root, 100.0).with_actual_rows(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn sales_order_has_three_joins() {
        assert_eq!(sales_order_plan().join_count(), 3);
    }
    #[test] fn sales_order_has_three_hash_matches() {
        assert_eq!(sales_order_plan().hash_match_count(), 3);
    }
    #[test] fn sales_order_no_nested_loops() {
        assert_eq!(sales_order_plan().nested_loop_count(), 0);
    }
    #[test] fn sales_order_actual_rows() {
        assert_eq!(sales_order_plan().actual_rows, 6069);
    }
    #[test] fn sales_order_no_key_lookups() {
        assert!(!sales_order_plan().has_key_lookups());
    }
    #[test] fn simple_plan_has_one_nested_loop() {
        assert_eq!(simple_nested_loop_plan().nested_loop_count(), 1);
    }
    #[test] fn simple_plan_has_table_scan() {
        assert!(simple_nested_loop_plan().has_table_scans());
    }
    #[test] fn highest_cost_node_label() {
        let plan = sales_order_plan();
        let node = plan.highest_cost_node();
        assert!(node.cost_pct >= 60.0, "root should dominate");
    }
    #[test] fn op_kind_labels_non_empty() {
        let ops: Vec<OpKind> = vec![
            OpKind::HashMatch(JoinType::Inner),
            OpKind::NestedLoopJoin(JoinType::LeftOuter),
            OpKind::TableScan { name: "T".into() },
            OpKind::KeyLookup { table: "T".into() },
            OpKind::ComputeScalar,
            OpKind::Sort { is_distinct: false },
            OpKind::Spool { is_eager: true },
        ];
        for op in ops { assert!(!op.label().is_empty()); }
    }
    #[test] fn nested_loop_is_nested_loop() {
        assert!(OpKind::NestedLoopJoin(JoinType::Inner).is_nested_loop());
        assert!(!OpKind::HashMatch(JoinType::Inner).is_nested_loop());
    }
    #[test] fn table_scan_flag() {
        assert!(OpKind::TableScan { name: "X".into() }.is_table_scan());
        assert!(!OpKind::IndexSeek { name: "X".into(), scan_type: ScanType::Clustered }.is_table_scan());
    }
    #[test] fn cardinality_ratio_none_when_zero() {
        let plan = QueryPlan::new("empty", PlanNode::new(OpKind::ComputeScalar, 0.0, 0, 0), 0.0);
        assert!(plan.cardinality_ratio().is_none());
    }
}
