//! GCC-PHAT (phase transform) cross-correlation delay estimator.
//! Valid on non-dispersive (metal) pipe. Pure Rust via rustfft.

use rustfft::{num_complex::Complex, FftPlanner};

use super::DelayEstimate;

/// Estimate the delay of `y` relative to `x` (positive = y lags x).
pub fn gcc_phat_delay(x: &[f64], y: &[f64], sample_rate_hz: f64) -> Option<DelayEstimate> {
    let n = x.len();
    if n < 8 || y.len() != n || sample_rate_hz <= 0.0 {
        return None;
    }
    // Zero-pad to 2n (next power of two) to avoid circular aliasing.
    let m = (2 * n).next_power_of_two();
    let mut fx: Vec<Complex<f64>> = x
        .iter()
        .map(|&v| Complex::new(v, 0.0))
        .chain(std::iter::repeat(Complex::new(0.0, 0.0)))
        .take(m)
        .collect();
    let mut fy: Vec<Complex<f64>> = y
        .iter()
        .map(|&v| Complex::new(v, 0.0))
        .chain(std::iter::repeat(Complex::new(0.0, 0.0)))
        .take(m)
        .collect();

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(m);
    let ifft = planner.plan_fft_inverse(m);
    fft.process(&mut fx);
    fft.process(&mut fy);

    // Cross-spectrum with PHAT weighting: R = X*conj(Y) / |X*conj(Y)|
    let mut r: Vec<Complex<f64>> = fx
        .iter()
        .zip(fy.iter())
        .map(|(a, b)| {
            let c = a * b.conj();
            let mag = c.norm();
            if mag > 1e-12 {
                c / mag
            } else {
                Complex::new(0.0, 0.0)
            }
        })
        .collect();
    ifft.process(&mut r);

    // Peak search over lags, wraparound-aware.
    let mut best_idx = 0usize;
    let mut best_val = f64::MIN;
    let mut energy = 0.0_f64;
    for (i, c) in r.iter().enumerate() {
        let v = c.re;
        energy += v.abs();
        if v > best_val {
            best_val = v;
            best_idx = i;
        }
    }
    if energy <= 0.0 {
        return None;
    }
    // Index > m/2 means a negative lag: y LEADS x.
    let lag_samples = if best_idx > m / 2 { best_idx as f64 - m as f64 } else { best_idx as f64 };
    // Sign convention check: R = X*conj(Y) peaks at lag = -(delay of y).
    // We therefore negate to report "delay of y relative to x".
    let delay_s = -lag_samples / sample_rate_hz;

    Some(DelayEstimate { delay_s, peak_quality: (best_val / energy).clamp(0.0, 1.0) })
}
