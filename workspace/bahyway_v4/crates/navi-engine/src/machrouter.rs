//! MachineRouter — KAKI-native A* routing over `ParticleMap`.
//!
//! Routes between any two particles on the sovereign map.  Unlike `RouteEngine`
//! (which routes between NaviGraph node IDs), `MachineRouter` operates on
//! particle KAKI hashes — every route is a sovereign chain of KAKI identities.
//!
//! ## Routing modes
//!
//! | Mode        | Traverses         | Cost factor                        |
//! |-------------|-------------------|------------------------------------|
//! | Pedestrian  | Roads + footways  | Base distance × road class mult    |
//! | Vehicle     | Roads (no footway)| Base distance × road class mult    |
//! | Pilgrimage  | Roads + grave paths| Sacred weight preferred            |
//! | Pipeline    | Pipeline segments | Flow direction awareness            |
//! | Emergency   | Any passable      | Distance only (no multipliers)     |
//!
//! ## A* priority queue
//!
//! `f32` costs are scaled ×10,000 and stored as `u64` in a `BinaryHeap`
//! (min-heap via `Reverse`).  No floating-point comparison in the heap.

#![forbid(unsafe_code)]

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::mapparticle::{FlowDir, MapKind, MapParticle, PipelineKind};
use crate::particle::NaviCoord;
use crate::particlemap::ParticleMap;
use crate::route::haversine_m;

// ── RoutingMode ───────────────────────────────────────────────────────────────

/// Routing strategy — controls which particle kinds are traversable and
/// how edge costs are weighted.
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingMode {
    /// Pedestrian / pilgrim on foot — all road classes including footways.
    Pedestrian,
    /// Wheeled vehicle — road classes excluding footways.
    Vehicle,
    /// Pilgrimage to a sacred site — prefers lower-sacred-weight zones.
    Pilgrimage {
        /// When true, particles with sacred_weight EAV < 1.0 get a cost bonus.
        prefer_sacred: bool,
    },
    /// Utility pipeline maintenance routing — respects flow direction.
    Pipeline {
        kind: PipelineKind,
        /// When true, only traverse edges in the pipeline flow direction.
        follow_flow: bool,
    },
    /// Emergency override — traverses any passable particle, no multipliers.
    Emergency,
}

impl RoutingMode {
    /// True if this particle kind may be traversed in this routing mode.
    pub fn allows(&self, p: &MapParticle) -> bool {
        if !p.is_passable() {
            return false;
        }
        match self {
            RoutingMode::Pedestrian => p.kind.pedestrian_accessible(),
            RoutingMode::Vehicle => p.kind.vehicle_accessible(),
            RoutingMode::Pilgrimage { .. } => {
                p.kind.pedestrian_accessible() || matches!(p.kind, MapKind::GraveMarker)
            }
            RoutingMode::Pipeline { kind, .. } => {
                matches!(&p.kind, MapKind::Pipeline { kind: k, .. } if k == kind)
                    || p.kind.is_junction()
            }
            RoutingMode::Emergency => true,
        }
    }
}

// ── Route output ──────────────────────────────────────────────────────────────

/// One hop in a route — from particle to particle.
#[derive(Debug, Clone)]
pub struct MachRouteSegment {
    pub from_particle_id: u32,
    pub to_particle_id: u32,
    pub distance_m: f32,
    pub bearing_deg: f32,
}

/// A complete route as a sequence of sovereign particle hops.
#[derive(Debug, Clone)]
pub struct MachRoute {
    /// Particle IDs in traversal order (start → end).
    pub waypoints: Vec<u32>,
    pub segments: Vec<MachRouteSegment>,
    pub total_dist_m: f32,
    pub total_cost: f32,
}

impl MachRoute {
    pub fn is_valid(&self) -> bool {
        self.waypoints.len() >= 2
    }
    pub fn hop_count(&self) -> usize {
        self.waypoints.len().saturating_sub(1)
    }

    pub fn start(&self) -> Option<u32> {
        self.waypoints.first().copied()
    }
    pub fn end(&self) -> Option<u32> {
        self.waypoints.last().copied()
    }
}

// ── A* priority queue entry ───────────────────────────────────────────────────

#[derive(Eq, PartialEq)]
struct PqEntry {
    f_cost: u64, // (g_cost + h_cost) × 10_000, integer for Ord
    id: u32,
}

impl Ord for PqEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap: smaller f_cost wins
        other.f_cost.cmp(&self.f_cost).then(self.id.cmp(&other.id))
    }
}
impl PartialOrd for PqEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ── MachineRouter ─────────────────────────────────────────────────────────────

pub struct MachineRouter<'a> {
    map: &'a ParticleMap,
}

impl<'a> MachineRouter<'a> {
    pub fn new(map: &'a ParticleMap) -> Self {
        MachineRouter { map }
    }

    /// Route between two particle IDs.  Returns `None` if no path exists.
    pub fn route(&self, from: u32, to: u32, mode: &RoutingMode) -> Option<MachRoute> {
        if from == to {
            return Some(self.trivial_route(from));
        }
        let path = self.astar(from, to, mode)?;
        Some(self.build_route(path))
    }

    /// Route between two KAKI hashes.
    pub fn route_kaki(
        &self,
        from_kaki: u32,
        to_kaki: u32,
        mode: &RoutingMode,
    ) -> Option<MachRoute> {
        let from = self.map.by_kaki(from_kaki)?.particle_id;
        let to = self.map.by_kaki(to_kaki)?.particle_id;
        self.route(from, to, mode)
    }

    /// Route from the nearest passable particle to `from_coord` to the nearest to `to_coord`.
    pub fn route_coord(
        &self,
        from_coord: NaviCoord,
        to_coord: NaviCoord,
        mode: &RoutingMode,
    ) -> Option<MachRoute> {
        let from = self.nearest_passable(from_coord, mode)?;
        let to = self.nearest_passable(to_coord, mode)?;
        self.route(from, to, mode)
    }

    /// Nearest passable particle to `coord` that is allowed by `mode`.
    pub fn nearest_passable(&self, coord: NaviCoord, mode: &RoutingMode) -> Option<u32> {
        let mut radius = 250.0f32;
        loop {
            let candidates = self.map.query_radius(coord.lat, coord.lon, radius);
            let best = candidates
                .into_iter()
                .filter(|p| mode.allows(p))
                .min_by(|a, b| {
                    let da = haversine_m(a.coord.lat, a.coord.lon, coord.lat, coord.lon);
                    let db = haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                });
            if let Some(p) = best {
                return Some(p.particle_id);
            }
            radius *= 2.0;
            if radius > 100_000.0 {
                return None;
            }
        }
    }

    // ── A* ────────────────────────────────────────────────────────────────────

    fn astar(&self, start: u32, goal: u32, mode: &RoutingMode) -> Option<Vec<u32>> {
        let goal_particle = self.map.get(goal)?;

        let mut g_cost: HashMap<u32, f32> = HashMap::new();
        let mut came_from: HashMap<u32, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        g_cost.insert(start, 0.0);
        heap.push(Reverse(PqEntry {
            f_cost: 0,
            id: start,
        }));

        while let Some(Reverse(PqEntry { id: current, .. })) = heap.pop() {
            if current == goal {
                return Some(reconstruct_path(&came_from, goal));
            }

            let g_cur = *g_cost.get(&current).unwrap_or(&f32::MAX);

            for &(neighbour, edge_cost) in self.map.neighbours(current) {
                let neighbour_p = match self.map.get(neighbour) {
                    Some(p) => p,
                    None => continue,
                };
                if !mode.allows(neighbour_p) {
                    continue;
                }

                // Pipeline flow direction check
                if let RoutingMode::Pipeline {
                    follow_flow: true, ..
                } = mode
                {
                    if let MapKind::Pipeline { flow, .. } = &neighbour_p.kind {
                        if *flow == FlowDir::UpStream {
                            // Travelling from neighbour TOWARD source — wrong direction
                            continue;
                        }
                    }
                }

                let step_cost = self.mode_cost(edge_cost, neighbour_p, mode);
                let new_g = g_cur + step_cost;

                if new_g < *g_cost.get(&neighbour).unwrap_or(&f32::MAX) {
                    g_cost.insert(neighbour, new_g);
                    came_from.insert(neighbour, current);
                    let h = haversine_m(
                        neighbour_p.coord.lat,
                        neighbour_p.coord.lon,
                        goal_particle.coord.lat,
                        goal_particle.coord.lon,
                    );
                    let f = ((new_g + h) * 10_000.0) as u64;
                    heap.push(Reverse(PqEntry {
                        f_cost: f,
                        id: neighbour,
                    }));
                }
            }
        }
        None
    }

    /// Apply routing-mode cost modifiers to an edge.
    fn mode_cost(&self, edge_cost: f32, p: &MapParticle, mode: &RoutingMode) -> f32 {
        match mode {
            RoutingMode::Emergency => edge_cost * 0.5, // fastest regardless
            RoutingMode::Pilgrimage { prefer_sacred } => {
                let sacred = if *prefer_sacred {
                    // Lower sacred weight = more revered = prefer (reduce cost)
                    p.eav
                        .get_float(crate::eav::ATTR_SACRED_WEIGHT)
                        .unwrap_or(1.0)
                } else {
                    1.0
                };
                edge_cost * sacred
            }
            _ => edge_cost,
        }
    }

    fn trivial_route(&self, id: u32) -> MachRoute {
        MachRoute {
            waypoints: vec![id],
            segments: Vec::new(),
            total_dist_m: 0.0,
            total_cost: 0.0,
        }
    }

    fn build_route(&self, path: Vec<u32>) -> MachRoute {
        let mut segments = Vec::with_capacity(path.len().saturating_sub(1));
        let mut total_dist = 0.0f32;
        let mut total_cost = 0.0f32;

        for w in path.windows(2) {
            let (a_id, b_id) = (w[0], w[1]);
            let a = self.map.get(a_id);
            let b = self.map.get(b_id);
            let dist = match (a, b) {
                (Some(a), Some(b)) => {
                    haversine_m(a.coord.lat, a.coord.lon, b.coord.lat, b.coord.lon)
                }
                _ => 0.0,
            };
            let bearing = match (a, b) {
                (Some(a), Some(b)) => {
                    bearing_deg(a.coord.lat, a.coord.lon, b.coord.lat, b.coord.lon)
                }
                _ => 0.0,
            };
            let cost = self
                .map
                .neighbours(a_id)
                .iter()
                .find(|(to, _)| *to == b_id)
                .map(|(_, c)| *c)
                .unwrap_or(dist);
            total_dist += dist;
            total_cost += cost;
            segments.push(MachRouteSegment {
                from_particle_id: a_id,
                to_particle_id: b_id,
                distance_m: dist,
                bearing_deg: bearing,
            });
        }

        MachRoute {
            waypoints: path,
            segments,
            total_dist_m: total_dist,
            total_cost,
        }
    }
}

fn reconstruct_path(came_from: &HashMap<u32, u32>, mut current: u32) -> Vec<u32> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

/// Compass bearing (degrees, 0=North, clockwise) from (lat1,lon1) to (lat2,lon2).
fn bearing_deg(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let d_lon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let y = d_lon.sin() * lat2r.cos();
    let x = lat1r.cos() * lat2r.sin() - lat1r.sin() * lat2r.cos() * d_lon.cos();
    let bearing = y.atan2(x).to_degrees();
    (bearing + 360.0) % 360.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapparticle::{MapKind, RoadClass};
    use crate::particle::NaviCoord;
    use crate::particlemap::{MapBounds, ParticleMap};
    use bahyway_core::TribeId;

    fn tribe() -> TribeId {
        TribeId::from_u16(0x0001)
    }

    /// Build a simple 3-node map: A —100m→ B —150m→ C
    fn three_node_map() -> ParticleMap {
        let mut m = ParticleMap::new(MapBounds::new(31.980, 32.010, 44.310, 44.340), 0.005);
        let a = m.place(
            NaviCoord::new(31.990, 44.310, 0.0),
            MapKind::Road {
                class: RoadClass::Secondary,
                one_way: false,
            },
            tribe(),
            2,
        );
        let b = m.place(
            NaviCoord::new(31.995, 44.320, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        let c = m.place(
            NaviCoord::new(32.000, 44.330, 0.0),
            MapKind::Road {
                class: RoadClass::Primary,
                one_way: false,
            },
            tribe(),
            2,
        );
        m.add_edge(a, b, 1000.0, true);
        m.add_edge(b, c, 1200.0, true);
        m
    }

    /// 4-node pipeline: Source—→—A—→—B—→—Drain
    fn pipeline_map() -> (ParticleMap, u32, u32) {
        let mut m = ParticleMap::new(MapBounds::new(31.980, 32.010, 44.310, 44.360), 0.005);
        let src = m.place(
            NaviCoord::new(31.990, 44.310, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        let pa = m.place(
            NaviCoord::new(31.993, 44.316, 0.0),
            MapKind::Pipeline {
                kind: PipelineKind::WaterSupply,
                flow: FlowDir::DownStream,
            },
            tribe(),
            3,
        );
        let pb = m.place(
            NaviCoord::new(31.996, 44.322, 0.0),
            MapKind::Pipeline {
                kind: PipelineKind::WaterSupply,
                flow: FlowDir::DownStream,
            },
            tribe(),
            3,
        );
        let drn = m.place(
            NaviCoord::new(31.999, 44.328, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        m.add_edge(src, pa, 800.0, false); // directed: source → pa
        m.add_edge(pa, pb, 700.0, false);
        m.add_edge(pb, drn, 600.0, false);
        (m, src, drn)
    }

    #[test]
    fn route_a_to_c() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        let ids = m.all_ids();
        let (a, c) = (*ids.iter().min().unwrap(), *ids.iter().max().unwrap());
        let route = r.route(a, c, &RoutingMode::Vehicle).expect("route A→C");
        assert!(route.is_valid());
        assert_eq!(route.start(), Some(a));
        assert_eq!(route.end(), Some(c));
    }

    #[test]
    fn trivial_route_same_particle() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        let id = *m.all_ids().iter().next().unwrap();
        let route = r.route(id, id, &RoutingMode::Pedestrian).unwrap();
        assert_eq!(route.hop_count(), 0);
        assert!(!route.is_valid()); // 1 waypoint, not 2
    }

    #[test]
    fn route_nonexistent_returns_none() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        let id = *m.all_ids().iter().next().unwrap();
        assert!(r.route(id, 9999, &RoutingMode::Pedestrian).is_none());
    }

    #[test]
    fn route_hop_count_correct() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        let mut ids: Vec<u32> = m.all_ids();
        ids.sort();
        let route = r.route(ids[0], ids[2], &RoutingMode::Vehicle).unwrap();
        assert_eq!(route.hop_count(), 2); // A→B→C
    }

    #[test]
    fn total_distance_positive() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        let mut ids: Vec<u32> = m.all_ids();
        ids.sort();
        let route = r.route(ids[0], ids[2], &RoutingMode::Pedestrian).unwrap();
        assert!(route.total_dist_m > 0.0);
    }

    #[test]
    fn bearing_north_is_zero() {
        let b = bearing_deg(31.0, 44.0, 32.0, 44.0);
        assert!(
            b < 10.0 || b > 350.0,
            "northward bearing should be near 0°, got {b}"
        );
    }

    #[test]
    fn bearing_east_is_90() {
        let b = bearing_deg(31.0, 44.0, 31.0, 45.0);
        assert!(
            (b - 90.0).abs() < 5.0,
            "eastward bearing should be near 90°, got {b}"
        );
    }

    #[test]
    fn pipeline_route_follows_flow() {
        let (m, src, drn) = pipeline_map();
        let r = MachineRouter::new(&m);
        let mode = RoutingMode::Pipeline {
            kind: PipelineKind::WaterSupply,
            follow_flow: true,
        };
        let route = r.route(src, drn, &mode).expect("pipeline route");
        assert!(route.is_valid());
        assert_eq!(route.start(), Some(src));
        assert_eq!(route.end(), Some(drn));
    }

    #[test]
    fn vehicle_mode_excludes_footway() {
        let mut m = ParticleMap::new(MapBounds::new(31.980, 32.010, 44.310, 44.350), 0.005);
        let a = m.place(
            NaviCoord::new(31.990, 44.310, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        let b = m.place(
            NaviCoord::new(31.995, 44.320, 0.0),
            MapKind::Road {
                class: RoadClass::Footway,
                one_way: false,
            },
            tribe(),
            3,
        );
        let c = m.place(
            NaviCoord::new(32.000, 44.330, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        m.add_edge(a, b, 500.0, true);
        m.add_edge(b, c, 500.0, true);
        let r = MachineRouter::new(&m);
        // Vehicle mode cannot traverse footway → no route A→C
        let route = r.route(a, c, &RoutingMode::Vehicle);
        assert!(route.is_none(), "vehicle must not traverse footway");
    }

    #[test]
    fn pedestrian_mode_traverses_footway() {
        let mut m = ParticleMap::new(MapBounds::new(31.980, 32.010, 44.310, 44.350), 0.005);
        let a = m.place(
            NaviCoord::new(31.990, 44.310, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        let b = m.place(
            NaviCoord::new(31.995, 44.320, 0.0),
            MapKind::Road {
                class: RoadClass::Footway,
                one_way: false,
            },
            tribe(),
            3,
        );
        let c = m.place(
            NaviCoord::new(32.000, 44.330, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        m.add_edge(a, b, 500.0, true);
        m.add_edge(b, c, 500.0, true);
        let r = MachineRouter::new(&m);
        let route = r.route(a, c, &RoutingMode::Pedestrian);
        assert!(route.is_some(), "pedestrian must traverse footway");
    }

    #[test]
    fn emergency_mode_traverses_everything() {
        let mut m = ParticleMap::new(MapBounds::new(31.980, 32.010, 44.310, 44.350), 0.005);
        let a = m.place(
            NaviCoord::new(31.990, 44.310, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        let b = m.place(
            NaviCoord::new(31.995, 44.320, 0.0),
            MapKind::Pipeline {
                kind: PipelineKind::WaterSupply,
                flow: FlowDir::DownStream,
            },
            tribe(),
            3,
        );
        let c = m.place(
            NaviCoord::new(32.000, 44.330, 0.0),
            MapKind::Junction,
            tribe(),
            2,
        );
        m.add_edge(a, b, 500.0, true);
        m.add_edge(b, c, 500.0, true);
        let r = MachineRouter::new(&m);
        let route = r.route(a, c, &RoutingMode::Emergency);
        assert!(route.is_some(), "emergency must traverse pipeline");
    }

    #[test]
    fn route_coord_finds_nearest_entry_point() {
        let m = three_node_map();
        let r = MachineRouter::new(&m);
        // Coordinates near first and last particles
        let from = NaviCoord::new(31.990, 44.310, 0.0);
        let to = NaviCoord::new(32.000, 44.330, 0.0);
        let route = r.route_coord(from, to, &RoutingMode::Pedestrian);
        assert!(route.is_some());
    }
}
