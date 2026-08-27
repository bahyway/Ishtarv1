//! ashnan-ingest — Sovereign ingest entry for ASHNAN (Climate-agriculture-livestock intelligence)
//! Source of truth: BC-AGRI-001
//!
//! Thin tribe-aware wrapper over `naramsin_bridge::process()`. Redesigned
//! against the real bridge API — an earlier playbook draft (PB-116) assumed
//! `FileProvenance` / `NaruJournal` / `SchemaRegistry` types that were never
//! actually implemented in `naramsin-bridge`.
#![forbid(unsafe_code)]

use naramsin_audit::DestinationModule;
use naramsin_bridge::process;
use std::time::Instant;
use uuid::Uuid;

pub const TRIBE_ID: &str = "ashnan_diyala";

/// Result of a single ingest attempt, summarised for the CLI runner
/// and for handoff to the module's KAKI schema builder (once it exists).
#[derive(Debug, Clone)]
pub struct IngestReport {
    pub archive_id: u128,
    pub filename: String,
    pub outcome_label: &'static str, // "CLEAN" / "PARTIAL" / "REJECTED"
    pub row_count: usize,
    pub nrm_code: &'static str,
    pub journal_line: String,
    pub blocked: bool,
}

/// Ingest a single file (raw bytes) for the ASHNAN tribe.
///
/// Called by BeeMDM 7-Gates ETL Stage 0 (MASHSHARU gate entry). The caller
/// inspects `.blocked` to decide whether to proceed to MASHSHARU or route
/// to KIBRATU.
///
/// Note: `naramsin_bridge::process()` requires a `&'static str` filename.
/// For a one-shot CLI invocation this leak is harmless (the process exits
/// immediately after); a long-running ingest service would need a
/// different bridge signature to avoid leaking per-request.
pub fn ingest(bytes: &[u8], filename: String) -> IngestReport {
    let archive_id = Uuid::new_v4().as_u128();
    let static_filename: &'static str = Box::leak(filename.clone().into_boxed_str());
    let t0 = Instant::now();

    let outcome = process(
        bytes,
        static_filename,
        DestinationModule::Ashnan,
        archive_id,
        0,
    );
    let elapsed_ms = t0.elapsed().as_millis() as u32;

    match outcome {
        naramsin_bridge::NaramsinOutcome::Clean(rows) => IngestReport {
            archive_id,
            filename,
            outcome_label: "CLEAN",
            row_count: rows.len(),
            nrm_code: "NRM_CLEAN",
            journal_line: format!(
                "NARAMSIN|{archive_id}|{static_filename}|dest=ASHNAN|rows={}|{elapsed_ms}ms",
                rows.len()
            ),
            blocked: false,
        },
        naramsin_bridge::NaramsinOutcome::Partial { rows, audit } => IngestReport {
            archive_id,
            filename,
            outcome_label: "PARTIAL",
            row_count: rows.len(),
            nrm_code: audit.integrity_code.label(),
            journal_line: audit.to_journal_line(),
            blocked: false,
        },
        naramsin_bridge::NaramsinOutcome::Rejected { code, audit } => IngestReport {
            archive_id,
            filename,
            outcome_label: "REJECTED",
            row_count: 0,
            nrm_code: code.label(),
            journal_line: audit.to_journal_line(),
            blocked: true,
        },
    }
}

/// Stub MASHSHARU handoff — to be implemented once ASHNAN's KAKI schema exists
/// (the real ashnan-kaki crate — PB-117 was never actually produced despite
/// being marked "WRITTEN" in the roadmap addendum).
pub fn hand_to_mashsharu(report: &IngestReport) {
    if report.blocked {
        tracing::warn!(
            "ashnan-ingest: NARAMSIN blocked ({}), skipping MASHSHARU",
            report.nrm_code
        );
        return;
    }
    tracing::info!(
        "ashnan-ingest: {} rows -> MASHSHARU (stub) | archive_id={}",
        report.row_count,
        report.archive_id
    );
    // TODO: hand row stream to ASHNAN-specific KAKI schema builder once it exists.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_returns_report_with_matching_filename() {
        let report = ingest(b"not a real archive", "test.zip".to_string());
        assert_eq!(report.filename, "test.zip");
    }

    #[test]
    fn ingest_empty_input_is_rejected_and_blocked() {
        let report = ingest(b"", "empty.bin".to_string());
        assert!(report.blocked);
        assert_eq!(report.nrm_code, "NRM_MALFORMED");
    }

    #[test]
    fn ingest_unrecognised_signature_is_treated_as_plain_file() {
        let report = ingest(b"not a real archive", "test.bin".to_string());
        assert!(!report.blocked);
        assert_eq!(report.nrm_code, "NRM_CLEAN");
    }
}
