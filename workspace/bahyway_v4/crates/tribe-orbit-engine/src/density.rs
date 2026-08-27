//! SPH Density Field — Smoothed-Particle Hydrodynamics for tribe clusters.
//!
//! Each particle contributes a cubic SPH kernel to its neighbourhood density.
//! The density field drives Rule 7 (overflow sink) and visual cluster detection.
//!
//! Three density bands:
//!   Isolated     0–2  neighbours within resonance_radius
//!   Cluster      3–10
//!   Rule7Overflow ≥11 → triggers Rule 7 (P07)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::constants::{CLUSTER_THRESH, RESONANCE_RADIUS, TAU_R7};

/// Density band of a particle based on its neighbourhood count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityBand {
    /// 0–2 neighbours — particle is isolated.
    Isolated,
    /// 3–10 neighbours — forming a cluster.
    Cluster,
    /// ≥ 11 neighbours — Rule 7 overflow sink fires.
    Rule7Overflow,
}

impl DensityBand {
    pub fn from_count(count: usize) -> Self {
        if count >= TAU_R7 {
            Self::Rule7Overflow
        } else if count >= CLUSTER_THRESH {
            Self::Cluster
        } else {
            Self::Isolated
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Isolated => "ISOLATED",
            Self::Cluster => "CLUSTER",
            Self::Rule7Overflow => "RULE7_OVERFLOW",
        }
    }
}

/// Cubic SPH kernel W(r, h).
///
/// Returns the kernel weight for a particle at distance `r` from the
/// evaluation point, with smoothing length `h = resonance_radius`.
/// Returns 0.0 for r ≥ h (outside the kernel support).
#[inline]
pub fn sph_kernel(r: f32, h: f32) -> f32 {
    if r >= h || h <= 0.0 {
        return 0.0;
    }
    let q = r / h;
    let base = 1.0 - q;
    base * base * base
}

/// Compute the neighbourhood particle count for each particle in `positions`.
///
/// Two particles are neighbours if their 3D Euclidean distance < `resonance_radius`.
/// The particle itself is excluded from its own count.
///
/// Returns `Vec<usize>` where `counts[i]` = number of neighbours of particle `i`.
pub fn neighbourhood_counts(positions: &[[f32; 3]]) -> Vec<usize> {
    let r2_max = RESONANCE_RADIUS * RESONANCE_RADIUS;

    positions
        .iter()
        .enumerate()
        .map(|(i, pi)| {
            positions
                .iter()
                .enumerate()
                .filter(|(j, pj)| {
                    if *j == i {
                        return false;
                    }
                    let dx = pi[0] - pj[0];
                    let dy = pi[1] - pj[1];
                    let dz = pi[2] - pj[2];
                    dx * dx + dy * dy + dz * dz < r2_max
                })
                .count()
        })
        .collect()
}

/// Compute SPH density for each particle.
///
/// `density[i]` = Σⱼ W(|posᵢ − posⱼ|, h) for all j ≠ i.
pub fn sph_densities(positions: &[[f32; 3]]) -> Vec<f32> {
    let h = RESONANCE_RADIUS;

    positions
        .iter()
        .enumerate()
        .map(|(i, pi)| {
            let mut rho = 0.0f32;
            for (j, pj) in positions.iter().enumerate() {
                if j == i {
                    continue;
                }
                let dx = pi[0] - pj[0];
                let dy = pi[1] - pj[1];
                let dz = pi[2] - pj[2];
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                rho += sph_kernel(r, h);
            }
            rho
        })
        .collect()
}

/// Classify each particle into its density band.
pub fn classify_density(positions: &[[f32; 3]]) -> Vec<DensityBand> {
    neighbourhood_counts(positions)
        .into_iter()
        .map(DensityBand::from_count)
        .collect()
}

/// Identify particle indices that form a Rule 7 overflow cluster.
///
/// Returns groups of particle indices where density ≥ τ_R7=11.
pub fn find_rule7_clusters(positions: &[[f32; 3]]) -> Vec<Vec<usize>> {
    let counts = neighbourhood_counts(positions);
    let overflow: Vec<usize> = counts
        .iter()
        .enumerate()
        .filter(|(_, &c)| c >= TAU_R7)
        .map(|(i, _)| i)
        .collect();

    if overflow.is_empty() {
        return Vec::new();
    }

    // Group into connected components (within resonance_radius)
    let r2_max = RESONANCE_RADIUS * RESONANCE_RADIUS;
    let mut visited = vec![false; overflow.len()];
    let mut clusters = Vec::new();

    for start in 0..overflow.len() {
        if visited[start] {
            continue;
        }
        let mut cluster = vec![overflow[start]];
        visited[start] = true;

        let mut queue = vec![start];
        while let Some(qi) = queue.pop() {
            let pi = positions[overflow[qi]];
            for next in 0..overflow.len() {
                if visited[next] {
                    continue;
                }
                let pj = positions[overflow[next]];
                let dx = pi[0] - pj[0];
                let dy = pi[1] - pj[1];
                let dz = pi[2] - pj[2];
                if dx * dx + dy * dy + dz * dz < r2_max {
                    visited[next] = true;
                    cluster.push(overflow[next]);
                    queue.push(next);
                }
            }
        }
        clusters.push(cluster);
    }
    clusters
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cluster(n: usize, centre: [f32; 3], jitter: f32) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| {
                let angle = i as f32 * 0.1;
                [
                    centre[0] + angle.cos() * jitter,
                    centre[1] + angle.sin() * jitter,
                    centre[2],
                ]
            })
            .collect()
    }

    #[test]
    fn sph_kernel_zero_outside_support() {
        assert_eq!(sph_kernel(0.2, 0.15), 0.0);
        assert_eq!(sph_kernel(0.15, 0.15), 0.0);
    }

    #[test]
    fn sph_kernel_positive_inside_support() {
        assert!(sph_kernel(0.0, 0.15) > 0.0);
        assert!(sph_kernel(0.05, 0.15) > 0.0);
    }

    #[test]
    fn sph_kernel_decreasing() {
        let h = RESONANCE_RADIUS;
        assert!(sph_kernel(0.0, h) > sph_kernel(0.05, h));
        assert!(sph_kernel(0.05, h) > sph_kernel(0.10, h));
    }

    #[test]
    fn isolated_particle_zero_neighbours() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let counts = neighbourhood_counts(&positions);
        assert_eq!(counts[0], 0);
    }

    #[test]
    fn close_particles_count_each_other() {
        let positions = vec![[0.0f32, 0.0, 0.0], [0.05, 0.0, 0.0]];
        let counts = neighbourhood_counts(&positions);
        assert_eq!(counts[0], 1);
        assert_eq!(counts[1], 1);
    }

    #[test]
    fn density_band_isolated() {
        assert_eq!(DensityBand::from_count(0), DensityBand::Isolated);
        assert_eq!(DensityBand::from_count(2), DensityBand::Isolated);
    }

    #[test]
    fn density_band_cluster() {
        assert_eq!(DensityBand::from_count(3), DensityBand::Cluster);
        assert_eq!(DensityBand::from_count(10), DensityBand::Cluster);
    }

    #[test]
    fn density_band_rule7() {
        assert_eq!(DensityBand::from_count(11), DensityBand::Rule7Overflow);
        assert_eq!(DensityBand::from_count(20), DensityBand::Rule7Overflow);
    }

    #[test]
    fn dense_cluster_triggers_rule7() {
        // 12 particles packed tightly within resonance_radius
        let positions = make_cluster(12, [0.0, 0.0, 0.0], 0.02);
        let bands = classify_density(&positions);
        let overflow_count = bands
            .iter()
            .filter(|&&b| b == DensityBand::Rule7Overflow)
            .count();
        assert!(
            overflow_count > 0,
            "12 particles packed tightly should trigger Rule 7"
        );
    }

    #[test]
    fn sparse_particles_all_isolated() {
        // 5 particles spaced 1.0 apart — far outside resonance_radius=0.15
        let positions: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let bands = classify_density(&positions);
        for band in bands {
            assert_eq!(band, DensityBand::Isolated);
        }
    }

    #[test]
    fn find_rule7_clusters_empty_for_sparse() {
        let positions: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let clusters = find_rule7_clusters(&positions);
        assert!(clusters.is_empty());
    }

    #[test]
    fn tau_r7_is_sovereign() {
        assert_eq!(TAU_R7, 11, "τ_R7 must be 11 — sovereign constant");
    }
}
