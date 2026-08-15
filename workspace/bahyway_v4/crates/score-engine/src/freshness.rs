//! FreshnessDecay — the δₜ metric (data age and currency).
//!
//! Freshness starts at 1.0 at birth and decays over time.
//! It maps to the BLUE byte of color_rgb.
//! The decay rate is set per-template (fast for sensor streams, slow for civil records).

/// Freshness decay model.
#[derive(Clone, Debug)]
pub struct FreshnessDecay {
    /// Decay constant λ (per second).  Higher = faster decay.
    pub lambda: f32,
}

impl FreshnessDecay {
    /// Slow decay — civil registry records (birth/death): ~1 year half-life.
    pub fn civil_registry() -> Self { FreshnessDecay { lambda: 1.0 / (365.25 * 86400.0) } }

    /// Medium decay — operational data: ~30-day half-life.
    pub fn operational() -> Self { FreshnessDecay { lambda: 1.0 / (30.0 * 86400.0) } }

    /// Fast decay — sensor streams: ~1-hour half-life.
    pub fn sensor_stream() -> Self { FreshnessDecay { lambda: 1.0 / 3600.0 } }

    /// Compute freshness at `elapsed_seconds` since birth.
    /// δₜ = e^(-λ·t), clamped to [0.0, 1.0].
    pub fn compute(&self, elapsed_seconds: f64) -> f32 {
        let v = (-self.lambda as f64 * elapsed_seconds).exp() as f32;
        v.clamp(0.0, 1.0)
    }

    pub fn to_blue_byte(&self, elapsed_seconds: f64) -> u8 {
        (self.compute(elapsed_seconds) * 255.0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_at_birth_is_one() {
        let d = FreshnessDecay::operational();
        assert!((d.compute(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn freshness_decays_over_time() {
        let d = FreshnessDecay::sensor_stream();
        let fresh = d.compute(0.0);
        let later = d.compute(7200.0); // 2 hours later
        assert!(later < fresh, "freshness should decay over time");
    }

    #[test]
    fn civil_registry_stays_fresh_for_days() {
        let d = FreshnessDecay::civil_registry();
        let one_week = d.compute(7.0 * 86400.0);
        // After 7 days with 1-year half-life: e^(-7/365.25) ≈ 0.981 — still very fresh
        assert!(one_week > 0.97, "civil registry should stay fresh for weeks");
    }
}
