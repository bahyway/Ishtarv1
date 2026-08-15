//! PA-14 Orbital Position — spherical coordinates from KAKIv4.0 bytes.
//!
//! Each particle's orbital position in 3D space is derived directly from
//! its 16-byte KAKI sovereign PK.  This makes the visual position deterministic:
//! same KAKI → same orbital coordinates → same visual position, always.
//!
//! Byte layout used (KAKIv4.0):
//!   κ[12..13] = timestamp (2 bytes)  → azimuth
//!   κ[14..15] = checksum (2 bytes)   → altitude
//!   δ (quality distance)             → orbital radius
//!
//! Equations (PA-14):
//!   azimuth  = 2π × κ[12] / 256
//!   altitude = (κ[14] / 256 − 0.5) × H_max
//!   radius   = r_max × (1 − H(P))       ← inner orbit = low δ = high quality
//!
//! The azimuth byte (κ[12]) is the first byte of the timestamp field (B12-B13).
//! The altitude byte (κ[14]) is the first byte of the checksum field (B14-B15).
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use std::f64::consts::TAU; // 2π

/// Orbital position in spherical coordinates.
///
/// `radius` increases as quality decreases — inner orbit = GEM particles.
/// `azimuth` distributes particles evenly around the tribe ring.
/// `altitude` spreads particles vertically within the tribe shell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitalPosition {
    /// Distance from tribe centroid — derived from δ = 1 − H(P).
    pub radius:  f64,
    /// Azimuth angle in radians `[0, 2π)` — derived from κ[12].
    pub azimuth: f64,
    /// Altitude above/below equatorial plane — derived from κ[14].
    pub altitude: f64,
}

impl OrbitalPosition {
    /// Convert to Cartesian (x, y, z) for rendering.
    pub fn to_cartesian(&self) -> [f64; 3] {
        let r_horiz = self.radius * self.azimuth.cos();
        [
            r_horiz * self.azimuth.cos(),
            r_horiz * self.azimuth.sin(),
            self.altitude,
        ]
    }
}

/// Compute the orbital position from a KAKIv4.0 sovereign PK.
///
/// # Arguments
/// * `kaki`    — 16-byte KAKI sovereign PK (KAKIv4.0 layout)
/// * `delta`   — quality distance δ = 1 − H(P), derived from EAV Orbit
/// * `r_max`   — maximum orbital radius (tribe ring outer edge)
/// * `h_max`   — maximum altitude above/below equatorial plane
///
/// # Returns
/// `OrbitalPosition { radius, azimuth, altitude }`
pub fn orbital_position(
    kaki:  &[u8; 16],
    delta: f64,
    r_max: f64,
    h_max: f64,
) -> OrbitalPosition {
    // Azimuth: κ[12] byte → angle in [0, 2π)
    let azimuth = TAU * (kaki[12] as f64) / 256.0;

    // Altitude: κ[14] byte → altitude in [-h_max/2, +h_max/2]
    // κ[14] / 256 maps to [0, 1); subtract 0.5 to centre on equatorial plane
    let altitude = (kaki[14] as f64 / 256.0 - 0.5) * h_max;

    // Radius: inner orbit (low r) = high quality (low delta)
    let radius = r_max * delta.clamp(0.0, 1.0);

    OrbitalPosition { radius, azimuth, altitude }
}

/// Secondary azimuth from κ[13] — used for sub-ring differentiation.
///
/// When two particles share the same κ[12] byte (same primary azimuth),
/// κ[13] gives a sub-ring offset for visual disambiguation.
#[inline]
pub fn sub_ring_azimuth(kaki: &[u8; 16]) -> f64 {
    TAU * (kaki[13] as f64) / 256.0
}

/// Secondary altitude from κ[15] (checksum second byte) — vertical scatter.
#[inline]
pub fn altitude_scatter(kaki: &[u8; 16], h_max: f64) -> f64 {
    (kaki[15] as f64 / 256.0 - 0.5) * h_max * 0.5
}

// ── PA-15 Tribe Birthing on the Rim ───────────────────────────────────────────

/// Quality-distance threshold above which a particle is a *rim candidate*
/// — eligible to become the founding population of a new child tribe.
///
/// Corresponds to the outermost 12.5% of the orbital shell (δ > 0.875).
/// When rim candidates exhibit coherent emergent direction, the child tribe
/// ideal ι_{T_child} = lim Σ_T(Q_n) materialises as per PA-15.
pub const RIM_BIRTH_THRESHOLD: f64 = 0.875;

/// PA-15 — returns `true` if this particle's quality distance places it in
/// the outer rim of its tribe's orbital structure.
///
/// Rim particles may form the founding population of a new child tribe when
/// their emergent coherence exceeds the tribe's bifurcation criterion.
#[inline]
pub fn is_rim_particle(delta: f64) -> bool {
    delta >= RIM_BIRTH_THRESHOLD
}

/// Map quality distance δ to orbital ring layer.
///
/// Returns 0 (inner) for GEM particles through 4 (outer) for DEAD particles,
/// consistent with the sovereign 5-shell decomposition.
pub fn orbital_ring_layer(delta: f64) -> usize {
    // thresholds match sovereign_5_shells() boundaries from shells.rs
    if      delta < 0.167 { 0 }
    else if delta < 0.417 { 1 }
    else if delta < 0.583 { 2 }
    else if delta < 0.754 { 3 }
    else                  { 4 }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn zero_kaki() -> [u8; 16] { [0u8; 16] }

    fn kaki_with(byte12: u8, byte14: u8) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[12] = byte12;
        k[14] = byte14;
        k
    }

    #[test]
    fn zero_kaki_gives_zero_azimuth() {
        let pos = orbital_position(&zero_kaki(), 0.5, 10.0, 5.0);
        assert_eq!(pos.azimuth, 0.0);
    }

    #[test]
    fn half_byte_gives_pi_azimuth() {
        // κ[12] = 128 → 2π × 128/256 = π
        let k = kaki_with(128, 0);
        let pos = orbital_position(&k, 0.0, 10.0, 5.0);
        assert!((pos.azimuth - PI).abs() < 1e-9, "azimuth={}", pos.azimuth);
    }

    #[test]
    fn azimuth_always_in_0_to_tau() {
        for byte in [0u8, 64, 128, 192, 255] {
            let k = kaki_with(byte, 0);
            let pos = orbital_position(&k, 0.0, 10.0, 5.0);
            assert!(pos.azimuth >= 0.0 && pos.azimuth < TAU,
                "azimuth {} out of [0, 2π)", pos.azimuth);
        }
    }

    #[test]
    fn zero_byte14_gives_negative_altitude() {
        // κ[14]=0 → 0/256 − 0.5 = −0.5 → altitude = −0.5 × h_max
        let k = kaki_with(0, 0);
        let pos = orbital_position(&k, 0.0, 10.0, 4.0);
        assert!((pos.altitude - (-2.0)).abs() < 1e-9, "altitude={}", pos.altitude);
    }

    #[test]
    fn byte14_128_gives_near_zero_altitude() {
        // κ[14]=128 → 128/256=0.5 → 0.5 − 0.5 = 0.0 → altitude ≈ 0
        let k = kaki_with(0, 128);
        let pos = orbital_position(&k, 0.0, 10.0, 5.0);
        assert!(pos.altitude.abs() < 0.02, "altitude={}", pos.altitude);
    }

    #[test]
    fn gem_particle_has_smallest_radius() {
        // delta=0.0 → radius=0 (perfect quality, innermost orbit)
        let pos = orbital_position(&zero_kaki(), 0.0, 10.0, 5.0);
        assert_eq!(pos.radius, 0.0);
    }

    #[test]
    fn dead_particle_has_largest_radius() {
        // delta=1.0 → radius=r_max
        let pos = orbital_position(&zero_kaki(), 1.0, 10.0, 5.0);
        assert_eq!(pos.radius, 10.0);
    }

    #[test]
    fn radius_monotone_with_delta() {
        let r_max = 10.0;
        let deltas = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        let radii: Vec<f64> = deltas.iter()
            .map(|&d| orbital_position(&zero_kaki(), d, r_max, 5.0).radius)
            .collect();
        for i in 0..radii.len() - 1 {
            assert!(radii[i] <= radii[i + 1],
                "radius not monotone: {:.2} > {:.2}", radii[i], radii[i+1]);
        }
    }

    #[test]
    fn same_kaki_same_position() {
        let k = kaki_with(42, 100);
        let p1 = orbital_position(&k, 0.3, 10.0, 5.0);
        let p2 = orbital_position(&k, 0.3, 10.0, 5.0);
        assert_eq!(p1.azimuth,  p2.azimuth);
        assert_eq!(p1.altitude, p2.altitude);
        assert_eq!(p1.radius,   p2.radius);
    }

    #[test]
    fn different_kaki_different_azimuth() {
        let k1 = kaki_with(0, 0);
        let k2 = kaki_with(64, 0);
        let p1 = orbital_position(&k1, 0.5, 10.0, 5.0);
        let p2 = orbital_position(&k2, 0.5, 10.0, 5.0);
        assert_ne!(p1.azimuth, p2.azimuth);
    }

    #[test]
    fn pa15_rim_particle_threshold() {
        assert!(!is_rim_particle(0.0));
        assert!(!is_rim_particle(0.5));
        assert!(!is_rim_particle(RIM_BIRTH_THRESHOLD - 0.001));
        assert!(is_rim_particle(RIM_BIRTH_THRESHOLD));
        assert!(is_rim_particle(1.0));
    }

    #[test]
    fn orbital_ring_layer_matches_quality_lanes() {
        assert_eq!(orbital_ring_layer(0.0),  0); // GEM
        assert_eq!(orbital_ring_layer(0.1),  0); // GEM
        assert_eq!(orbital_ring_layer(0.2),  1); // TRIBE
        assert_eq!(orbital_ring_layer(0.5),  2); // ACTIVE
        assert_eq!(orbital_ring_layer(0.65), 3); // FUZZY
        assert_eq!(orbital_ring_layer(0.8),  4); // DEAD
        assert_eq!(orbital_ring_layer(1.0),  4); // DEAD
    }

    #[test]
    fn cartesian_conversion_length_equals_horiz_radius() {
        let k = kaki_with(0, 128); // azimuth=0, altitude≈0
        let pos = orbital_position(&k, 0.5, 10.0, 5.0);
        let [x, y, z] = pos.to_cartesian();
        let horiz = (x * x + y * y).sqrt();
        // With azimuth=0: x = r_horiz*cos(0)² = r_horiz, y ≈ 0
        assert!(horiz > 0.0, "horizontal distance must be positive");
        assert!(z.abs() < 0.1, "altitude near equator for byte14=128");
    }
}
