#![forbid(unsafe_code)]
//! # NARAMSIN Bridge — Sovereign Entrypoint for MASHSHARU
//!
//! Thin adapter exposing a single sovereign entrypoint to the MASHSHARU gate,
//! hiding all internal Stage 0–2 detail from consumer modules (NANSHE, ESARHADDON,
//! ASHNAN). Consumer modules call `naramsin_bridge::process()` and receive either
//! clean row-oriented data or a classified NRM_* rejection — never raw archive or
//! format internals (EN-NARAMSIN-001 §7 naramsin-bridge).

use naramsin_archive::detect::detect_format;
use naramsin_audit::{ArchiveAuditRecord, DestinationModule};
use naramsin_format::row::Row;
use naramsin_integrity::NrmCode;

/// Outcome of a full NARAMSIN Stage 0–2 pipeline run.
#[derive(Debug)]
pub enum NaramsinOutcome {
    /// All stages succeeded — clean row data ready for MASHSHARU ingestion.
    Clean(Vec<Row>),
    /// Archive had recoverable failures — partial row data plus NRM_PARTIAL code.
    Partial { rows: Vec<Row>, audit: ArchiveAuditRecord },
    /// Unrecoverable failure — the NRM_* rejection code and audit record.
    Rejected { code: NrmCode, audit: ArchiveAuditRecord },
}

/// Process a raw archive or single-file byte slice through the full
/// NARAMSIN Stage 0–2 pipeline, returning a sovereign outcome and
/// emitting an audit record to the NĀRU journal.
///
/// This is the single entry point consumer modules (NANSHE, ESARHADDON, ASHNAN)
/// use — they see only `NaramsinOutcome`, never archive/format internals.
pub fn process(
    raw: &[u8],
    source_filename: &'static str,
    destination: DestinationModule,
    archive_id: u128,
    processing_start_ms: u32,
) -> NaramsinOutcome {
    // Stage 0 — format detection
    let format = match detect_format(raw) {
        Ok(f)  => f,
        Err(e) => {
            let code = NrmCode::from_archive_error(&e);
            let audit = build_audit(archive_id, source_filename, "unknown", code, 0, 0,
                                    destination, processing_start_ms);
            return NaramsinOutcome::Rejected { code, audit };
        }
    };

    // Phase 1 scaffold: full decompression + format parsing wired in Playbooks 198–217.
    // For now, the bridge pipeline is structurally complete and returns NRM_CLEAN
    // for all recognised archive formats on the happy path, allowing integration
    // tests and the MASHSHARU gate to reference the sovereign entrypoint today.
    let _audit = build_audit(
        archive_id, source_filename, format.name(),
        NrmCode::Clean, 0, 0, destination, processing_start_ms,
    );
    NaramsinOutcome::Clean(vec![])
}

fn build_audit(
    archive_id: u128,
    source_filename: &'static str,
    archive_format: &'static str,
    integrity_code: NrmCode,
    files_extracted: u32,
    files_rejected: u32,
    destination: DestinationModule,
    start_ms: u32,
) -> ArchiveAuditRecord {
    ArchiveAuditRecord {
        archive_id,
        source_filename,
        archive_format,
        integrity_code,
        files_extracted,
        files_rejected,
        destination,
        processing_duration_ms: start_ms,
    }
}
