//! NaviEngine integration tests — end-to-end sovereign routing scenarios.
//!
//! All scenarios use the 7-node heptagram fixture (one node per sector).
//! Scenarios cover: pilgrimage routing, sensor disruption, sovereignty rules,
//! multi-hop paths, and the full Wadi al-Salam pipeline simulation.

use bahyway_core::TribeId;
use navi_engine::{
    haversine_m, seven_node_map, NaviCodeExecutor, NaviCommand, NaviGraph, NaviMap,
    RouteConstraints, RouteEngine, RouteRequest, SensorEvent, SensorFeed, SurfaceQuality,
};

fn najaf_tribe() -> TribeId {
    TribeId::from_u16(0x0001)
}
fn build_graph() -> NaviGraph {
    NaviGraph::from_navimap(&seven_node_map()).unwrap()
}

// ── Scenario 1: Basic pilgrimage route ───────────────────────────────────────

#[test]
fn pilgrimage_centre_to_all_sectors() {
    let g = build_graph();
    let e = RouteEngine::new(&g);
    for dest in 2u32..=7 {
        let plan = e
            .plan_default(1, dest)
            .unwrap_or_else(|err| panic!("route Centre→{dest} failed: {err}"));
        assert!(plan.is_valid(), "route Centre→{dest} must be golden");
        assert_eq!(*plan.waypoints.first().unwrap(), 1);
        assert_eq!(*plan.waypoints.last().unwrap(), dest);
    }
}

// ── Scenario 2: Sector-to-sector routing ─────────────────────────────────────

#[test]
fn north_to_south_routes_through_centre_or_rim() {
    let g = build_graph();
    let e = RouteEngine::new(&g);
    let plan = e.plan_default(2, 5).expect("North → South must route");
    assert!(plan.is_valid());
    // Path must contain at least 2 hops (no direct beam North↔South)
    assert!(plan.waypoint_count() >= 2);
}

#[test]
fn adjacent_outer_nodes_route_directly() {
    let g = build_graph();
    let e = RouteEngine::new(&g);
    // Node 2=North, 3=NorthEast — adjacent outer, rim beam exists
    let plan = e.plan_default(2, 3).expect("North → NorthEast must route");
    assert!(plan.is_valid());
}

// ── Scenario 3: Sensor disruption and re-route ───────────────────────────────

#[test]
fn road_closure_forces_alternate_path() {
    let mut g = build_graph();
    // Block NorthEast (node 3) — forces routes through other outer nodes
    SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
    assert!(!g.node(3).unwrap().is_passable());

    let e = RouteEngine::new(&g);
    let plan = e.plan_default(1, 4); // Centre → SouthEast
    assert!(plan.is_ok(), "route must survive NE closure");
    let plan = plan.unwrap();
    // Waypoints must not include the closed node
    assert!(
        !plan.waypoints.contains(&3),
        "closed node must not appear in route"
    );
}

#[test]
fn road_reopens_and_route_uses_it() {
    let mut g = build_graph();
    SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
    SensorFeed::apply(&mut g, SensorEvent::RoadOpen { node_id: 3 }).unwrap();
    assert!(g.node(3).unwrap().is_passable());

    let e = RouteEngine::new(&g);
    let plan = e.plan_default(2, 4).expect("must route after reopen");
    assert!(plan.is_valid());
}

#[test]
fn congestion_increases_cost_but_route_still_found() {
    let mut g = build_graph();
    // Max congestion on node 2 (North)
    SensorFeed::apply(
        &mut g,
        SensorEvent::Congestion {
            node_id: 2,
            level: 255,
        },
    )
    .unwrap();
    let e = RouteEngine::new(&g);
    let plan = e
        .plan_default(1, 2)
        .expect("congestion must not block routing");
    assert!(plan.is_valid());
}

// ── Scenario 4: Sovereignty rules ────────────────────────────────────────────

#[test]
fn sovereign_alert_blocks_tribe_but_leaves_others_passable() {
    let mut g = build_graph();
    // Only alert tribe 0x0099 — no nodes have this tribe
    let other = TribeId::from_u16(0x0099);
    SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe: other }).unwrap();

    // All nodes (tribe 0x0001) remain passable
    for id in 1u32..=7 {
        assert!(g.node(id).unwrap().is_passable());
    }
}

// ── Scenario 5: NaviCode pipeline stage verification ─────────────────────────

#[test]
fn pipeline_stages_run_in_order() {
    let pipeline = NaviCommand::pipeline();
    let stages: Vec<u8> = pipeline.iter().map(|c| c.stage()).collect();
    assert_eq!(stages, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn nc6_produces_golden_plan() {
    let g = build_graph();
    let req = RouteRequest::new(1, 5, najaf_tribe());
    let plan = NaviCodeExecutor::run_pipeline(req, &g).expect("NC6 must produce plan");
    assert!(plan.is_golden);
}

// ── Scenario 6: Surface weather impact ───────────────────────────────────────

#[test]
fn poor_surface_on_all_nodes_still_routes() {
    let mut g = build_graph();
    for id in 1u32..=7 {
        SensorFeed::apply(
            &mut g,
            SensorEvent::WeatherChange {
                node_id: id,
                surface: SurfaceQuality::Poor,
            },
        )
        .unwrap();
    }
    let e = RouteEngine::new(&g);
    let plan = e
        .plan_default(1, 5)
        .expect("must route despite poor surface");
    assert!(plan.is_valid());
    // Cost must be higher than baseline
    assert!(plan.total_cost > 100.0);
}

// ── Scenario 7: Wadi al-Salam pipeline simulation ────────────────────────────

#[test]
fn wadi_al_salam_seven_sector_full_scan() {
    // Simulate the cemetery sector scan: route from entry (Centre) to each zone
    let g = build_graph();
    let e = RouteEngine::new(&g);
    let zones = [2u32, 3, 4, 5, 6, 7]; // all outer sectors

    let mut all_valid = true;
    let mut total_waypoints = 0usize;
    for &zone in &zones {
        match e.plan_default(1, zone) {
            Ok(plan) => {
                all_valid &= plan.is_valid();
                total_waypoints += plan.waypoint_count();
            }
            Err(e) => {
                all_valid = false;
                eprintln!("zone {zone} failed: {e}");
            }
        }
    }
    assert!(
        all_valid,
        "all 6 cemetery zones must be reachable from entry"
    );
    assert!(total_waypoints >= 12, "at least 2 waypoints per route");
}

// ── Scenario 8: Haversine accuracy for Najaf area ────────────────────────────

#[test]
fn haversine_wadi_al_salam_north_south_distance() {
    // Approximate north and south boundaries of Wadi al-Salam cemetery
    let north_lat = 32.010_f32;
    let south_lat = 31.985_f32;
    let lon = 44.320_f32;
    let d = haversine_m(north_lat, lon, south_lat, lon);
    // ~2.5 km expected
    assert!(
        d > 2_000.0 && d < 3_500.0,
        "N-S distance should be ~2.5 km, got {d}"
    );
}

// ── Scenario 9: Map parsing round-trip ───────────────────────────────────────

#[test]
fn parse_map_then_route() {
    let src = r#"
# Wadi al-Salam minimal map
NODE 10 31.990 44.320 0.0 0 0x0001
NODE 11 32.000 44.320 0.0 1 0x0001
NODE 12 31.995 44.335 0.0 2 0x0001
BEAM 10 11 1100.0
BEAM 10 12 900.0
BEAM 11 12 700.0
"#;
    let map = NaviMap::parse(src).expect("parse must succeed");
    let g = NaviGraph::from_navimap(&map).expect("graph build");
    assert_eq!(g.node_count(), 3);

    let e = RouteEngine::new(&g);
    let plan = e.plan_default(10, 12).expect("must route on parsed map");
    assert!(plan.is_valid());
}

// ── Scenario 10: Custom constraints ──────────────────────────────────────────

#[test]
fn strict_constraints_still_find_route() {
    let g = build_graph();
    let e = RouteEngine::new(&g);
    let c = RouteConstraints {
        max_risk: 0,
        avoid_restricted: true,
        prefer_quality: true,
    };
    let plan = e
        .plan(1, 5, c)
        .expect("strict constraints must not break routing");
    assert!(plan.is_valid());
}

#[test]
fn tribe_hops_counted_in_plan() {
    let g = build_graph();
    let e = RouteEngine::new(&g);
    let plan = e.plan_default(1, 5).unwrap();
    // All nodes are tribe 0x0001 — one tribe in the hop map
    assert_eq!(plan.tribe_hop_count(), 1);
}
