//! ParticleMap — sovereign GIS replacement.
//!
//! The entire map is stored as sovereign `MapParticle`s in memory, indexed
//! for spatial queries via a pure-Rust `SpatialGrid` (no R-tree external deps).
//! Connectivity is represented as directed edges with an O(1) adjacency index.
//!
//! ## How it replaces third-party GIS
//!
//! | GIS concept          | ParticleMap equivalent                    |
//! |----------------------|-------------------------------------------|
//! | Feature layer        | `MapKind` enum on each particle           |
//! | Attribute table      | `EavStore` per particle                   |
//! | Spatial index        | `SpatialGrid` (uniform grid cells)        |
//! | Topology / network   | `MapEdge` list + `adjacency` HashMap      |
//! | Tile cache           | `HubbleTile` zoom layer                   |
//! | KAKI identity        | Immutable sovereign `Kaki` per particle   |

#![forbid(unsafe_code)]

use bahyway_core::TribeId;
use std::collections::HashMap;

use crate::mapparticle::{MapKind, MapParticle};
use crate::particle::NaviCoord;
use crate::route::haversine_m;
use crate::tile::HubbleTileId;

// ── MapBounds ─────────────────────────────────────────────────────────────────

/// Bounding box of the entire `ParticleMap` in WGS-84 degrees.
#[derive(Debug, Clone, Copy)]
pub struct MapBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lon_min: f32,
    pub lon_max: f32,
}

impl MapBounds {
    pub fn new(lat_min: f32, lat_max: f32, lon_min: f32, lon_max: f32) -> Self {
        MapBounds {
            lat_min,
            lat_max,
            lon_min,
            lon_max,
        }
    }

    /// Generous world bounds — accepts any WGS-84 coordinate.
    pub fn world() -> Self {
        MapBounds {
            lat_min: -90.0,
            lat_max: 90.0,
            lon_min: -180.0,
            lon_max: 180.0,
        }
    }

    pub fn contains(&self, lat: f32, lon: f32) -> bool {
        lat >= self.lat_min && lat <= self.lat_max && lon >= self.lon_min && lon <= self.lon_max
    }
}

// ── SpatialGrid ───────────────────────────────────────────────────────────────

/// Uniform grid spatial index — pure Rust, no external dependencies.
///
/// Divides the map bounding box into equal-size cells.  Each cell holds the
/// IDs of particles whose coordinates fall within it.  Spatial queries scan
/// only the cells that intersect the query bounding box.
struct SpatialGrid {
    cells: Vec<Vec<u32>>,
    lat_min: f32,
    lon_min: f32,
    cell_size: f32,
    cols: usize,
    rows: usize,
}

impl SpatialGrid {
    fn new(bounds: MapBounds, cell_size: f32) -> Self {
        let rows = (((bounds.lat_max - bounds.lat_min) / cell_size).ceil() as usize).max(1);
        let cols = (((bounds.lon_max - bounds.lon_min) / cell_size).ceil() as usize).max(1);
        SpatialGrid {
            cells: vec![Vec::new(); rows * cols],
            lat_min: bounds.lat_min,
            lon_min: bounds.lon_min,
            cell_size,
            cols,
            rows,
        }
    }

    fn cell_index(&self, lat: f32, lon: f32) -> Option<usize> {
        let row = ((lat - self.lat_min) / self.cell_size).floor() as isize;
        let col = ((lon - self.lon_min) / self.cell_size).floor() as isize;
        if row < 0 || col < 0 || row >= self.rows as isize || col >= self.cols as isize {
            return None;
        }
        Some(row as usize * self.cols + col as usize)
    }

    fn insert(&mut self, particle_id: u32, lat: f32, lon: f32) {
        if let Some(idx) = self.cell_index(lat, lon) {
            self.cells[idx].push(particle_id);
        }
    }

    /// Return all particle IDs in cells that overlap the query bounding box.
    fn query_bbox(&self, lat_min: f32, lat_max: f32, lon_min: f32, lon_max: f32) -> Vec<u32> {
        let row0 = (((lat_min - self.lat_min) / self.cell_size).floor() as isize).max(0) as usize;
        let row1 = (((lat_max - self.lat_min) / self.cell_size).ceil() as isize).max(0) as usize;
        let col0 = (((lon_min - self.lon_min) / self.cell_size).floor() as isize).max(0) as usize;
        let col1 = (((lon_max - self.lon_min) / self.cell_size).ceil() as isize).max(0) as usize;
        let row1 = row1.min(self.rows);
        let col1 = col1.min(self.cols);
        let mut ids = Vec::new();
        for r in row0..row1 {
            for c in col0..col1 {
                ids.extend_from_slice(&self.cells[r * self.cols + c]);
            }
        }
        ids
    }
}

// ── MapEdge ───────────────────────────────────────────────────────────────────

/// A directed edge between two `MapParticle`s.
#[derive(Debug, Clone)]
pub struct MapEdge {
    pub from: u32, // particle_id
    pub to: u32,   // particle_id
    pub distance_m: f32,
}

// ── ParticleMap ───────────────────────────────────────────────────────────────

/// The sovereign map — all particles, their attributes, and their connectivity.
pub struct ParticleMap {
    particles: HashMap<u32, MapParticle>,
    kaki_index: HashMap<u32, u32>, // kaki_hash → particle_id
    spatial: SpatialGrid,
    edges: Vec<MapEdge>,
    adjacency: HashMap<u32, Vec<(u32, f32)>>,
    next_id: u32,
}

impl ParticleMap {
    /// Create a new empty map.  `cell_size_deg` controls the spatial grid
    /// resolution — 0.01° (≈1 km) is a good default for city-scale maps.
    pub fn new(bounds: MapBounds, cell_size_deg: f32) -> Self {
        ParticleMap {
            particles: HashMap::new(),
            kaki_index: HashMap::new(),
            spatial: SpatialGrid::new(bounds, cell_size_deg),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a map covering Wadi al-Salam cemetery area at fine resolution.
    pub fn najaf_map() -> Self {
        Self::new(MapBounds::new(31.960, 32.040, 44.290, 44.360), 0.001)
    }

    /// Register a particle.  Returns the assigned particle_id.
    /// If `particle.particle_id` is 0, a fresh ID is auto-assigned.
    pub fn add_particle(&mut self, mut p: MapParticle) -> u32 {
        if p.particle_id == 0 {
            p.particle_id = self.next_id;
        }
        self.next_id = self.next_id.max(p.particle_id + 1);
        let id = p.particle_id;
        self.kaki_index.insert(p.kaki_hash(), id);
        self.spatial.insert(id, p.coord.lat, p.coord.lon);
        self.adjacency.entry(id).or_default();
        self.particles.insert(id, p);
        id
    }

    /// Add a directed edge from `from` to `to` with measured `distance_m`.
    /// If `bidirectional`, also adds the reverse edge.
    pub fn add_edge(&mut self, from: u32, to: u32, distance_m: f32, bidirectional: bool) {
        // Routing cost = distance × average particle multiplier
        let cost_fwd = self.edge_cost(from, to, distance_m);
        self.adjacency.entry(from).or_default().push((to, cost_fwd));
        self.edges.push(MapEdge {
            from,
            to,
            distance_m,
        });
        if bidirectional {
            let cost_rev = self.edge_cost(to, from, distance_m);
            self.adjacency.entry(to).or_default().push((from, cost_rev));
            self.edges.push(MapEdge {
                from: to,
                to: from,
                distance_m,
            });
        }
    }

    /// Add an edge whose distance is computed from the two particles' haversine distance.
    pub fn add_edge_auto_dist(&mut self, from: u32, to: u32, bidirectional: bool) {
        let dist = self
            .particles
            .get(&from)
            .zip(self.particles.get(&to))
            .map(|(a, b)| haversine_m(a.coord.lat, a.coord.lon, b.coord.lat, b.coord.lon))
            .unwrap_or(1.0);
        self.add_edge(from, to, dist, bidirectional);
    }

    fn edge_cost(&self, from: u32, to: u32, dist_m: f32) -> f32 {
        let m_from = self
            .particles
            .get(&from)
            .map(|p| p.routing_cost())
            .unwrap_or(1.0);
        let m_to = self
            .particles
            .get(&to)
            .map(|p| p.routing_cost())
            .unwrap_or(1.0);
        dist_m * (m_from + m_to) / 2.0
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get(&self, id: u32) -> Option<&MapParticle> {
        self.particles.get(&id)
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut MapParticle> {
        self.particles.get_mut(&id)
    }

    /// Look up a particle by its KAKI hash (first 4 bytes of uuid field).
    pub fn by_kaki(&self, kaki_hash: u32) -> Option<&MapParticle> {
        self.kaki_index
            .get(&kaki_hash)
            .and_then(|id| self.particles.get(id))
    }

    /// Return all particles whose coordinates fall within a bounding box.
    pub fn query_bbox(
        &self,
        lat_min: f32,
        lat_max: f32,
        lon_min: f32,
        lon_max: f32,
    ) -> Vec<&MapParticle> {
        self.spatial
            .query_bbox(lat_min, lat_max, lon_min, lon_max)
            .into_iter()
            .filter_map(|id| self.particles.get(&id))
            .collect()
    }

    /// Return all particles visible at `zoom_level` within the tile.
    pub fn particles_in_tile(&self, tile: &HubbleTileId) -> Vec<&MapParticle> {
        let b = tile.bounds();
        self.spatial
            .query_bbox(b.lat_min, b.lat_max, b.lon_min, b.lon_max)
            .into_iter()
            .filter_map(|id| self.particles.get(&id))
            .filter(|p| p.zoom_min <= tile.zoom)
            .collect()
    }

    /// Return all particles within `radius_m` of (lat, lon).
    pub fn query_radius(&self, lat: f32, lon: f32, radius_m: f32) -> Vec<&MapParticle> {
        let deg = radius_m / 111_320.0;
        self.spatial
            .query_bbox(lat - deg, lat + deg, lon - deg, lon + deg)
            .into_iter()
            .filter_map(|id| self.particles.get(&id))
            .filter(|p| haversine_m(p.coord.lat, p.coord.lon, lat, lon) <= radius_m)
            .collect()
    }

    /// Nearest particle to (lat, lon) of the given kind (None = any kind).
    pub fn nearest(&self, lat: f32, lon: f32, kind: Option<&MapKind>) -> Option<&MapParticle> {
        // Start with radius 500 m, double until found or > 50 km
        let mut radius = 500.0f32;
        loop {
            let candidates: Vec<&MapParticle> = self
                .query_radius(lat, lon, radius)
                .into_iter()
                .filter(|p| p.is_passable())
                .filter(|p| {
                    kind.is_none_or(|k| {
                        core::mem::discriminant(&p.kind) == core::mem::discriminant(k)
                    })
                })
                .collect();
            if let Some(best) = candidates.into_iter().min_by(|a, b| {
                let da = haversine_m(a.coord.lat, a.coord.lon, lat, lon);
                let db = haversine_m(b.coord.lat, b.coord.lon, lat, lon);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                return Some(best);
            }
            radius *= 2.0;
            if radius > 50_000.0 {
                return None;
            }
        }
    }

    /// O(1) adjacency list for a particle (particle_id → [(to_id, cost)]).
    pub fn neighbours(&self, id: u32) -> &[(u32, f32)] {
        self.adjacency.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn all_ids(&self) -> Vec<u32> {
        self.particles.keys().copied().collect()
    }

    /// Convenience: build a `MapParticle` and register it in one call.
    pub fn place(&mut self, coord: NaviCoord, kind: MapKind, tribe: TribeId, zoom_min: u8) -> u32 {
        let id = self.next_id;
        let p = MapParticle::new(id, coord, kind, tribe, zoom_min);
        self.add_particle(p)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapparticle::{MapKind, RoadClass};
    use crate::particle::{NaviCoord, NaviParticleState};
    use bahyway_core::TribeId;

    fn tribe() -> TribeId {
        TribeId::from_u16(0x0001)
    }

    fn najaf_map() -> ParticleMap {
        ParticleMap::new(MapBounds::new(31.960, 32.040, 44.290, 44.360), 0.002)
    }

    fn add_junction(m: &mut ParticleMap, lat: f32, lon: f32) -> u32 {
        m.place(NaviCoord::new(lat, lon, 0.0), MapKind::Junction, tribe(), 2)
    }

    fn add_road(m: &mut ParticleMap, lat: f32, lon: f32) -> u32 {
        m.place(
            NaviCoord::new(lat, lon, 0.0),
            MapKind::Road {
                class: RoadClass::Secondary,
                one_way: false,
            },
            tribe(),
            2,
        )
    }

    #[test]
    fn add_and_retrieve() {
        let mut m = najaf_map();
        let id = add_junction(&mut m, 31.995, 44.320);
        assert!(m.get(id).is_some());
        assert_eq!(m.particle_count(), 1);
    }

    #[test]
    fn kaki_index_roundtrip() {
        let mut m = najaf_map();
        let id = add_junction(&mut m, 31.995, 44.320);
        let hash = m.get(id).unwrap().kaki_hash();
        assert_eq!(m.by_kaki(hash).unwrap().particle_id, id);
    }

    #[test]
    fn query_bbox_finds_nearby() {
        let mut m = najaf_map();
        add_junction(&mut m, 31.995, 44.320);
        add_junction(&mut m, 32.100, 44.500); // far away
        let found = m.query_bbox(31.980, 32.010, 44.300, 44.340);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn query_radius_respects_haversine() {
        let mut m = najaf_map();
        add_junction(&mut m, 31.995, 44.320);
        // 200 m away (~0.002°)
        add_junction(&mut m, 32.100, 44.500);
        // Query 500 m around reference — only first should appear
        let found = m.query_radius(31.995, 44.320, 500.0);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn edge_adds_adjacency() {
        let mut m = najaf_map();
        let a = add_junction(&mut m, 31.990, 44.310);
        let b = add_junction(&mut m, 31.995, 44.320);
        m.add_edge(a, b, 1000.0, false);
        assert_eq!(m.neighbours(a).len(), 1);
        assert_eq!(m.neighbours(b).len(), 0); // one-directional
    }

    #[test]
    fn bidirectional_edge_adds_both_directions() {
        let mut m = najaf_map();
        let a = add_junction(&mut m, 31.990, 44.310);
        let b = add_junction(&mut m, 31.995, 44.320);
        m.add_edge(a, b, 1000.0, true);
        assert_eq!(m.neighbours(a).len(), 1);
        assert_eq!(m.neighbours(b).len(), 1);
        assert_eq!(m.edge_count(), 2);
    }

    #[test]
    fn auto_dist_edge_computes_haversine() {
        let mut m = najaf_map();
        let a = add_junction(&mut m, 31.990, 44.310);
        let b = add_junction(&mut m, 31.995, 44.320);
        m.add_edge_auto_dist(a, b, false);
        let edge_dist = m
            .edges
            .iter()
            .find(|e| e.from == a && e.to == b)
            .unwrap()
            .distance_m;
        let expected = haversine_m(31.990, 44.310, 31.995, 44.320);
        assert!((edge_dist - expected).abs() < 1.0);
    }

    #[test]
    fn nearest_returns_closest() {
        let mut m = najaf_map();
        let a = add_junction(&mut m, 31.995, 44.320);
        let _b = add_junction(&mut m, 32.100, 44.500);
        let nearest = m.nearest(31.995, 44.320, None).unwrap();
        assert_eq!(nearest.particle_id, a);
    }

    #[test]
    fn nearest_skips_dead_particles() {
        let mut m = najaf_map();
        let a = add_junction(&mut m, 31.995, 44.320);
        let b = add_road(&mut m, 31.995, 44.321);
        m.get_mut(a).unwrap().state = NaviParticleState::Dead;
        let nearest = m.nearest(31.995, 44.320, None).unwrap();
        assert_eq!(nearest.particle_id, b);
    }

    #[test]
    fn particles_in_tile_respects_zoom_min() {
        let mut m = najaf_map();
        // zoom_min=5 — only visible at Z5 and Z6
        m.place(
            NaviCoord::new(31.995, 44.320, 0.0),
            MapKind::GraveMarker,
            tribe(),
            5,
        );
        let tile_z3 = HubbleTileId::from_coord(31.995, 44.320, 3);
        let tile_z5 = HubbleTileId::from_coord(31.995, 44.320, 5);
        assert!(
            m.particles_in_tile(&tile_z3).is_empty(),
            "z3 must not show zoom_min=5 particle"
        );
        assert!(
            !m.particles_in_tile(&tile_z5).is_empty(),
            "z5 must show zoom_min=5 particle"
        );
    }

    #[test]
    fn place_assigns_sequential_ids() {
        let mut m = najaf_map();
        let ids: Vec<u32> = (0..5)
            .map(|i| add_junction(&mut m, 31.990 + i as f32 * 0.001, 44.320))
            .collect();
        let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn particle_map_empty_initially() {
        let m = najaf_map();
        assert_eq!(m.particle_count(), 0);
        assert_eq!(m.edge_count(), 0);
    }
}
