//! Dispersion-compensated correlation for viscoelastic (PE/PVC) pipe.
//!
//! On plastic, phase velocity is frequency-dependent; a single-v correlator
//! is mathematically wrong and can miss by metres. We de-chirp the
//! cross-spectrum before the inverse transform using a first-order
//! phase-velocity model calibrated per segment (reference-tap sweep):
//!
//! >   v(f) = v_ref * (1 + alpha * (f - f_ref)/f_ref)
//!
//! The dispersion-induced excess phase over a propagation range R is
//! >   Δφ(f) = 2*pi*f*R * (1/v(f) - 1/v_ref)
//!
//! which requires a coarse range estimate -- so compensation is ITERATIVE:
//! run plain GCC-PHAT for a coarse R, compensate, re-estimate, repeat.
//! `iterate_compensated_delay` performs this loop. alpha = 0 degrades
//! gracefully to plain GCC-PHAT.

use rustfft::{num_complex::Complex, FftPlanner};

use super::gcc_phat::gcc_phat_delay;
use super::DelayEstimate;

#[derive(Debug, Clone, Copy)]
pub struct DispersionModel {
    pub v_ref_mps: f64,
    pub f_ref_hz: f64,
    /// First-order dispersion slope from per-segment calibration sweep.
    pub alpha: f64,
}

impl DispersionModel {
    pub fn v_of_f(&self, f_hz: f64) -> f64 {
        let rel = (f_hz.abs() - self.f_ref_hz) / self.f_ref_hz.max(1e-9);
        (self.v_ref_mps * (1.0 + self.alpha * rel)).max(1e-3)
    }
}

/// Single compensation pass at an assumed propagation range.
pub fn compensated_delay(
    x: &[f64],
    y: &[f64],
    sample_rate_hz: f64,
    model: &DispersionModel,
    assumed_range_m: f64,
) -> Option<DelayEstimate> {
    let n = x.len();
    if n < 8 || y.len() != n || sample_rate_hz <= 0.0 {
        return None;
    }
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

    let mut r: Vec<Complex<f64>> = (0..m)
        .map(|k| {
            let c = fx[k] * fy[k].conj();
            let mag = c.norm();
            if mag <= 1e-12 {
                return Complex::new(0.0, 0.0);
            }
            let phat = c / mag;
            let f = if k <= m / 2 {
                k as f64 * sample_rate_hz / m as f64
            } else {
                (k as f64 - m as f64) * sample_rate_hz / m as f64
            };
            // Remove dispersion-induced excess phase over the assumed range.
            let excess = 2.0
                * std::f64::consts::PI
                * f
                * assumed_range_m
                * (1.0 / model.v_of_f(f) - 1.0 / model.v_ref_mps);
            phat * Complex::from_polar(1.0, excess)
        })
        .collect();
    ifft.process(&mut r);

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
    let lag_samples = if best_idx > m / 2 {
        best_idx as f64 - m as f64
    } else {
        best_idx as f64
    };
    Some(DelayEstimate {
        delay_s: -lag_samples / sample_rate_hz,
        peak_quality: (best_val / energy).clamp(0.0, 1.0),
    })
}

/// Iterative refinement: coarse GCC-PHAT -> compensate -> converge.
pub fn iterate_compensated_delay(
    x: &[f64],
    y: &[f64],
    sample_rate_hz: f64,
    model: &DispersionModel,
    sensor_spacing_m: f64,
    max_iter: usize,
) -> Option<DelayEstimate> {
    let mut est = gcc_phat_delay(x, y, sample_rate_hz)?;
    for _ in 0..max_iter.max(1) {
        let d =
            ((sensor_spacing_m - model.v_ref_mps * est.delay_s) / 2.0).clamp(0.0, sensor_spacing_m);
        // Dominant propagation range = the longer leg to a sensor.
        let range = d.max(sensor_spacing_m - d);
        let next = compensated_delay(x, y, sample_rate_hz, model, range)?;
        let converged = (next.delay_s - est.delay_s).abs() * model.v_ref_mps < 0.01;
        est = next;
        if converged {
            break;
        }
    }
    Some(est)
}
