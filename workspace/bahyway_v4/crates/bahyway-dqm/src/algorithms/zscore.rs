//! Z-score statistical anomaly detection.
//!
//! Uses Welford's online algorithm to maintain a running mean and variance
//! without storing the full data history.  O(1) per observation, O(1) space.
//!
//! A value with |z| > threshold is flagged as an anomaly.
//! Standard threshold: 3.0 (flags ~0.3% of normally-distributed values).
//! Strict threshold:   2.5 (flags ~1.2% of normally-distributed values).
//!
//! Used by: DqmDimension::Validity (statistical anomaly detection on numeric fields)

/// Running statistics accumulator using Welford's online algorithm.
/// Maintains mean and variance incrementally — no stored history.
#[derive(Debug, Clone)]
pub struct RunningStats {
    count: u64,
    mean:  f64,
    m2:    f64,  // sum of squared deviations from mean
}

impl RunningStats {
    pub fn new() -> Self {
        RunningStats { count: 0, mean: 0.0, m2: 0.0 }
    }

    /// Incorporate one new observation using Welford's method.
    pub fn update(&mut self, x: f64) {
        self.count += 1;
        let delta  = x - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = x - self.mean;
        self.m2   += delta * delta2;
    }

    /// Incremental mean.
    pub fn mean(&self) -> f64 { self.mean }

    /// Population variance.
    pub fn variance(&self) -> f64 {
        if self.count < 2 { return 0.0; }
        self.m2 / self.count as f64
    }

    /// Population standard deviation.
    pub fn std_dev(&self) -> f64 { self.variance().sqrt() }

    /// Number of observations incorporated.
    pub fn count(&self) -> u64 { self.count }

    /// Z-score for a new value: (x - mean) / std_dev.
    /// Returns None if fewer than 2 observations (std_dev undefined).
    pub fn z_score(&self, x: f64) -> Option<f64> {
        let sd = self.std_dev();
        if sd < f64::EPSILON { return None; }
        Some((x - self.mean) / sd)
    }

    /// True when `x` is an outlier at the given `threshold` (|z| > threshold).
    /// Standard threshold is 3.0; use 2.5 for stricter detection.
    pub fn is_outlier(&self, x: f64, threshold: f64) -> bool {
        self.z_score(x).map(|z| z.abs() > threshold).unwrap_or(false)
    }

    /// Quality score for `x`: 1.0 when z=0, approaches 0.0 as |z| → threshold.
    /// Score = max(0, 1 - |z| / threshold).
    pub fn anomaly_score(&self, x: f64, threshold: f64) -> f32 {
        match self.z_score(x) {
            None    => 1.0,
            Some(z) => (1.0 - z.abs() / threshold).max(0.0) as f32,
        }
    }
}

impl Default for RunningStats {
    fn default() -> Self { Self::new() }
}

/// Batch Z-score: compute z-scores for all values given pre-computed statistics.
pub fn batch_z_scores(values: &[f64], mean: f64, std_dev: f64) -> Vec<f64> {
    if std_dev < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    values.iter().map(|&x| (x - mean) / std_dev).collect()
}

/// Compute mean and population std_dev over a slice.
pub fn slice_stats(values: &[f64]) -> (f64, f64) {
    if values.is_empty() { return (0.0, 0.0); }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var  = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(stats: &mut RunningStats, values: &[f64]) {
        for &v in values { stats.update(v); }
    }

    #[test]
    fn welford_mean_matches_naive() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut s = RunningStats::new();
        feed(&mut s, &data);
        let naive = data.iter().sum::<f64>() / data.len() as f64;
        assert!((s.mean() - naive).abs() < 1e-10);
    }

    #[test]
    fn welford_std_dev_matches_naive() {
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mut s = RunningStats::new();
        feed(&mut s, &data);
        // Population std dev of this set is exactly 2.0
        assert!((s.std_dev() - 2.0).abs() < 1e-8);
    }

    #[test]
    fn z_score_of_mean_is_zero() {
        let mut s = RunningStats::new();
        feed(&mut s, &[1.0, 2.0, 3.0, 4.0, 5.0]);
        let z = s.z_score(s.mean()).unwrap();
        assert!(z.abs() < 1e-10);
    }

    #[test]
    fn extreme_outlier_detected() {
        let mut s = RunningStats::new();
        feed(&mut s, &[10.0, 11.0, 10.5, 10.8, 11.2, 10.3]);
        // Value 100.0 is far outside the distribution
        assert!(s.is_outlier(100.0, 3.0));
    }

    #[test]
    fn normal_value_not_flagged() {
        let mut s = RunningStats::new();
        feed(&mut s, &[10.0, 11.0, 10.5, 10.8, 11.2, 10.3]);
        assert!(!s.is_outlier(10.9, 3.0));
    }

    #[test]
    fn anomaly_score_one_for_mean() {
        let mut s = RunningStats::new();
        feed(&mut s, &[1.0, 2.0, 3.0, 4.0, 5.0]);
        let score = s.anomaly_score(s.mean(), 3.0);
        assert!((score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn anomaly_score_zero_for_extreme_outlier() {
        let mut s = RunningStats::new();
        feed(&mut s, &[10.0, 11.0, 10.5, 10.8, 11.2, 10.3]);
        let score = s.anomaly_score(1000.0, 3.0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn single_observation_no_z_score() {
        let mut s = RunningStats::new();
        s.update(5.0);
        assert!(s.z_score(5.0).is_none());
    }

    #[test]
    fn slice_stats_known_values() {
        let (mean, sd) = slice_stats(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((mean - 5.0).abs() < 1e-10);
        assert!((sd - 2.0).abs() < 1e-8);
    }
}
