//! 𒁾 Pipeline Bridge — wires HeptaScore into BeeMDM stations S0–S7.
//!
//! Each BeeMDM pipeline station writes its quality assessment into one or
//! more `StationScores` fields. `score_from_station_outputs()` then computes
//! the final H(P) → B11 → QualityLane.
//!
//! Station → Dimension mapping:
//! ```text
//! S0 VaultGate    → timeliness    (file freshness)
//! S1 KAKI         → completeness + validity
//! S2 CompareWay   → consistency + uniqueness
//! S3 CleansingWay → accuracy      (Arabic normalisation)
//! S7 EnkiWay      → integrity     (EAV referential completeness)
//! ```
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::domain::{HeptaVector, TribeIdealPoint, HeptaScore};

/// Quality outputs from all BeeMDM pipeline stations.
///
/// Each station writes its score (0.0–1.0) into the relevant field.
/// `None` means the station did not run or was not applicable.
#[derive(Debug, Clone, Default)]
pub struct StationScores {
    // S0 VaultGate
    pub timeliness_score:   Option<f32>,
    // S1 KAKI Issuance
    pub completeness_score: Option<f32>,
    pub validity_score:     Option<f32>,
    // S2 CompareWay
    pub consistency_score:  Option<f32>,
    pub uniqueness_score:   Option<f32>,
    // S3 CleansingWay
    pub accuracy_score:     Option<f32>,
    // S7 EnkiWay
    pub integrity_score:    Option<f32>,
}

impl StationScores {
    /// Returns true when all 7 station scores are populated.
    pub fn is_complete(&self) -> bool {
        self.accuracy_score.is_some()
            && self.completeness_score.is_some()
            && self.consistency_score.is_some()
            && self.validity_score.is_some()
            && self.uniqueness_score.is_some()
            && self.timeliness_score.is_some()
            && self.integrity_score.is_some()
    }
}

/// Convert `StationScores` into a `HeptaScore`.
///
/// Missing station scores use a conservative default of 0.5 — not assuming
/// quality when unverified.
pub fn score_from_station_outputs(
    stations: &StationScores,
    tribe:    &TribeIdealPoint,
) -> HeptaScore {
    const DEFAULT_UNSCORED: f32 = 0.5;

    let hepta = HeptaVector::new(
        stations.accuracy_score    .unwrap_or(DEFAULT_UNSCORED),
        stations.completeness_score.unwrap_or(DEFAULT_UNSCORED),
        stations.consistency_score .unwrap_or(DEFAULT_UNSCORED),
        stations.validity_score    .unwrap_or(DEFAULT_UNSCORED),
        stations.uniqueness_score  .unwrap_or(DEFAULT_UNSCORED),
        stations.timeliness_score  .unwrap_or(DEFAULT_UNSCORED),
        stations.integrity_score   .unwrap_or(DEFAULT_UNSCORED),
    );

    hepta.score(tribe)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_stations(q: f32) -> StationScores {
        StationScores {
            accuracy_score:     Some(q),
            completeness_score: Some(q),
            consistency_score:  Some(q),
            validity_score:     Some(q),
            uniqueness_score:   Some(q),
            timeliness_score:   Some(q),
            integrity_score:    Some(q),
        }
    }

    #[test]
    fn full_perfect_stations_gives_gem() {
        let tribe = TribeIdealPoint::sovereign_arabic_mdm();
        let score = score_from_station_outputs(&all_stations(1.0), &tribe);
        assert!(score.is_gem(), "perfect → GEM, got B11={}", score.b11);
    }

    #[test]
    fn full_poor_stations_gives_active() {
        // q=0.1: d=√(0.81*1)=0.9, H=0.526, B11=126 → Active
        // Dead is unreachable: T=[1;7], Σwᵢ=1 → H_min=0.5 → B11_min=120
        let tribe = TribeIdealPoint::sovereign_arabic_mdm();
        let score = score_from_station_outputs(&all_stations(0.1), &tribe);
        assert_eq!(score.lane, crate::domain::QualityLane::Active,
            "poor → Active, got B11={}", score.b11);
    }

    #[test]
    fn missing_stations_give_mid_range_b11() {
        let tribe = TribeIdealPoint::sovereign_arabic_mdm();
        let score = score_from_station_outputs(&StationScores::default(), &tribe);
        assert!(score.b11 > 0 && score.b11 < 200,
            "all-default B11={} should be mid-range", score.b11);
    }

    #[test]
    fn is_complete_false_when_any_missing() {
        let mut s = StationScores::default();
        s.accuracy_score = Some(0.9);
        assert!(!s.is_complete());
    }

    #[test]
    fn is_complete_true_when_all_set() {
        assert!(all_stations(0.9).is_complete());
    }
}
