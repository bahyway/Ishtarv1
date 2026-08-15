//! ColorRgb — the three-byte color encoding for a particle's assessment state.
//!
//! RED   = domain classification byte (set once by template, rarely changes)
//! GREEN = quality (HPS score mapped 0.0→0x00 .. 1.0→0xFF)
//! BLUE  = freshness (δₜ metric mapped 0.0→0x00 .. 1.0→0xFF)
//!
//! Gray   = (0x80, 0x80, 0x80) — Dead/Ground State
//! Golden = (domain, 0xFF, 0xFF) — perfect quality + perfect freshness

use bahyway_core::ParticleState;

/// The three color bytes encoding a particle's current assessment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorRgb {
    /// D3 — domain classification byte (0x00..0xFF per template).
    pub r: u8,
    /// D4 — quality (HPS score → u8).
    pub g: u8,
    /// D5 — freshness (δₜ → u8).
    pub b: u8,
}

impl ColorRgb {
    pub const GRAY:   ColorRgb = ColorRgb { r: 0x80, g: 0x80, b: 0x80 };
    pub const GOLDEN: ColorRgb = ColorRgb { r: 0xFF, g: 0xFF, b: 0xFF };

    pub fn new(r: u8, g: u8, b: u8) -> Self { ColorRgb { r, g, b } }

    /// Serialize to 3 bytes (wire / EAV storage format).
    pub fn to_bytes(self) -> [u8; 3] { [self.r, self.g, self.b] }

    pub fn from_bytes(raw: [u8; 3]) -> Self { ColorRgb { r: raw[0], g: raw[1], b: raw[2] } }

    /// Euclidean distance to the origin color (0, 0, 0) — proxy for "how gray".
    pub fn magnitude(self) -> f32 {
        ((self.r as f32).powi(2)
            + (self.g as f32).powi(2)
            + (self.b as f32).powi(2))
        .sqrt()
    }

    /// Drift toward Gray: distance from a pure Gray (128, 128, 128).
    pub fn drift_from_gray(self) -> f32 {
        let dr = (self.r as f32 - 128.0).powi(2);
        let dg = (self.g as f32 - 128.0).powi(2);
        let db = (self.b as f32 - 128.0).powi(2);
        (dr + dg + db).sqrt()
    }
}

/// Compute the ColorRgb for a particle from its current quality + freshness scores.
/// Domain byte is passed in from the Template definition.
pub fn compute_color(
    domain_byte: u8,
    quality:     f32,     // HPS 0.0–1.0
    freshness:   f32,     // δₜ 0.0–1.0
    state:       ParticleState,
) -> ColorRgb {
    if state == ParticleState::Dead {
        return ColorRgb::GRAY;
    }
    let g = (quality.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (freshness.clamp(0.0, 1.0) * 255.0) as u8;
    ColorRgb::new(domain_byte, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_particle_is_gray() {
        let c = compute_color(0xFF, 1.0, 1.0, ParticleState::Dead);
        assert_eq!(c, ColorRgb::GRAY);
    }

    #[test]
    fn perfect_quality_and_freshness() {
        let c = compute_color(0xAA, 1.0, 1.0, ParticleState::Golden);
        assert_eq!(c.g, 0xFF);
        assert_eq!(c.b, 0xFF);
        assert_eq!(c.r, 0xAA);
    }

    #[test]
    fn quality_decay_lowers_green() {
        let c = compute_color(0x10, 0.5, 1.0, ParticleState::Golden);
        assert_eq!(c.g, 127); // 0.5 * 255 = 127
        assert_eq!(c.b, 0xFF);
    }

    #[test]
    fn round_trip_bytes() {
        let c  = ColorRgb::new(10, 200, 50);
        let c2 = ColorRgb::from_bytes(c.to_bytes());
        assert_eq!(c, c2);
    }
}
