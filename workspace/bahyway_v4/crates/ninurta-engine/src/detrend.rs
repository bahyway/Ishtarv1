//! Detrending -- the honesty machinery's first step (per the
//! PIK-Potsdam CSD paper this calculus is grounded in, Ben-Yami, Skiba,
//! Bathiany & Boers, Nature Communications 2023): subtract a running
//! mean before computing any indicator, or the mean trend contaminates
//! the variance/autocorrelation/lambda estimate.

/// Centered running-mean detrend: each point has the mean of its own
/// `window`-wide neighborhood (clamped at the series' edges) subtracted.
/// `window` is floored at 1 -- at that floor, each point's own
/// "neighborhood" is itself, so every residual is exactly 0.
pub fn running_mean_detrend(series: &[f64], window: usize) -> Vec<f64> {
    let window = window.max(1);
    let n = series.len();
    let half = window / 2;
    (0..n)
        .map(|i| {
            let lo = i.saturating_sub(half);
            let hi = (i + half + 1).min(n);
            let mean: f64 = series[lo..hi].iter().sum::<f64>() / (hi - lo) as f64;
            series[i] - mean
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detrending_a_constant_series_yields_all_zeros() {
        let series = vec![5.0; 10];
        let out = running_mean_detrend(&series, 4);
        assert!(out.iter().all(|v| v.abs() < 1e-9));
    }

    #[test]
    fn detrending_preserves_length() {
        let series: Vec<f64> = (0..17).map(|i| i as f64).collect();
        let out = running_mean_detrend(&series, 5);
        assert_eq!(out.len(), series.len());
    }

    #[test]
    fn detrending_removes_a_linear_trend_from_the_middle_of_the_series() {
        // A pure ramp: the running mean at an interior point equals the
        // point itself for a symmetric window, so the residual is ~0 --
        // exactly the "mean trend contaminates the variance" problem
        // this step exists to remove.
        let series: Vec<f64> = (0..21).map(|i| i as f64).collect();
        let out = running_mean_detrend(&series, 7);
        assert!(out[10].abs() < 1e-9, "interior residual should be ~0 for a linear ramp, got {}", out[10]);
    }

    #[test]
    fn window_of_one_zeros_every_residual() {
        let series = vec![1.0, 4.0, -2.0, 7.0];
        let out = running_mean_detrend(&series, 1);
        assert!(out.iter().all(|v| v.abs() < 1e-9), "window=1: each point IS its own neighborhood mean, so every residual is 0, got {out:?}");
    }
}
