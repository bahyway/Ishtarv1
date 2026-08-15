//! Index 5 — ColorID Spatial Index (§9.3, §4.5)
//! D3/D4/D5 (color_rgb bytes) + radial position → particle set.
//! Powers ColorID drift diagnostic queries for AlertEngine and DubSar IDE.
//!
//! Structure: hash map storing (r, g, b, radial) 4-tuples, queried by
//! bounding-box scan. Tree-free -- "R-tree variant" was aspirational
//! language from an earlier draft that never matched this implementation;
//! there is no tree here today, and per "no tree in the Orbits" (PB-182,
//! 2026-07-13), a future spatial structure for this index should not be
//! one either.

use std::collections::HashMap;

/// A 4-dimensional ColorID spatial point: RED (domain), GREEN (quality),
/// BLUE (freshness), RADIAL (distance from tribe nucleus in orbital space).
#[derive(Clone, Debug, PartialEq)]
pub struct ColorIdPoint {
    pub r:      u8,
    pub g:      u8,
    pub b:      u8,
    pub radial: f32,
}

impl ColorIdPoint {
    pub fn new(r: u8, g: u8, b: u8, radial: f32) -> Self {
        ColorIdPoint { r, g, b, radial }
    }

    /// Distance from the "perfect golden" point (255, 255, 255, 0.0).
    /// Used for drift detection: larger = more degraded.
    pub fn drift_distance(&self) -> f32 {
        let dr = (255.0 - self.r as f32).powi(2);
        let dg = (255.0 - self.g as f32).powi(2);
        let db = (255.0 - self.b as f32).powi(2);
        (dr + dg + db + self.radial.powi(2)).sqrt()
    }
}

/// ColorID spatial index: uuid_hash → ColorIdPoint.
pub struct ColorIdIndex {
    entries: HashMap<u32, ColorIdPoint>,
}

impl ColorIdIndex {
    pub fn new() -> Self {
        ColorIdIndex { entries: HashMap::new() }
    }

    pub fn upsert(&mut self, uuid_hash: u32, point: ColorIdPoint) {
        self.entries.insert(uuid_hash, point);
    }

    /// Return all particles whose drift_distance exceeds the threshold.
    /// Used by AlertEngine for early-warning detection (§4.5).
    pub fn query_drift_exceeding(&self, threshold: f32) -> Vec<(u32, f32)> {
        self.entries
            .iter()
            .map(|(&h, p)| (h, p.drift_distance()))
            .filter(|&(_, d)| d > threshold)
            .collect()
    }

    /// Return all particles in a bounding box of the color space.
    pub fn query_bbox(
        &self,
        r_min: u8, r_max: u8,
        g_min: u8, g_max: u8,
        b_min: u8, b_max: u8,
    ) -> Vec<u32> {
        self.entries
            .iter()
            .filter(|(_, p)| {
                p.r >= r_min && p.r <= r_max &&
                p.g >= g_min && p.g <= g_max &&
                p.b >= b_min && p.b <= b_max
            })
            .map(|(&h, _)| h)
            .collect()
    }
}

impl Default for ColorIdIndex { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_distance_golden() {
        let p = ColorIdPoint::new(255, 255, 255, 0.0);
        assert!((p.drift_distance() - 0.0).abs() < 0.001);
    }

    #[test]
    fn drift_detection() {
        let mut idx = ColorIdIndex::new();
        idx.upsert(0x0001, ColorIdPoint::new(255, 255, 255, 0.0)); // golden
        idx.upsert(0x0002, ColorIdPoint::new(100, 100, 100, 50.0)); // degraded
        let drifters = idx.query_drift_exceeding(100.0);
        assert_eq!(drifters.len(), 1);
        assert_eq!(drifters[0].0, 0x0002);
    }

    #[test]
    fn bbox_query() {
        let mut idx = ColorIdIndex::new();
        idx.upsert(0x0001, ColorIdPoint::new(200, 200, 200, 0.0));
        idx.upsert(0x0002, ColorIdPoint::new(50,  50,  50,  0.0));
        let results = idx.query_bbox(100, 255, 100, 255, 100, 255);
        assert!(results.contains(&0x0001));
        assert!(!results.contains(&0x0002));
    }
}
