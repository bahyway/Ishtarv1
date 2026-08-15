//! HpsScore — Holistic Particle Score (quality metric, 0.0–1.0).
//!
//! The HPS is computed from validation results at ingestion and updated
//! by the Score Engine as Event-Kakis arrive.  It maps to the GREEN byte
//! of color_rgb.  A particle at 0.95 is "healthy Golden", at 0.15 is Fuzzy.

/// HPS score clamped to [0.0, 1.0].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct HpsScore(f32);

impl HpsScore {
    pub const PERFECT:  HpsScore = HpsScore(1.0);
    pub const ZERO:     HpsScore = HpsScore(0.0);

    pub fn new(v: f32) -> Self { HpsScore(v.clamp(0.0, 1.0)) }

    pub fn value(self) -> f32 { self.0 }

    /// Classify into Golden / Fuzzy / Dead based on HPS thresholds.
    /// Thresholds can be overridden by Template — these are defaults.
    pub fn to_state(self) -> bahyway_core::ParticleState {
        match self.0 {
            v if v >= 0.8 => bahyway_core::ParticleState::Golden,
            v if v >= 0.3 => bahyway_core::ParticleState::Fuzzy,
            _             => bahyway_core::ParticleState::Dead,
        }
    }

    pub fn to_green_byte(self) -> u8 {
        (self.0 * 255.0) as u8
    }
}

impl From<f32> for HpsScore {
    fn from(v: f32) -> Self { HpsScore::new(v) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds() {
        assert_eq!(HpsScore::new(0.95).to_state(), bahyway_core::ParticleState::Golden);
        assert_eq!(HpsScore::new(0.50).to_state(), bahyway_core::ParticleState::Fuzzy);
        assert_eq!(HpsScore::new(0.10).to_state(), bahyway_core::ParticleState::Dead);
    }

    #[test]
    fn clamping() {
        assert_eq!(HpsScore::new(2.0).value(), 1.0);
        assert_eq!(HpsScore::new(-1.0).value(), 0.0);
    }

    #[test]
    fn green_byte_mapping() {
        assert_eq!(HpsScore::new(1.0).to_green_byte(), 255);
        assert_eq!(HpsScore::new(0.0).to_green_byte(), 0);
    }
}
