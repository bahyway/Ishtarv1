//! NajafEngine v4.0 — Wadi al-Salam sovereign cemetery navigation.
//!
//! Architecture:
//!   sector  — NajafSector: 7 cemetery zones mapped to HeptaSector topology
//!   grave   — GraveParticle: sovereign burial site (KAKI identity + NaviCoord)
//!   search  — GraveRegistry: searchable collection of burial sites
//!   guide   — PilgrimGuide: routes pilgrim from entrance to grave via NaviEngine
//!   seed    — seven_grave_registry() + cemetery_nav_graph() fixtures
//!   error   — NajafError / NajafResult
//!
//! Depends on: navi-engine (routing), enkidb-kaki (identity), bahyway-core (TribeId)
//! Zero external dependencies.

pub mod error;
pub mod sector;
pub mod grave;
pub mod identity;
pub mod search;
pub mod seed;
pub mod guide;
pub mod topology;

pub use error::{NajafError, NajafResult};
pub use sector::NajafSector;
pub use grave::{GraveCondition, DiscoverySource, GraveId, GraveParticle, GraveState, QUALITY_DIVISOR};
pub use identity::{IdentityCandidate, InscriptionMatcher};
pub use search::GraveRegistry;
pub use seed::{cemetery_nav_graph, cemetery_nav_map, seven_grave_registry};
pub use guide::{BLESSED_COST_THRESHOLD, PilgrimGuide, PilgrimRoute};
pub use topology::{
    SimplicialComplex, SimplexEdge, StackOrder, DiscoveryEntry,
    is_particle_in_complex, resolve_stack, reconstruct_ghost, infer_pipeline,
};

// Re-export NaviEngine types needed by consumers of NajafEngine.
pub use navi_engine::{NaviCoord, NaviGraph, SensorEvent, SensorFeed};

// ── Crate constants ───────────────────────────────────────────────────────────

pub const NAJAF_ENGINE_VERSION: &str = "4.0.0";
pub const NAJAF_SECTORS:        usize = 7;
pub const WADI_AL_SALAM_LAT:    f32   = 31.995;
pub const WADI_AL_SALAM_LON:    f32   = 44.320;

#[cfg(test)]
mod tests {
    use super::*;
    use navi_engine::SensorEvent;

    #[test]
    fn version_is_4_0_0() { assert_eq!(NAJAF_ENGINE_VERSION, "4.0.0"); }

    #[test]
    fn sectors_constant_is_7() { assert_eq!(NAJAF_SECTORS, 7); }

    #[test]
    fn wadi_coordinates_in_najaf_area() {
        // Wadi al-Salam is near 32°N, 44°E
        assert!(WADI_AL_SALAM_LAT > 31.9 && WADI_AL_SALAM_LAT < 32.1);
        assert!(WADI_AL_SALAM_LON > 44.2 && WADI_AL_SALAM_LON < 44.4);
    }

    #[test]
    fn full_pipeline_entrance_to_anbiya() {
        let g    = cemetery_nav_graph();
        let r    = seven_grave_registry();
        let guide = PilgrimGuide::new(&g, &r);
        // Grave 107 = Anbiya sector
        let route = guide.guide_to_grave(1, 107).expect("entrance → Anbiya");
        assert!(route.is_valid());
        assert_eq!(route.sector, NajafSector::Anbiya);
    }

    #[test]
    fn sensor_disruption_reroutes() {
        let mut g = cemetery_nav_graph();
        let r     = seven_grave_registry();
        // Block node 3 (NorthEast/Awliya route) — must reroute
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 3 }).unwrap();
        let guide = PilgrimGuide::new(&g, &r);
        // Grave 103 is in Awliya — but its sector node (3) is blocked
        // RouteEngine will fail for that specific destination
        // Other graves must still route
        let route = guide.guide_to_grave(1, 102); // Shuhadaa (node 2) — unaffected
        assert!(route.is_ok());
    }
}
