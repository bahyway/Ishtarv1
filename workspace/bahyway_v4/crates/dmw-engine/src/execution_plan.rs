//! execution_plan.rs — Plan Visualization Graph
//!
//! Provides `PlanOperator` (16 SQL operators with colour coding) and `PlanGraph`
//! (a flat, id-referenced node graph suitable for rendering in the DubSar IDE).
//!
//! `PlanComparator` generates two side-by-side graphs from a v4.0 `QueryPlan`:
//!   - `legacy_graph()` — reflects the plan as-is, with bottleneck operators
//!   - `sovereign_graph()` — the ideal DMW-optimised plan (HashJoin + IndexSeeks)
//!
//! The before/after comparison is the core visual story of DeepMotionWay.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::plan::{OpKind, PlanNode as V4PlanNode, QueryPlan, ScanType};

// ── Plan Operator ─────────────────────────────────────────────────────────────

/// A SQL execution plan operator — maps to an operator box in SSMS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanOperator {
    NestedLoopJoin,
    HashJoin,
    MergeJoin,
    CrossJoin,
    ClusteredIndexSeek,
    IndexSeek,
    IndexScan,
    TableScan,
    StreamAggregate,
    HashAggregate,
    SortOperator,
    ComputeScalar,
    Filter,
    Select,
    Assert,
    Parallelism,
}

impl PlanOperator {
    pub fn display_name(&self) -> &'static str {
        match self {
            PlanOperator::NestedLoopJoin      => "Nested Loops",
            PlanOperator::HashJoin            => "Hash Match (Join)",
            PlanOperator::MergeJoin           => "Merge Join",
            PlanOperator::CrossJoin           => "Cross Join",
            PlanOperator::ClusteredIndexSeek  => "Clustered Index Seek",
            PlanOperator::IndexSeek           => "Index Seek",
            PlanOperator::IndexScan           => "Index Scan",
            PlanOperator::TableScan           => "Table Scan",
            PlanOperator::StreamAggregate     => "Stream Aggregate",
            PlanOperator::HashAggregate       => "Hash Aggregate",
            PlanOperator::SortOperator        => "Sort",
            PlanOperator::ComputeScalar       => "Compute Scalar",
            PlanOperator::Filter              => "Filter",
            PlanOperator::Select              => "SELECT",
            PlanOperator::Assert              => "Assert",
            PlanOperator::Parallelism         => "Parallelism",
        }
    }

    /// RGB colour for visualisation — red=bottleneck, green=optimal, teal=neutral.
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            PlanOperator::NestedLoopJoin |
            PlanOperator::CrossJoin      |
            PlanOperator::TableScan      => (220, 50, 50),   // red — bottleneck
            PlanOperator::IndexScan      |
            PlanOperator::SortOperator   => (255, 180, 0),   // amber — warning
            PlanOperator::HashJoin       |
            PlanOperator::MergeJoin      |
            PlanOperator::ClusteredIndexSeek |
            PlanOperator::IndexSeek      => (0, 200, 100),   // green — optimal
            _                            => (0, 180, 180),   // teal — neutral
        }
    }

    /// `true` when this operator is a known performance bottleneck.
    pub fn is_bottleneck(&self) -> bool {
        matches!(self,
            PlanOperator::NestedLoopJoin |
            PlanOperator::CrossJoin      |
            PlanOperator::TableScan)
    }

    /// `true` when this operator represents an efficient, sovereign execution path.
    pub fn is_sovereign(&self) -> bool {
        matches!(self,
            PlanOperator::HashJoin            |
            PlanOperator::ClusteredIndexSeek  |
            PlanOperator::IndexSeek)
    }
}

// ── Plan Node ────────────────────────────────────────────────────────────────

/// A single node in the flat `PlanGraph` (id-referenced, not tree-embedded).
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub id:             u32,
    pub operator:       PlanOperator,
    pub cost_pct:       f32,
    pub estimated_rows: u64,
    pub object_name:    Option<String>,
    pub children:       Vec<u32>,
    /// Oracle fix hint to show in the DubSar IDE tooltip.
    pub oracle_fix:     Option<String>,
}

impl PlanNode {
    pub fn new(id: u32, operator: PlanOperator, cost_pct: f32, estimated_rows: u64) -> Self {
        PlanNode { id, operator, cost_pct, estimated_rows,
                   object_name: None, children: Vec::new(), oracle_fix: None }
    }

    pub fn with_object(mut self, name: impl Into<String>) -> Self {
        self.object_name = Some(name.into()); self
    }

    pub fn with_children(mut self, children: Vec<u32>) -> Self {
        self.children = children; self
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        let s = fix.into();
        if !s.is_empty() { self.oracle_fix = Some(s); }
        self
    }
}

// ── Plan Graph ───────────────────────────────────────────────────────────────

/// Flat, id-referenced plan graph for DubSar IDE rendering.
#[derive(Debug, Clone)]
pub struct PlanGraph {
    pub title:           String,
    pub nodes:           Vec<PlanNode>,
    pub root_id:         u32,
    pub total_cost:      f32,
    pub is_sovereign:    bool,
    pub alignment_score: f32,
}

impl PlanGraph {
    pub fn node(&self, id: u32) -> Option<&PlanNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn bottleneck_nodes(&self) -> Vec<&PlanNode> {
        self.nodes.iter().filter(|n| n.operator.is_bottleneck()).collect()
    }

    pub fn total_bottleneck_cost(&self) -> f32 {
        self.bottleneck_nodes().iter().map(|n| n.cost_pct).sum()
    }

    pub fn sovereign_nodes(&self) -> Vec<&PlanNode> {
        self.nodes.iter().filter(|n| n.operator.is_sovereign()).collect()
    }
}

// ── Plan Comparator ───────────────────────────────────────────────────────────

/// Generates legacy and sovereign `PlanGraph` instances from a v4.0 `QueryPlan`.
pub struct PlanComparator;

impl PlanComparator {
    /// Build a `PlanGraph` that mirrors the actual `QueryPlan` tree.
    pub fn legacy_graph(plan: &QueryPlan) -> PlanGraph {
        let mut nodes: Vec<PlanNode> = Vec::new();
        let root_id = flatten_v4_node(&plan.root, &mut nodes);

        let bottleneck_cost: f32 = nodes.iter()
            .filter(|n| n.operator.is_bottleneck())
            .map(|n| n.cost_pct)
            .sum();
        let alignment = (1.0 - bottleneck_cost / 100.0).clamp(0.0, 1.0);

        PlanGraph {
            title:           format!("Legacy Plan — {}", plan.query_id),
            nodes,
            root_id,
            total_cost:      plan.total_cost,
            is_sovereign:    false,
            alignment_score: alignment,
        }
    }

    /// Build an idealised sovereign `PlanGraph` (HashJoin + IndexSeeks only).
    pub fn sovereign_graph(plan: &QueryPlan) -> PlanGraph {
        let mut nodes: Vec<PlanNode> = Vec::new();
        let join_count = plan.join_count().max(1);
        let leaf_cost  = (100.0 - 12.0) / join_count as f32;

        let mut leaf_ids = Vec::new();
        for i in 0..join_count {
            let id = nodes.len() as u32;
            nodes.push(
                PlanNode::new(id, PlanOperator::ClusteredIndexSeek,
                    leaf_cost, plan.root.estimated_rows as u64 / (i as u64 + 1))
                    .with_object(format!("Table_{}_Sovereign", i + 1))
            );
            leaf_ids.push(id);
        }

        let join_id = nodes.len() as u32;
        nodes.push(
            PlanNode::new(join_id, PlanOperator::HashJoin, 12.0, plan.root.estimated_rows as u64)
                .with_children(leaf_ids)
        );

        let root_id = nodes.len() as u32;
        nodes.push(
            PlanNode::new(root_id, PlanOperator::Select, 0.0, plan.root.estimated_rows as u64)
                .with_children(vec![join_id])
        );

        PlanGraph {
            title:           format!("DMW Sovereign Plan — {}", plan.query_id),
            nodes,
            root_id,
            total_cost:      100.0,
            is_sovereign:    true,
            alignment_score: 0.88,
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Post-order traversal of v4.0's tree `PlanNode` → flat id-referenced Vec.
/// Returns the id assigned to this node.
fn flatten_v4_node(v4: &V4PlanNode, flat: &mut Vec<PlanNode>) -> u32 {
    let child_ids: Vec<u32> = v4.children.iter()
        .map(|c| flatten_v4_node(c, flat))
        .collect();
    let my_id = flat.len() as u32;
    flat.push(
        PlanNode::new(my_id, map_op_kind(&v4.op), v4.cost_pct, v4.estimated_rows as u64)
            .with_children(child_ids)
    );
    my_id
}

fn map_op_kind(op: &OpKind) -> PlanOperator {
    match op {
        OpKind::HashMatch(_)                                      => PlanOperator::HashJoin,
        OpKind::NestedLoopJoin(_)                                 => PlanOperator::NestedLoopJoin,
        OpKind::MergeJoin(_)                                      => PlanOperator::MergeJoin,
        OpKind::IndexSeek { scan_type: ScanType::Clustered, .. }  => PlanOperator::ClusteredIndexSeek,
        OpKind::IndexSeek { .. }                                  => PlanOperator::IndexSeek,
        OpKind::IndexScan { .. }                                  => PlanOperator::IndexScan,
        OpKind::TableScan { .. }                                  => PlanOperator::TableScan,
        OpKind::KeyLookup { .. }                                  => PlanOperator::IndexSeek,
        OpKind::Sort { .. }                                       => PlanOperator::SortOperator,
        OpKind::ComputeScalar                                     => PlanOperator::ComputeScalar,
        OpKind::Filter                                            => PlanOperator::Filter,
        OpKind::StreamAggregate                                   => PlanOperator::StreamAggregate,
        OpKind::HashAggregate                                     => PlanOperator::HashAggregate,
        OpKind::Spool { .. }                                      => PlanOperator::ComputeScalar,
        OpKind::Top { .. }                                        => PlanOperator::Select,
        OpKind::Concatenation                                     => PlanOperator::Select,
        OpKind::Parallelism                                       => PlanOperator::Parallelism,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan};

    #[test]
    fn legacy_graph_nested_loop_for_simple_plan() {
        let g = PlanComparator::legacy_graph(&simple_nested_loop_plan());
        assert!(g.nodes.iter().any(|n| n.operator == PlanOperator::NestedLoopJoin),
            "simple plan must contain NestedLoopJoin");
    }

    #[test]
    fn legacy_graph_table_scan_for_simple_plan() {
        let g = PlanComparator::legacy_graph(&simple_nested_loop_plan());
        assert!(g.nodes.iter().any(|n| n.operator == PlanOperator::TableScan));
    }

    #[test]
    fn legacy_graph_hash_join_for_sales_order() {
        let g = PlanComparator::legacy_graph(&sales_order_plan());
        assert!(g.nodes.iter().any(|n| n.operator == PlanOperator::HashJoin));
    }

    #[test]
    fn legacy_graph_no_duplicate_ids() {
        let g = PlanComparator::legacy_graph(&simple_nested_loop_plan());
        let mut ids: Vec<u32> = g.nodes.iter().map(|n| n.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), g.nodes.len(), "all node IDs must be unique");
    }

    #[test]
    fn legacy_graph_root_id_valid() {
        let g = PlanComparator::legacy_graph(&sales_order_plan());
        assert!(g.node(g.root_id).is_some(), "root_id must resolve to a node");
    }

    #[test]
    fn sovereign_graph_no_nested_loop() {
        let g = PlanComparator::sovereign_graph(&simple_nested_loop_plan());
        assert!(!g.nodes.iter().any(|n| n.operator == PlanOperator::NestedLoopJoin));
    }

    #[test]
    fn sovereign_graph_has_hash_join() {
        let g = PlanComparator::sovereign_graph(&simple_nested_loop_plan());
        assert!(g.nodes.iter().any(|n| n.operator == PlanOperator::HashJoin));
    }

    #[test]
    fn sovereign_graph_is_flagged_sovereign() {
        let g = PlanComparator::sovereign_graph(&sales_order_plan());
        assert!(g.is_sovereign);
    }

    #[test]
    fn sovereign_alignment_higher_than_legacy() {
        let legacy    = PlanComparator::legacy_graph(&simple_nested_loop_plan());
        let sovereign = PlanComparator::sovereign_graph(&simple_nested_loop_plan());
        assert!(sovereign.alignment_score > legacy.alignment_score,
            "sovereign={:.2} legacy={:.2}", sovereign.alignment_score, legacy.alignment_score);
    }

    #[test]
    fn bottleneck_nodes_empty_for_sovereign() {
        let g = PlanComparator::sovereign_graph(&simple_nested_loop_plan());
        assert_eq!(g.bottleneck_nodes().len(), 0);
    }

    #[test]
    fn operator_display_names_non_empty() {
        for op in [
            PlanOperator::NestedLoopJoin, PlanOperator::HashJoin, PlanOperator::MergeJoin,
            PlanOperator::CrossJoin, PlanOperator::ClusteredIndexSeek, PlanOperator::IndexSeek,
            PlanOperator::TableScan, PlanOperator::SortOperator,
        ] { assert!(!op.display_name().is_empty()); }
    }

    #[test]
    fn color_rgb_red_for_bottlenecks() {
        let (r, _, _) = PlanOperator::NestedLoopJoin.color_rgb();
        assert!(r > 150, "NestedLoop must have high red channel");
    }

    #[test]
    fn color_rgb_green_for_optimal() {
        let (_, g, _) = PlanOperator::HashJoin.color_rgb();
        assert!(g > 150, "HashJoin must have high green channel");
    }
}
