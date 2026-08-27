//! HubbleTile — 7-level sovereign zoom system (Z0–Z6).
//!
//! The "Hubble" metaphor: zooming in reveals progressively finer particles,
//! from country-level regions (Z0) down to sub-meter precision (Z6).
//!
//! | Level | Name        | Tile size  | Typical content                      |
//! |-------|-------------|------------|--------------------------------------|
//! | Z0    | Global      | 10°        | Countries, major cities              |
//! | Z1    | Regional    |  2°        | Provinces, major roads               |
//! | Z2    | City        |  0.5°      | Districts, streets, neighbourhoods   |
//! | Z3    | District    |  0.1°      | Blocks, individual buildings         |
//! | Z4    | Block       |  0.02°     | Cemetery sections, manholes          |
//! | Z5    | Feature     |  0.005°    | Individual graves, pipe joints       |
//! | Z6    | Precision   |  0.001°    | Sub-meter landmarks, inscriptions    |
//!
//! Every `MapParticle` has `zoom_min: u8` — the minimum zoom level at which
//! it becomes visible.  `HubbleTile::contains_particle()` uses this to filter.

#![forbid(unsafe_code)]

/// Maximum zoom level (7 levels: Z0–Z6, matching 7D particle architecture).
pub const HUBBLE_ZOOM_MAX: u8 = 6;

/// Tile size in degrees at each zoom level.
/// Index 0 = Z0, index 6 = Z6.
const TILE_SIZES_DEG: [f32; 7] = [10.0, 2.0, 0.5, 0.1, 0.02, 0.005, 0.001];

// ── TileBounds ────────────────────────────────────────────────────────────────

/// Bounding box of a `HubbleTile` in WGS-84 degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lon_min: f32,
    pub lon_max: f32,
}

impl TileBounds {
    /// Epsilon for boundary membership — ~1 m tolerance, safe for GPS.
    const EPS: f32 = 1e-5;

    pub fn contains(&self, lat: f32, lon: f32) -> bool {
        lat >= self.lat_min - Self::EPS
            && lat < self.lat_max + Self::EPS
            && lon >= self.lon_min - Self::EPS
            && lon < self.lon_max + Self::EPS
    }

    pub fn overlaps(&self, other: &TileBounds) -> bool {
        self.lat_min < other.lat_max
            && self.lat_max > other.lat_min
            && self.lon_min < other.lon_max
            && self.lon_max > other.lon_min
    }

    /// Centre of the tile.
    pub fn centre(&self) -> (f32, f32) {
        (
            (self.lat_min + self.lat_max) / 2.0,
            (self.lon_min + self.lon_max) / 2.0,
        )
    }
}

// ── HubbleTileId ─────────────────────────────────────────────────────────────

/// Sovereign tile identifier — zoom level + grid X (lon) + grid Y (lat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HubbleTileId {
    pub zoom: u8,
    pub x: u32, // longitude grid index
    pub y: u32, // latitude grid index
}

impl HubbleTileId {
    pub fn new(zoom: u8, x: u32, y: u32) -> Self {
        HubbleTileId {
            zoom: zoom.min(HUBBLE_ZOOM_MAX),
            x,
            y,
        }
    }

    /// Tile size in degrees at this zoom level.
    pub fn tile_size_deg(&self) -> f32 {
        TILE_SIZES_DEG[self.zoom as usize]
    }

    /// Compute the tile that contains (lat, lon) at `zoom`.
    ///
    /// Uses a global grid anchored at (-90°, -180°).
    /// Internal f64 arithmetic avoids f32 precision loss at fine zoom levels.
    pub fn from_coord(lat: f32, lon: f32, zoom: u8) -> Self {
        let zoom = zoom.min(HUBBLE_ZOOM_MAX);
        let size = TILE_SIZES_DEG[zoom as usize] as f64;
        let y = ((lat as f64 + 90.0) / size).floor() as u32;
        let x = ((lon as f64 + 180.0) / size).floor() as u32;
        HubbleTileId { zoom, x, y }
    }

    /// Bounding box of this tile.
    /// Internal f64 arithmetic for precision; bounds stored as f32.
    pub fn bounds(&self) -> TileBounds {
        let size = TILE_SIZES_DEG[self.zoom as usize] as f64;
        let lat_min = (self.y as f64 * size - 90.0) as f32;
        let lon_min = (self.x as f64 * size - 180.0) as f32;
        TileBounds {
            lat_min,
            lat_max: (lat_min as f64 + size) as f32,
            lon_min,
            lon_max: (lon_min as f64 + size) as f32,
        }
    }

    /// Parent tile (one zoom level coarser).  Returns `None` at Z0.
    pub fn parent(&self) -> Option<HubbleTileId> {
        if self.zoom == 0 {
            return None;
        }
        Some(HubbleTileId::from_coord(
            self.bounds().centre().0,
            self.bounds().centre().1,
            self.zoom - 1,
        ))
    }

    /// All child tiles at the next finer zoom level that are within this tile.
    ///
    /// Tile sizes do not follow 2:1 ratios (e.g. Z3 0.1° → Z4 0.02° = 5×5 = 25 children).
    /// Returns empty at Z6.
    pub fn children(&self) -> Vec<HubbleTileId> {
        if self.zoom >= HUBBLE_ZOOM_MAX {
            return Vec::new();
        }
        let next = self.zoom + 1;
        let parent_sz = TILE_SIZES_DEG[self.zoom as usize] as f64;
        let child_sz = TILE_SIZES_DEG[next as usize] as f64;
        let count = (parent_sz / child_sz).round() as u32;
        let b = self.bounds();
        let lat0 = b.lat_min as f64;
        let lon0 = b.lon_min as f64;
        let mut result = Vec::with_capacity((count * count) as usize);
        for r in 0..count {
            for c in 0..count {
                let lat = lat0 + (r as f64 + 0.5) * child_sz;
                let lon = lon0 + (c as f64 + 0.5) * child_sz;
                result.push(HubbleTileId::from_coord(lat as f32, lon as f32, next));
            }
        }
        result
    }

    /// Number of children per dimension (tile-size ratio to next zoom level).
    pub fn children_per_dim(&self) -> u32 {
        if self.zoom >= HUBBLE_ZOOM_MAX {
            return 0;
        }
        let p = TILE_SIZES_DEG[self.zoom as usize] as f64;
        let c = TILE_SIZES_DEG[(self.zoom + 1) as usize] as f64;
        (p / c).round() as u32
    }
}

// ── HubbleTile ────────────────────────────────────────────────────────────────

/// A zoom tile with a list of `MapParticle` IDs visible at this zoom level.
pub struct HubbleTile {
    pub id: HubbleTileId,
    /// Particle IDs (from `ParticleMap`) visible within this tile at its zoom level.
    pub particle_ids: Vec<u32>,
}

impl HubbleTile {
    pub fn new(id: HubbleTileId) -> Self {
        HubbleTile {
            id,
            particle_ids: Vec::new(),
        }
    }

    pub fn add_particle(&mut self, particle_id: u32) {
        if !self.particle_ids.contains(&particle_id) {
            self.particle_ids.push(particle_id);
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particle_ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.particle_ids.is_empty()
    }
    pub fn bounds(&self) -> TileBounds {
        self.id.bounds()
    }

    /// True if the particle's `zoom_min` ≤ this tile's zoom level.
    pub fn contains_particle(&self, particle_zoom_min: u8) -> bool {
        particle_zoom_min <= self.id.zoom
    }
}

// ── Zoom utilities ────────────────────────────────────────────────────────────

/// Return the appropriate zoom level for a search radius in metres.
///
/// Chooses the coarsest zoom whose tile fits at least the given radius.
pub fn zoom_for_radius_m(radius_m: f32) -> u8 {
    // 1 degree ≈ 111,320 m at equator
    let radius_deg = radius_m / 111_320.0;
    for zoom in (0..=HUBBLE_ZOOM_MAX).rev() {
        if TILE_SIZES_DEG[zoom as usize] >= radius_deg {
            return zoom;
        }
    }
    0
}

/// Return all tile IDs that intersect the bounding box at `zoom`.
pub fn tiles_in_bbox(
    lat_min: f32,
    lat_max: f32,
    lon_min: f32,
    lon_max: f32,
    zoom: u8,
) -> Vec<HubbleTileId> {
    let size = TILE_SIZES_DEG[zoom.min(HUBBLE_ZOOM_MAX) as usize];
    let origin_lat = -90.0f32;
    let origin_lon = -180.0f32;
    let y0 = ((lat_min - origin_lat) / size).floor() as u32;
    let y1 = ((lat_max - origin_lat) / size).ceil() as u32;
    let x0 = ((lon_min - origin_lon) / size).floor() as u32;
    let x1 = ((lon_max - origin_lon) / size).ceil() as u32;
    let mut ids = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            ids.push(HubbleTileId::new(zoom, x, y));
        }
    }
    ids
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn najaf_tile(zoom: u8) -> HubbleTileId {
        // Wadi al-Salam: lat=31.995, lon=44.320
        HubbleTileId::from_coord(31.995, 44.320, zoom)
    }

    #[test]
    fn tile_contains_its_own_coord() {
        for zoom in 0..=HUBBLE_ZOOM_MAX {
            let t = najaf_tile(zoom);
            assert!(
                t.bounds().contains(31.995, 44.320),
                "z{zoom}: tile {:?} must contain Najaf coord",
                t
            );
        }
    }

    #[test]
    fn tile_size_decreases_with_zoom() {
        for zoom in 0..(HUBBLE_ZOOM_MAX) {
            let cur = TILE_SIZES_DEG[zoom as usize];
            let next = TILE_SIZES_DEG[(zoom + 1) as usize];
            assert!(cur > next, "z{zoom}: {cur} must be > z{}: {next}", zoom + 1);
        }
    }

    #[test]
    fn parent_child_round_trip() {
        let t = najaf_tile(4);
        let p = t.parent().unwrap();
        let children = p.children();
        assert!(
            children.iter().any(|c| c == &t),
            "tile must be among its parent's children"
        );
    }

    #[test]
    fn z0_has_no_parent() {
        assert!(najaf_tile(0).parent().is_none());
    }

    #[test]
    fn z6_has_no_children() {
        assert!(najaf_tile(6).children().is_empty());
    }

    #[test]
    fn children_count_is_n_squared_below_z6() {
        // Tile size ratios: Z0→Z1=5, Z1→Z2=4, Z2→Z3=5, Z3→Z4=5, Z4→Z5=4, Z5→Z6=5
        let expected_per_dim: [u32; 6] = [5, 4, 5, 5, 4, 5];
        for zoom in 0..HUBBLE_ZOOM_MAX {
            let t = najaf_tile(zoom);
            let n = expected_per_dim[zoom as usize];
            let kids = t.children();
            assert_eq!(
                kids.len() as u32,
                n * n,
                "z{zoom}: expected {}×{}={} children, got {}",
                n,
                n,
                n * n,
                kids.len()
            );
        }
    }

    #[test]
    fn tile_bounds_width_equals_tile_size() {
        for zoom in 0..=HUBBLE_ZOOM_MAX {
            let b = najaf_tile(zoom).bounds();
            let width = b.lon_max - b.lon_min;
            let size = TILE_SIZES_DEG[zoom as usize];
            assert!(
                (width - size).abs() < 0.0001,
                "z{zoom}: width {width} ≠ size {size}"
            );
        }
    }

    #[test]
    fn two_distant_coords_land_in_different_tiles_at_z4() {
        let t1 = HubbleTileId::from_coord(31.995, 44.320, 4);
        let t2 = HubbleTileId::from_coord(32.100, 44.500, 4);
        assert_ne!(t1, t2);
    }

    #[test]
    fn two_nearby_coords_share_tile_at_z2() {
        let t1 = HubbleTileId::from_coord(31.994, 44.319, 2);
        let t2 = HubbleTileId::from_coord(31.996, 44.321, 2);
        assert_eq!(t1, t2);
    }

    #[test]
    fn hubble_tile_add_particle_no_duplicates() {
        let mut tile = HubbleTile::new(najaf_tile(4));
        tile.add_particle(1);
        tile.add_particle(1);
        tile.add_particle(2);
        assert_eq!(tile.particle_count(), 2);
    }

    #[test]
    fn contains_particle_zoom_min_check() {
        let tile = HubbleTile::new(najaf_tile(3));
        assert!(tile.contains_particle(0)); // zoom_min=0 visible at Z3
        assert!(tile.contains_particle(3)); // zoom_min=3 visible at Z3
        assert!(!tile.contains_particle(4)); // zoom_min=4 NOT visible at Z3
    }

    #[test]
    fn zoom_for_radius_coarser_for_larger_radius() {
        let z_50m = zoom_for_radius_m(50.0);
        let z_5000m = zoom_for_radius_m(5000.0);
        assert!(z_50m >= z_5000m, "larger radius → coarser zoom");
    }

    #[test]
    fn tiles_in_bbox_covers_najaf_area() {
        // Small bbox around cemetery
        let ids = tiles_in_bbox(31.980, 32.010, 44.300, 44.340, 4);
        assert!(!ids.is_empty());
        let has_najaf = ids.iter().any(|t| t.bounds().contains(31.995, 44.320));
        assert!(has_najaf, "Najaf coord must be covered");
    }

    #[test]
    fn tiles_in_bbox_single_point_at_z6() {
        let ids = tiles_in_bbox(31.9950, 31.9951, 44.3200, 44.3201, 6);
        assert!(!ids.is_empty());
    }

    #[test]
    fn tile_bounds_overlap_adjacent() {
        let t1 = najaf_tile(4);
        let b1 = t1.bounds();
        // Tile to the immediate east
        let t2 = HubbleTileId::from_coord(b1.lat_min, b1.lon_max + 0.0001, 4);
        let b2 = t2.bounds();
        // They share an edge — should NOT overlap in the strict sense
        assert!(!b1.overlaps(&b2), "adjacent tiles must not overlap");
    }
}
