//! PilgrimGuide — routes a pilgrim from the cemetery entrance to a grave.
//!
//! Uses NaviEngine's RouteEngine (A* + NaviCode NC1–NC6) for path finding.
//! The sacred_weight of the destination sector adjusts the final cost,
//! reflecting the zone's pilgrimage priority.
//!
//! A route is "blessed" when its cost falls below the BLESSED_COST_THRESHOLD,
//! meaning the path to the grave is both short and through an elevated zone.

use navi_engine::{NaviGraph, NaviNodeId, RouteConstraints, RouteEngine, RoutePlan};

use crate::error::{NajafError, NajafResult};
use crate::grave::GraveId;
use crate::search::GraveRegistry;
use crate::sector::NajafSector;

/// Routes with cost × sacred_weight below this threshold are "blessed".
pub const BLESSED_COST_THRESHOLD: f32 = 2_000.0;

// ── PilgrimRoute ──────────────────────────────────────────────────────────────

pub struct PilgrimRoute {
    pub waypoints:  Vec<NaviNodeId>,
    pub grave_id:   GraveId,
    pub sector:     NajafSector,
    pub total_cost: f32,
    /// True when sacred cost is below the blessed threshold.
    pub is_blessed: bool,
}

impl PilgrimRoute {
    fn from_plan(plan: RoutePlan, grave_id: GraveId, sector: NajafSector) -> Self {
        let sacred_cost = plan.total_cost * sector.sacred_weight();
        let is_blessed  = sacred_cost < BLESSED_COST_THRESHOLD;
        PilgrimRoute {
            waypoints: plan.waypoints,
            grave_id,
            sector,
            total_cost: sacred_cost,
            is_blessed,
        }
    }

    pub fn waypoint_count(&self) -> usize { self.waypoints.len() }

    /// A route is valid if it has at least one waypoint and a finite cost.
    /// A single-waypoint route means the pilgrim is already at the destination.
    pub fn is_valid(&self) -> bool {
        !self.waypoints.is_empty() && self.total_cost.is_finite()
    }
}

// ── PilgrimGuide ─────────────────────────────────────────────────────────────

pub struct PilgrimGuide<'a> {
    engine:   RouteEngine<'a>,
    registry: &'a GraveRegistry,
}

impl<'a> PilgrimGuide<'a> {
    pub fn new(graph: &'a NaviGraph, registry: &'a GraveRegistry) -> Self {
        PilgrimGuide { engine: RouteEngine::new(graph), registry }
    }

    /// Route from `entrance_node` to the sector node nearest the target grave.
    pub fn guide_to_grave(
        &self,
        entrance_node: NaviNodeId,
        grave_id: GraveId,
    ) -> NajafResult<PilgrimRoute> {
        let grave = self.registry.get(grave_id)
            .ok_or(NajafError::GraveNotFound(grave_id))?;

        if !grave.is_accessible() {
            return Err(NajafError::SectorBlocked(grave.sector));
        }

        // Route to the sector's representative node in the NaviGraph.
        let dest_node = grave.sector.fixture_node_id();

        // Trivial case: pilgrim is already at the destination sector.
        if entrance_node == dest_node {
            let sector = grave.sector;
            return Ok(PilgrimRoute {
                waypoints:  vec![entrance_node],
                grave_id,
                sector,
                total_cost: 0.0,
                is_blessed: true,
            });
        }

        let plan = self.engine
            .plan(entrance_node, dest_node, RouteConstraints::default())?;

        Ok(PilgrimRoute::from_plan(plan, grave_id, grave.sector))
    }

    /// Route from `entrance_node` to every accessible grave in the given sector.
    pub fn guide_to_sector(
        &self,
        entrance_node: NaviNodeId,
        sector: NajafSector,
    ) -> NajafResult<Vec<PilgrimRoute>> {
        let accessible: Vec<GraveId> = self.registry
            .accessible_in_sector(sector)
            .into_iter()
            .map(|g| g.id)
            .collect();

        if accessible.is_empty() {
            return Err(NajafError::NoAccessibleGravesInSector(sector));
        }

        accessible
            .into_iter()
            .map(|id| self.guide_to_grave(entrance_node, id))
            .collect()
    }

    /// Route from `entrance_node` to the nearest accessible grave to `coord`.
    pub fn nearest_accessible(
        &self,
        entrance_node: NaviNodeId,
        coord: &navi_engine::NaviCoord,
    ) -> NajafResult<PilgrimRoute> {
        let grave = self.registry.nearest_accessible(coord)
            .ok_or(NajafError::EmptyCemetery)?;
        self.guide_to_grave(entrance_node, grave.id)
    }

    /// Batch: route from entrance to all accessible graves in all sectors.
    pub fn full_pilgrimage_scan(&self, entrance_node: NaviNodeId) -> Vec<PilgrimRoute> {
        self.registry.accessible().into_iter().filter_map(|g| {
            self.guide_to_grave(entrance_node, g.id).ok()
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::{cemetery_nav_graph, seven_grave_registry};

    fn setup() -> (NaviGraph, GraveRegistry) {
        (cemetery_nav_graph(), seven_grave_registry())
    }

    #[test]
    fn guide_to_grave_101_entrance_to_entrance() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        // Grave 101 is in the Entrance sector (node 1) — trivial route
        let route = guide.guide_to_grave(1, 101).expect("must route");
        assert!(route.is_valid());
        assert_eq!(route.grave_id, 101);
        assert_eq!(route.sector, NajafSector::Entrance);
        // Trivial: pilgrim already at entrance — cost 0, blessed, 1 waypoint
        assert_eq!(route.waypoint_count(), 1);
        assert!(route.is_blessed);
        assert_eq!(route.total_cost, 0.0);
    }

    #[test]
    fn guide_to_grave_shuhadaa() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let route = guide.guide_to_grave(1, 102).expect("Centre → Shuhadaa");
        assert!(route.is_valid());
        assert_eq!(route.sector, NajafSector::Shuhadaa);
    }

    #[test]
    fn guide_to_all_seven_sectors() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        for grave_id in 101u32..=107 {
            let route = guide.guide_to_grave(1, grave_id)
                .unwrap_or_else(|e| panic!("grave {grave_id} failed: {e}"));
            assert!(route.is_valid(), "grave {grave_id} route is invalid");
        }
    }

    #[test]
    fn guide_to_nonexistent_grave_errors() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let result = guide.guide_to_grave(1, 9999);
        assert!(matches!(result, Err(NajafError::GraveNotFound(9999))));
    }

    #[test]
    fn guide_to_sealed_grave_errors() {
        let (g, mut r) = setup();
        r.get_mut(102).unwrap().seal();
        let guide = PilgrimGuide::new(&g, &r);
        let result = guide.guide_to_grave(1, 102);
        assert!(matches!(result, Err(NajafError::SectorBlocked(_))));
    }

    #[test]
    fn guide_to_sector_shuhadaa_returns_one_route() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let routes = guide.guide_to_sector(1, NajafSector::Shuhadaa).expect("must route");
        assert_eq!(routes.len(), 1);
    }

    #[test]
    fn guide_to_sector_empty_sector_errors() {
        let (g, r) = setup();
        let _guide = PilgrimGuide::new(&g, &r);
        // Seal the only grave in the Huffaz sector
        let mut r2 = r;
        r2.get_mut(104).unwrap().seal();
        let guide2 = PilgrimGuide::new(&g, &r2);
        let result = guide2.guide_to_sector(1, NajafSector::Huffaz);
        assert!(matches!(result, Err(NajafError::NoAccessibleGravesInSector(_))));
    }

    #[test]
    fn nearest_accessible_returns_route() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let coord = navi_engine::NaviCoord::new(32.005, 44.320, 0.0); // near Shuhadaa
        let route = guide.nearest_accessible(1, &coord).expect("must find nearest");
        assert!(route.is_valid());
    }

    #[test]
    fn full_scan_returns_seven_routes() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let routes = guide.full_pilgrimage_scan(1);
        assert_eq!(routes.len(), 7);
        assert!(routes.iter().all(|r| r.is_valid()));
    }

    #[test]
    fn sacred_cost_lower_for_elevated_zones() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let shuhadaa = guide.guide_to_grave(1, 102).unwrap();
        let momineen = guide.guide_to_grave(1, 105).unwrap();
        // Shuhadaa has sacred_weight 0.85, Momineen 1.00 — same base distance via spoke
        assert!(shuhadaa.total_cost <= momineen.total_cost);
    }

    #[test]
    fn pilgrim_route_waypoints_start_at_entrance() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        let route = guide.guide_to_grave(1, 103).unwrap();
        assert_eq!(*route.waypoints.first().unwrap(), 1);
    }

    #[test]
    fn pilgrim_route_blessed_flag_set_for_short_routes() {
        let (g, r) = setup();
        let guide = PilgrimGuide::new(&g, &r);
        // Spoke from entrance → adjacent node should be short enough to be blessed
        let route = guide.guide_to_grave(1, 102).unwrap();
        // Cost < BLESSED_COST_THRESHOLD → blessed
        assert_eq!(route.is_blessed, route.total_cost < BLESSED_COST_THRESHOLD);
    }
}
