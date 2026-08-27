//! OrbitalSnapshot — a point-in-time capture of a particle's orbital state.
//!
//! Snapshots are taken after every successful score-engine pass. The probe
//! compares two consecutive snapshots to detect unexplained motion.

use fuzzy_engine::QualityTier;
use tribe_orbit_engine::{OrbitalAssignment, OrbitalRing};

/// Immutable orbital state captured after one scoring cycle.
#[derive(Debug, Clone)]
pub struct OrbitalSnapshot {
    /// Epoch at which this snapshot was taken.
    pub epoch: u64,
    /// B11 quality byte that drove ring assignment.
    pub b11: u8,
    /// Quality tier at snapshot time.
    pub tier: QualityTier,
    /// Orbital ring assigned.
    pub ring: OrbitalRing,
    /// Full orbital coordinates.
    pub assignment: OrbitalAssignment,
    /// SPH neighbour count at snapshot time.
    pub neighbour_count: usize,
    /// Freshness BLUE byte at snapshot time (0–255, maps from 0.0–1.0).
    pub freshness_byte: u8,
    /// FNV-1a hash of the rules.rs evaluate_rules output constants — used for
    /// rule-change detection in Step 1. Computed once at snapshot time.
    pub rules_fingerprint: u64,
}

impl OrbitalSnapshot {
    pub fn new(
        epoch: u64,
        b11: u8,
        tier: QualityTier,
        assignment: OrbitalAssignment,
        neighbour_count: usize,
        freshness_byte: u8,
        rules_fingerprint: u64,
    ) -> Self {
        Self {
            epoch,
            b11,
            tier,
            ring: assignment.ring,
            assignment,
            neighbour_count,
            freshness_byte,
            rules_fingerprint,
        }
    }

    /// Euclidean distance between two orbital Cartesian positions.
    pub fn cartesian_distance(&self, other: &Self) -> f32 {
        let [x1, y1, z1] = self.assignment.to_cartesian();
        let [x2, y2, z2] = other.assignment.to_cartesian();
        let dx = x1 - x2;
        let dy = y1 - y2;
        let dz = z1 - z2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// True when both snapshots are assigned to the same orbital ring.
    pub fn same_ring(&self, other: &Self) -> bool {
        self.ring == other.ring
    }
}

/// The B11 hysteresis band around ring boundaries.
/// A particle within ±HYSTERESIS of a boundary is in "noise territory".
///
/// Boundaries: Inner/Mid = 200, Mid/Outer = 100.
pub const HYSTERESIS: u8 = 5;

/// Returns true if `b11` is within the hysteresis band of any ring boundary.
pub fn near_boundary(b11: u8) -> bool {
    b11.abs_diff(200) <= HYSTERESIS || b11.abs_diff(100) <= HYSTERESIS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_boundary_inner_mid() {
        assert!(near_boundary(200));
        assert!(near_boundary(195));
        assert!(near_boundary(205));
        assert!(!near_boundary(194));
    }

    #[test]
    fn near_boundary_mid_outer() {
        assert!(near_boundary(100));
        assert!(near_boundary(97));
        assert!(near_boundary(103));
        assert!(!near_boundary(94));
    }

    #[test]
    fn away_from_boundary_not_near() {
        assert!(!near_boundary(240)); // deep Gem
        assert!(!near_boundary(150)); // mid Tribe
        assert!(!near_boundary(50)); // deep Dead
    }
}
