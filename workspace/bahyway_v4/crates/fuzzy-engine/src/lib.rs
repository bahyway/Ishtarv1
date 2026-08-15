// ============================================================================
// fuzzy-engine/src/lib.rs
// FuzzyEngine v3.5.1 — Mamdani Fuzzy Logic Quality Scoring (ADR-005, ADR-006)
// Pure std — zero third-party dependencies.
// ============================================================================

pub mod defuzz;
pub mod dimensions;
pub mod format_layer;
pub mod membership;
pub mod rules;

pub use defuzz::{centroid_defuzz, quality_tier_label};
pub use dimensions::{FuzzifiedDimensions, FuzzyDimensions, SourceTrust};
pub use format_layer::FormatLayer;
pub use rules::{evaluate_rules, AggregatedOutput, QualityTier, RuleFiring};

pub const FUZZY_ENGINE_VERSION: &str = "3.5.1";

/// B11 threshold: scores ≥ this value are Gem (Golden Record).
pub const GEM_THRESHOLD:    u8 = 200;
/// B11 threshold: scores ≥ this value are Tribe (near-golden).
pub const TRIBE_THRESHOLD:  u8 = 140;
/// B11 threshold: scores ≥ this value are Active (fuzzy / uncertain).
pub const ACTIVE_THRESHOLD: u8 = 100;

/// Top-level engine — single entry point for scoring a particle.
pub struct FuzzyEngine;

impl FuzzyEngine {
    /// Score a particle given its fuzzy dimensions and format layer.
    ///
    /// Pipeline: fuzzify → evaluate_rules → centroid_defuzz → apply layer multiplier.
    /// Returns quality byte B11 (0–255).
    pub fn score(dims: &FuzzyDimensions, format: FormatLayer) -> u8 {
        let raw = Self::score_raw(dims);
        let scaled = (raw as f32 * format.score_multiplier()).round().clamp(0.0, 255.0);
        scaled as u8
    }

    /// Score without format layer multiplier (raw centroid output).
    pub fn score_raw(dims: &FuzzyDimensions) -> u8 {
        let fuzzified = dims.fuzzify();
        let aggregated = evaluate_rules(&fuzzified);
        centroid_defuzz(&aggregated)
    }

    /// Classify a B11 quality byte into a QualityTier.
    pub fn classify(b11: u8) -> QualityTier {
        if b11 >= GEM_THRESHOLD    { QualityTier::Gem }
        else if b11 >= TRIBE_THRESHOLD  { QualityTier::Tribe }
        else if b11 >= ACTIVE_THRESHOLD { QualityTier::Active }
        else                            { QualityTier::NonActive }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_record_scores_gem() {
        let dims = FuzzyDimensions::perfect();
        let b11  = FuzzyEngine::score(&dims, FormatLayer::CivilRegistry);
        assert!(b11 >= GEM_THRESHOLD, "perfect CivilRegistry must be Gem, got {b11}");
    }

    #[test]
    fn format_layer_multiplier_lowers_score() {
        let dims   = FuzzyDimensions::perfect();
        let raw    = FuzzyEngine::score_raw(&dims);
        let scaled = FuzzyEngine::score(&dims, FormatLayer::GenericEav);
        assert!(scaled <= raw, "GenericEav multiplier must not exceed raw score");
    }

    #[test]
    fn classify_boundaries() {
        assert_eq!(FuzzyEngine::classify(255), QualityTier::Gem);
        assert_eq!(FuzzyEngine::classify(200), QualityTier::Gem);
        assert_eq!(FuzzyEngine::classify(199), QualityTier::Tribe);
        assert_eq!(FuzzyEngine::classify(140), QualityTier::Tribe);
        assert_eq!(FuzzyEngine::classify(139), QualityTier::Active);
        assert_eq!(FuzzyEngine::classify(100), QualityTier::Active);
        assert_eq!(FuzzyEngine::classify(99),  QualityTier::NonActive);
        assert_eq!(FuzzyEngine::classify(0),   QualityTier::NonActive);
    }

    #[test]
    fn version_string_is_correct() {
        assert_eq!(FUZZY_ENGINE_VERSION, "3.5.1");
    }

    #[test]
    fn score_raw_and_score_civil_registry_equal_for_perfect() {
        let dims = FuzzyDimensions::perfect();
        let raw  = FuzzyEngine::score_raw(&dims);
        let with_layer = FuzzyEngine::score(&dims, FormatLayer::CivilRegistry);
        // CivilRegistry multiplier is 1.0 — should be identical
        assert_eq!(raw, with_layer);
    }
}
