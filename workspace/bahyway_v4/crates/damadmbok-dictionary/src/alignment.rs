//! BahyWay component ↔ DAMA-DMBOK term alignment table.
//!
//! Maps each DAMA code to the specific BahyWay crate and concept that
//! implements or fulfils that governance requirement.

/// A mapping from a DAMA-DMBOK term to a BahyWay component.
#[derive(Debug, Clone, Copy)]
pub struct BahywayAlignment {
    /// DAMA-DMBOK term code (e.g. "DQ.COMPLETENESS").
    pub dama_code:       &'static str,
    /// BahyWay crate name (e.g. "fuzzy-engine").
    pub bahyway_crate:   &'static str,
    /// Specific concept / type / function within the crate.
    pub bahyway_concept: &'static str,
    /// Brief note explaining the mapping.
    pub mapping_note:    &'static str,
}

/// Complete alignment table — DAMA code → BahyWay component.
pub static ALIGNMENTS: &[BahywayAlignment] = &[
    // ── Data Quality ─────────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DQ.COMPLETENESS",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyDimension D1",
        mapping_note:    "Name completeness scored 0.0–1.0 via fuzzy membership function.",
    },
    BahywayAlignment {
        dama_code:       "DQ.CONSISTENCY",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyDimension D2",
        mapping_note:    "Arabic BERT cosine-similarity score measuring cross-field coherence.",
    },
    BahywayAlignment {
        dama_code:       "DQ.VALIDITY",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyDimension D3+D4",
        mapping_note:    "D3 validates ID format; D4 validates date format and logical ranges.",
    },
    BahywayAlignment {
        dama_code:       "DQ.FRESHNESS",
        bahyway_crate:   "score-engine",
        bahyway_concept: "FreshnessDecay",
        mapping_note:    "Exponential decay function δₜ = e^−λt applied per-particle.",
    },
    BahywayAlignment {
        dama_code:       "DQ.UNIQUENESS",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyDimension D7",
        mapping_note:    "Duplicate proximity score inverted: high proximity → low uniqueness.",
    },
    BahywayAlignment {
        dama_code:       "DQ.FUZZY_SCORE",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyEngine::score()",
        mapping_note:    "Aggregates D1–D8 into the single B11 quality byte (0–255).",
    },
    BahywayAlignment {
        dama_code:       "DQ.B11_BYTE",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "QualityByte (B11)",
        mapping_note:    "The 11th byte of the IdentityKaki encoding the aggregate quality score.",
    },
    BahywayAlignment {
        dama_code:       "DQ.GEM_TIER",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "QualityTier::Gem",
        mapping_note:    "B11 >= 200 → Gem tier; triggers Golden particle promotion.",
    },
    BahywayAlignment {
        dama_code:       "DQ.ENTROPY",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "NabaMetrics::entropy_pct()",
        mapping_note:    "Computed as (non-Golden particles / total) * 100.",
    },
    BahywayAlignment {
        dama_code:       "DQ.COHERENCE",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "NabaMetrics::coherence_pct()",
        mapping_note:    "100 - entropy_pct(); high coherence means most particles are Golden.",
    },
    // ── Reference & Master Data ───────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "RMD.GOLDRECORD",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "QualityTier::Gem (B11 >= 200)",
        mapping_note:    "Gem-tier particles are promoted to ParticleState::Golden as golden records.",
    },
    BahywayAlignment {
        dama_code:       "RMD.PARTICLE_STATE",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "ParticleState (Golden/Fuzzy/Dead)",
        mapping_note:    "Tri-state enum governing particle lifecycle in the sovereign journal.",
    },
    BahywayAlignment {
        dama_code:       "RMD.MASTERDATA",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "PersistedDb particle registry",
        mapping_note:    "All registered particles constitute the sovereign master data population.",
    },
    BahywayAlignment {
        dama_code:       "RMD.DEDUPLICATION",
        bahyway_crate:   "fuzzy-engine",
        bahyway_concept: "FuzzyDimension D7 (duplicate proximity)",
        mapping_note:    "D7 scores similarity to existing particles to surface candidate duplicates.",
    },
    // ── Data Governance ───────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DG.STEWARDSHIP",
        bahyway_crate:   "data-steward-station",
        bahyway_concept: "DataStewardEngine",
        mapping_note:    "Orchestrates quality triage, promotion/demotion decisions, and remediation workflows.",
    },
    BahywayAlignment {
        dama_code:       "DG.LINEAGE",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "JournalEntry (append-only)",
        mapping_note:    "Every write appends an immutable EAV entry providing full event lineage.",
    },
    BahywayAlignment {
        dama_code:       "DG.AUDIT_TRAIL",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "append_writer / COMMIT_MARKER",
        mapping_note:    "COMMIT_MARKER=0xCA seals each write batch; replay reconstructs the audit trail.",
    },
    BahywayAlignment {
        dama_code:       "DG.COMPLIANCE",
        bahyway_crate:   "musaru-security",
        bahyway_concept: "check_sovereignty()",
        mapping_note:    "Validates every particle against tribal sovereignty rules before acceptance.",
    },
    // ── Metadata ──────────────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "MD.PROVENANCE",
        bahyway_crate:   "enkidb-kaki",
        bahyway_concept: "IdentityKaki (16-byte sovereign identity)",
        mapping_note:    "Embeds tribe, type, UUID hash, and quality in every particle identity.",
    },
    BahywayAlignment {
        dama_code:       "MD.ATTR_HASH",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "EavTriple.attr_hash (FNV-1a 32-bit)",
        mapping_note:    "Attribute names are hashed to u32 keys for compact EAV storage.",
    },
    BahywayAlignment {
        dama_code:       "MD.KAKI_METADATA",
        bahyway_crate:   "enkidb-kaki",
        bahyway_concept: "IdentityKaki + EventKaki",
        mapping_note:    "IdentityKaki=16 bytes particle metadata; EventKaki=16 bytes event metadata.",
    },
    BahywayAlignment {
        dama_code:       "MD.DATA_DICTIONARY",
        bahyway_crate:   "damadmbok-dictionary",
        bahyway_concept: "DICTIONARY static table",
        mapping_note:    "This crate IS the sovereign data dictionary — 100+ DAMA terms with definitions.",
    },
    // ── Data Storage & Operations ─────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DSO.APPEND_ONLY",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "Journal (no UPDATE/DELETE)",
        mapping_note:    "Journal is strictly append-only; mutations are new events, not overwrites.",
    },
    BahywayAlignment {
        dama_code:       "DSO.CRASH_RECOVERY",
        bahyway_crate:   "enkidb-storage",
        bahyway_concept: "AppendWriter / COMMIT_MARKER=0xCA",
        mapping_note:    "Incomplete writes without 0xCA marker are truncated at next startup.",
    },
    BahywayAlignment {
        dama_code:       "DSO.SNAPSHOT",
        bahyway_crate:   "enkidb-snapshot",
        bahyway_concept: "SnapshotRecord",
        mapping_note:    "Periodic snapshots compress journal state for faster replay and backup.",
    },
    BahywayAlignment {
        dama_code:       "DSO.JOURNAL",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "Journal",
        mapping_note:    "Core append-only event log; single source of truth for all particle history.",
    },
    BahywayAlignment {
        dama_code:       "DSO.FSYNC",
        bahyway_crate:   "enkidb-storage",
        bahyway_concept: "FsyncPolicy::PerCommit",
        mapping_note:    "PerCommit policy calls fsync after each batch ensuring physical durability.",
    },
    BahywayAlignment {
        dama_code:       "DSO.REPLAY",
        bahyway_crate:   "enkidb-persist",
        bahyway_concept: "PersistedDb::open() journal replay",
        mapping_note:    "On startup, journal entries are replayed to reconstruct in-memory particle state.",
    },
    // ── Data Integration ──────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DI.ETL",
        bahyway_crate:   "enkidw",
        bahyway_concept: "EtlPipeline",
        mapping_note:    "Orchestrates extract-transform-load into the sovereign data warehouse.",
    },
    BahywayAlignment {
        dama_code:       "DI.LANDING_ZONE",
        bahyway_crate:   "enkidw",
        bahyway_concept: "LandingZone",
        mapping_note:    "Raw arrivals land here before quality scoring and particle promotion.",
    },
    BahywayAlignment {
        dama_code:       "DI.SCHEMA_COMPARISON",
        bahyway_crate:   "compare-tribe-schema",
        bahyway_concept: "compare_versions()",
        mapping_note:    "Compares V1 and V2 schema snapshots to surface breaking changes.",
    },
    BahywayAlignment {
        dama_code:       "DI.DEAD_LETTER",
        bahyway_crate:   "compare-tribe-schema",
        bahyway_concept: "DeadVerdict",
        mapping_note:    "Records that fail schema comparison are emitted as DeadVerdict entries.",
    },
    BahywayAlignment {
        dama_code:       "DI.VALIDATION",
        bahyway_crate:   "vgca-validation",
        bahyway_concept: "validate()",
        mapping_note:    "Validates EAV triples against VGCA template rules before journal commit.",
    },
    // ── Data Architecture ─────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DA.EAV",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "EavTriple",
        mapping_note:    "All particle attributes stored as (attr_hash, value_bytes) EAV triples.",
    },
    BahywayAlignment {
        dama_code:       "DA.KAKI",
        bahyway_crate:   "enkidb-kaki",
        bahyway_concept: "IdentityKaki + EventKaki",
        mapping_note:    "16-byte identity structures encoding all sovereign particle metadata.",
    },
    BahywayAlignment {
        dama_code:       "DA.TRIBE",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "TribeId",
        mapping_note:    "16-bit namespace value partitioning all particles into sovereign domains.",
    },
    BahywayAlignment {
        dama_code:       "DA.SOVEREIGN_PARTICLE",
        bahyway_crate:   "bahyway-core",
        bahyway_concept: "ParticleState + IdentityKaki",
        mapping_note:    "A particle is the atomic data unit: identity + state + EAV attribute bag.",
    },
    // ── Data Warehousing ──────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DW.GOLDEN_RATIO",
        bahyway_crate:   "enkidw",
        bahyway_concept: "DwReport.golden_ratio()",
        mapping_note:    "Computes golden / total as a 0.0–1.0 sovereignty health metric.",
    },
    BahywayAlignment {
        dama_code:       "DW.TIME_TRAVEL",
        bahyway_crate:   "enkidb-engine",
        bahyway_concept: "EnkiDb.project_at_epoch()",
        mapping_note:    "Reconstructs particle state at any past epoch by replaying journal entries.",
    },
    BahywayAlignment {
        dama_code:       "DW.EPOCH",
        bahyway_crate:   "enkidb-journal",
        bahyway_concept: "JournalEntry.epoch (u32)",
        mapping_note:    "Monotonic u32 counter stamped on every journal entry.",
    },
    // ── Data Security ─────────────────────────────────────────────────────────
    BahywayAlignment {
        dama_code:       "DS.SOVEREIGNTY",
        bahyway_crate:   "musaru-security",
        bahyway_concept: "check_sovereignty()",
        mapping_note:    "Enforces that each particle belongs to its declared tribe namespace.",
    },
    BahywayAlignment {
        dama_code:       "DS.WAY_FILE",
        bahyway_crate:   "enkidw",
        bahyway_concept: "WayFile (.way WAYv2.0 security language)",
        mapping_note:    "WAY files declare sovereign access rules for data warehouse exports.",
    },
];

/// Look up alignments for a specific DAMA code.
pub fn alignments_for(dama_code: &str) -> Vec<&'static BahywayAlignment> {
    ALIGNMENTS.iter().filter(|a| a.dama_code == dama_code).collect()
}

/// Return all alignments for a given BahyWay crate.
pub fn alignments_by_crate(crate_name: &str) -> Vec<&'static BahywayAlignment> {
    ALIGNMENTS.iter().filter(|a| a.bahyway_crate == crate_name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignments_not_empty() {
        assert!(!ALIGNMENTS.is_empty());
    }

    #[test]
    fn lookup_dq_freshness_alignment() {
        let hits = alignments_for("DQ.FRESHNESS");
        assert!(!hits.is_empty());
        assert_eq!(hits[0].bahyway_crate, "score-engine");
    }

    #[test]
    fn fuzzy_engine_has_multiple_alignments() {
        let hits = alignments_by_crate("fuzzy-engine");
        assert!(hits.len() >= 4);
    }

    #[test]
    fn all_alignments_have_non_empty_fields() {
        for a in ALIGNMENTS {
            assert!(!a.dama_code.is_empty());
            assert!(!a.bahyway_crate.is_empty());
            assert!(!a.bahyway_concept.is_empty());
            assert!(!a.mapping_note.is_empty());
        }
    }
}
