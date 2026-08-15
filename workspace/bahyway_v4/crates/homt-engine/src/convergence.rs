//! HOMT Convergence Conditions and Stability Analysis.
//!
//! Convergence Theorem (PA-∞):
//!   If Ψ ∈ C¹, Φ bounded, step η < 2/L
//!   then lim(t→∞) K_t → K*  where ∇Ψ(K*) = 0.
//!
//! The quality boundaries (0.2 and 0.6) are mathematically grounded as the
//! convergence boundaries of the HOMT fixed-point iteration:
//!   φ < 0.2  → K diverges to ground state (DEAD)
//!   φ ≥ 0.6  → K converges to GOLDEN stable orbit
//!   0.2 ≤ φ < 0.6 → FUZZY (basin of attraction not yet determined)

/// The DEAD boundary — below this density the particle diverges to ground state.
pub const QUALITY_DEAD_BOUNDARY: f64 = 0.2;

/// The GOLDEN boundary — above this density the particle is in a stable golden orbit.
pub const QUALITY_GOLDEN_BOUNDARY: f64 = 0.6;

/// The maximum stable Euler step size for a given Lipschitz constant L.
///
/// η_max = 2 / L  — from the convergence theorem.
pub fn max_stable_step(lipschitz_constant: f64) -> f64 {
    debug_assert!(lipschitz_constant > 0.0, "Lipschitz constant must be positive");
    2.0 / lipschitz_constant
}

/// Default maximum step size assuming L = 2 (conservative estimate).
pub const MAX_STABLE_STEP: f64 = 1.0; // = 2/2

/// Determine if a density value has converged to a stable orbit.
///
/// Returns the convergence basin the density falls into.
pub fn is_converged(density: f64) -> ConvergenceState {
    if density < QUALITY_DEAD_BOUNDARY {
        ConvergenceState::Dead
    } else if density >= QUALITY_GOLDEN_BOUNDARY {
        ConvergenceState::Golden
    } else {
        ConvergenceState::Fuzzy
    }
}

/// Estimate the Lipschitz constant L from a finite set of field samples.
///
/// L = max |f(x) - f(y)| / |x - y|  over all sampled pairs.
/// Returns `None` if fewer than 2 samples are given.
pub fn lipschitz_bound(samples: &[(f64, f64)]) -> Option<f64> {
    if samples.len() < 2 {
        return None;
    }
    let mut l: f64 = 0.0;
    for i in 0..samples.len() {
        for j in (i + 1)..samples.len() {
            let dx = (samples[i].0 - samples[j].0).abs();
            if dx < f64::EPSILON { continue; }
            let dy = (samples[i].1 - samples[j].1).abs();
            l = l.max(dy / dx);
        }
    }
    Some(l)
}

/// The convergence basin of a particle's orbit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceState {
    /// φ ≥ 0.6 — stable golden orbit, ∇Ψ = 0.
    Golden,
    /// 0.2 ≤ φ < 0.6 — undetermined basin of attraction.
    Fuzzy,
    /// φ < 0.2 — diverging to ground state.
    Dead,
}

impl ConvergenceState {
    pub fn as_str(self) -> &'static str {
        match self {
            ConvergenceState::Golden => "GOLDEN",
            ConvergenceState::Fuzzy  => "FUZZY",
            ConvergenceState::Dead   => "DEAD",
        }
    }
}

impl core::fmt::Display for ConvergenceState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Jordan Discrete-Time State Evolution: X(t+1) = J·X(t) + B·U(t)
// ─────────────────────────────────────────────────────────────────────────────

/// A 7×7 sovereign matrix (row-major, f64).
/// Represents the Jordan transition matrix J or the input gain matrix B.
pub type Matrix7 = [[f64; 7]; 7];

/// A 7D state vector — maps to the Hepta-score dimensions of a particle.
pub type StateVec7 = [f64; 7];

/// The 7×7 identity matrix (zero B and identity J = no change).
pub const IDENTITY_7: Matrix7 = [
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
];

/// The 7×7 zero matrix (zero input gain).
pub const ZERO_7: Matrix7 = [[0.0; 7]; 7];

/// Advance a particle's 7D Hepta-state by one discrete time step.
///
/// **Equation:** X(t+1) = J·X(t) + B·U(t)
///
/// - `state_x`   — current Hepta-score state vector X(t)
/// - `j_matrix`  — Jordan transition matrix (autonomous orbit evolution)
/// - `b_matrix`  — input gain matrix (amplifies control events from Adad Gate)
/// - `control_u` — 7D control input U(t) (event/perturbation at time t)
pub fn state_evolution(
    state_x:   &StateVec7,
    j_matrix:  &Matrix7,
    b_matrix:  &Matrix7,
    control_u: &StateVec7,
) -> StateVec7 {
    let mut result = [0.0f64; 7];
    for i in 0..7 {
        let jx: f64 = (0..7).map(|k| j_matrix[i][k] * state_x[k]).sum();
        let bu: f64 = (0..7).map(|k| b_matrix[i][k] * control_u[k]).sum();
        result[i] = jx + bu;
    }
    result
}

/// Apply `state_evolution` for `steps` time steps with a constant U(t).
pub fn trajectory(
    initial:   &StateVec7,
    j_matrix:  &Matrix7,
    b_matrix:  &Matrix7,
    control_u: &StateVec7,
    steps:     usize,
) -> Vec<StateVec7> {
    let mut states = Vec::with_capacity(steps + 1);
    states.push(*initial);
    for _ in 0..steps {
        let next = state_evolution(states.last().unwrap(), j_matrix, b_matrix, control_u);
        states.push(next);
    }
    states
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_boundaries() {
        assert_eq!(is_converged(0.0),  ConvergenceState::Dead);
        assert_eq!(is_converged(0.19), ConvergenceState::Dead);
        assert_eq!(is_converged(0.2),  ConvergenceState::Fuzzy);
        assert_eq!(is_converged(0.59), ConvergenceState::Fuzzy);
        assert_eq!(is_converged(0.6),  ConvergenceState::Golden);
        assert_eq!(is_converged(1.0),  ConvergenceState::Golden);
    }

    #[test]
    fn max_step_is_inversely_proportional_to_lipschitz() {
        assert!((max_stable_step(2.0) - 1.0).abs() < f64::EPSILON);
        assert!((max_stable_step(1.0) - 2.0).abs() < f64::EPSILON);
        assert!((max_stable_step(4.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn lipschitz_bound_linear_function() {
        // f(x) = 3x → L = 3
        let samples: Vec<(f64, f64)> = (0..5).map(|i| {
            let x = i as f64;
            (x, 3.0 * x)
        }).collect();
        let l = lipschitz_bound(&samples).unwrap();
        assert!((l - 3.0).abs() < 1e-9);
    }

    #[test]
    fn lipschitz_bound_requires_at_least_two_samples() {
        assert!(lipschitz_bound(&[]).is_none());
        assert!(lipschitz_bound(&[(1.0, 1.0)]).is_none());
        assert!(lipschitz_bound(&[(1.0, 1.0), (2.0, 2.0)]).is_some());
    }

    // ── state_evolution ───────────────────────────────────────────────────────

    #[test]
    fn identity_j_zero_b_leaves_state_unchanged() {
        let x: StateVec7 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let u: StateVec7 = [0.0; 7];
        let x1 = state_evolution(&x, &IDENTITY_7, &ZERO_7, &u);
        for i in 0..7 {
            assert!((x1[i] - x[i]).abs() < f64::EPSILON, "dim {i} changed");
        }
    }

    #[test]
    fn zero_j_zero_b_collapses_to_zero() {
        let x: StateVec7 = [5.0; 7];
        let u: StateVec7 = [0.0; 7];
        let x1 = state_evolution(&x, &ZERO_7, &ZERO_7, &u);
        for v in x1 { assert_eq!(v, 0.0); }
    }

    #[test]
    fn control_input_adds_through_identity_b() {
        let x: StateVec7 = [0.0; 7];
        let u: StateVec7 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        // B = I, J = 0 → X(1) = I·U = U
        let x1 = state_evolution(&x, &ZERO_7, &IDENTITY_7, &u);
        for i in 0..7 {
            assert!((x1[i] - u[i]).abs() < f64::EPSILON, "dim {i}");
        }
    }

    #[test]
    fn nilpotent_j_decays_state_to_zero() {
        // A nilpotent 7×7 Jordan block: J[i][i+1] = 1, everything else 0.
        let mut j: Matrix7 = [[0.0; 7]; 7];
        for i in 0..6 { j[i][i + 1] = 1.0; }

        let mut x: StateVec7 = [1.0; 7];
        let u: StateVec7 = [0.0; 7];
        for _ in 0..7 { x = state_evolution(&x, &j, &ZERO_7, &u); }
        // After N steps a nilpotent N×N matrix gives zero.
        for v in x { assert_eq!(v, 0.0); }
    }

    #[test]
    fn trajectory_length_is_steps_plus_one() {
        let x: StateVec7 = [1.0; 7];
        let u: StateVec7 = [0.0; 7];
        let path = trajectory(&x, &IDENTITY_7, &ZERO_7, &u, 5);
        assert_eq!(path.len(), 6); // initial + 5 steps
    }
}
