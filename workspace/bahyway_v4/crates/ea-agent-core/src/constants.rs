//! constants.rs — EaAgent sovereign mathematical constants.
#![forbid(unsafe_code)]

/// QUALITY_DIVISOR — Plimpton 322, Babylonian base-60 (4×60). Eternal.
pub const QUALITY_DIVISOR: f64 = 240.0;

/// GEM threshold — B11 ≥ 200 = Golden Record.
pub const GEM_B11: u8 = 200;

/// TAU_R7 — SPH density overflow threshold (Sovereign Law §7 sink).
pub const TAU_R7: u32 = 11;

/// RESONANCE_RADIUS — SPH kernel neighbourhood radius.
pub const RESONANCE_RADIUS: f64 = 0.15;

/// GEM_RATE_TARGET — 35.4% of particles must be GEM (ADR-004).
pub const GEM_RATE_TARGET: f64 = 0.354;

/// PAULI_EXCLUSION_THRESHOLD — minimum Hepta distance between two particles.
/// Below this → Pauli Violation → repulsion protocol triggered.
pub const PAULI_EXCLUSION_THRESHOLD: f64 = 0.05;

/// JORDAN_STABILITY_THRESHOLD — spectral radius below this = unstable tribe.
pub const JORDAN_STABILITY_THRESHOLD: f64 = 0.85;

/// JORDAN_COLLAPSE_THRESHOLD — spectral radius below this = imminent collapse.
pub const JORDAN_COLLAPSE_THRESHOLD: f64 = 0.50;

/// EA_AGENT_TRIBE_ID — EaAgent particle namespace.
pub const EA_AGENT_TRIBE_ID: u16 = 0x1300;

/// ABZU_PRECISION — epsilon for floating point comparisons.
pub const ABZU_PRECISION: f64 = 1e-10;

/// Hepta dimension weights (D1..D7) — must sum to 1.0.
pub const HEPTA_WEIGHTS: [f64; 7] = [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05];

/// Hepta dimension names (Ibn Wahshiyya 7D basis).
pub const HEPTA_NAMES: [&str; 7] = [
    "D1 Identity",
    "D2 Belonging",
    "D3 Domain (RED)",
    "D4 Quality (GREEN)",
    "D5 Freshness (BLUE)",
    "D6 Temporal",
    "D7 Integrity",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hepta_weights_sum_to_one() {
        let sum: f64 = HEPTA_WEIGHTS.iter().sum();
        assert!((sum - 1.0).abs() < ABZU_PRECISION,
            "Hepta weights must sum to 1.0, got {sum}");
    }

    #[test]
    fn quality_divisor_is_240() {
        assert_eq!(QUALITY_DIVISOR, 240.0);
    }

    #[test]
    fn gem_b11_is_200() {
        assert_eq!(GEM_B11, 200);
    }

    #[test]
    fn pauli_threshold_is_positive() {
        assert!(PAULI_EXCLUSION_THRESHOLD > 0.0);
    }

    #[test]
    fn jordan_collapse_below_stability() {
        assert!(JORDAN_COLLAPSE_THRESHOLD < JORDAN_STABILITY_THRESHOLD);
    }
}
