//! Harmony coefficient — cluster-wide quality uniformity measure.
//!
//! Measures how harmoniously a tribe's B11 scores are distributed.
//! A perfectly uniform tribe (all same B11) scores 1.0.
//! High variance → lower harmony → approaching fragmentation.
//!
//! Used by riksu-engine to weight arc strength, and by shedu-engine
//! to assess cluster-wide particle health during failure scenarios.

/// Compute the harmony coefficient for a tribe given their B11 quality bytes.
///
/// `H_harmony = 1 / (1 + σ/240)` where σ = standard deviation of B11 values.
///
/// Returns 0.0 for empty input, 1.0 for a perfectly uniform tribe.
pub fn harmony_coefficient(b11s: &[u8]) -> f32 {
    if b11s.is_empty() { return 0.0; }
    if b11s.len() == 1 { return 1.0; }

    let mean = b11s.iter().map(|&b| b as f32).sum::<f32>() / b11s.len() as f32;
    let variance = b11s.iter()
        .map(|&b| (b as f32 - mean).powi(2))
        .sum::<f32>()
        / b11s.len() as f32;
    let sigma = variance.sqrt();

    1.0 / (1.0 + sigma / 240.0)
}

/// Mean B11 quality for the tribe.
pub fn mean_b11(b11s: &[u8]) -> f32 {
    if b11s.is_empty() { return 0.0; }
    b11s.iter().map(|&b| b as f32).sum::<f32>() / b11s.len() as f32
}

/// Fraction of the tribe in GEM lane (B11 ≥ 200).
pub fn gem_fraction(b11s: &[u8]) -> f32 {
    if b11s.is_empty() { return 0.0; }
    b11s.iter().filter(|&&b| b >= 200).count() as f32 / b11s.len() as f32
}

/// Fraction of the tribe in DEAD lane (B11 < 60).
pub fn dead_fraction(b11s: &[u8]) -> f32 {
    if b11s.is_empty() { return 0.0; }
    b11s.iter().filter(|&&b| b < 60).count() as f32 / b11s.len() as f32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_zero() {
        assert_eq!(harmony_coefficient(&[]), 0.0);
    }

    #[test]
    fn single_particle_returns_one() {
        assert_eq!(harmony_coefficient(&[200]), 1.0);
    }

    #[test]
    fn uniform_tribe_returns_one() {
        let b11s = vec![180u8; 7];
        assert!((harmony_coefficient(&b11s) - 1.0).abs() < 1e-5,
            "uniform tribe must be 1.0");
    }

    #[test]
    fn mixed_tribe_less_than_one() {
        let b11s = [240u8, 200, 150, 100, 60, 30, 10];
        let h = harmony_coefficient(&b11s);
        assert!(h < 1.0);
        assert!(h > 0.0);
    }

    #[test]
    fn higher_variance_lower_harmony() {
        let uniform   = [180u8, 180, 180, 180, 180, 180, 180];
        let dispersed = [240u8, 200, 150, 100, 60, 30, 10];
        assert!(harmony_coefficient(&uniform) > harmony_coefficient(&dispersed));
    }

    #[test]
    fn gem_fraction_all_gem() {
        let b11s = [210u8, 220, 200, 240, 210, 200, 215];
        assert!((gem_fraction(&b11s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn dead_fraction_none_dead() {
        let b11s = [180u8, 160, 140, 120, 100, 80, 65];
        assert_eq!(dead_fraction(&b11s), 0.0);
    }

    #[test]
    fn mean_b11_correct() {
        let b11s = [100u8, 200];
        assert!((mean_b11(&b11s) - 150.0).abs() < 1e-4);
    }
}
