// ============================================================================
// fuzzy-engine/src/dimensions.rs
// 8 sovereign fuzzy dimensions (ADR-005)
// serde removed — pure std (BahyWay v4 zero-dependency constraint)
// ============================================================================
#![allow(missing_docs)]

use crate::membership::*;

/// Source trust level for D6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTrust {
    Authoritative, // 1.0
    Official,      // 0.75
    Community,     // 0.40
    Unknown,       // 0.10
}

impl SourceTrust {
    pub fn score(self) -> f32 {
        match self {
            SourceTrust::Authoritative => 1.00,
            SourceTrust::Official => 0.75,
            SourceTrust::Community => 0.40,
            SourceTrust::Unknown => 0.10,
        }
    }

    /// Apply an orbital trust penalty (from orbital-trust-probe) to derive a
    /// degraded effective score. The penalty [0.0, 1.0] linearly reduces the
    /// current score toward the Unknown floor (0.10), ensuring a single
    /// unexplained orbital deviation never silences a particle entirely.
    ///
    /// degraded = score - penalty × (score - UNKNOWN_FLOOR)
    pub fn penalised_score(self, penalty: f32) -> f32 {
        const UNKNOWN_FLOOR: f32 = 0.10;
        let base = self.score();
        let degraded = base - penalty.clamp(0.0, 1.0) * (base - UNKNOWN_FLOOR);
        degraded.clamp(UNKNOWN_FLOOR, 1.0)
    }

    /// Construct a FuzzyDimensions override that applies the orbital penalty
    /// to D6 before scoring. Callers pass the accumulated penalty from
    /// `OrbitalDeviationJournal::accumulated_penalty()`.
    pub fn with_orbital_penalty(self, penalty: f32) -> f32 {
        self.penalised_score(penalty)
    }
}

/// The 8 fuzzy input dimensions (ADR-005).
/// All values normalised to [0.0, 1.0] before membership evaluation.
#[derive(Debug, Clone)]
pub struct FuzzyDimensions {
    /// D1 — name completeness: fraction of 5 name parts present (0.0–1.0).
    pub d1_name_completeness: f32,
    /// D2 — name consistency: Arabic BERT cosine similarity (0.0–1.0).
    pub d2_name_consistency: f32,
    /// D3 — ID validity: 1.0 if valid 12-digit Iraqi national ID, else 0.0.
    pub d3_id_validity: f32,
    /// D4 — date validity: Hijri + Gregorian cross-check score (0.0–1.0).
    pub d4_date_validity: f32,
    /// D5 — geographic precision: H3 resolution / 15.0 (0.0–1.0).
    pub d5_geo_precision: f32,
    /// D6 — source trust level.
    pub d6_source_trust: SourceTrust,
    /// D7 — duplicate proximity: 1.0 - (cosine dist to nearest particle).
    /// High value = close to duplicate = bad.
    pub d7_duplicate_proximity: f32,
    /// D8 — Arabic density: VGCA-Λ arabic_density from FSV d3 (0.0–1.0).
    pub d8_arabic_density: f32,
    /// D9 — Orbital trust penalty: accumulated penalty from orbital-trust-probe
    /// (0.0 = no violations observed, 1.0 = maximum sustained deviation).
    /// Feeds back into D6 effective score before fuzzification.
    /// Default 0.0 — no penalty.
    pub d9_orbital_trust_penalty: f32,
}

impl FuzzyDimensions {
    /// Construct a perfect-score dimensions record (all 1.0, Authoritative, no duplicates).
    pub fn perfect() -> Self {
        Self {
            d1_name_completeness: 1.0,
            d2_name_consistency: 1.0,
            d3_id_validity: 1.0,
            d4_date_validity: 1.0,
            d5_geo_precision: 1.0,
            d6_source_trust: SourceTrust::Authoritative,
            d7_duplicate_proximity: 0.0,
            d8_arabic_density: 1.0,
            d9_orbital_trust_penalty: 0.0,
        }
    }

    /// Fuzzify all 8 dimensions. Returns (low, medium, high) membership for each.
    pub fn fuzzify(&self) -> FuzzifiedDimensions {
        // D9 orbital penalty degrades the effective D6 trust score before fuzzification.
        // Zero penalty → no effect; 1.0 penalty → D6 collapses to the Unknown floor (0.10).
        let trust = self
            .d6_source_trust
            .with_orbital_penalty(self.d9_orbital_trust_penalty);
        FuzzifiedDimensions {
            d1: fuzzify_completeness(self.d1_name_completeness),
            d2: fuzzify_similarity(self.d2_name_consistency),
            d3: fuzzify_validity(self.d3_id_validity),
            d4: fuzzify_validity(self.d4_date_validity),
            d5: fuzzify_precision(self.d5_geo_precision),
            d6: fuzzify_trust(trust),
            d7: fuzzify_proximity(self.d7_duplicate_proximity),
            d8: fuzzify_density(self.d8_arabic_density),
        }
    }
}

/// Fuzzified membership values for all 8 dimensions.
/// Each field is (low, medium, high) membership tuple.
#[derive(Debug, Clone)]
pub struct FuzzifiedDimensions {
    pub d1: (f32, f32, f32),
    pub d2: (f32, f32, f32),
    pub d3: (f32, f32, f32),
    pub d4: (f32, f32, f32),
    pub d5: (f32, f32, f32),
    pub d6: (f32, f32, f32),
    pub d7: (f32, f32, f32), // proximity: (low=good, medium, high=bad)
    pub d8: (f32, f32, f32),
}

fn fuzzify_completeness(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.3, 0.6),
        f_tri(x, 0.3, 0.65, 0.9),
        f_right(x, 0.8, 1.0),
    )
}

fn fuzzify_similarity(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.4, 0.7),
        f_tri(x, 0.4, 0.7, 0.9),
        f_right(x, 0.85, 1.0),
    )
}

fn fuzzify_validity(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.3, 0.6),
        f_tri(x, 0.4, 0.6, 0.85),
        f_right(x, 0.75, 1.0),
    )
}

fn fuzzify_precision(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.2, 0.5),
        f_tri(x, 0.3, 0.6, 0.85),
        f_right(x, 0.7, 1.0),
    )
}

fn fuzzify_trust(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.2, 0.5),
        f_tri(x, 0.3, 0.55, 0.8),
        f_right(x, 0.7, 1.0),
    )
}

fn fuzzify_proximity(x: f32) -> (f32, f32, f32) {
    // Low proximity = good (no duplicates); high = bad
    (
        f_left(x, 0.1, 0.3),
        f_tri(x, 0.1, 0.35, 0.6),
        f_right(x, 0.5, 0.9),
    )
}

fn fuzzify_density(x: f32) -> (f32, f32, f32) {
    (
        f_left(x, 0.2, 0.5),
        f_tri(x, 0.3, 0.6, 0.85),
        f_right(x, 0.7, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_dimensions_all_fuzzify_to_high() {
        let d = FuzzyDimensions::perfect();
        let f = d.fuzzify();
        assert!(f.d1.2 > 0.9, "d1 high should be ~1.0");
        assert!(f.d3.2 > 0.9, "d3 high should be ~1.0");
        assert!(f.d6.2 > 0.9, "d6 high should be ~1.0");
    }

    #[test]
    fn zero_completeness_fuzzifies_to_low() {
        let mut d = FuzzyDimensions::perfect();
        d.d1_name_completeness = 0.0;
        let f = d.fuzzify();
        assert_eq!(f.d1.0, 1.0, "zero completeness must be fully low");
    }

    #[test]
    fn unknown_trust_fuzzifies_to_low() {
        let mut d = FuzzyDimensions::perfect();
        d.d6_source_trust = SourceTrust::Unknown;
        let f = d.fuzzify();
        assert!(f.d6.0 > 0.5, "unknown trust should be mostly low");
    }

    #[test]
    fn high_proximity_fuzzifies_to_high() {
        let mut d = FuzzyDimensions::perfect();
        d.d7_duplicate_proximity = 0.95;
        let f = d.fuzzify();
        assert!(f.d7.2 > 0.9, "near-duplicate proximity should be high");
    }

    #[test]
    fn source_trust_scores_are_ordered() {
        assert!(SourceTrust::Authoritative.score() > SourceTrust::Official.score());
        assert!(SourceTrust::Official.score() > SourceTrust::Community.score());
        assert!(SourceTrust::Community.score() > SourceTrust::Unknown.score());
    }

    #[test]
    fn zero_penalty_leaves_score_unchanged() {
        let trust = SourceTrust::Authoritative;
        assert!((trust.penalised_score(0.0) - trust.score()).abs() < 1e-6);
    }

    #[test]
    fn full_penalty_collapses_to_unknown_floor() {
        let trust = SourceTrust::Authoritative;
        // Full penalty drives score to UNKNOWN_FLOOR = 0.10
        assert!((trust.penalised_score(1.0) - 0.10).abs() < 1e-6);
    }

    #[test]
    fn partial_penalty_is_between_floor_and_base() {
        let trust = SourceTrust::Official; // base = 0.75
        let degraded = trust.penalised_score(0.5);
        assert!(degraded > 0.10, "must stay above floor");
        assert!(degraded < 0.75, "must be below base score");
    }

    #[test]
    fn orbital_penalty_lowers_b11_via_fuzzify() {
        let mut d_clean = FuzzyDimensions::perfect();
        d_clean.d9_orbital_trust_penalty = 0.0;
        let mut d_penalised = FuzzyDimensions::perfect();
        d_penalised.d9_orbital_trust_penalty = 0.8;

        let f_clean = d_clean.fuzzify();
        let f_penalised = d_penalised.fuzzify();
        // D6 high membership should be lower when penalised
        assert!(
            f_penalised.d6.2 < f_clean.d6.2,
            "penalty must reduce D6 high membership: clean={} penalised={}",
            f_clean.d6.2,
            f_penalised.d6.2
        );
    }

    #[test]
    fn perfect_with_zero_penalty_unchanged() {
        let d = FuzzyDimensions::perfect(); // d9 defaults to 0.0
        let f = d.fuzzify();
        assert!(f.d6.2 > 0.9, "zero penalty must leave d6 high at maximum");
    }
}
