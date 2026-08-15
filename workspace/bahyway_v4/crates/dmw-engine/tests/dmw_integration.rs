//! DMWEngine integration tests — end-to-end scenario coverage.
//!
//! Each test runs through DmwAnalyzer::analyze (bottleneck → journey → oracle)
//! and verifies the coherence of the full report for known plan fixtures.

use dmw_engine::{
    DmwAnalyzer,
    DmwState,
    BottleneckKind,
    RotationType,
    FragmentationMap,
    FragmentationState,
    plan::{sales_order_plan, simple_nested_loop_plan, QueryPlan, PlanNode, OpKind, ScanType},
    fragmentation::sales_order_frag_map,
    journey::JourneyLevelId,
};

// ── Sales Order plan ─────────────────────────────────────────────────────────

#[test]
fn sales_order_state_not_critical() {
    // Sales-order uses hash joins throughout; only Level A is penalised (deadlock risk).
    // Plan execution efficiency is high → state must be Stable or Golden.
    let report = DmwAnalyzer::analyze(&sales_order_plan());
    assert!(
        matches!(report.state, DmwState::Stable | DmwState::Golden),
        "Expected Stable or Golden, got {:?} ({:.1}%)", report.state, report.alignment_pct
    );
}

#[test]
fn sales_order_has_deadlock_risk_bottleneck() {
    let report = DmwAnalyzer::analyze(&sales_order_plan());
    assert!(
        report.bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::DeadlockRisk { .. })),
        "Two chained LeftOuter joins must produce DeadlockRisk"
    );
}

#[test]
fn sales_order_oracle_recommends_stage_outer_joins() {
    let report = DmwAnalyzer::analyze(&sales_order_plan());
    assert!(
        report.recommendations.iter().any(|r| r.rotation == RotationType::StageOuterJoins),
        "Oracle must recommend StageOuterJoins for chained outer joins"
    );
}

#[test]
fn sales_order_projected_higher_than_actual() {
    let report = DmwAnalyzer::analyze(&sales_order_plan());
    assert!(report.projected_pct >= report.alignment_pct,
        "Projection must be ≥ current alignment");
}

#[test]
fn sales_order_all_seven_journey_levels_scored() {
    let report = DmwAnalyzer::analyze(&sales_order_plan());
    for id in [
        JourneyLevelId::ME, JourneyLevelId::GU, JourneyLevelId::SAG,
        JourneyLevelId::A,  JourneyLevelId::IZI, JourneyLevelId::UD,
        JourneyLevelId::URU,
    ] {
        let score = report.journey.level_score(id);
        assert!(score >= 0.0 && score <= 1.0, "{:?} score out of range: {score}", id);
    }
}

// ── Simple NestedLoop plan ────────────────────────────────────────────────────

#[test]
fn simple_plan_nested_loop_bottleneck_detected() {
    let report = DmwAnalyzer::analyze(&simple_nested_loop_plan());
    assert!(report.bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::NestedLoopOnLargeSet { .. })));
}

#[test]
fn simple_plan_table_scan_bottleneck_detected() {
    let report = DmwAnalyzer::analyze(&simple_nested_loop_plan());
    assert!(report.bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::UnsargablePredicate { .. })));
}

#[test]
fn simple_plan_oracle_recommends_loop_to_hash() {
    let report = DmwAnalyzer::analyze(&simple_nested_loop_plan());
    assert!(report.recommendations.iter().any(|r| r.rotation == RotationType::LoopToHashConversion),
        "NestedLoop must trigger LoopToHashConversion recommendation");
}

#[test]
fn simple_plan_alignment_score_below_65() {
    let report = DmwAnalyzer::analyze(&simple_nested_loop_plan());
    // NestedLoop(50k rows) + TableScan degrades L3, L4, L5 → score ≤ 65 %
    assert!(report.alignment_pct <= 65.0,
        "NestedLoop + TableScan should score ≤ 65%, got {:.1}%", report.alignment_pct);
}

// ── Perfect plan (no bottlenecks) ────────────────────────────────────────────

#[test]
fn perfect_plan_golden_state() {
    let root   = PlanNode::new(OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered }, 100.0, 1000, 50);
    let plan   = QueryPlan::new("Perfect Plan", root, 100.0).with_actual_rows(1000);
    let report = DmwAnalyzer::analyze(&plan);
    assert_eq!(report.state, DmwState::Golden, "alignment={:.1}%", report.alignment_pct);
    assert!(report.bottlenecks.is_empty(), "No bottlenecks expected");
}

#[test]
fn perfect_plan_no_recommendations() {
    let root   = PlanNode::new(OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered }, 100.0, 500, 25);
    let plan   = QueryPlan::new("Perfect", root, 100.0).with_actual_rows(500);
    let report = DmwAnalyzer::analyze(&plan);
    assert!(report.recommendations.is_empty());
}

// ── Stale statistics scenario ─────────────────────────────────────────────────

#[test]
fn stale_statistics_plan_detected() {
    let root = PlanNode::new(
        OpKind::IndexSeek { name: "IX_T_col".into(), scan_type: ScanType::NonClustered },
        100.0, 10, 500,   // estimated 10 rows but 5000 actual
    );
    let plan = QueryPlan::new("Stale Stats", root, 100.0).with_actual_rows(5000);
    let report = DmwAnalyzer::analyze(&plan);
    assert!(report.bottlenecks.iter().any(|b| matches!(b.kind, BottleneckKind::StaleStatistics { .. })));
    assert!(report.recommendations.iter().any(|r| r.rotation == RotationType::UpdateStatistics));
}

// ── Fragmentation map ─────────────────────────────────────────────────────────

#[test]
fn sales_frag_map_critical_count() {
    let map = sales_order_frag_map();
    // IX_Person_LastName (38.4%) + IX_SalesOrder_CustomerID (31.0%) = 2 rebuild targets
    assert_eq!(map.critical_count(), 2);
}

#[test]
fn sales_frag_map_has_rebuild_targets() {
    let map = sales_order_frag_map();
    assert!(!map.rebuild_targets().is_empty());
    for t in map.rebuild_targets() {
        assert!(t.fragmentation_pct >= 30.0, "{} at {:.1}%", t.index_name, t.fragmentation_pct);
    }
}

#[test]
fn sales_frag_map_health_score_between_0_and_100() {
    let score = sales_order_frag_map().overall_health_score();
    assert!(score >= 0.0 && score <= 100.0, "health score = {score:.1}");
}

#[test]
fn empty_frag_map_is_fully_healthy() {
    let map = FragmentationMap::new(vec![]);
    assert_eq!(map.overall_health_score(), 100.0);
    assert_eq!(map.critical_count(), 0);
}

#[test]
fn fragmentation_state_ordering_respected() {
    assert!(FragmentationState::Critical > FragmentationState::Healthy);
}
