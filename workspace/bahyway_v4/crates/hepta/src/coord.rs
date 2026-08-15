#![forbid(unsafe_code)]

// ============================================================================
// hepta/src/coord.rs — Coordinate system bridge: WGS84 ↔ KAKI_7D
//
// Pure std math, zero external deps.
// ============================================================================

use core::f64::consts::PI;
use crate::{
    ast::{Distance, DistanceUnit},
    error::HeptaError,
    kaki::{HeptaSector, Kaki7dPk},
};

// ─── Constants ───────────────────────────────────────────────────────────────

/// 360° / 7 ≈ 51.4286° — the fundamental heptagram sector step.
pub const HEPTA_STEP_DEG: f64 = 360.0 / 7.0;
pub const HEPTA_STEP_RAD: f64 = 2.0 * PI / 7.0;

// ─── Heptagram Geometry ──────────────────────────────────────────────────────

/// (x, y) of the i-th vertex of a unit heptagram centred at (cx, cy).
/// Vertex 0 is at the top (north, −π/2 rotation).
pub fn heptagram_vertex(index: u8, radius: f64, cx: f64, cy: f64) -> (f64, f64) {
    let angle = HEPTA_STEP_RAD * (index as f64) - PI / 2.0;
    (cx + radius * angle.cos(), cy + radius * angle.sin())
}

/// Chord length for a {7/k} chord on a circle of radius R.
///   length = 2 R sin(k π / 7)
pub fn chord_length(radius: f64, skip: u8) -> f64 {
    2.0 * radius * (skip as f64 * PI / 7.0).sin()
}

/// Perimeter segment length (the {7/1} edge).
pub fn perimeter_segment(radius: f64) -> f64 {
    chord_length(radius, 1)
}

/// Angular granularity advantage of Hepta vs H3 hexagons.
///   H3 sector angle:    360° / 6 = 60°
///   Hepta sector angle: 360° / 7 ≈ 51.43°
///   Improvement: (60 − 51.43) / 60 ≈ 14.3%
pub fn angular_granularity_advantage() -> f64 {
    let h3_sector    = 360.0 / 6.0;
    let hepta_sector = HEPTA_STEP_DEG;
    (h3_sector - hepta_sector) / h3_sector
}

/// Path reduction for a {7/k} chord compared to walking k perimeter segments.
///   savings = (k × segment − chord) / (k × segment)
pub fn chord_path_reduction(skip: u8, radius: f64) -> f64 {
    let seg   = perimeter_segment(radius);
    let chord = chord_length(radius, skip);
    let walk  = skip as f64 * seg;
    (walk - chord) / walk
}

/// Mean path reduction in a heptagram using the average of {7/2} and {7/3}.
pub fn mean_hepta_advantage(radius: f64) -> f64 {
    (chord_path_reduction(2, radius) + chord_path_reduction(3, radius)) / 2.0
}

// ─── WGS84 → Sector Assignment ───────────────────────────────────────────────

/// Return which of the 7 primary sectors a point falls into (0=SUN/core).
pub fn assign_sector(
    anchor_lon: f64, anchor_lat: f64,
    point_lon:  f64, point_lat:  f64,
) -> u8 {
    let bearing = bearing_deg(anchor_lon, anchor_lat, point_lon, point_lat);
    (bearing / HEPTA_STEP_DEG).floor() as u8 % 7
}

/// Forward bearing from (lon1, lat1) → (lon2, lat2), degrees [0, 360).
fn bearing_deg(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let lat1_r = lat1.to_radians();
    let lat2_r = lat2.to_radians();
    let dlon_r = (lon2 - lon1).to_radians();
    let y = dlon_r.sin() * lat2_r.cos();
    let x = lat1_r.cos() * lat2_r.sin()
          - lat1_r.sin() * lat2_r.cos() * dlon_r.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

/// Great-circle distance between two WGS84 points in metres (Haversine).
pub fn haversine_m(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
          + lat1.to_radians().cos() * lat2.to_radians().cos()
          * (dlon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

/// Hyperbolic density depth (0–63) given distance from anchor.
///   63 = at the core, 0 = at or beyond the perimeter.
pub fn density_from_distance(distance_m: f64, radius_m: f64) -> u8 {
    if radius_m <= 0.0 { return 0; }
    let norm = (1.0 - (distance_m / radius_m).min(1.0)).max(0.0);
    (norm * 63.0).round() as u8
}

// ─── Full WGS84 → Kaki7dPk conversion ───────────────────────────────────────

/// Build a 16-byte KAKI_7D sovereign spatial key for a WGS84 coordinate.
pub fn wgs84_to_kaki7d(
    lon:        f64,
    lat:        f64,
    elev_m:     f64,
    epoch_sec:  u64,
    tier:       u8,
    anchor_lon: f64,
    anchor_lat: f64,
    radius_m:   f64,
    project_id: &str,
) -> Result<Kaki7dPk, HeptaError> {
    let primary = assign_sector(anchor_lon, anchor_lat, lon, lat);
    let dist    = haversine_m(anchor_lon, anchor_lat, lon, lat);
    let density = density_from_distance(dist, radius_m);
    let sector  = HeptaSector::new(primary, 0, 0)?;
    Kaki7dPk::new(lon, lat, elev_m, epoch_sec, tier, sector, density, project_id)
}

/// Convert a `Distance` AST node to metres.
pub fn distance_to_metres(d: &Distance) -> f64 {
    match d.unit {
        DistanceUnit::Km => d.value * 1_000.0,
        DistanceUnit::M  => d.value,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hepta_step_is_correct() {
        assert!((HEPTA_STEP_DEG - 51.4285714).abs() < 0.0001);
    }

    #[test]
    fn angular_advantage_over_h3() {
        let adv = angular_granularity_advantage();
        assert!((adv - 0.142857).abs() < 0.001, "angular advantage = {:.4}", adv);
    }

    #[test]
    fn chord_72_shorter_than_walk() {
        let r = 15_000.0;
        let reduction = chord_path_reduction(2, r);
        assert!(reduction > 0.0 && reduction < 1.0);
    }

    #[test]
    fn chord_73_longer_than_chord_72() {
        let r = 15_000.0;
        assert!(chord_length(r, 3) > chord_length(r, 2));
    }

    #[test]
    fn sector_assignment_north_is_zero() {
        let s0 = assign_sector(44.36, 33.33, 44.36, 34.33);
        assert_eq!(s0, 0, "point directly north → sector 0 (SUN)");
    }

    #[test]
    fn haversine_baghdad_to_najaf() {
        let d = haversine_m(44.40, 33.34, 44.31, 31.99);
        assert!(d > 140_000.0 && d < 155_000.0, "Baghdad–Najaf = {:.0} m", d);
    }

    #[test]
    fn density_at_centre_is_max() {
        assert_eq!(density_from_distance(0.0, 15_000.0), 63);
    }

    #[test]
    fn density_at_perimeter_is_zero() {
        assert_eq!(density_from_distance(15_000.0, 15_000.0), 0);
    }

    #[test]
    fn density_beyond_perimeter_clamps_to_zero() {
        assert_eq!(density_from_distance(30_000.0, 15_000.0), 0);
    }

    #[test]
    fn wgs84_to_kaki7d_produces_valid_pk() {
        let pk = wgs84_to_kaki7d(
            44.3148, 31.9957, // Najaf lon, lat
            30.0,             // elev_m
            1_700_000_000,    // epoch
            1,                // tier
            44.3148, 31.9957, // anchor = same point
            500.0,            // radius_m
            "NajafWay",
        ).unwrap();
        assert_eq!(pk.as_bytes().len(), 16);
    }

    #[test]
    fn heptagram_vertex_zero_is_at_top() {
        let (x, y) = heptagram_vertex(0, 1.0, 0.0, 0.0);
        assert!(x.abs() < 1e-9, "vertex 0 x ≈ 0");
        assert!(y < 0.0, "vertex 0 y < 0 (above centre in screen coords)");
    }

    #[test]
    fn mean_hepta_advantage_is_positive() {
        let adv = mean_hepta_advantage(15_000.0);
        assert!(adv > 0.0 && adv < 1.0);
    }
}
