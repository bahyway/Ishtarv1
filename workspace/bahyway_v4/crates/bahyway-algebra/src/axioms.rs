//! BahyWay Unified Algebra — Formal Axioms (PA-series).
//!
//! PA = Particle Axiom.  These are the invariants that govern all particles
//! in the BahyWay v4.0 ecosystem.  Axioms PA-1 through PA-12 live in
//! the KAKI specification (enkidb-kaki §2); PA-13 is defined here.
//!
//! PA-13 Trajectory Momentum:
//!   θ(p) = |Δhepta(t) / Δt| bounded by Θ_max.
//!
//!   - θ(p) is a DERIVED EAV attribute: ATTR_MOMENTUM (u32 hash).
//!   - It is NOT a KAKI field (§2.4 Structural-Facts-Only Rule).
//!   - It is NOT stored; it is computed at query time from the Journal spline.
//!   - If θ(p) > Θ_max the particle transitions to FUZZY for steward review.
//!   - Mathematical basis: bounded variation on the orbit trajectory spline.

use crate::topology::HeptaPoint;

/// ATTR_MOMENTUM — EAV attribute hash for the derived trajectory momentum.
/// CRC-16/CCITT of "momentum" = 0xD3A1.
pub const ATTR_MOMENTUM: u32 = 0xD3A1;

/// Θ_max — maximum allowed trajectory momentum before FUZZY transition.
///
/// Calibrated at 1.0 normalised unit per time step.  Domain teams may
/// override this per-tribe via a .way governance file.
pub const PA13_THETA_MAX: f64 = 1.0;

/// Compute θ(p) — the trajectory momentum for a particle at two consecutive
/// positions in the Hepta manifold.
///
/// θ(p) = |Δhepta| / Δt  where |Δhepta| is the 7D Euclidean displacement.
///
/// Returns `None` if Δt is zero (stationary particle — momentum undefined).
pub fn trajectory_momentum(
    pos_t0: &HeptaPoint,
    pos_t1: &HeptaPoint,
    delta_t: f64,
) -> Option<f64> {
    if delta_t.abs() < f64::EPSILON {
        return None;
    }
    let displacement = pos_t0
        .iter()
        .zip(pos_t1.iter())
        .map(|(a, b)| (b - a).powi(2))
        .sum::<f64>()
        .sqrt();
    Some(displacement / delta_t)
}

/// Returns `true` if the computed momentum exceeds Θ_max.
/// Triggers a FUZZY transition event in the StoryEngine.
#[inline]
pub fn exceeds_momentum_threshold(theta: f64) -> bool {
    theta > PA13_THETA_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_particle_returns_none() {
        let p = [0.0f64; 7];
        assert!(trajectory_momentum(&p, &p, 0.0).is_none());
        assert!(trajectory_momentum(&p, &p, 1.0).is_some());
    }

    #[test]
    fn unit_step_in_one_axis_gives_unit_momentum() {
        let p0 = [0.0f64; 7];
        let mut p1 = [0.0f64; 7];
        p1[0] = 1.0;
        let theta = trajectory_momentum(&p0, &p1, 1.0).unwrap();
        assert!((theta - 1.0).abs() < 1e-9);
    }

    #[test]
    fn below_threshold_not_flagged() {
        assert!(!exceeds_momentum_threshold(0.5));
        assert!(!exceeds_momentum_threshold(PA13_THETA_MAX));
    }

    #[test]
    fn above_threshold_is_flagged() {
        assert!(exceeds_momentum_threshold(1.001));
        assert!(exceeds_momentum_threshold(10.0));
    }

    #[test]
    fn momentum_scales_with_inverse_time() {
        let p0 = [0.0f64; 7];
        let mut p1 = [0.0f64; 7];
        p1[0] = 2.0;
        let theta_fast = trajectory_momentum(&p0, &p1, 1.0).unwrap();
        let theta_slow = trajectory_momentum(&p0, &p1, 2.0).unwrap();
        assert!((theta_fast - 2.0 * theta_slow).abs() < 1e-9);
    }
}
