//! LAW-ALE-3: v is measured, not remembered.

use crate::material::PipeMaterial;

#[derive(Debug, Clone, Copy)]
pub struct SegmentCalibration {
    pub material: PipeMaterial,
    pub length_m: f64,
    /// In-situ wave speed from a timed reference tap AT MEASUREMENT TIME.
    pub v_mps: f64,
    /// Relative uncertainty of v (e.g. 0.001 = 0.1%). Feeds sigma_d directly.
    pub sigma_v_rel: f64,
    /// Water temperature at calibration (degC) -- recalibrate if drifted.
    pub water_temp_c: f64,
}

impl SegmentCalibration {
    /// Calibrate v from a reference tap: known distance / measured delay.
    pub fn from_reference_tap(
        material: PipeMaterial,
        length_m: f64,
        tap_distance_m: f64,
        tap_delay_s: f64,
        tap_timing_sd_s: f64,
        water_temp_c: f64,
    ) -> Option<Self> {
        if tap_delay_s <= 0.0 || tap_distance_m <= 0.0 {
            return None;
        }
        let v = tap_distance_m / tap_delay_s;
        Some(Self {
            material,
            length_m,
            v_mps: v,
            sigma_v_rel: tap_timing_sd_s / tap_delay_s,
            water_temp_c,
        })
    }
}

/// Mixed-material path: total travel time integrates piecewise,
/// dt = sum(Li / vi). A single-v model across mixed segments is unlawful.
pub fn piecewise_travel_time_s(path: &[SegmentCalibration]) -> f64 {
    path.iter().map(|s| s.length_m / s.v_mps).sum()
}

/// Two-sensor localization on a single calibrated segment:
/// d(from sensor A) = (L - v*dt) / 2, where dt = t_A - t_B.
pub fn localize_two_sensor(seg: &SegmentCalibration, delta_t_s: f64) -> f64 {
    (seg.length_m - seg.v_mps * delta_t_s) / 2.0
}
