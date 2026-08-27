// ============================================================================
// score-engine/src/fuzzy_scorer.rs
// Bridge: FuzzyDimensions → B11 → HpsScore → ParticleState → ColorRgb → EAV
//
// Tier mapping (ADR-005/ADR-006):
//   Gem    (B11 ≥ 200) → ParticleState::Golden
//   Tribe  (B11 140–199) → ParticleState::Golden
//   Active (B11 100–139) → ParticleState::Fuzzy
//   NonActive (B11 < 100) → ParticleState::Dead
// ============================================================================
#![allow(missing_docs)]

use bahyway_core::ParticleState;
use enkidb_journal::entry::EavTriple;
use fuzzy_engine::{FormatLayer, FuzzyDimensions, FuzzyEngine, QualityTier};
use story_engine::projection::{
    encode_state, ATTR_COLOR_RGB, ATTR_FRESHNESS, ATTR_QUALITY, ATTR_STATE,
};

use crate::color_id::{compute_color, ColorRgb};
use crate::freshness::FreshnessDecay;
use crate::hps::HpsScore;

/// Full input for scoring one particle.
pub struct ScoreInput {
    /// The 8 fuzzy dimensions for this particle (D1–D8).
    pub dims: FuzzyDimensions,
    /// Format layer — applies a score multiplier (ADR-006).
    pub format: FormatLayer,
    /// RED byte for ColorRgb — domain classification from Template.
    pub domain_byte: u8,
    /// Seconds elapsed since this particle was first registered (for freshness).
    pub elapsed_seconds: f64,
    /// Freshness decay model (civil_registry, operational, sensor_stream, or custom λ).
    pub decay: FreshnessDecay,
    /// Accumulated orbital trust penalty from orbital-trust-probe (0.0–1.0).
    /// Written to D9 of FuzzyDimensions before scoring. Default: 0.0 (no violations).
    pub orbital_trust_penalty: f32,
}

impl ScoreInput {
    /// Convenience constructor with civil_registry decay (most common sovereign use case).
    /// Orbital trust penalty defaults to 0.0 — no violations observed.
    pub fn civil(dims: FuzzyDimensions, domain_byte: u8, elapsed_seconds: f64) -> Self {
        ScoreInput {
            dims,
            format: FormatLayer::CivilRegistry,
            domain_byte,
            elapsed_seconds,
            decay: FreshnessDecay::civil_registry(),
            orbital_trust_penalty: 0.0,
        }
    }

    /// Convenience constructor that includes an orbital trust penalty.
    pub fn civil_with_penalty(
        dims: FuzzyDimensions,
        domain_byte: u8,
        elapsed_seconds: f64,
        penalty: f32,
    ) -> Self {
        ScoreInput {
            dims,
            format: FormatLayer::CivilRegistry,
            domain_byte,
            elapsed_seconds,
            decay: FreshnessDecay::civil_registry(),
            orbital_trust_penalty: penalty,
        }
    }
}

/// Complete scoring result for one particle.
#[derive(Debug, Clone)]
pub struct ScoreResult {
    /// Quality byte B11 from the fuzzy pipeline (0–255).
    pub b11: u8,
    /// Quality tier from fuzzy output.
    pub tier: QualityTier,
    /// Sovereign lifecycle state derived from tier.
    pub state: ParticleState,
    /// Holistic Particle Score — B11 normalised to [0.0, 1.0].
    pub hps: HpsScore,
    /// Three-byte color encoding for this particle.
    pub color: ColorRgb,
    /// Freshness δₜ at the given elapsed time.
    pub freshness: f32,
}

impl ScoreResult {
    /// Human-readable quality tier label: "Gem", "Tribe", "Active", or "Dead".
    pub fn tier_label(&self) -> &'static str {
        fuzzy_engine::quality_tier_label(self.b11)
    }

    /// Produce the 4 mandatory assessment EAV triples.
    /// Ready to pass directly to `PersistedDb::commit()`.
    pub fn to_eav_triples(&self) -> Vec<EavTriple> {
        vec![
            EavTriple::new(ATTR_STATE, encode_state(self.state).to_vec()),
            EavTriple::new(ATTR_QUALITY, encode_f32(self.hps.value())),
            EavTriple::new(ATTR_COLOR_RGB, self.color.to_bytes().to_vec()),
            EavTriple::new(ATTR_FRESHNESS, encode_f32(self.freshness)),
        ]
    }
}

/// Map a fuzzy quality tier to the sovereign particle lifecycle state.
pub fn tier_to_state(tier: QualityTier) -> ParticleState {
    match tier {
        QualityTier::Gem | QualityTier::Tribe => ParticleState::Golden,
        QualityTier::Active => ParticleState::Fuzzy,
        QualityTier::NonActive => ParticleState::Dead,
    }
}

/// Score a particle from its fuzzy dimensions plus context.
///
/// Pipeline:
///   FuzzyDimensions (D1–D8) + D9 orbital_trust_penalty
///   → fuzzify (penalty degrades D6 effective score)
///   → evaluate_rules → centroid_defuzz
///   → × format_multiplier → B11
///   → tier_to_state → HpsScore → FreshnessDecay → ColorRgb
pub fn score(input: &ScoreInput) -> ScoreResult {
    // Apply the orbital trust penalty to D9 before scoring.
    let mut dims = input.dims.clone();
    dims.d9_orbital_trust_penalty = input.orbital_trust_penalty;
    let b11 = FuzzyEngine::score(&dims, input.format);
    let tier = FuzzyEngine::classify(b11);
    let state = tier_to_state(tier);
    let hps = HpsScore::new(b11 as f32 / 255.0);
    let freshness = input.decay.compute(input.elapsed_seconds);
    let color = compute_color(input.domain_byte, hps.value(), freshness, state);
    ScoreResult {
        b11,
        tier,
        state,
        hps,
        color,
        freshness,
    }
}

/// Encode a [0.0, 1.0] float as a 5-char ASCII decimal string (e.g. b"0.800").
/// Pure std — no format! macro allocation beyond Vec.
fn encode_f32(v: f32) -> Vec<u8> {
    let v = v.clamp(0.0, 1.0);
    // Integer part: always 0 or 1
    let int_part = v as u32;
    // Fractional part: 3 digits (multiply by 1000 then round)
    let frac = ((v - int_part as f32) * 1000.0).round() as u32;
    // Handle 1.000 edge case where frac could be 1000 due to rounding
    let (int_part, frac) = if frac >= 1000 {
        (int_part + 1, 0)
    } else {
        (int_part, frac)
    };
    let int_part = int_part.min(1); // clamp to max 1
    vec![
        b'0' + int_part as u8,
        b'.',
        b'0' + (frac / 100) as u8,
        b'0' + ((frac / 10) % 10) as u8,
        b'0' + (frac % 10) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuzzy_engine::{FuzzyDimensions, SourceTrust};

    fn perfect_input() -> ScoreInput {
        ScoreInput::civil(FuzzyDimensions::perfect(), 0xA0, 0.0)
    }

    #[test]
    fn perfect_particle_scores_gem_golden() {
        let result = score(&perfect_input());
        assert_eq!(result.tier, QualityTier::Gem);
        assert_eq!(result.state, ParticleState::Golden);
        assert!(
            result.b11 >= 200,
            "perfect must be Gem, got B11={}",
            result.b11
        );
    }

    #[test]
    fn perfect_particle_color_has_correct_domain_byte() {
        let result = score(&perfect_input());
        assert_eq!(result.color.r, 0xA0, "domain byte must be 0xA0");
        assert!(result.color.g > 200, "quality green byte should be high");
        assert!(
            result.color.b > 250,
            "freshness blue should be near max at t=0"
        );
    }

    #[test]
    fn dead_particle_renders_gray() {
        let mut dims = FuzzyDimensions::perfect();
        dims.d1_name_completeness = 0.0;
        dims.d8_arabic_density = 0.0;
        dims.d3_id_validity = 0.0;
        dims.d4_date_validity = 0.0;
        let input = ScoreInput::civil(dims, 0xFF, 0.0);
        let result = score(&input);
        if result.state == ParticleState::Dead {
            assert_eq!(result.color, ColorRgb::GRAY, "dead particle must be gray");
        }
    }

    #[test]
    fn active_tier_maps_to_fuzzy_state() {
        assert_eq!(tier_to_state(QualityTier::Active), ParticleState::Fuzzy);
        assert_eq!(tier_to_state(QualityTier::NonActive), ParticleState::Dead);
        assert_eq!(tier_to_state(QualityTier::Tribe), ParticleState::Golden);
        assert_eq!(tier_to_state(QualityTier::Gem), ParticleState::Golden);
    }

    #[test]
    fn eav_triples_contain_four_attrs() {
        let result = score(&perfect_input());
        let triples = result.to_eav_triples();
        assert_eq!(triples.len(), 4);
        let hashes: Vec<u32> = triples.iter().map(|t| t.attr_hash).collect();
        assert!(hashes.contains(&ATTR_STATE));
        assert!(hashes.contains(&ATTR_QUALITY));
        assert!(hashes.contains(&ATTR_COLOR_RGB));
        assert!(hashes.contains(&ATTR_FRESHNESS));
    }

    #[test]
    fn eav_state_is_golden_for_perfect() {
        let result = score(&perfect_input());
        let triples = result.to_eav_triples();
        let state_triple = triples.iter().find(|t| t.attr_hash == ATTR_STATE).unwrap();
        assert_eq!(state_triple.value, b"GOLDEN");
    }

    #[test]
    fn format_layer_multiplier_affects_b11() {
        let dims = FuzzyDimensions::perfect();
        let civil = score(&ScoreInput {
            dims: dims.clone(),
            format: FormatLayer::CivilRegistry,
            domain_byte: 0,
            elapsed_seconds: 0.0,
            decay: FreshnessDecay::civil_registry(),
            orbital_trust_penalty: 0.0,
        });
        let generic = score(&ScoreInput {
            dims,
            format: FormatLayer::GenericEav,
            domain_byte: 0,
            elapsed_seconds: 0.0,
            decay: FreshnessDecay::civil_registry(),
            orbital_trust_penalty: 0.0,
        });
        assert!(
            civil.b11 >= generic.b11,
            "CivilRegistry must score >= GenericEav"
        );
    }

    #[test]
    fn encode_f32_round_trips() {
        assert_eq!(encode_f32(0.0), b"0.000");
        assert_eq!(encode_f32(1.0), b"1.000");
        assert_eq!(encode_f32(0.8), b"0.800");
        assert_eq!(encode_f32(0.995), b"0.995");
    }

    #[test]
    fn freshness_decays_over_time() {
        let dims = FuzzyDimensions::perfect();
        let fresh = score(&ScoreInput::civil(dims.clone(), 0, 0.0));
        let stale = score(&ScoreInput::civil(dims, 0, 365.25 * 86400.0 * 2.0)); // 2 years
        assert!(
            stale.freshness < fresh.freshness,
            "freshness must decay over time"
        );
        assert!(
            stale.color.b < fresh.color.b,
            "blue byte must decay with freshness"
        );
    }

    #[test]
    fn tier_label_matches_b11() {
        let result = score(&perfect_input());
        assert_eq!(result.tier_label(), "Gem");
    }

    #[test]
    fn orbital_penalty_lowers_b11() {
        let dims = FuzzyDimensions::perfect();
        let clean = score(&ScoreInput::civil(dims.clone(), 0, 0.0));
        let penalised = score(&ScoreInput::civil_with_penalty(dims, 0, 0.0, 0.9));
        assert!(
            penalised.b11 <= clean.b11,
            "orbital penalty must not increase b11: clean={} penalised={}",
            clean.b11,
            penalised.b11
        );
    }

    #[test]
    fn zero_orbital_penalty_preserves_score() {
        let dims = FuzzyDimensions::perfect();
        let normal = score(&ScoreInput::civil(dims.clone(), 0, 0.0));
        let explicit = score(&ScoreInput::civil_with_penalty(dims, 0, 0.0, 0.0));
        assert_eq!(
            normal.b11, explicit.b11,
            "zero penalty must leave b11 identical"
        );
    }

    #[test]
    fn unknown_trust_lowers_score_vs_authoritative() {
        let mut dims_auth = FuzzyDimensions::perfect();
        dims_auth.d6_source_trust = fuzzy_engine::SourceTrust::Authoritative;
        let mut dims_unkn = FuzzyDimensions::perfect();
        dims_unkn.d6_source_trust = SourceTrust::Unknown;

        let auth = score(&ScoreInput::civil(dims_auth, 0, 0.0));
        let unkn = score(&ScoreInput::civil(dims_unkn, 0, 0.0));
        assert!(
            auth.b11 >= unkn.b11,
            "authoritative trust must score >= unknown"
        );
    }
}
