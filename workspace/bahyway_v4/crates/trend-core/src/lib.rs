//! trend-core — least-squares trend-to-threshold, implemented once.
//!
//! This is the same math three places in the ecosystem independently
//! want: Enbilulu's horizon_weeks (bahyway-algebra::enbilulu), the
//! Girra/Gibil horizon (egd-engine::horizon), and Marduk's T_golden
//! (BC-MRD-001 §3.1, "golden horizon"). Rather than a fourth copy, this
//! crate provides the one core primitive and lets each domain wrap it
//! with its own threshold, time unit, and sign convention (BC-MRD-001
//! §3.6: some domains treat inward drift as healthy, others the
//! opposite -- that convention belongs to the domain, not the trend
//! fit itself).
//!
//! NOTE: `bahyway-algebra::enbilulu::enbi_horizon_weeks` is NOT changed
//! to depend on this crate. WPDEngine/Enbilulu is sealed source (BLK-1
//! lesson) and is left untouched; this crate exists so *new* domains
//! (Gibil, Nabu Calculus) don't add a fourth reimplementation.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendError {
    /// Fewer than two samples: no line can be fit.
    InsufficientHistory,
}

/// One (t, value) sample. `t` is domain time relative to "now" (t=0);
/// negative t = past. Callers choose the unit (days, weeks, ...) and
/// must use it consistently within one history.
pub type Sample = (f64, f64);

/// Ordinary least-squares fit of `history`, then solves for the time
/// at which the fitted line crosses `threshold`.
///
///   `Ok(Some(d))` with `d >= 0.0` — crossing predicted in `d` time units
///   `Ok(Some(0.0))`                — already at or over threshold now
///   `Ok(None)`                     — flat or improving trend; never crosses
///   `Err(InsufficientHistory)`     — fewer than 2 samples
pub fn time_to_threshold(history: &[Sample], threshold: f64) -> Result<Option<f64>, TrendError> {
    let n = history.len() as f64;
    if n < 2.0 {
        return Err(TrendError::InsufficientHistory);
    }
    let (mut st, mut sv, mut stt, mut stv) = (0.0, 0.0, 0.0, 0.0);
    for &(t, v) in history {
        st += t;
        sv += v;
        stt += t * t;
        stv += t * v;
    }
    let denom = n * stt - st * st;
    if denom.abs() < 1e-12 {
        return Ok(None); // no time spread in the samples; no trend to fit
    }
    let slope = (n * stv - st * sv) / denom;
    let intercept = (sv - slope * st) / n; // value at t=0 (now)
    if intercept >= threshold {
        return Ok(Some(0.0));
    }
    // A perfectly flat series' least-squares slope is rarely exactly
    // 0.0 in floating point (e.g. ~1e-17) -- an exact `slope <= 0.0`
    // check lets that noise through and divides by it, producing a
    // nonsensical huge "days to threshold". Require a slope clearly
    // distinguishable from flat before projecting a crossing.
    const FLAT_SLOPE_EPSILON: f64 = 1e-9;
    if slope < FLAT_SLOPE_EPSILON {
        return Ok(None); // not meaningfully degrading toward the threshold
    }
    Ok(Some((threshold - intercept) / slope))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insufficient_history_is_an_error() {
        assert_eq!(time_to_threshold(&[(0.0, 1.0)], 5.0), Err(TrendError::InsufficientHistory));
        assert_eq!(time_to_threshold(&[], 5.0), Err(TrendError::InsufficientHistory));
    }

    #[test]
    fn degrading_series_predicts_correct_crossing() {
        // value rising 0.01/day, now at 0.5, threshold 1.0 -> ~50 days
        let h: Vec<Sample> = (0..60).map(|i| (-(60 - i) as f64, 0.5 - (60 - i) as f64 * 0.01)).collect();
        match time_to_threshold(&h, 1.0) {
            Ok(Some(d)) => assert!((d - 50.0).abs() < 2.0, "expected ~50, got {d}"),
            other => panic!("expected Some(~50), got {other:?}"),
        }
    }

    #[test]
    fn stable_series_never_crosses() {
        let h: Vec<Sample> = (0..60).map(|i| (-(i as f64), 0.3)).collect();
        assert_eq!(time_to_threshold(&h, 1.0), Ok(None));
    }

    #[test]
    fn already_over_threshold_returns_zero() {
        let h: Vec<Sample> = vec![(-10.0, 2.0), (-5.0, 2.0), (0.0, 2.0)];
        assert_eq!(time_to_threshold(&h, 1.0), Ok(Some(0.0)));
    }

    #[test]
    fn improving_series_never_crosses() {
        // mirror of degrading_series_predicts_correct_crossing: value
        // falling 0.01/day, now at 0.5, well under threshold 1.0, and
        // moving further away from it -> never crosses.
        let h: Vec<Sample> = (0..60).map(|i| (-(60 - i) as f64, 0.5 + (60 - i) as f64 * 0.01)).collect();
        assert_eq!(time_to_threshold(&h, 1.0), Ok(None));
    }
}
