//! Orbital ring assignment — maps QualityLane to 3-ring torus geometry.
//!
//! Each tribe has 3 orbital rings:
//!   Ring 0 (inner)  — GEM particles  (radius ≈ 1.0)
//!   Ring 1 (mid)    — TRIBE/ACTIVE   (radius ≈ 2.2)
//!   Ring 2 (outer)  — FUZZY/DEAD     (radius ≈ 3.5)
//!
//! Within a ring, particles are distributed by azimuth (from κ[12])
//! and altitude (from κ[14]), exactly as PA-14 specifies.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::constants::{ORBIT_RADIUS_GEM, ORBIT_RADIUS_MAX};

/// The three orbital rings of a tribe torus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitalRing {
    Inner,  // Ring 0: GEM
    Mid,    // Ring 1: TRIBE + ACTIVE
    Outer,  // Ring 2: FUZZY + DEAD
}

impl OrbitalRing {
    /// Central radius of this ring.
    pub fn base_radius(self) -> f32 {
        match self {
            Self::Inner => ORBIT_RADIUS_GEM,        // 1.0
            Self::Mid   => ORBIT_RADIUS_GEM + 1.2,  // 2.2
            Self::Outer => ORBIT_RADIUS_MAX  - 0.5, // 3.5
        }
    }

    /// Width of the ring band (radius variation within the ring).
    pub fn band_width(self) -> f32 {
        match self {
            Self::Inner => 0.3,
            Self::Mid   => 0.5,
            Self::Outer => 0.8,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Inner => "INNER",
            Self::Mid   => "MID",
            Self::Outer => "OUTER",
        }
    }
}

/// Assign an orbital ring from a B11 quality byte.
///
/// B11 ≥ 200 → Inner (GEM)
/// B11 ≥ 100 → Mid  (TRIBE + ACTIVE)
/// B11 <  100 → Outer (FUZZY + DEAD)
pub fn ring_from_b11(b11: u8) -> OrbitalRing {
    if      b11 >= 200 { OrbitalRing::Inner }
    else if b11 >= 100 { OrbitalRing::Mid   }
    else               { OrbitalRing::Outer  }
}

/// Full orbital assignment for one particle.
#[derive(Debug, Clone, Copy)]
pub struct OrbitalAssignment {
    /// Assigned ring.
    pub ring:     OrbitalRing,
    /// Orbital radius (ring base ± δ from quality).
    pub radius:   f32,
    /// Azimuth in radians [0, 2π) — from κ[12].
    pub azimuth:  f32,
    /// Altitude above/below equatorial plane — from κ[14].
    pub altitude: f32,
}

impl OrbitalAssignment {
    /// Derive the full orbital assignment from KAKI bytes and B11.
    ///
    /// κ[12] → azimuth, κ[14] → altitude, B11 → ring + radius.
    pub fn from_kaki(kaki: &[u8; 16], b11: u8) -> Self {
        use std::f32::consts::TAU;

        let ring    = ring_from_b11(b11);
        let delta   = 1.0 - (b11 as f32 / 240.0); // quality distance
        let radius  = ring.base_radius() + delta * ring.band_width();
        let azimuth = TAU * (kaki[12] as f32) / 256.0;
        let altitude = (kaki[14] as f32 / 256.0 - 0.5) * 1.0; // ±0.5 max

        Self { ring, radius, azimuth, altitude }
    }

    /// Cartesian position (x, y, z) for renderer.
    pub fn to_cartesian(&self) -> [f32; 3] {
        let x = self.radius * self.azimuth.cos();
        let y = self.radius * self.azimuth.sin();
        let z = self.altitude;
        [x, y, z]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gem_b11_240_gives_inner_ring() {
        assert_eq!(ring_from_b11(240), OrbitalRing::Inner);
        assert_eq!(ring_from_b11(200), OrbitalRing::Inner);
    }

    #[test]
    fn tribe_b11_gives_mid_ring() {
        assert_eq!(ring_from_b11(199), OrbitalRing::Mid);
        assert_eq!(ring_from_b11(140), OrbitalRing::Mid);
        assert_eq!(ring_from_b11(100), OrbitalRing::Mid);
    }

    #[test]
    fn fuzzy_dead_b11_gives_outer_ring() {
        assert_eq!(ring_from_b11(99), OrbitalRing::Outer);
        assert_eq!(ring_from_b11(0),  OrbitalRing::Outer);
    }

    #[test]
    fn inner_ring_smallest_radius() {
        assert!(OrbitalRing::Inner.base_radius() < OrbitalRing::Mid.base_radius());
        assert!(OrbitalRing::Mid.base_radius()   < OrbitalRing::Outer.base_radius());
    }

    #[test]
    fn same_kaki_same_orbital_assignment() {
        let kaki = [0x01u8, 0, 0, 0, 0, 1, 2, 1, 0, 0, 0, 0, 42, 0, 128, 0];
        let a1 = OrbitalAssignment::from_kaki(&kaki, 210);
        let a2 = OrbitalAssignment::from_kaki(&kaki, 210);
        assert_eq!(a1.ring, a2.ring);
        assert!((a1.azimuth - a2.azimuth).abs() < 1e-6);
        assert!((a1.altitude - a2.altitude).abs() < 1e-6);
    }

    #[test]
    fn cartesian_uses_radius_and_azimuth() {
        let kaki = [0u8; 16]; // azimuth = 0 (κ[12]=0), altitude ≈ -0.5 (κ[14]=0)
        let a = OrbitalAssignment::from_kaki(&kaki, 240);
        let [x, y, _z] = a.to_cartesian();
        // azimuth=0 → cos(0)=1, sin(0)=0
        assert!((x - a.radius).abs() < 1e-5, "x={x}, radius={}", a.radius);
        assert!(y.abs() < 1e-5, "y={y}");
    }

    #[test]
    fn gem_particle_radius_near_inner_base() {
        let kaki = [0u8; 16];
        let a = OrbitalAssignment::from_kaki(&kaki, 240); // perfect quality
        // delta=0 → radius = base_radius + 0 = 1.0
        assert!((a.radius - OrbitalRing::Inner.base_radius()).abs() < 0.01);
    }
}
