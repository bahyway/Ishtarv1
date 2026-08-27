//! 𒁾 The Sovereign Hepta Equation
//!
//! ```text
//!                    1
//! H(P)  =  ─────────────────────────────
//!            1 + √ Σᵢ₌₁⁷  wᵢ (Pᵢ − Tᵢ)²
//! ```
//!
//! Pure function — same inputs always produce the same output.
//! No state, no allocations.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

/// Compute H(P) for a 7D particle vector, tribe ideal, and weight vector.
///
/// # Arguments
/// * `p` — Particle dimension scores `[P₁..P₇]`, each in `[0.0, 1.0]`
/// * `t` — Tribe ideal point `[T₁..T₇]`, each in `[0.0, 1.0]`
/// * `w` — Sovereign weights `[w₁..w₇]`, must sum to 1.0
///
/// # Returns
/// `H(P)` in `(0.0, 1.0]`. Returns `1.0` exactly when `P == T`.
#[inline]
pub fn hepta_health_score(p: &[f32; 7], t: &[f32; 7], w: &[f32; 7]) -> f32 {
    let weighted_sq_dist: f32 = p
        .iter()
        .zip(t.iter())
        .zip(w.iter())
        .map(|((pi, ti), wi)| wi * (pi - ti).powi(2))
        .sum();
    1.0 / (1.0 + weighted_sq_dist.sqrt())
}

/// Weighted Euclidean distance √Σwᵢ(Pᵢ−Tᵢ)² — the particle's orbital distance.
///
/// A larger value means the particle is further from its tribe ideal
/// and orbits in a wider, slower ring.
#[inline]
pub fn weighted_orbit_distance(p: &[f32; 7], t: &[f32; 7], w: &[f32; 7]) -> f32 {
    p.iter()
        .zip(t.iter())
        .zip(w.iter())
        .map(|((pi, ti), wi)| wi * (pi - ti).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Per-dimension contributions wᵢ(Pᵢ−Tᵢ)² for remediation analysis.
///
/// The dimension with the highest contribution is the `fix_priority`.
pub fn dimension_contributions(p: &[f32; 7], t: &[f32; 7], w: &[f32; 7]) -> [f32; 7] {
    std::array::from_fn(|i| w[i] * (p[i] - t[i]).powi(2))
}

/// Project H(P) onto the sovereign B11 byte: `B11 = round(H(P) × 240.0)`.
///
/// **ADR-001**: ALWAYS 240.0, NEVER 255.0.
#[inline]
pub fn health_to_b11(health: f32) -> u8 {
    (health * 240.0).round().clamp(0.0, 240.0) as u8
}

/// Inverse: derive approximate H(P) from a B11 byte.
#[inline]
pub fn b11_to_health(b11: u8) -> f32 {
    b11 as f32 / 240.0
}

/// Sensitivity ∂H/∂Pᵢ — how much does H improve if Pᵢ increases by δ?
///
/// `∂H/∂Pᵢ = wᵢ(Tᵢ − Pᵢ) / [(1 + d)² × d]`  where `d = √Σwⱼ(Pⱼ−Tⱼ)²`
///
/// Used by the remediation engine to calculate fix ROI per dimension.
pub fn sensitivity(p: &[f32; 7], t: &[f32; 7], w: &[f32; 7]) -> [f32; 7] {
    let d = weighted_orbit_distance(p, t, w);
    if d < 1e-8 {
        return [0.0; 7]; // already perfect
    }
    let factor = 1.0 / ((1.0 + d).powi(2) * d);
    std::array::from_fn(|i| w[i] * (t[i] - p[i]) * factor)
}

/// Simulate the new H(P) after fixing dimension `dim` to `new_val`.
///
/// Used in ShoWay "what-if" remediation simulator.
pub fn simulate_fix(p: &[f32; 7], t: &[f32; 7], w: &[f32; 7], dim: usize, new_val: f32) -> f32 {
    let mut p_fixed = *p;
    p_fixed[dim] = new_val.clamp(0.0, 1.0);
    hepta_health_score(&p_fixed, t, w)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const T: [f32; 7] = [1.0; 7];
    const W_EQUAL: [f32; 7] = [1.0 / 7.0; 7];
    const W_SOVEREIGN: [f32; 7] = [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05];

    #[test]
    fn perfect_particle_scores_exactly_one() {
        let h = hepta_health_score(&[1.0; 7], &T, &W_EQUAL);
        assert_eq!(h, 1.0);
    }

    #[test]
    fn zero_particle_gives_half() {
        // With T=[1;7] and Σwᵢ=1, d_max=1 → H_min=0.5 (mathematical floor)
        let h = hepta_health_score(&[0.0; 7], &T, &W_EQUAL);
        assert!((h - 0.5).abs() < 1e-6, "H(zero)={h}");
    }

    #[test]
    fn score_bounded_between_0_and_1() {
        for q in [0.0f32, 0.1, 0.3, 0.5, 0.7, 0.9, 1.0] {
            let h = hepta_health_score(&[q; 7], &T, &W_EQUAL);
            assert!(h > 0.0 && h <= 1.0, "H({q})={h}");
        }
    }

    #[test]
    fn score_is_monotone() {
        let levels = [0.0f32, 0.2, 0.4, 0.6, 0.8, 0.95, 1.0];
        let scores: Vec<f32> = levels
            .iter()
            .map(|&q| hepta_health_score(&[q; 7], &T, &W_EQUAL))
            .collect();
        for i in 0..scores.len() - 1 {
            assert!(
                scores[i] < scores[i + 1],
                "not monotone: h({})={:.4} >= h({})={:.4}",
                levels[i],
                scores[i],
                levels[i + 1],
                scores[i + 1]
            );
        }
    }

    #[test]
    fn sovereign_weights_sum_to_one() {
        let sum: f32 = W_SOVEREIGN.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "sum={sum}");
    }

    #[test]
    fn near_perfect_b11_is_gem() {
        let h = hepta_health_score(&[0.97; 7], &T, &W_SOVEREIGN);
        assert!(health_to_b11(h) >= 200);
    }

    #[test]
    fn very_poor_b11_is_active() {
        // P=[0.2;7]: d=√(Σ(1/7)*0.64)=0.8, H=0.556, B11=133 → Active
        // Dead (B11<59) requires d>3.07 — impossible with T=[1;7], P∈[0,1]
        let h = hepta_health_score(&[0.2; 7], &T, &W_EQUAL);
        let b11 = health_to_b11(h);
        assert!(b11 >= 100 && b11 < 140, "B11={b11} should be Active range");
    }

    #[test]
    fn b11_roundtrip_within_one() {
        let original = 180u8;
        let h = b11_to_health(original);
        let b11 = health_to_b11(h);
        assert!((original as i32 - b11 as i32).abs() <= 1);
    }

    #[test]
    fn contributions_sum_to_squared_distance() {
        let p = [0.7f32; 7];
        let contribs = dimension_contributions(&p, &T, &W_SOVEREIGN);
        let total: f32 = contribs.iter().sum();
        let direct = weighted_orbit_distance(&p, &T, &W_SOVEREIGN).powi(2);
        assert!((total - direct).abs() < 1e-5, "{total} vs {direct}");
    }

    #[test]
    fn sensitivity_positive_when_below_ideal() {
        let p = [0.7f32; 7];
        let sens = sensitivity(&p, &T, &W_EQUAL);
        for (i, &s) in sens.iter().enumerate() {
            assert!(s > 0.0, "sensitivity[{i}]={s}");
        }
    }

    #[test]
    fn simulate_fix_improves_score() {
        let p = [0.7f32; 7];
        let original = hepta_health_score(&p, &T, &W_EQUAL);
        let fixed = simulate_fix(&p, &T, &W_EQUAL, 0, 1.0);
        assert!(fixed > original, "{original} → {fixed}");
    }

    #[test]
    fn orbit_distance_zero_for_perfect() {
        let d = weighted_orbit_distance(&[1.0; 7], &T, &W_EQUAL);
        assert!(d.abs() < 1e-6, "d={d}");
    }
}
