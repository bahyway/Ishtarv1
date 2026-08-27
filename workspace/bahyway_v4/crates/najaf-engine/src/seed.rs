//! Seed data — canonical 7-grave fixture for Wadi al-Salam tests.
//!
//! One grave per sector, tribe 0x0001, spanning Hijri years 1400–1460.
//! Coordinates approximate the real cemetery area (Najaf, Iraq).
//! Grave IDs 101–107 (avoids overlap with NaviEngine node IDs 1–7).

use bahyway_core::TribeId;
use navi_engine::{seven_node_map, NaviCoord, NaviGraph, NaviMap};

use crate::grave::GraveParticle;
use crate::search::GraveRegistry;
use crate::sector::NajafSector;

/// Build the canonical 7-grave cemetery registry fixture.
/// Each grave occupies one NajafSector, tribe = 0x0001.
pub fn seven_grave_registry() -> GraveRegistry {
    let tribe = TribeId::from_u16(0x0001);
    let mut reg = GraveRegistry::new();

    // Approximate bounding box of Wadi al-Salam: 31.980–32.010°N, 44.310–44.340°E
    reg.register(GraveParticle::new(
        101,
        NaviCoord::new(31.995, 44.320, 0.0),
        NajafSector::Entrance,
        tribe,
        1400,
    ));
    reg.register(GraveParticle::new(
        102,
        NaviCoord::new(32.005, 44.320, 0.0),
        NajafSector::Shuhadaa,
        tribe,
        1410,
    ));
    reg.register(GraveParticle::new(
        103,
        NaviCoord::new(32.003, 44.333, 0.0),
        NajafSector::Awliya,
        tribe,
        1420,
    ));
    reg.register(GraveParticle::new(
        104,
        NaviCoord::new(31.988, 44.333, 0.0),
        NajafSector::Huffaz,
        tribe,
        1430,
    ));
    reg.register(GraveParticle::new(
        105,
        NaviCoord::new(31.983, 44.320, 0.0),
        NajafSector::Momineen,
        tribe,
        1440,
    ));
    reg.register(GraveParticle::new(
        106,
        NaviCoord::new(31.988, 44.307, 0.0),
        NajafSector::Ulamaa,
        tribe,
        1450,
    ));
    reg.register(GraveParticle::new(
        107,
        NaviCoord::new(32.003, 44.307, 0.0),
        NajafSector::Anbiya,
        tribe,
        1460,
    ));

    reg
}

/// Build the NaviGraph used for pilgrimage routing in seed scenarios.
/// Re-uses the NaviEngine seven_node_map fixture.
pub fn cemetery_nav_graph() -> NaviGraph {
    NaviGraph::from_navimap(&seven_node_map()).expect("cemetery_nav_graph: fixture must build")
}

/// Build the NaviMap underlying the cemetery graph (for inspection / re-parsing).
pub fn cemetery_nav_map() -> NaviMap {
    seven_node_map()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_graves_registered() {
        let r = seven_grave_registry();
        assert_eq!(r.count(), 7);
    }

    #[test]
    fn each_sector_has_exactly_one_grave() {
        let r = seven_grave_registry();
        for &s in NajafSector::all().iter() {
            assert_eq!(
                r.by_sector(s).len(),
                1,
                "sector {s:?} must have exactly 1 grave"
            );
        }
    }

    #[test]
    fn all_graves_occupied_and_accessible() {
        let r = seven_grave_registry();
        assert_eq!(r.accessible().len(), 7);
    }

    #[test]
    fn all_graves_same_tribe() {
        let r = seven_grave_registry();
        assert_eq!(r.by_tribe(TribeId::from_u16(0x0001)).len(), 7);
        assert_eq!(r.by_tribe(TribeId::from_u16(0x0002)).len(), 0);
    }

    #[test]
    fn epoch_span_1400_to_1460() {
        let r = seven_grave_registry();
        let in_range = r.by_epoch_range(1400, 1460);
        assert_eq!(in_range.len(), 7);
        let out_range = r.by_epoch_range(1461, 1500);
        assert_eq!(out_range.len(), 0);
    }

    #[test]
    fn cemetery_nav_graph_has_seven_nodes() {
        let g = cemetery_nav_graph();
        assert_eq!(g.node_count(), 7);
    }

    #[test]
    fn cemetery_nav_graph_has_24_edges() {
        let g = cemetery_nav_graph();
        // 6 bidirectional spokes (12) + 6 bidirectional rim (12) = 24
        assert_eq!(g.edge_count(), 24);
    }
}
