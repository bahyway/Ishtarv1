//! Kullback-Leibler divergence — ALGEBRA_GLOSSARY.md Part V, Layer 7.
//! Confirmed genuinely absent before this file (no hit for KL/kl_divergence
//! anywhere in the workspace). Complements the real Shannon entropy already
//! in `fsv::shannon_entropy` / `bfv::compute_bfv`.
//!
//! D_KL(P||Q) = Σ p(x) log2( p(x) / q(x) )
//!
//! Sovereign Rule (ALGEBRA_GLOSSARY.md): smoothing constant epsilon = 1e-10
//! is fixed, applied to Q to avoid division by / log of zero when a symbol
//! observed under P is unobserved under Q.

/// Fixed smoothing constant applied to Q, per the glossary's Sovereign Rule.
pub const KL_EPSILON: f64 = 1e-10;

/// KL divergence of P from Q, in bits. `p` and `q` must be the same length
/// and are treated as (not necessarily pre-normalized) probability masses;
/// zero-mass symbols under P contribute 0 by convention (0 log 0 := 0).
///
/// # Panics
/// Panics if `p.len() != q.len()`.
pub fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    assert_eq!(p.len(), q.len(), "KL divergence requires equal-length distributions");
    p.iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| {
            if pi <= 0.0 {
                0.0
            } else {
                let qi_smoothed = qi.max(KL_EPSILON);
                pi * (pi / qi_smoothed).log2()
            }
        })
        .sum()
}

/// Normalizes a raw frequency/count vector into a probability distribution.
/// Returns an all-zero vector unchanged if the total is zero.
pub fn normalize(counts: &[f64]) -> Vec<f64> {
    let total: f64 = counts.iter().sum();
    if total <= 0.0 {
        return counts.to_vec();
    }
    counts.iter().map(|&c| c / total).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kl_of_distribution_with_itself_is_zero() {
        let p = normalize(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        let d = kl_divergence(&p, &p);
        assert!(d.abs() < 1e-9, "D_KL(P||P) should be 0, got {d}");
    }

    #[test]
    fn kl_is_nonnegative_gibbs_inequality() {
        let p = normalize(&[0.1, 0.2, 0.3, 0.4]);
        let q = normalize(&[0.4, 0.3, 0.2, 0.1]);
        assert!(kl_divergence(&p, &q) >= -1e-12);
        assert!(kl_divergence(&q, &p) >= -1e-12);
    }

    #[test]
    fn kl_is_asymmetric() {
        let p = normalize(&[0.9, 0.1]);
        let q = normalize(&[0.5, 0.5]);
        let pq = kl_divergence(&p, &q);
        let qp = kl_divergence(&q, &p);
        assert!((pq - qp).abs() > 1e-6, "D_KL(P||Q) should differ from D_KL(Q||P)");
    }

    #[test]
    fn kl_smoothing_prevents_blowup_on_zero_support() {
        // P has mass on a symbol Q assigns exactly zero -- without
        // smoothing this would be +infinity (log of zero).
        let p = vec![0.5, 0.5, 0.0];
        let q = vec![0.5, 0.0, 0.5];
        let d = kl_divergence(&p, &q);
        assert!(d.is_finite(), "expected finite divergence with epsilon smoothing, got {d}");
        assert!(d > 0.0);
    }

    #[test]
    fn normalize_all_zero_input_stays_all_zero() {
        let n = normalize(&[0.0, 0.0, 0.0]);
        assert_eq!(n, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn normalize_sums_to_one() {
        let n = normalize(&[1.0, 2.0, 3.0, 4.0]);
        let sum: f64 = n.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }
}
