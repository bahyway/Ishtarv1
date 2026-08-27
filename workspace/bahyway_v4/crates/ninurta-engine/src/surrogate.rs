//! Fourier-surrogate significance testing -- the honesty machinery's
//! third, non-negotiable step: never alarm on a rising lambda alone.
//! Generate many random series with the same power spectrum (hence the
//! same variance and autocorrelation structure) as the real series, and
//! alarm only if the real lambda-trend exceeds the surrogates' more
//! often than chance allows (p < 0.05).
//!
//! Implemented as a direct discrete Fourier transform (O(n^2)), not an
//! FFT -- this workspace forbids external crates (Sovereign Constraint
//! Section 0.3), and the sliding-window sizes this calculus operates on
//! (tens to low hundreds of ticks, per the Girsu CSD Notebook design
//! tablet's own `WINDOW LAST 90 TICKS` example) make an O(n^2)
//! transform genuinely fine, not a hidden performance trap.

use std::f64::consts::PI;

/// xorshift64star -- fast, deterministic, no crypto needed for
/// generating random phases. Same generator family `bin/najaf-gen`
/// uses, for the same reason (reproducible synthetic data from a seed).
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// A uniform f64 in [0, 2*PI).
    fn next_phase(&mut self) -> f64 {
        (self.next_u64() as f64 / u64::MAX as f64) * 2.0 * PI
    }
}

/// Direct DFT: returns (real, imaginary) coefficients per frequency bin.
fn dft(x: &[f64]) -> Vec<(f64, f64)> {
    let n = x.len();
    (0..n)
        .map(|k| {
            let mut re = 0.0;
            let mut im = 0.0;
            for (t, &xt) in x.iter().enumerate() {
                let angle = -2.0 * PI * (k as f64) * (t as f64) / (n as f64);
                re += xt * angle.cos();
                im += xt * angle.sin();
            }
            (re, im)
        })
        .collect()
}

/// Direct inverse DFT, real part only (the caller guarantees conjugate
/// symmetry so the imaginary part is ~0 up to floating-point error).
fn idft_real(coeffs: &[(f64, f64)]) -> Vec<f64> {
    let n = coeffs.len();
    (0..n)
        .map(|t| {
            let mut sum = 0.0;
            for (k, &(re, im)) in coeffs.iter().enumerate() {
                let angle = 2.0 * PI * (k as f64) * (t as f64) / (n as f64);
                sum += re * angle.cos() - im * angle.sin();
            }
            sum / n as f64
        })
        .collect()
}

/// One Fourier surrogate of `series`: same amplitude spectrum (hence
/// the same variance and autocorrelation function), independently
/// randomized phases. Conjugate symmetry is preserved by construction
/// (mirrored bins get the negated phase) so the result is real-valued.
fn fourier_surrogate(series: &[f64], rng: &mut Rng) -> Vec<f64> {
    let n = series.len();
    let coeffs = dft(series);
    let mut randomized = vec![(0.0, 0.0); n];
    let half = n / 2;
    for k in 0..=half {
        let (re, im) = coeffs[k];
        let mag = (re * re + im * im).sqrt();
        // Bin 0 (DC) and, for even n, the Nyquist bin (k == n/2) must
        // stay real-valued or the series picks up a spurious imaginary
        // component -- both are self-conjugate under mirroring, so
        // leave their original value alone rather than randomizing.
        if k == 0 || (n.is_multiple_of(2) && k == half) {
            randomized[k] = (re, im);
            continue;
        }
        let phase = rng.next_phase();
        let new_re = mag * phase.cos();
        let new_im = mag * phase.sin();
        randomized[k] = (new_re, new_im);
        randomized[n - k] = (new_re, -new_im);
    }
    idft_real(&randomized)
}

/// Generate `count` independent Fourier surrogates, seeded
/// deterministically from `seed` (varied per surrogate so the batch
/// isn't `count` copies of one draw).
pub fn fourier_surrogates(series: &[f64], count: usize, seed: u64) -> Vec<Vec<f64>> {
    (0..count)
        .map(|i| {
            let mut rng = Rng::new(seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            fourier_surrogate(series, &mut rng)
        })
        .collect()
}

/// One-sided surrogate significance test: the fraction of
/// `surrogate_trends` at least as extreme (>=) as `real_trend` -- the
/// classic surrogate p-value. This IS the tau-score on the alert (per
/// the Girsu CSD Notebook's own law): "how sure am I this isn't noise."
/// Returns 1.0 (never significant) for an empty surrogate set rather
/// than panicking or dividing by zero.
pub fn surrogate_p_value(real_trend: f64, surrogate_trends: &[f64]) -> f64 {
    if surrogate_trends.is_empty() {
        return 1.0;
    }
    let at_least_as_extreme = surrogate_trends
        .iter()
        .filter(|&&t| t >= real_trend)
        .count();
    at_least_as_extreme as f64 / surrogate_trends.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::variance;

    #[test]
    fn fourier_surrogate_preserves_length() {
        let series: Vec<f64> = (0..20).map(|i| (i as f64).sin()).collect();
        let mut rng = Rng::new(1);
        let surr = fourier_surrogate(&series, &mut rng);
        assert_eq!(surr.len(), series.len());
    }

    #[test]
    fn fourier_surrogate_preserves_variance_closely() {
        let series: Vec<f64> = (0..40)
            .map(|i| (i as f64 * 0.3).sin() + (i as f64 * 0.05))
            .collect();
        let mut rng = Rng::new(7);
        let surr = fourier_surrogate(&series, &mut rng);
        let var_orig = variance(&series);
        let var_surr = variance(&surr);
        let rel_diff = (var_orig - var_surr).abs() / var_orig.max(1e-9);
        assert!(rel_diff < 0.05, "surrogate variance {var_surr} should closely match original {var_orig}, rel diff {rel_diff}");
    }

    #[test]
    fn different_surrogates_in_a_batch_are_not_identical() {
        let series: Vec<f64> = (0..24).map(|i| (i as f64 * 0.4).cos()).collect();
        let batch = fourier_surrogates(&series, 5, 42);
        assert_ne!(
            batch[0], batch[1],
            "independent surrogates must not be identical draws"
        );
    }

    #[test]
    fn surrogate_p_value_of_extreme_real_trend_is_low() {
        let surrogates = vec![0.01, -0.02, 0.005, -0.01, 0.02];
        let p = surrogate_p_value(10.0, &surrogates);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn surrogate_p_value_of_typical_real_trend_is_high() {
        let surrogates = vec![0.5, 0.6, 0.4, 0.55, 0.45];
        let p = surrogate_p_value(0.5, &surrogates);
        assert!(p > 0.5, "a real trend in the middle of the surrogate distribution must not look significant, got p={p}");
    }

    #[test]
    fn empty_surrogate_set_is_never_significant() {
        assert_eq!(surrogate_p_value(100.0, &[]), 1.0);
    }
}
