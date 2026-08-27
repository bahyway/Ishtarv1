//! HUBURU FORENSICS — the noise is a witness too (PB-607).
//! After explained structure is removed from HSI / LiDAR residuals,
//! honest noise should be white: no spatial memory (lag-1 autocorrelation
//! near zero) and no heavy tails (excess kurtosis near zero). Structured
//! residue betrays a hidden object. A cell whose noise testifies as
//! structured is demoted from certification and targeted — the pattern
//! discovered INSIDE the noise.

#[derive(Debug, Clone, PartialEq)]
pub enum NoiseVerdict {
    White,
    Structured { autocorr: f64, kurtosis: f64, cause: String },
}

pub fn lag1_autocorr(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 3 {
        return 0.0;
    }
    let mean = xs.iter().sum::<f64>() / n as f64;
    let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>();
    if var == 0.0 {
        return 0.0;
    }
    let cov: f64 = xs.windows(2).map(|w| (w[0] - mean) * (w[1] - mean)).sum();
    cov / var
}

pub fn excess_kurtosis(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 4 {
        return 0.0;
    }
    let mean = xs.iter().sum::<f64>() / n as f64;
    let m2 = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    if m2 == 0.0 {
        return 0.0;
    }
    let m4 = xs.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n as f64;
    m4 / (m2 * m2) - 3.0
}

/// Judge a residual series. Thresholds are court-set; defaults below are
/// generous so honest noise passes and real structure does not.
pub fn judge_noise(xs: &[f64], r_max: f64, k_max: f64) -> NoiseVerdict {
    let r = lag1_autocorr(xs);
    let k = excess_kurtosis(xs);
    if r.abs() > r_max {
        return NoiseVerdict::Structured {
            autocorr: r,
            kurtosis: k,
            cause: format!("spatial memory: |lag-1 autocorr| {:.2} > {:.2}", r.abs(), r_max),
        };
    }
    if k.abs() > k_max {
        return NoiseVerdict::Structured {
            autocorr: r,
            kurtosis: k,
            cause: format!("heavy tails: |excess kurtosis| {:.2} > {:.2}", k.abs(), k_max),
        };
    }
    NoiseVerdict::White
}

pub const R_MAX_DEFAULT: f64 = 0.20;
pub const K_MAX_DEFAULT: f64 = 1.00;

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic near-Gaussian noise: sum of four LCG uniforms
    /// (Irwin–Hall; excess kurtosis ≈ −0.3, no lag memory).
    fn honest_noise(n: usize) -> Vec<f64> {
        let mut state: u64 = 3600;
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 33) as f64) / (u32::MAX as f64 + 1.0)
        };
        (0..n).map(|_| next() + next() + next() + next() - 2.0).collect()
    }

    #[test]
    fn honest_noise_passes_as_white() {
        let xs = honest_noise(4000);
        assert_eq!(judge_noise(&xs, R_MAX_DEFAULT, K_MAX_DEFAULT), NoiseVerdict::White);
    }

    #[test]
    fn buried_periodicity_betrays_itself_by_memory() {
        let mut xs = honest_noise(4000);
        for (i, x) in xs.iter_mut().enumerate() {
            *x += 1.4 * (i as f64 * 0.35).sin(); // smooth buried structure
        }
        match judge_noise(&xs, R_MAX_DEFAULT, K_MAX_DEFAULT) {
            NoiseVerdict::Structured { cause, .. } => assert!(cause.contains("spatial memory")),
            _ => panic!("periodic residue must be flagged"),
        }
    }

    #[test]
    fn sparse_spikes_betray_themselves_by_tails() {
        let mut xs = honest_noise(4000);
        for i in (0..xs.len()).step_by(500) {
            xs[i] += 14.0; // rare localized anomalies
        }
        match judge_noise(&xs, R_MAX_DEFAULT, K_MAX_DEFAULT) {
            NoiseVerdict::Structured { cause, .. } => assert!(cause.contains("heavy tails")),
            _ => panic!("spiked residue must be flagged"),
        }
    }

    #[test]
    fn degenerate_series_do_not_lie() {
        assert_eq!(judge_noise(&[0.0, 0.0], 0.2, 1.0), NoiseVerdict::White);
        assert_eq!(lag1_autocorr(&[1.0, 1.0, 1.0]), 0.0, "zero variance → no memory claim");
    }
}
