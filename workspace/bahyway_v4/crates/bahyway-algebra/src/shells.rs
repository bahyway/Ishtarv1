//! PA-13 Shell Decomposition — logarithmic quality-space binning.
//!
//! The quality distance δ(p) = 1 − H(P) partitions the particle space
//! into concentric shells.  Each shell boundary is placed at an exponentially
//! growing distance from the ideal point so that inner shells (near-perfect
//! particles) have fine resolution and outer shells (degraded particles) have
//! coarser resolution — matching the Babylonian logarithmic number system.
//!
//! The key invariant:
//!   shell_index(delta, &shell_boundaries(n, ε)) runs in O(log n) via
//!   binary search and is guaranteed to return a valid index for any δ ∈ [0,1].
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

/// Euclidean quality distance between two H(P) health scores.
///
/// `δ = |H(p₁) − H(p₂)|` — measures how far apart two particles are
/// in the quality dimension.  Always returns a value in `[0.0, 1.0]`.
#[inline]
pub fn quality_distance(h1: f64, h2: f64) -> f64 {
    (h1 - h2).abs()
}

/// Construct logarithmic shell boundaries for `n` shells.
///
/// Each boundary bᵢ = ε × rⁱ where r is chosen so that bₙ₋₁ ≈ 1.0.
/// The result has `n - 1` boundary values; particles with δ < b₀ land in
/// shell 0 (innermost / highest quality), and δ ≥ b_{n-2} land in shell n-1.
///
/// # Arguments
/// * `n`       — number of shells (minimum 2)
/// * `epsilon` — inner-shell resolution (smallest distinguishable δ)
///
/// # Panics
/// Panics if `n < 2` or `epsilon <= 0`.
pub fn shell_boundaries(n: usize, epsilon: f64) -> Vec<f64> {
    assert!(n >= 2, "need at least 2 shells");
    assert!(epsilon > 0.0 && epsilon < 1.0, "epsilon must be in (0, 1)");

    let num_boundaries = n - 1;
    // Logarithmic ratio: each boundary is r × the previous one.
    // b₀ = ε, b_{k} = ε × r^k, b_{n-2} ≤ 1.0
    // r = (1.0 / ε)^(1 / (n-2)) when n > 2, else r doesn't matter.
    let r = if num_boundaries == 1 {
        1.0 // single boundary: just use epsilon itself
    } else {
        (1.0_f64 / epsilon).powf(1.0 / (num_boundaries - 1) as f64)
    };

    (0..num_boundaries)
        .map(|k| (epsilon * r.powi(k as i32)).min(1.0))
        .collect()
}

/// Determine which shell a particle belongs to in O(log n).
///
/// Returns the shell index (0 = innermost, highest quality).
/// A particle with δ < boundaries[0] is in shell 0.
/// A particle with δ ≥ boundaries[last] is in the outermost shell.
///
/// # Arguments
/// * `delta`      — quality distance δ(p) = 1 − H(P)
/// * `boundaries` — shell boundary vector from `shell_boundaries()`
pub fn shell_index(delta: f64, boundaries: &[f64]) -> usize {
    // binary search: find leftmost boundary that delta < boundary
    match boundaries.binary_search_by(|b| b.partial_cmp(&delta).unwrap()) {
        Ok(i) => i + 1, // exact match → next shell
        Err(i) => i,    // i = position where delta would be inserted
    }
}

/// Map a B11 quality byte to a shell index.
///
/// `B11 = round(H(P) × 240)` — converts to H(P) then to δ, then bins.
pub fn b11_to_shell(b11: u8, boundaries: &[f64]) -> usize {
    let health = b11 as f64 / 240.0;
    let delta = 1.0 - health;
    shell_index(delta, boundaries)
}

/// Sovereign 5-shell decomposition (BahyWay default).
///
/// Mirrors the 5 QualityLane bands: GEM / TRIBE / ACTIVE / FUZZY / DEAD.
/// Boundaries chosen so that B11 thresholds (200, 140, 100, 59) fall inside
/// the correct shell.
pub fn sovereign_5_shells() -> Vec<f64> {
    // δ at each B11 boundary: δ = 1 − B11/240
    // B11=200 → δ=0.167  (GEM → TRIBE boundary)
    // B11=140 → δ=0.417  (TRIBE → ACTIVE boundary)
    // B11=100 → δ=0.583  (ACTIVE → FUZZY boundary)
    // B11=59  → δ=0.754  (FUZZY → DEAD boundary)
    vec![0.167, 0.417, 0.583, 0.754]
}

/// Shell name for the sovereign 5-shell decomposition.
pub fn sovereign_shell_label(shell: usize) -> &'static str {
    match shell {
        0 => "GEM",
        1 => "TRIBE",
        2 => "ACTIVE",
        3 => "FUZZY",
        4 => "DEAD",
        _ => "OUTER",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_distance_symmetric() {
        assert!((quality_distance(0.8, 0.6) - 0.2).abs() < 1e-9);
        assert!((quality_distance(0.6, 0.8) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn quality_distance_same_score_is_zero() {
        assert_eq!(quality_distance(0.75, 0.75), 0.0);
    }

    #[test]
    fn quality_distance_bounded_in_unit() {
        let d = quality_distance(0.0, 1.0);
        assert!((d - 1.0).abs() < 1e-9);
    }

    #[test]
    fn shell_boundaries_length() {
        let bs = shell_boundaries(5, 0.01);
        assert_eq!(bs.len(), 4, "n shells → n-1 boundaries");
    }

    #[test]
    fn shell_boundaries_monotone() {
        let bs = shell_boundaries(6, 0.001);
        for w in bs.windows(2) {
            assert!(w[0] < w[1], "boundaries must be strictly increasing");
        }
    }

    #[test]
    fn shell_boundaries_all_in_unit() {
        let bs = shell_boundaries(8, 0.01);
        for b in &bs {
            assert!(*b > 0.0 && *b <= 1.0, "boundary {b} out of [0,1]");
        }
    }

    #[test]
    fn shell_index_inner_shell_for_perfect() {
        let bs = sovereign_5_shells();
        // perfect particle: H=1.0, δ=0.0 → shell 0 (GEM)
        assert_eq!(shell_index(0.0, &bs), 0);
    }

    #[test]
    fn shell_index_outer_shell_for_worst() {
        let bs = sovereign_5_shells();
        // worst particle: δ=1.0 → shell 4 (DEAD)
        assert_eq!(shell_index(1.0, &bs), 4);
    }

    #[test]
    fn sovereign_shells_match_quality_lanes() {
        let bs = sovereign_5_shells();
        // B11=240 → GEM
        assert_eq!(b11_to_shell(240, &bs), 0);
        assert_eq!(sovereign_shell_label(0), "GEM");

        // B11=150 → TRIBE
        assert_eq!(b11_to_shell(150, &bs), 1);
        assert_eq!(sovereign_shell_label(1), "TRIBE");

        // B11=110 → ACTIVE
        assert_eq!(b11_to_shell(110, &bs), 2);
        assert_eq!(sovereign_shell_label(2), "ACTIVE");

        // B11=70 → FUZZY
        assert_eq!(b11_to_shell(70, &bs), 3);
        assert_eq!(sovereign_shell_label(3), "FUZZY");

        // B11=30 → DEAD
        assert_eq!(b11_to_shell(30, &bs), 4);
        assert_eq!(sovereign_shell_label(4), "DEAD");
    }

    #[test]
    fn shell_index_o1_lookup_consistent() {
        let bs = shell_boundaries(10, 0.001);
        for i in 0..10 {
            let delta = i as f64 * 0.1;
            let s = shell_index(delta, &bs);
            assert!(s < 10, "shell {s} out of range for δ={delta}");
        }
    }

    #[test]
    fn two_shell_minimum_accepted() {
        let bs = shell_boundaries(2, 0.5);
        assert_eq!(bs.len(), 1);
        assert_eq!(shell_index(0.0, &bs), 0);
        assert_eq!(shell_index(1.0, &bs), 1);
    }
}
