//! Welford's online algorithm: single-pass, numerically stable mean and
//! variance per dimension. No second pass over the data, no accumulated
//! sum-of-squares catastrophic cancellation.

use crate::D;

#[derive(Debug, Clone, Copy)]
pub struct WelfordStats {
    pub count: u64,
    pub mean: [f64; D],
    /// Sum of squared differences from the running mean (Welford's M2).
    m2: [f64; D],
}

impl Default for WelfordStats {
    fn default() -> Self {
        WelfordStats { count: 0, mean: [0.0; D], m2: [0.0; D] }
    }
}

impl WelfordStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fold in one 7D sample.
    pub fn update(&mut self, x: &[f64; D]) {
        self.count += 1;
        let n = self.count as f64;
        for i in 0..D {
            let delta = x[i] - self.mean[i];
            self.mean[i] += delta / n;
            let delta2 = x[i] - self.mean[i];
            self.m2[i] += delta * delta2;
        }
    }

    /// Sample variance per dimension (n-1 divisor). Zero for count < 2.
    pub fn variance(&self) -> [f64; D] {
        let mut out = [0.0; D];
        if self.count < 2 {
            return out;
        }
        let denom = (self.count - 1) as f64;
        for i in 0..D {
            out[i] = self.m2[i] / denom;
        }
        out
    }

    pub fn std_dev(&self) -> [f64; D] {
        let var = self.variance();
        let mut out = [0.0; D];
        for i in 0..D {
            out[i] = var[i].sqrt();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_direct_calculation_on_known_data() {
        // Dimension 0 only carries interesting data; others held constant
        // at zero so their variance is trivially zero.
        let xs = [2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mut w = WelfordStats::new();
        for &x in &xs {
            let mut v = [0.0; D];
            v[0] = x;
            w.update(&v);
        }
        let n = xs.len() as f64;
        let mean = xs.iter().sum::<f64>() / n;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);

        assert!((w.mean[0] - mean).abs() < 1e-9);
        assert!((w.variance()[0] - var).abs() < 1e-9);
        // Known result for this classic example: mean=5, sample var=4.5714...
        assert!((w.mean[0] - 5.0).abs() < 1e-9);
        assert!((w.variance()[0] - 32.0 / 7.0).abs() < 1e-9);
    }

    #[test]
    fn single_sample_has_zero_variance() {
        let mut w = WelfordStats::new();
        w.update(&[1.0; D]);
        assert_eq!(w.variance(), [0.0; D]);
        assert_eq!(w.count, 1);
    }

    #[test]
    fn empty_stats_are_safe() {
        let w = WelfordStats::new();
        assert_eq!(w.count, 0);
        assert_eq!(w.mean, [0.0; D]);
        assert_eq!(w.variance(), [0.0; D]);
    }
}
