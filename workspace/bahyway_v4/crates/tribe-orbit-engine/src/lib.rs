//! 𒁾 tribe-orbit-engine — Tribal Orbital Mechanics v4.0
//!
//! Implements the J_phys term of the AMMAS master kinetic equation:
//!
//!   J_phys[f] = orbital motion + Rule 7 density sink
//!
//! Sovereign constants:
//!   PARTICLES_PER_TRIBE = 7    (Heptagon)
//!   SHC                = 1/255 (Sovereign Heptagon Constant)
//!   τ_R7               = 11    (Rule 7 density trigger)
//!   resonance_radius   = 0.15  (SPH kernel bandwidth)
//!   cluster_thresh     = 3
//!
//! 7 Tribal Laws priority: P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)
//!
//! 3 density bands: Isolated(0–2) · Cluster(3–10) · Rule7Overflow(≥11)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

#![forbid(unsafe_code)]

pub mod constants;
pub mod density;
pub mod law;
pub mod orbit;

pub use constants::{
    PARTICLES_PER_TRIBE, SHC, TAU_R7, CLUSTER_THRESH,
    RESONANCE_RADIUS, DENSITY_ISOLATED_MAX, DENSITY_CLUSTER_MAX,
    ORBIT_RADIUS_MAX, ORBIT_RADIUS_GEM, LAW_PRIORITY_ORDER,
};
pub use density::{
    DensityBand, sph_kernel, sph_densities,
    neighbourhood_counts, classify_density, find_rule7_clusters,
};
pub use law::{TribeLaw, resolve_conflict};
pub use orbit::{OrbitalRing, OrbitalAssignment, ring_from_b11};

/// High-level entry point: compute all orbital assignments for a batch of particles.
///
/// `kaki_batch` — slice of 16-byte KAKI PKs
/// `b11_batch`  — corresponding B11 quality bytes (same length)
///
/// Returns `Vec<OrbitalAssignment>` in the same order.
pub fn assign_orbits(
    kaki_batch: &[[u8; 16]],
    b11_batch:  &[u8],
) -> Vec<OrbitalAssignment> {
    assert_eq!(kaki_batch.len(), b11_batch.len(), "kaki and b11 slices must be same length");
    kaki_batch.iter()
        .zip(b11_batch.iter())
        .map(|(kaki, &b11)| OrbitalAssignment::from_kaki(kaki, b11))
        .collect()
}

/// Compute density bands for all particles given their 3D positions.
///
/// Returns `(DensityBand per particle, Rule7 cluster groups)`.
pub fn analyse_density(
    positions: &[[f32; 3]],
) -> (Vec<DensityBand>, Vec<Vec<usize>>) {
    let bands    = classify_density(positions);
    let clusters = find_rule7_clusters(positions);
    (bands, clusters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_orbits_batch_same_length() {
        let kakis = vec![[0u8; 16]; 5];
        let b11s  = vec![240u8, 180, 120, 70, 30];
        let assignments = assign_orbits(&kakis, &b11s);
        assert_eq!(assignments.len(), 5);
        assert_eq!(assignments[0].ring, OrbitalRing::Inner);
        assert_eq!(assignments[4].ring, OrbitalRing::Outer);
    }

    #[test]
    fn analyse_density_sparse_no_clusters() {
        let positions: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let (bands, clusters) = analyse_density(&positions);
        assert_eq!(bands.len(), 5);
        assert!(clusters.is_empty());
    }

    #[test]
    fn sovereign_constants_unchanged() {
        assert_eq!(PARTICLES_PER_TRIBE, 7);
        assert_eq!(TAU_R7, 11);
        assert_eq!(CLUSTER_THRESH, 3);
        assert!((SHC - 1.0 / 255.0).abs() < 1e-7);
        assert!((RESONANCE_RADIUS - 0.15).abs() < 1e-6);
    }
}
