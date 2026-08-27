//! NaviEngine v4.0 — Sovereign routing engine.
//!
//! NaviCode (NC1–NC6) pipeline → quality-weighted A* → haversine heuristic.
//!
//! Architecture:
//!   navimap    — NaviMap: sovereign .navimap text format (NODE / BEAM)
//!   particle   — NaviParticle: 7-dimensional spatial position
//!   graph      — NaviGraph: heptagram nodes + edges + O(1) adjacency index
//!   navicode   — NC1–NC6 pipeline: EdgeCostMatrix, TribeCluster, Dijkstra
//!   route      — RouteEngine: A* wrapper with haversine heuristic
//!   sensor     — SensorFeed: real-time local event updates
//!   error      — NaviError / NaviResult
//!
//! Sovereign map layer (no third-party GIS):
//!   eav         — EavStore: mandatory/optional typed attribute store
//!   mapparticle — MapParticle: KAKI-identity map atom (Road/Pipeline/POI/…)
//!   tile        — HubbleTile: Z0–Z6 zoom system (7 levels, 10°→0.001°)
//!   particlemap — ParticleMap: SpatialGrid + KAKI index + adjacency
//!   machrouter  — MachineRouter: KAKI-native A* over ParticleMap
//!   bookmark    — BookmarkStore: sovereign named locations
//!   pipeline    — WpdPipelineMap: pipeline domain layer (WPD / Najaf)

pub mod error;
pub mod graph;
pub mod navicode;
pub mod navimap;
pub mod particle;
pub mod route;
pub mod sensor;

// ── Sovereign map layer ───────────────────────────────────────────────────────

pub mod bookmark;
pub mod eav;
pub mod machrouter;
pub mod mapparticle;
pub mod particlemap;
pub mod pipeline;
pub mod tile;

// ── Original re-exports ───────────────────────────────────────────────────────

pub use error::{NaviError, NaviResult};
pub use graph::{HeptaChordType, HeptaSector, NaviEdge, NaviGraph, NaviNode};
pub use navicode::{
    EdgeCostMatrix, ExecutionContext, NaviCodeExecutor, NaviCommand, RouteConstraints, RoutePlan,
    RouteRequest, TribeClusterMap,
};
pub use navimap::{seven_node_map, MapBeam, MapNode, NaviMap};
pub use particle::{
    NaviCoord, NaviNodeId, NaviParticle, NaviParticleState, NaviSignal, SurfaceQuality,
};
pub use route::{haversine_m, RouteEngine, RouteMetrics};
pub use sensor::{SensorEvent, SensorFeed};

// ── Sovereign map re-exports ──────────────────────────────────────────────────

pub use bookmark::{Bookmark, BookmarkCategory, BookmarkStore};
pub use eav::{
    AttrKey, AttrValue, EavAttr, EavStore, ATTR_ACCESS_LEVEL, ATTR_AGE_YEARS, ATTR_DIAMETER_MM,
    ATTR_ELEVATION_M, ATTR_FLOW_DIR, ATTR_LANES, ATTR_NAME, ATTR_NAME_ARABIC, ATTR_ONE_WAY,
    ATTR_PIPE_MATERIAL, ATTR_PRESSURE_KPA, ATTR_ROAD_CLASS, ATTR_SACRED_WEIGHT, ATTR_SPEED_LIMIT,
};
pub use machrouter::{MachRoute, MachRouteSegment, MachineRouter, RoutingMode};
pub use mapparticle::{FlowDir, MapKind, MapParticle, PipelineKind, PoiCategory, RoadClass};
pub use particlemap::{MapBounds, MapEdge, ParticleMap};
pub use pipeline::{
    LeakCandidate, LeakDetectionSource, PipeMaterial, PipelineSegmentData, WpdPipelineMap,
    QUALITY_DIVISOR as PIPELINE_QUALITY_DIVISOR,
};
pub use tile::{
    tiles_in_bbox, zoom_for_radius_m, HubbleTile, HubbleTileId, TileBounds, HUBBLE_ZOOM_MAX,
};

// ── Crate constants ───────────────────────────────────────────────────────────

pub const NAVI_ENGINE_VERSION: &str = "4.0.0";
pub const NAVI_SECTORS: usize = 7;
pub const NAVICODE_STAGES: usize = 6;
pub const MAX_ROUTE_WAYPOINTS: usize = 512;
pub const GOLDEN_PATH_SYMBOL: &str = "✦";

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;

    #[test]
    fn version_is_4_0_0() {
        assert_eq!(NAVI_ENGINE_VERSION, "4.0.0");
    }
    #[test]
    fn sectors_is_7() {
        assert_eq!(NAVI_SECTORS, 7);
    }
    #[test]
    fn navicode_stages_is_6() {
        assert_eq!(NAVICODE_STAGES, 6);
    }

    #[test]
    fn full_pipeline_centre_to_south() {
        let map = seven_node_map();
        let g = NaviGraph::from_navimap(&map).expect("graph build");
        let e = RouteEngine::new(&g);
        let plan = e.plan_default(1, 5).expect("route to South");
        assert!(plan.is_valid());
        assert_eq!(*plan.waypoints.first().unwrap(), 1);
        assert_eq!(*plan.waypoints.last().unwrap(), 5);
    }

    #[test]
    fn sensor_disrupts_route_and_rereroutes() {
        let map = seven_node_map();
        let mut g = NaviGraph::from_navimap(&map).expect("graph build");

        // Block node 5 (South) — direct spoke from Centre will be tombstoned
        SensorFeed::apply(&mut g, SensorEvent::RoadClosure { node_id: 5 }).unwrap();

        // Route still succeeds via rim path through other sectors
        let e = RouteEngine::new(&g);
        let plan = e.plan(1, 4, RouteConstraints::default()); // Centre → SouthEast
        assert!(plan.is_ok(), "must find alternative when node 5 is dead");
    }

    #[test]
    fn sovereign_alert_blocks_entire_tribe() {
        let map = seven_node_map();
        let mut g = NaviGraph::from_navimap(&map).expect("graph build");

        // All nodes are tribe 0x0001 — alert blocks everything
        let tribe = TribeId::from_u16(0x0001);
        SensorFeed::apply(&mut g, SensorEvent::SovereignAlert { tribe }).unwrap();

        // All nodes restricted — even the origin is not passable
        for id in 1u32..=7 {
            assert!(!g.node(id).unwrap().is_passable());
        }
    }
}
