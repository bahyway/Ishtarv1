//! The Purussum Calculus -- NinurtaEngine's own reading of a
//! particle-collection's scalar observable in motion: detrend, estimate
//! the restoring rate lambda and its corroborating indicators over a
//! sliding window, test significance against Fourier surrogates, and
//! render a tau-scored verdict.
//!
//! Per the Topological Engine Division (GL-TED-001), this calculus IS
//! NinurtaEngine's own -- there is no separate "statistics engine."
//! The Girsu CSD Notebook design tablet's stray reference to a fourth
//! "ShamashEngine (statistics)" component is corrected here to
//! NinurtaEngine itself: Shamash already names this ecosystem's real
//! Gate 4 / State Exclusion (zombie/reincarnation judgment on Dead
//! particles -- see `docs/04_gates/shamash_gate.md`), an unrelated,
//! already-sealed name. Naming this calculus's home a second
//! "ShamashEngine" would collide with that seal.
//!
//! CSD is probabilistic evidence of destabilisation, never a schedule.
//! This module's whole output is a verdict with an honest confidence,
//! never a claimed transition tick.

use crate::detrend::running_mean_detrend;
use crate::indicators::{sliding_indicators, trend_slope};
use crate::surrogate::{fourier_surrogates, surrogate_p_value};

/// A verdict below this p-value is "significant." Matches the PIK
/// paper's own threshold; not a hidden magic number -- named here so
/// a caller reads the honesty gate rather than a bare `0.05` literal.
pub const SIGNIFICANCE_THRESHOLD: f64 = 0.05;

/// The rendered verdict: NinurtaEngine's answer to "is this shape
/// about to break," always carrying its own honesty.
#[derive(Debug, Clone, Copy)]
pub struct PurussumVerdict {
    /// The most recent window's lambda estimate.
    pub lambda: f64,
    /// Slope of lambda across successive windows -- positive means
    /// lambda is rising toward 0, the CSD alarm signal.
    pub lambda_trend: f64,
    /// Slope of lag-1 autocorrelation across successive windows --
    /// corroborates lambda_trend, never trusted alone.
    pub autocorrelation_trend: f64,
    /// The surrogate p-value on lambda_trend -- doubles as the
    /// tau-confidence: lower is more significant.
    pub p_value: f64,
    /// True only when lambda_trend > 0 AND p_value < SIGNIFICANCE_THRESHOLD
    /// -- both the direction and the honesty gate must agree.
    pub significant: bool,
}

/// Run the full Purussum Calculus over `raw_series` (a tribe/orbit/
/// BIGRING's own scalar observable history, oldest-first): detrend,
/// slide `window_size`-wide windows (stepping by `step`) computing
/// lambda/variance/autocorrelation per window, test the lambda trend's
/// significance against `surrogate_count` Fourier surrogates of the
/// detrended series. Returns `None` if there are too few windows
/// (fewer than 2 -- a trend needs at least two points) to compute a
/// trend at all.
pub fn render_verdict(
    raw_series: &[f64],
    window_size: usize,
    step: usize,
    detrend_window: usize,
    surrogate_count: usize,
    seed: u64,
) -> Option<PurussumVerdict> {
    let detrended = running_mean_detrend(raw_series, detrend_window);
    let windows = sliding_indicators(&detrended, window_size, step);
    if windows.len() < 2 {
        return None;
    }
    let lambdas: Vec<f64> = windows.iter().map(|w| w.lambda).collect();
    let autocorrs: Vec<f64> = windows.iter().map(|w| w.autocorrelation_lag1).collect();
    let lambda_trend = trend_slope(&lambdas)?;
    let autocorrelation_trend = trend_slope(&autocorrs).unwrap_or(0.0);

    let surrogates = fourier_surrogates(&detrended, surrogate_count, seed);
    let surrogate_trends: Vec<f64> = surrogates
        .iter()
        .filter_map(|s| {
            let sw = sliding_indicators(s, window_size, step);
            if sw.len() < 2 {
                return None;
            }
            let sl: Vec<f64> = sw.iter().map(|w| w.lambda).collect();
            trend_slope(&sl)
        })
        .collect();
    let p_value = surrogate_p_value(lambda_trend, &surrogate_trends);

    Some(PurussumVerdict {
        lambda: windows.last().unwrap().lambda,
        lambda_trend,
        autocorrelation_trend,
        p_value,
        significant: lambda_trend > 0.0 && p_value < SIGNIFICANCE_THRESHOLD,
    })
}

/// The EAV trichotomy this calculus watches for, per the Self-Modelling
/// Data Trilogy's mapping: GOLDEN (stable, lambda well below zero),
/// FUZZY (drifting toward the bifurcation edge), DEAD (at or past the
/// fixed point, lambda >= 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LambdaTrichotomy {
    Golden,
    Fuzzy,
    Dead,
}

/// Maps a lambda estimate to the trichotomy. `fuzzy_threshold` is the
/// GOLDEN/FUZZY boundary (must be < 0.0) -- a tunable boundary the
/// caller supplies (e.g. loaded from a governance record, mirroring
/// this ecosystem's own `eps_geo`/`h_min`-from-governance convention),
/// never a magic constant hidden inside this function.
pub fn trichotomy_for_lambda(lambda: f64, fuzzy_threshold: f64) -> LambdaTrichotomy {
    if lambda >= 0.0 {
        LambdaTrichotomy::Dead
    } else if lambda >= fuzzy_threshold {
        LambdaTrichotomy::Fuzzy
    } else {
        LambdaTrichotomy::Golden
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic synthetic Ornstein-Uhlenbeck-like process:
    /// `x_{t+1} = x_t * (1 + lambda_t) + noise_t`, `lambda_t` sliding
    /// linearly from `lambda_start` to `lambda_end` across the series,
    /// with a FIXED-amplitude additive noise term injected every step
    /// (not just at t=0) so the process keeps real dynamic range at
    /// every point rather than decaying to the noise floor and
    /// staying there once mean-reversion is strong -- a stationary
    /// AR(1) process's own variance is `noise_var / (1 - (1+lambda)^2)`,
    /// which genuinely grows as lambda rises toward 0, giving the
    /// later windows real signal to regress on, not just noise.
    fn synthetic_series(len: usize, lambda_start: f64, lambda_end: f64, noise_scale: f64, seed: u64) -> Vec<f64> {
        let mut x = 0.0_f64;
        let mut out = Vec::with_capacity(len);
        let mut s = seed;
        for i in 0..len {
            out.push(x);
            let frac = i as f64 / len.max(1) as f64;
            let lambda = lambda_start + frac * (lambda_end - lambda_start);
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let u = (s % 100_000) as f64 / 100_000.0 - 0.5;
            let noise = u * noise_scale;
            x = x * (1.0 + lambda) + noise;
        }
        out
    }

    /// lambda weakens from -0.9 (strongly stable) to -0.05 (near the
    /// bifurcation edge) across the series -- the real critical-
    /// slowing-down signature this calculus exists to catch.
    fn destabilizing_series(len: usize) -> Vec<f64> {
        synthetic_series(len, -0.9, -0.05, 1.0, 12345)
    }

    /// lambda stays deeply negative the whole way through -- stable,
    /// no approach to any transition.
    fn stable_series(len: usize) -> Vec<f64> {
        synthetic_series(len, -0.9, -0.9, 1.0, 999)
    }

    #[test]
    fn a_destabilizing_series_produces_a_positive_lambda_trend() {
        let series = destabilizing_series(120);
        let verdict = render_verdict(&series, 20, 5, 5, 200, 1).expect("enough windows to compute a trend");
        assert!(verdict.lambda_trend > 0.0, "expected lambda to trend upward toward instability, got {}", verdict.lambda_trend);
    }

    #[test]
    fn a_stable_series_has_a_lambda_trend_near_zero() {
        let series = stable_series(120);
        let verdict = render_verdict(&series, 20, 5, 5, 200, 2).expect("enough windows to compute a trend");
        assert!(verdict.lambda_trend.abs() < 0.05, "a stable series' lambda should not trend meaningfully, got {}", verdict.lambda_trend);
    }

    #[test]
    fn too_short_a_series_returns_none() {
        let series = vec![1.0, 2.0, 3.0];
        assert!(render_verdict(&series, 20, 5, 5, 50, 1).is_none());
    }

    #[test]
    fn trichotomy_golden_below_threshold() {
        assert_eq!(trichotomy_for_lambda(-0.9, -0.3), LambdaTrichotomy::Golden);
    }

    #[test]
    fn trichotomy_fuzzy_between_threshold_and_zero() {
        assert_eq!(trichotomy_for_lambda(-0.1, -0.3), LambdaTrichotomy::Fuzzy);
    }

    #[test]
    fn trichotomy_dead_at_or_past_zero() {
        assert_eq!(trichotomy_for_lambda(0.0, -0.3), LambdaTrichotomy::Dead);
        assert_eq!(trichotomy_for_lambda(0.2, -0.3), LambdaTrichotomy::Dead);
    }
}
