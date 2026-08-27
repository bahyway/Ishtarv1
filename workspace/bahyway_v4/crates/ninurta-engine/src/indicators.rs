//! The three CSD indicators computed per sliding window, per Ben-Yami,
//! Skiba, Bathiany & Boers (Nature Communications, 2023): the restoring
//! rate lambda (preferred, robust), variance (treacherous alone -- can
//! shift from population-size artifacts, never trusted in isolation),
//! and lag-1 autocorrelation (corroborates lambda).

/// Simple OLS slope of `y` on `x` (with intercept): slope =
/// cov(x,y)/var(x). `None` if `x` has ~zero variance (degenerate).
fn ols_slope(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len() as f64;
    if n < 2.0 {
        return None;
    }
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_x = 0.0;
    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        cov += dx * (y[i] - mean_y);
        var_x += dx * dx;
    }
    if var_x <= f64::EPSILON {
        return None;
    }
    Some(cov / var_x)
}

/// Restoring-rate estimate (the Purussum Calculus's preferred
/// indicator): regress the increment `dx_t = x[t+1] - x[t]` on `x[t]`
/// (near equilibrium, dx/dt ~ lambda*x + eta, discretized at unit time
/// step). Returns the OLS slope, `lambda`. Negative for a stable state;
/// rises toward 0 as a transition approaches. `None` if fewer than 3
/// points or `x` has ~zero variance.
pub fn restoring_rate(series: &[f64]) -> Option<f64> {
    if series.len() < 3 {
        return None;
    }
    let xs = &series[..series.len() - 1];
    let dxs: Vec<f64> = series.windows(2).map(|w| w[1] - w[0]).collect();
    ols_slope(xs, &dxs)
}

/// Population variance (divide by N, not N-1) of the series.
pub fn variance(series: &[f64]) -> f64 {
    if series.is_empty() {
        return 0.0;
    }
    let n = series.len() as f64;
    let mean = series.iter().sum::<f64>() / n;
    series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
}

/// Lag-1 autocorrelation: correlation between `series[t]` and
/// `series[t+1]`. `None` if fewer than 3 points or ~zero variance on
/// either side.
pub fn lag1_autocorrelation(series: &[f64]) -> Option<f64> {
    if series.len() < 3 {
        return None;
    }
    let a = &series[..series.len() - 1];
    let b = &series[1..];
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    if var_a <= f64::EPSILON || var_b <= f64::EPSILON {
        return None;
    }
    Some(cov / (var_a.sqrt() * var_b.sqrt()))
}

/// Slope of a simple linear regression of `values` against their own
/// index (0, 1, 2, ...) -- "is this indicator trending upward over
/// successive windows." `None` if fewer than 2 points.
pub fn trend_slope(values: &[f64]) -> Option<f64> {
    let x: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
    ols_slope(&x, values)
}

/// One sliding window's worth of CSD indicators.
#[derive(Debug, Clone, Copy)]
pub struct WindowIndicators {
    pub lambda: f64,
    pub variance: f64,
    pub autocorrelation_lag1: f64,
}

/// Slide a `window_size`-wide window across `detrended` in steps of
/// `step`, computing (lambda, variance, lag-1 autocorrelation) per
/// window. A window whose lambda or autocorrelation is undefined
/// (degenerate/zero-variance) is skipped, not zero-filled -- a skipped
/// window must never silently read as "lambda = 0," a stable-looking
/// value that isn't real.
pub fn sliding_indicators(
    detrended: &[f64],
    window_size: usize,
    step: usize,
) -> Vec<WindowIndicators> {
    let step = step.max(1);
    let mut out = Vec::new();
    if window_size < 3 || detrended.len() < window_size {
        return out;
    }
    let mut start = 0;
    while start + window_size <= detrended.len() {
        let w = &detrended[start..start + window_size];
        if let (Some(lambda), Some(ac)) = (restoring_rate(w), lag1_autocorrelation(w)) {
            out.push(WindowIndicators {
                lambda,
                variance: variance(w),
                autocorrelation_lag1: ac,
            });
        }
        start += step;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strongly_mean_reverting_series_has_negative_lambda() {
        // x_{t+1} = 0.2 * x_t (strong mean reversion): dx = x_{t+1}-x_t
        // = -0.8*x_t, so the true lambda is exactly -0.8.
        let mut series = vec![10.0];
        for _ in 0..20 {
            series.push(0.2 * series.last().unwrap());
        }
        let lambda = restoring_rate(&series).unwrap();
        assert!(
            (lambda - (-0.8)).abs() < 0.05,
            "expected lambda close to -0.8, got {lambda}"
        );
    }

    #[test]
    fn a_flat_series_has_no_defined_lambda() {
        let series = vec![3.0; 10];
        assert!(restoring_rate(&series).is_none());
    }

    #[test]
    fn variance_of_constant_series_is_zero() {
        assert_eq!(variance(&[7.0; 5]), 0.0);
    }

    #[test]
    fn lag1_autocorrelation_of_a_strong_ramp_is_close_to_one() {
        let series: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let ac = lag1_autocorrelation(&series).unwrap();
        assert!(
            ac > 0.99,
            "a smooth ramp should be almost perfectly lag-1 autocorrelated, got {ac}"
        );
    }

    #[test]
    fn trend_slope_detects_a_rising_sequence() {
        let values = vec![-0.9, -0.8, -0.7, -0.6, -0.5];
        let slope = trend_slope(&values).unwrap();
        assert!(
            (slope - 0.1).abs() < 1e-9,
            "expected slope 0.1, got {slope}"
        );
    }

    #[test]
    fn sliding_indicators_produces_one_entry_per_window() {
        let series: Vec<f64> = (0..30).map(|i| (i as f64 * 0.3).sin()).collect();
        let windows = sliding_indicators(&series, 10, 5);
        // (30 - 10) / 5 + 1 = 5 windows
        assert_eq!(windows.len(), 5);
    }

    #[test]
    fn sliding_indicators_returns_empty_for_too_short_a_series() {
        let series = vec![1.0, 2.0, 3.0];
        assert!(sliding_indicators(&series, 10, 1).is_empty());
    }
}
