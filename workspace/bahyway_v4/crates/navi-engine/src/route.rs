//! RouteEngine — quality-weighted A* over NaviGraph.
//!
//! Wraps the NaviCode pipeline with a haversine-based spatial heuristic.
//! The A* heuristic guides exploration toward the destination using real
//! geographic distance — the NaviCode Dijkstra inside NC6 does the final
//! shortest-path resolution after costs are refined by NC1–NC5.

use crate::error::{NaviError, NaviResult};
use crate::graph::NaviGraph;
use crate::navicode::{NaviCodeExecutor, RouteConstraints, RoutePlan, RouteRequest};
use crate::particle::NaviNodeId;
use bahyway_core::TribeId;

// ── Haversine distance ────────────────────────────────────────────────────────

/// Great-circle distance in metres using the haversine formula.
pub fn haversine_m(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    const R: f32 = 6_371_000.0; // Earth radius in metres
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

// ── RouteMetrics ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RouteMetrics {
    pub waypoint_count: usize,
    pub total_cost: f32,
    pub tribe_hop_count: usize,
    pub algorithm: &'static str,
}

impl RouteMetrics {
    pub fn from_plan(plan: &RoutePlan) -> Self {
        RouteMetrics {
            waypoint_count: plan.waypoint_count(),
            total_cost: plan.total_cost,
            tribe_hop_count: plan.tribe_hop_count(),
            algorithm: "NaviCode-A*",
        }
    }
}

// ── RouteEngine ───────────────────────────────────────────────────────────────

pub struct RouteEngine<'a> {
    graph: &'a NaviGraph,
}

impl<'a> RouteEngine<'a> {
    pub fn new(graph: &'a NaviGraph) -> Self {
        RouteEngine { graph }
    }

    /// Plan a route from `origin` to `destination`.
    /// Runs the full NC1–NC6 NaviCode pipeline with the supplied constraints.
    pub fn plan(
        &self,
        origin: NaviNodeId,
        destination: NaviNodeId,
        constraints: RouteConstraints,
    ) -> NaviResult<RoutePlan> {
        if self.graph.node_count() == 0 {
            return Err(NaviError::EmptyGraph);
        }
        let dest_tribe = self
            .graph
            .node(destination)
            .map(|n| n.particle.tribe)
            .unwrap_or_else(|| TribeId::from_u16(0x0001));

        let req = RouteRequest {
            origin,
            destination,
            constraints,
            dest_tribe,
        };
        NaviCodeExecutor::run_pipeline(req, self.graph)
    }

    /// Plan with default constraints.
    pub fn plan_default(
        &self,
        origin: NaviNodeId,
        destination: NaviNodeId,
    ) -> NaviResult<RoutePlan> {
        self.plan(origin, destination, RouteConstraints::default())
    }

    /// Returns RouteMetrics for a completed plan.
    pub fn metrics(plan: &RoutePlan) -> RouteMetrics {
        RouteMetrics::from_plan(plan)
    }

    /// Heuristic: straight-line haversine distance from `node` to `destination`.
    pub fn heuristic(&self, node_id: NaviNodeId, dest_id: NaviNodeId) -> f32 {
        let n = self.graph.node(node_id);
        let d = self.graph.node(dest_id);
        match (n, d) {
            (Some(n), Some(d)) => haversine_m(
                n.particle.coord.lat,
                n.particle.coord.lon,
                d.particle.coord.lat,
                d.particle.coord.lon,
            ),
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::NaviGraph;
    use crate::navimap::seven_node_map;

    fn engine() -> NaviGraph {
        NaviGraph::from_navimap(&seven_node_map()).unwrap()
    }

    #[test]
    fn plan_centre_to_north_succeeds() {
        let g = engine();
        let e = RouteEngine::new(&g);
        let plan = e.plan_default(1, 2).expect("route must succeed");
        assert!(plan.is_valid());
    }

    #[test]
    fn plan_same_node_succeeds_trivially() {
        let g = engine();
        let e = RouteEngine::new(&g);
        // origin == destination: path length 1, cost 0
        let plan = e.plan_default(1, 1).expect("trivial route");
        assert!(plan.waypoints.contains(&1));
    }

    #[test]
    fn plan_nonexistent_destination_fails() {
        let g = engine();
        let e = RouteEngine::new(&g);
        assert!(e.plan_default(1, 99).is_err());
    }

    #[test]
    fn metrics_waypoint_count_matches_plan() {
        let g = engine();
        let e = RouteEngine::new(&g);
        let plan = e.plan_default(1, 5).unwrap();
        let m = RouteEngine::metrics(&plan);
        assert_eq!(m.waypoint_count, plan.waypoint_count());
        assert_eq!(m.algorithm, "NaviCode-A*");
    }

    #[test]
    fn metrics_total_cost_positive() {
        let g = engine();
        let e = RouteEngine::new(&g);
        let plan = e.plan_default(1, 5).unwrap();
        let m = RouteEngine::metrics(&plan);
        assert!(m.total_cost > 0.0);
    }

    #[test]
    fn heuristic_same_node_is_zero() {
        let g = engine();
        let e = RouteEngine::new(&g);
        assert!(e.heuristic(1, 1) < 0.01);
    }

    #[test]
    fn heuristic_different_nodes_positive() {
        let g = engine();
        let e = RouteEngine::new(&g);
        assert!(e.heuristic(1, 5) > 0.0);
    }

    #[test]
    fn heuristic_nonexistent_node_is_zero() {
        let g = engine();
        let e = RouteEngine::new(&g);
        assert!(e.heuristic(1, 99) < 0.01);
    }

    #[test]
    fn haversine_same_point_is_zero() {
        let d = haversine_m(31.99, 44.32, 31.99, 44.32);
        assert!(d < 0.01);
    }

    #[test]
    fn haversine_one_degree_lat_approx_111km() {
        let d = haversine_m(31.0, 44.0, 32.0, 44.0);
        assert!(d > 100_000.0 && d < 120_000.0);
    }
}
