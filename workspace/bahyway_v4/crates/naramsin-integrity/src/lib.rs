#![forbid(unsafe_code)]
//! # NARAMSIN Stage 2 — Sovereign Integrity Code Classification
//!
//! Every Stage 0–2 outcome receives exactly one deterministic NRM_* integrity
//! code, recorded immutably and passed forward as EAV context to MASHSHARU
//! and KIBRATU (EN-NARAMSIN-001 §5).
//!
//! This module is purely rule-based — no LLM, no reasoning, no external dependencies.
//! An NRM_TRUNCATED classification is the exact output the ASHNAN BeeMDM test
//! corpus's intentionally corrupted zip (truncated to 60% of valid size) was
//! designed to exercise.
//!
//! ## NRM_* Code Taxonomy
//! | Code | Severity | Action |
//! |------|----------|--------|
//! | NRM_CLEAN | None | Pass to MASHSHARU normally |
//! | NRM_PARTIAL | Warning | Pass recovered files with provenance flag |
//! | NRM_TRUNCATED | Reject | Full reject, no partial extraction |
//! | NRM_MALFORMED | Reject | Reject specific file; archive NRM_PARTIAL if others clean |
//! | NRM_UNKNOWN_FORMAT | Reject | Reject with "unsupported format" log |
//! | NRM_SCHEMA_DRIFT | Advisory | Pass forward with advisory flag for KIBRATU |

use naramsin_archive::error::ArchiveError;
use naramsin_format::error::FormatError;

/// Deterministic NRM_* integrity code for a NARAMSIN processing outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NrmCode {
    /// Archive and all contained files parsed without error.
    Clean,
    /// Archive recoverable but one or more contained files failed to parse.
    Partial,
    /// Archive truncated mid-transfer — no partial extraction attempted.
    Truncated,
    /// Contained file fails format-layer parse despite valid archive.
    Malformed,
    /// Archive or file signature does not match any supported format.
    UnknownFormat,
    /// File parses cleanly but column/field set differs from last known schema.
    SchemaDrift,
}

impl NrmCode {
    /// Derive NRM code from a Stage 0 archive error.
    pub fn from_archive_error(e: &ArchiveError) -> Self {
        match e {
            ArchiveError::Truncated => Self::Truncated,
            ArchiveError::UnknownFormat => Self::UnknownFormat,
            ArchiveError::EmptyInput => Self::Malformed,
            ArchiveError::MaxRecursionDepth(_) => Self::Malformed,
            ArchiveError::UnsupportedFormat(_) => Self::UnknownFormat,
        }
    }

    /// Derive NRM code from a Stage 1 format error.
    pub fn from_format_error(e: &FormatError) -> Self {
        match e {
            FormatError::RaggedCsvRow { .. } => Self::Malformed,
            FormatError::InvalidJson { .. } => Self::Malformed,
            FormatError::InvalidXml { .. } => Self::Malformed,
            FormatError::SchemaDrift { .. } => Self::SchemaDrift,
        }
    }

    /// True if this code results in a sovereign rejection (no forward pass to MASHSHARU).
    pub fn is_rejection(&self) -> bool {
        matches!(
            self,
            Self::Truncated | Self::Malformed | Self::UnknownFormat
        )
    }

    /// Human-readable label for NĀRU audit journal.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Clean => "NRM_CLEAN",
            Self::Partial => "NRM_PARTIAL",
            Self::Truncated => "NRM_TRUNCATED",
            Self::Malformed => "NRM_MALFORMED",
            Self::UnknownFormat => "NRM_UNKNOWN_FORMAT",
            Self::SchemaDrift => "NRM_SCHEMA_DRIFT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_archive_is_rejection() {
        let code = NrmCode::from_archive_error(&ArchiveError::Truncated);
        assert_eq!(code, NrmCode::Truncated);
        assert!(code.is_rejection());
    }

    #[test]
    fn schema_drift_is_not_rejection() {
        let code = NrmCode::from_format_error(&FormatError::SchemaDrift { source_id: 42 });
        assert_eq!(code, NrmCode::SchemaDrift);
        assert!(!code.is_rejection());
    }

    #[test]
    fn clean_code_label() {
        assert_eq!(NrmCode::Clean.label(), "NRM_CLEAN");
    }
}
