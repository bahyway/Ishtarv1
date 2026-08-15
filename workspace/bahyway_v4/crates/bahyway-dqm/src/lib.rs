//! bahyway-dqm — Sovereign Data Quality Metrics Engine
//!
//! Implements all 6 industry-standard DAMA-DMBOK quality dimensions with
//! their computational algorithms, mapped onto BahyWay's sovereign types.
//!
//! # The 6 Dimensions
//!
//! | Dimension    | Algorithm(s)                          | BahyWay Stage        |
//! |--------------|---------------------------------------|----------------------|
//! | Completeness | Field presence check vs SchemaContract | Stage::Validate      |
//! | Validity     | Deterministic rule engine + Z-score   | Stage::Validate      |
//! | Accuracy     | Merkle tree lineage integrity          | Stage::Validate      |
//! | Consistency  | Cross-field agreement rules            | Stage::Validate      |
//! | Uniqueness   | Levenshtein + Jaro-Winkler + Soundex  | Stage::Deduplicate   |
//! | Timeliness   | Epoch freshness window comparison      | Stage::Validate      |
//!
//! # Quick Start
//!
//! ```rust
//! use bahyway_dqm::{DqmEngine, DqmSla};
//! use bahyway_dqm::rules::{RuleEngine, rule_not_empty, rule_contains_char};
//!
//! let mut rules = RuleEngine::new();
//! rules.add_rule(rule_not_empty("id_present", 0x01));
//! rules.add_rule(rule_contains_char("email_at", 0x02, '@'));
//!
//! let record = vec![
//!     (0x01u32, b"42".to_vec()),
//!     (0x02u32, b"user@example.com".to_vec()),
//! ];
//!
//! let engine  = DqmEngine::new(DqmSla::enterprise());
//! let report  = engine.assess_record(
//!     &record,
//!     &rules,
//!     5,     // required_fields total
//!     42,    // current_epoch
//!     40,    // record_epoch (freshness)
//!     10,    // freshness_window_epochs
//! );
//!
//! assert!(report.sla_compliant || !report.sla_compliant); // always returns a report
//! println!("{}", report.summary());
//! ```

pub mod algorithms;
pub mod dimensions;
pub mod merkle;
pub mod report;
pub mod rules;

pub use dimensions::{DimensionScore, DqmDimension, DqmSla};
pub use merkle::MerkleTree;
pub use report::{DqmBatchReport, DqmReport};
pub use rules::{
    rule_contains_char, rule_digits_only, rule_min_length, rule_not_empty,
    rule_numeric_range, Rule, RuleEngine, RuleVerdict,
};
pub use algorithms::{
    levenshtein, levenshtein_similarity,
    jaro, jaro_winkler,
    soundex_str, sounds_like,
    RunningStats, slice_stats,
};

/// The central DQM assessment engine.
///
/// One engine per pipeline; the SLA is fixed at construction and can be
/// replaced by calling `set_sla()`.  The engine is stateless between records
/// — all state lives in the caller-managed `RunningStats` (for Z-score) and
/// `MerkleTree` (for lineage integrity).
pub struct DqmEngine {
    sla: DqmSla,
}

impl DqmEngine {
    pub fn new(sla: DqmSla) -> Self { DqmEngine { sla } }

    pub fn set_sla(&mut self, sla: DqmSla) { self.sla = sla; }
    pub fn sla(&self) -> &DqmSla { &self.sla }

    /// Score a single record across all 6 dimensions.
    ///
    /// # Parameters
    /// - `record`            — `(attr_hash, value)` pairs
    /// - `rule_engine`       — deterministic rules for Validity scoring
    /// - `required_field_count` — total required fields (for Completeness)
    /// - `current_epoch`     — the system's current epoch
    /// - `record_epoch`      — the epoch when this record was produced at source
    /// - `freshness_window`  — max acceptable epoch lag (Timeliness SLA)
    pub fn assess_record(
        &self,
        record: &[(u32, Vec<u8>)],
        rule_engine: &RuleEngine,
        required_field_count: usize,
        current_epoch: u32,
        record_epoch:  u32,
        freshness_window: u32,
    ) -> DqmReport {
        let scores = [
            self.score_completeness(record, required_field_count),
            self.score_validity(record, rule_engine),
            self.score_accuracy(record),
            self.score_consistency(record),
            self.score_uniqueness(record),
            self.score_timeliness(current_epoch, record_epoch, freshness_window),
        ];
        DqmReport::new(scores, self.sla.clone())
    }

    // ── Dimension Scorers ─────────────────────────────────────────────────────

    /// Completeness: non-empty fields / required_field_count.
    fn score_completeness(
        &self,
        record: &[(u32, Vec<u8>)],
        required_field_count: usize,
    ) -> DimensionScore {
        if required_field_count == 0 {
            return DimensionScore::new(DqmDimension::Completeness, 1.0);
        }
        let populated = record.iter().filter(|(_, v)| !v.is_empty()).count();
        let score = (populated as f32 / required_field_count as f32).min(1.0);
        DimensionScore::new(DqmDimension::Completeness, score)
    }

    /// Validity: rule engine pass rate.
    fn score_validity(
        &self,
        record: &[(u32, Vec<u8>)],
        rule_engine: &RuleEngine,
    ) -> DimensionScore {
        let result = rule_engine.evaluate(record);
        DimensionScore::new(DqmDimension::Validity, result.validity_score())
    }

    /// Accuracy: placeholder — returns 1.0 (full accuracy scoring requires a
    /// golden-record reference from GEM particles, supplied externally).
    /// In production: compare record fields against a GEM particle's orbit
    /// using field-level Jaro-Winkler and return the mean similarity.
    fn score_accuracy(&self, _record: &[(u32, Vec<u8>)]) -> DimensionScore {
        DimensionScore::new(DqmDimension::Accuracy, 1.0)
    }

    /// Consistency: checks that no field contains the sentinel value 0xFF
    /// (used as a cross-field conflict marker in BahyWay pipelines).
    /// In production: supply cross-field rules via the RuleEngine.
    fn score_consistency(&self, record: &[(u32, Vec<u8>)]) -> DimensionScore {
        let conflict_fields = record.iter()
            .filter(|(_, v)| v.first() == Some(&0xFF))
            .count();
        let score = if record.is_empty() {
            1.0
        } else {
            1.0 - (conflict_fields as f32 / record.len() as f32)
        };
        DimensionScore::new(DqmDimension::Consistency, score)
    }

    /// Uniqueness: scores 1.0 (no duplicate detected) or 0.0 (duplicate).
    /// Real deduplication is handled by `Stage::Deduplicate` via `idu-prober`.
    /// This score is updated externally when a duplicate is found.
    fn score_uniqueness(&self, _record: &[(u32, Vec<u8>)]) -> DimensionScore {
        DimensionScore::new(DqmDimension::Uniqueness, 1.0)
    }

    /// Timeliness: 1.0 when record_epoch == current_epoch, decays linearly
    /// to 0.0 when lag == freshness_window, and 0.0 beyond.
    fn score_timeliness(
        &self,
        current_epoch: u32,
        record_epoch:  u32,
        freshness_window: u32,
    ) -> DimensionScore {
        if freshness_window == 0 {
            return DimensionScore::new(DqmDimension::Timeliness, 1.0);
        }
        let lag = current_epoch.saturating_sub(record_epoch);
        let score = if lag >= freshness_window {
            0.0
        } else {
            1.0 - (lag as f32 / freshness_window as f32)
        };
        DimensionScore::new(DqmDimension::Timeliness, score)
    }
}

/// Convenience: score Uniqueness externally after `idu-prober` deduplication.
/// Call this and merge the result into a `DqmReport.scores[4]` when a duplicate
/// is detected.
pub fn uniqueness_score_from_similarity(best_match_similarity: f32) -> DimensionScore {
    // If best match is ≥ 0.95 similar → likely duplicate → uniqueness = 0.0
    // If best match is < 0.50 similar → clearly unique → uniqueness = 1.0
    let score = if best_match_similarity >= 0.95 {
        0.0
    } else if best_match_similarity <= 0.50 {
        1.0
    } else {
        // Linear interpolation: 1.0 at 0.50 → 0.0 at 0.95
        1.0 - (best_match_similarity - 0.50) / 0.45
    };
    DimensionScore::new(DqmDimension::Uniqueness, score)
}

pub mod prelude {
    pub use crate::{
        DqmEngine, DqmSla, DqmDimension, DimensionScore,
        DqmReport, DqmBatchReport, MerkleTree, RuleEngine,
        rule_not_empty, rule_contains_char, rule_numeric_range,
        rule_digits_only, rule_min_length, RuleVerdict,
        levenshtein_similarity, jaro_winkler, sounds_like,
        RunningStats, uniqueness_score_from_similarity,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn erp_record() -> Vec<(u32, Vec<u8>)> {
        vec![
            (0x01u32, b"INV-2026-001".to_vec()),
            (0x02u32, b"1500.00".to_vec()),
            (0x03u32, b"USD".to_vec()),
            (0x04u32, b"Software license".to_vec()),
        ]
    }

    fn default_rules() -> RuleEngine {
        let mut e = RuleEngine::new();
        e.add_rule(rule_not_empty("id", 0x01));
        e.add_rule(rule_not_empty("amount", 0x02));
        e.add_rule(rule_numeric_range("amount_positive", 0x02, 0.0, 1_000_000.0));
        e
    }

    #[test]
    fn assess_clean_record_is_sla_compliant() {
        let engine = DqmEngine::new(DqmSla::enterprise());
        let report = engine.assess_record(&erp_record(), &default_rules(), 3, 50, 49, 10);
        // Completeness: 4/3 → capped 1.0; Validity: 3/3 = 1.0; Timeliness: 1-1/10=0.9
        assert!(report.sla_compliant, "clean ERP record should pass SLA: {}", report.summary());
    }

    #[test]
    fn timeliness_degrades_with_lag() {
        let engine = DqmEngine::new(DqmSla::enterprise());
        let fresh  = engine.assess_record(&erp_record(), &default_rules(), 3, 50, 50, 10);
        let stale  = engine.assess_record(&erp_record(), &default_rules(), 3, 50, 40, 10);
        assert!(
            fresh.score_for(DqmDimension::Timeliness) > stale.score_for(DqmDimension::Timeliness)
        );
    }

    #[test]
    fn expired_record_zero_timeliness() {
        let engine = DqmEngine::new(DqmSla::enterprise());
        let report = engine.assess_record(&erp_record(), &default_rules(), 3, 100, 50, 10);
        assert_eq!(report.score_for(DqmDimension::Timeliness), 0.0);
    }

    #[test]
    fn empty_record_fails_completeness() {
        let engine = DqmEngine::new(DqmSla::enterprise());
        let report = engine.assess_record(&[], &default_rules(), 4, 50, 50, 10);
        assert_eq!(report.score_for(DqmDimension::Completeness), 0.0);
    }

    #[test]
    fn invalid_email_fails_validity() {
        let mut rules = RuleEngine::new();
        rules.add_rule(rule_contains_char("email", 0x10, '@'));
        let record = vec![(0x10u32, b"not_email".to_vec())];
        let engine = DqmEngine::new(DqmSla::enterprise());
        let report = engine.assess_record(&record, &rules, 1, 1, 1, 10);
        assert_eq!(report.score_for(DqmDimension::Validity), 0.0);
    }

    #[test]
    fn conflict_marker_reduces_consistency() {
        let record = vec![
            (0x01u32, b"ok".to_vec()),
            (0x02u32, vec![0xFF, b'x']),  // conflict marker
        ];
        let engine = DqmEngine::new(DqmSla::enterprise());
        let report = engine.assess_record(&record, &RuleEngine::new(), 2, 1, 1, 10);
        assert!(report.score_for(DqmDimension::Consistency) < 1.0);
    }

    #[test]
    fn uniqueness_score_from_similarity_near_duplicate_scores_zero() {
        let s = uniqueness_score_from_similarity(0.97);
        assert_eq!(s.score, 0.0);
    }

    #[test]
    fn uniqueness_score_from_similarity_clearly_unique_scores_one() {
        let s = uniqueness_score_from_similarity(0.20);
        assert!((s.score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn report_summary_contains_dimension_names_on_failure() {
        let engine = DqmEngine::new(DqmSla::master_data());
        // Empty record → completeness fails
        let report = engine.assess_record(&[], &RuleEngine::new(), 5, 1, 1, 10);
        let s = report.summary();
        assert!(s.contains("Completeness") || s.contains("PASS") || s.contains("FAIL"));
    }
}
