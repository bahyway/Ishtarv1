#![forbid(unsafe_code)]
//! Provenance EAV — mandatory block for all ASHNAN external ingestion particles.
//!
//! Every CrossTribe particle (kaki_type=0x03) must carry this block.
//! No external bulletin is ever silently treated as sovereign ground truth
//! (BC-AGRI-001 §4.2, §9).

use crate::error::AshnanError;

/// Confidence tier for an external data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfidenceTier {
    /// TIER 0 — Sovereign sensor: Diyala ASHNAN pilot network.
    Tier0Sovereign = 0,
    /// TIER 1 — Official direct feed (national met API, satellite NDVI).
    Tier1Official = 1,
    /// TIER 2 — Official bulletin (FAO, WOAH, ministry press releases).
    Tier2Bulletin = 2,
    /// TIER 3 — Modelled / third-hand (regional indices, NGO field summaries).
    Tier3Modelled = 3,
}

/// Mandatory provenance block attached to every external ingestion particle.
#[derive(Debug, Clone)]
pub struct ProvenanceBlock {
    /// Originating organisation — never anonymous.
    pub source_org: &'static str,
    /// Confidence tier classification.
    pub tier: ConfidenceTier,
    /// Days between observed event and data availability (drives KIBRATU STALE detection).
    pub publication_lag_days: u16,
    /// Source schema version — enables KIBRATU MISCLASSIFIED detection on format drift.
    pub schema_version: u32,
    /// Agreement score against overlapping sovereign sensor reading, if available (0.0–1.0).
    pub cross_validation_score: Option<f32>,
}

impl ProvenanceBlock {
    /// Validate that required fields are present and coherent.
    pub fn validate(&self) -> Result<(), AshnanError> {
        if self.source_org.is_empty() {
            return Err(AshnanError::MissingProvenance("source_org"));
        }
        if let Some(cv) = self.cross_validation_score {
            if !(0.0..=1.0).contains(&cv) {
                return Err(AshnanError::MissingProvenance(
                    "cross_validation_score out of range",
                ));
            }
        }
        Ok(())
    }

    /// Returns true if the publication lag exceeds the freshness threshold for the domain.
    /// Threshold: 14 days for active growing season; 42 days otherwise.
    pub fn is_stale(&self, growing_season_active: bool) -> bool {
        let threshold = if growing_season_active { 14 } else { 42 };
        self.publication_lag_days > threshold
    }
}
