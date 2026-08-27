//! Integration tests: Hepta .hepta files → DFG plan → UEK kernel execution.

use dfg_engine::{EdgeKind, NodeKind};
use hdf_bridge::{translate, DefaultSteward, ExecuteOutcome, UekEvent, UekKernel};
use hepta_spatial::parse as hepta_parse;

const BAGHDAD: &str = include_str!("../../hepta-spatial/maps/baghdad_primary.hepta");
const WADI: &str = include_str!("../../hepta-spatial/maps/wadi_al_salam_primary.hepta");

// ── Baghdad plan ──────────────────────────────────────────────────────────────

#[test]
fn baghdad_plan_has_correct_node_count() {
    let file = hepta_parse(BAGHDAD).expect("baghdad parse");
    let plan = translate(&file);
    // 1 anchor + 6 sectors = 7 nodes (no NODE blocks in baghdad map)
    assert_eq!(plan.node_count(), 7, "expected anchor + 6 sectors");
}

#[test]
fn baghdad_plan_has_root_governance_node() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let roots: Vec<_> = plan.root_nodes().collect();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].kind, NodeKind::RootGovernance);
}

#[test]
fn baghdad_plan_has_six_clusters() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    assert_eq!(plan.cluster_nodes().count(), 6);
}

#[test]
fn baghdad_edges_include_chord72_and_chord73() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let has_72 = plan.edges.iter().any(|e| e.kind == EdgeKind::Chord72);
    let has_73 = plan.edges.iter().any(|e| e.kind == EdgeKind::Chord73);
    assert!(has_72, "expected at least one Chord72 edge");
    assert!(has_73, "expected at least one Chord73 edge");
}

#[test]
fn baghdad_moon_node_has_two_outgoing_edges() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let moon = plan.node_by_label("MOON").expect("MOON node");
    let out: Vec<_> = plan.edges_from(moon.id).collect();
    assert_eq!(
        out.len(),
        2,
        "MOON should have BEAM/2 and BEAM/3 outgoing edges"
    );
}

#[test]
fn baghdad_mars_has_elevated_risk_condition() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let mars = plan.node_by_label("MARS").expect("MARS node");
    let cond = mars.condition.as_deref().unwrap_or("");
    assert!(
        cond.contains("ELEVATED_RISK"),
        "MARS should carry ELEVATED_RISK condition"
    );
}

#[test]
fn baghdad_edges_all_resolved() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    for edge in &plan.edges {
        assert_ne!(
            edge.to,
            u32::MAX,
            "unresolved edge from {} → {} (label: {})",
            edge.from,
            edge.to,
            edge.target_label
        );
    }
}

#[test]
fn baghdad_attribs_translated_to_feature_vector() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let moon = plan.node_by_label("MOON").expect("MOON");
    // MOON has waste:demand attrib in baghdad_primary.hepta
    let has_waste = moon.attribs.iter().any(|a| a.domain == "waste");
    assert!(has_waste, "MOON should have waste domain attribs");
}

#[test]
fn baghdad_coord_system_preserved() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    assert!(
        plan.coord_system.contains("Kaki7d"),
        "coord_system should be Kaki7d"
    );
}

// ── Wadi al-Salam plan ────────────────────────────────────────────────────────

#[test]
fn wadi_plan_has_anchor_and_sectors() {
    let file = hepta_parse(WADI).expect("wadi parse");
    let plan = translate(&file);
    assert_eq!(plan.root_nodes().count(), 1, "1 anchor");
    assert!(plan.cluster_nodes().count() >= 4, "at least 4 sectors");
}

#[test]
fn wadi_plan_has_particle_container() {
    let file = hepta_parse(WADI).expect("wadi parse");
    let plan = translate(&file);
    assert!(
        plan.container_nodes().count() >= 1,
        "wadi has at least one NODE block → ParticleContainer"
    );
}

#[test]
fn wadi_anchor_carries_sacred_weight() {
    let file = hepta_parse(WADI).expect("wadi parse");
    let plan = translate(&file);
    let anchor = plan.root_nodes().next().expect("anchor");
    let sw = anchor.attrib("anchor", "sacred_weight");
    assert!(sw.is_some(), "anchor sacred_weight should be present");
    let val: f64 = sw.unwrap().parse().expect("numeric");
    assert!(val > 0.0);
}

#[test]
fn wadi_path_edges_present() {
    let file = hepta_parse(WADI).expect("wadi parse");
    let plan = translate(&file);
    let has_path = plan.edges.iter().any(|e| e.kind == EdgeKind::Path);
    assert!(has_path, "wadi map uses PATH beams");
}

#[test]
fn wadi_mercury_has_chord_link_attrib() {
    let file = hepta_parse(WADI).expect("wadi parse");
    let plan = translate(&file);
    let mercury = plan.node_by_label("MERCURY").expect("MERCURY");
    let has_link = mercury.attribs.iter().any(|a| a.key == "chord_link_target");
    assert!(has_link, "MERCURY should have chord_link_target attrib");
}

#[test]
fn wadi_project_id_preserved() {
    let file = hepta_parse(WADI).unwrap();
    let plan = translate(&file);
    assert!(plan.project_id.is_some(), "wadi should have project_id");
}

// ── UEK Kernel end-to-end ─────────────────────────────────────────────────────

#[test]
fn uek_kernel_executes_clean_events_on_baghdad_plan() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let mut kernel = UekKernel::new(plan, DefaultSteward);

    let event = UekEvent {
        kaki_id: [1u8; 16],
        timestamp: 1_000_000,
        attribute: "occupation_rate".into(),
        value: "0.72".into(),
        governance_score: 0.9,
    };

    let outcome = kernel.execute(event);
    assert_eq!(outcome, ExecuteOutcome::Accepted);
    assert_eq!(kernel.stalk_count(), 1);
}

#[test]
fn uek_kernel_accumulates_stalk_append_only() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let mut kernel = UekKernel::new(plan, DefaultSteward);

    let kaki = [2u8; 16];
    for ts in [100u64, 200, 300, 400, 500] {
        let e = UekEvent {
            kaki_id: kaki,
            timestamp: ts,
            attribute: "status".into(),
            value: "active".into(),
            governance_score: 0.95,
        };
        assert_eq!(kernel.execute(e), ExecuteOutcome::Accepted);
    }
    assert_eq!(kernel.stalk_count(), 1);
    assert_eq!(kernel.total_events(), 5);
}

#[test]
fn uek_kernel_journals_flagged_events_on_wadi() {
    use dfg_engine::{DataSteward, DfgNode, FeatureAttrib, GovernanceDecision};

    struct FlagAll;
    impl DataSteward for FlagAll {
        fn evaluate(&self, _: &DfgNode, _: &[FeatureAttrib]) -> GovernanceDecision {
            GovernanceDecision::Flag
        }
    }

    let file = hepta_parse(WADI).unwrap();
    let plan = translate(&file);
    let mut kernel = UekKernel::new(plan, FlagAll);

    kernel.execute(UekEvent {
        kaki_id: [3u8; 16],
        timestamp: 1,
        attribute: "x".into(),
        value: "y".into(),
        governance_score: 0.5,
    });
    assert_eq!(kernel.journal.len(), 1);
    assert_eq!(kernel.stalk_count(), 0);
}

#[test]
fn full_dfg_pipeline_baghdad_multinode() {
    let file = hepta_parse(BAGHDAD).unwrap();
    let plan = translate(&file);
    let mut kernel = UekKernel::new(plan, DefaultSteward);

    // Simulate events arriving for different KAKI particles
    let kakis: [[u8; 16]; 3] = [[10u8; 16], [20u8; 16], [30u8; 16]];
    for (i, kaki) in kakis.iter().enumerate() {
        for ts in 0u64..4 {
            let e = UekEvent {
                kaki_id: *kaki,
                timestamp: i as u64 * 100 + ts,
                attribute: "district_load".into(),
                value: format!("{}", 0.5 + ts as f64 * 0.1),
                governance_score: 0.85,
            };
            assert_eq!(kernel.execute(e), ExecuteOutcome::Accepted);
        }
    }
    assert_eq!(kernel.stalk_count(), 3);
    assert_eq!(kernel.total_events(), 12);
}
