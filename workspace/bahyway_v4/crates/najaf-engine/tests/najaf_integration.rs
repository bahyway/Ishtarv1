//! NajafEngine integration tests — end-to-end pilgrimage scenarios.
//!
//! All scenarios use the 7-grave, 7-sector cemetery fixture.
//! Tests cover: full pilgrimage scan, sensor disruption, sovereignty, KAKI
//! identity, epoch-based filtering, and Wadi al-Salam geographic accuracy.

use najaf_engine::{
    cemetery_nav_graph, seven_grave_registry,
    GraveRegistry, NajafSector, PilgrimGuide,
    SensorEvent, SensorFeed,
};
use bahyway_core::TribeId;
use navi_engine::haversine_m;

fn guide_setup() -> (navi_engine::NaviGraph, GraveRegistry) {
    (cemetery_nav_graph(), seven_grave_registry())
}

// ── Scenario 1: Full pilgrimage scan ─────────────────────────────────────────

#[test]
fn all_seven_graves_reachable_from_entrance() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);
    let routes = guide.full_pilgrimage_scan(1);
    assert_eq!(routes.len(), 7, "all 7 graves must be reachable");
    assert!(routes.iter().all(|r| r.is_valid()), "all routes must be valid");
}

// ── Scenario 2: Sector-by-sector routing ─────────────────────────────────────

#[test]
fn each_sector_routes_individually() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);
    for &sector in NajafSector::all().iter() {
        let routes = guide.guide_to_sector(1, sector)
            .unwrap_or_else(|e| panic!("sector {sector:?} failed: {e}"));
        assert!(!routes.is_empty(), "sector {sector:?} must produce at least one route");
        assert!(routes.iter().all(|r| r.is_valid()));
    }
}

// ── Scenario 3: Sacred cost ordering ─────────────────────────────────────────

#[test]
fn elevated_zones_have_lower_sacred_cost() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);

    // Martyrs (0.85) vs Believers (1.00) — both one spoke from entrance
    let shuhadaa = guide.guide_to_grave(1, 102).unwrap();
    let momineen = guide.guide_to_grave(1, 105).unwrap();
    assert!(
        shuhadaa.total_cost <= momineen.total_cost,
        "Shuhadaa sacred cost {:.1} must be ≤ Momineen {:.1}",
        shuhadaa.total_cost, momineen.total_cost
    );
}

// ── Scenario 4: Sensor disruption ────────────────────────────────────────────

#[test]
fn road_closure_on_spoke_node_forces_alternate() {
    let (mut g, r) = guide_setup();
    // Block node 3 = NorthEast = Awliya spoke
    SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
    let guide = PilgrimGuide::new(&g, &r);
    // Other sectors must still route
    let route = guide.guide_to_grave(1, 102).expect("Shuhadaa unaffected");
    assert!(route.is_valid());
    assert!(!route.waypoints.contains(&3), "closed node 3 must not appear");
}

#[test]
fn road_closure_then_reopen_restores_routing() {
    let (mut g, r) = guide_setup();
    SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 2 }).unwrap();
    SensorFeed::apply(&mut g, SensorEvent::RoadOpen    { node_id: 2 }).unwrap();
    assert!(g.node(2).unwrap().is_passable());
    let guide = PilgrimGuide::new(&g, &r);
    let route = guide.guide_to_grave(1, 102).expect("must route after reopen");
    assert!(route.is_valid());
}

// ── Scenario 5: Grave state transitions ──────────────────────────────────────

#[test]
fn sealed_grave_cannot_be_guided_to() {
    let (g, mut r) = guide_setup();
    r.get_mut(105).unwrap().seal();
    let guide = PilgrimGuide::new(&g, &r);
    let result = guide.guide_to_grave(1, 105);
    assert!(result.is_err(), "sealed grave must fail");
}

#[test]
fn reserved_grave_is_guidable() {
    let (g, mut r) = guide_setup();
    r.get_mut(106).unwrap().reserve();
    let guide = PilgrimGuide::new(&g, &r);
    let route = guide.guide_to_grave(1, 106).expect("reserved grave must route");
    assert!(route.is_valid());
}

// ── Scenario 6: Nearest accessible routing ───────────────────────────────────

#[test]
fn nearest_accessible_finds_route_from_coord() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);
    let coord  = navi_engine::NaviCoord::new(32.003, 44.333, 0.0); // near Awliya
    let route  = guide.nearest_accessible(1, &coord).expect("must find nearest");
    assert!(route.is_valid());
}

#[test]
fn nearest_accessible_skips_sealed_graves() {
    let (g, mut r) = guide_setup();
    // Seal the Awliya grave (103) nearest to this coordinate
    r.get_mut(103).unwrap().seal();
    let guide = PilgrimGuide::new(&g, &r);
    let coord = navi_engine::NaviCoord::new(32.003, 44.333, 0.0);
    let route = guide.nearest_accessible(1, &coord).expect("must skip sealed and find next");
    assert!(route.is_valid());
    assert_ne!(route.grave_id, 103, "sealed grave must not be returned");
}

// ── Scenario 7: KAKI identity integrity ──────────────────────────────────────

#[test]
fn all_graves_have_valid_kaki_checksums() {
    let r = seven_grave_registry();
    for id in 101u32..=107 {
        let g = r.get(id).unwrap();
        assert!(g.kaki.verify_checksum(), "grave {id} KAKI checksum invalid");
    }
}

#[test]
fn all_graves_kaki_tribe_matches_registry_tribe() {
    let r = seven_grave_registry();
    let tribe = TribeId::from_u16(0x0001);
    for id in 101u32..=107 {
        let g = r.get(id).unwrap();
        assert_eq!(g.kaki.tribe_id(), tribe, "grave {id} KAKI tribe mismatch");
    }
}

// ── Scenario 8: Epoch-based search ───────────────────────────────────────────

#[test]
fn epoch_filter_finds_early_interments() {
    let r = seven_grave_registry();
    let early = r.by_epoch_range(1400, 1420);
    assert_eq!(early.len(), 3); // 1400, 1410, 1420
}

#[test]
fn epoch_filter_finds_late_interments() {
    let r = seven_grave_registry();
    let late = r.by_epoch_range(1440, 1460);
    assert_eq!(late.len(), 3); // 1440, 1450, 1460
}

// ── Scenario 9: Geographic accuracy ──────────────────────────────────────────

#[test]
fn haversine_entrance_to_shuhadaa_distance() {
    // Entrance: 31.995°N, 44.320°E
    // Shuhadaa: 32.005°N, 44.320°E
    let d = haversine_m(31.995, 44.320, 32.005, 44.320);
    // ~1.1 km north-south
    assert!(d > 900.0 && d < 1_300.0, "N-S distance should be ~1.1km, got {d}");
}

#[test]
fn haversine_entrance_to_anbiya_distance() {
    // Entrance: 31.995°N, 44.320°E
    // Anbiya:   32.003°N, 44.307°E
    let d = haversine_m(31.995, 44.320, 32.003, 44.307);
    // NW diagonal — ~1.3 km
    assert!(d > 800.0 && d < 1_800.0, "diagonal distance should be ~1.3km, got {d}");
}

// ── Scenario 10: Sovereignty — tribe alert ───────────────────────────────────

#[test]
fn sovereign_alert_on_own_tribe_blocks_all_nodes() {
    let (mut g, r) = guide_setup();
    let tribe = TribeId::from_u16(0x0001);
    SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe }).unwrap();
    // All NaviGraph nodes are now restricted
    for id in 1u32..=7 {
        assert!(!g.node(id).unwrap().is_passable(), "node {id} must be restricted");
    }
    let guide  = PilgrimGuide::new(&g, &r);
    // Routing to any grave must fail (no passable nodes)
    let result = guide.guide_to_grave(1, 102);
    assert!(result.is_err(), "sovereign alert must block routing");
}

#[test]
fn sovereign_alert_on_other_tribe_leaves_routing_intact() {
    let (mut g, r) = guide_setup();
    let other = TribeId::from_u16(0x0099);
    SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe: other }).unwrap();
    // All nodes remain passable (they are tribe 0x0001, not 0x0099)
    let guide  = PilgrimGuide::new(&g, &r);
    let routes = guide.full_pilgrimage_scan(1);
    assert_eq!(routes.len(), 7);
}

// ── Scenario 11: Waypoint integrity ──────────────────────────────────────────

#[test]
fn every_route_starts_at_entrance_node() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);
    for grave_id in 101u32..=107 {
        let route = guide.guide_to_grave(1, grave_id).unwrap();
        assert_eq!(*route.waypoints.first().unwrap(), 1,
            "grave {grave_id} route must start at entrance node 1");
    }
}

#[test]
fn every_route_ends_at_sector_node() {
    let (g, r) = guide_setup();
    let guide  = PilgrimGuide::new(&g, &r);
    for grave_id in 101u32..=107 {
        let route = guide.guide_to_grave(1, grave_id).unwrap();
        let grave  = r.get(grave_id).unwrap();
        let expected_dest = grave.sector.fixture_node_id();
        // For the Entrance grave (trivial route, 1 waypoint), first == last.
        assert_eq!(*route.waypoints.last().unwrap(), expected_dest,
            "grave {grave_id} route must end at sector node {expected_dest}");
    }
}

// ── Scenario 12: GraveState::Available has no guide path ─────────────────────

#[test]
fn available_grave_is_not_a_pilgrimage_destination() {
    let (g, mut r) = guide_setup();
    r.get_mut(107).unwrap().release(); // Available
    let guide = PilgrimGuide::new(&g, &r);
    let result = guide.guide_to_grave(1, 107);
    assert!(result.is_err(), "Available grave is not a pilgrimage destination");
}

// ── Scenario 13: Full scan after partial sealing ─────────────────────────────

#[test]
fn partial_sealing_reduces_scan_count() {
    let (g, mut r) = guide_setup();
    r.get_mut(103).unwrap().seal();
    r.get_mut(106).unwrap().seal();
    let guide  = PilgrimGuide::new(&g, &r);
    let routes = guide.full_pilgrimage_scan(1);
    assert_eq!(routes.len(), 5, "only 5 accessible graves remain");
}
