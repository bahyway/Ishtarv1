#![forbid(unsafe_code)]
//! # NARAMSIN Audit — NĀRU Sovereign Query Journal Integration
//!
//! NARAMSIN mints one particle type of its own: a lightweight audit record
//! for every archive it processes, written to the NĀRU sovereign query journal
//! regardless of outcome (clean, partial, or rejected).
//!
//! This guarantees a permanent record of every file BahyWay ever attempted
//! to ingest, independent of whether that ingestion ultimately succeeded
//! (EN-NARAMSIN-001 §6.2).

use naramsin_integrity::NrmCode;

/// Destination module for a NARAMSIN-processed archive batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationModule {
    Nanshe,
    Esarhaddon,
    Ashnan,
    Other,
}

impl DestinationModule {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Nanshe => "NANSHE",
            Self::Esarhaddon => "ESARHADDON",
            Self::Ashnan => "ASHNAN",
            Self::Other => "OTHER",
        }
    }
}

/// Immutable audit record for a single NARAMSIN archive processing attempt.
/// Written to the NĀRU sovereign query journal for every attempt, regardless
/// of outcome.
#[derive(Debug, Clone)]
pub struct ArchiveAuditRecord {
    /// Unique UUID v7 per processing attempt.
    pub archive_id: u128,
    /// Original archive/file name as received.
    pub source_filename: &'static str,
    /// Detected archive format name (from ArchiveFormat::name()).
    pub archive_format: &'static str,
    /// Deterministic NRM_* integrity code for this attempt.
    pub integrity_code: NrmCode,
    /// Count of successfully extracted and parsed contained files.
    pub files_extracted: u32,
    /// Count of contained files that failed parsing.
    pub files_rejected: u32,
    /// Which module's MASHSHARU gate this batch was routed to.
    pub destination: DestinationModule,
    /// Stage 0–2 wall-clock processing duration in milliseconds.
    pub processing_duration_ms: u32,
}

impl ArchiveAuditRecord {
    /// Render this record as a compact NĀRU journal line.
    pub fn to_journal_line(&self) -> String {
        format!(
            "NARAMSIN|{}|{}|{}|{}|extracted={}|rejected={}|dest={}|{}ms",
            self.archive_id,
            self.source_filename,
            self.archive_format,
            self.integrity_code.label(),
            self.files_extracted,
            self.files_rejected,
            self.destination.name(),
            self.processing_duration_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_line_contains_integrity_code() {
        let rec = ArchiveAuditRecord {
            archive_id: 0xDEADBEEF,
            source_filename: "sensor_data.zip",
            archive_format: "zip",
            integrity_code: NrmCode::Clean,
            files_extracted: 3,
            files_rejected: 0,
            destination: DestinationModule::Ashnan,
            processing_duration_ms: 42,
        };
        let line = rec.to_journal_line();
        assert!(line.contains("NRM_CLEAN"));
        assert!(line.contains("ASHNAN"));
    }
}
