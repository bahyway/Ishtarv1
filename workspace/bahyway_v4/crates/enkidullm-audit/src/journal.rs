//! AuditJournal — immutable evidence record for plagiarism findings.
//!
//! Every probe result that is flagged becomes an append-only Tribe Journal entry.
//! The journal is the legal chain: sovereign, verifiable, not asserted.
//!
//! Entry KAKI: kaki_type=Event(0x02), kaki_role=Zikru(0x02), tribe=LINGUISTIC.
//! The FNV-1a hash of (suspect_uuid, source_uuid, timestamp) produces a
//! deterministic event_uuid — same finding at same time → same evidence record.

use crate::probe::ProbeResult;
use bahyway_core::TribeId;
use bahyway_crc::crc16;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use enkidullm_core::orbit::IduState;

/// The Linguistic tribe used for audit event KAKIs.
const AUDIT_TRIBE: TribeId = TribeId::from_u16(0x10FF);

/// An immutable audit journal entry — sovereign evidence of a plagiarism finding.
#[derive(Debug, Clone)]
pub struct AuditJournalEntry {
    /// Event KAKI (kaki_type=Event) — the identity of this evidence record.
    pub event_kaki: Kaki,
    /// Book being audited (suspect).
    pub suspect_uuid: u32,
    /// Original source book.
    pub source_uuid: u32,
    /// Knowledge graph overlap at detection time.
    pub overlap_score: f32,
    /// Citation coverage ratio at detection time.
    pub citation_gap: f32,
    /// IDU state after degradation.
    pub idu_state: IduState,
    /// Detection mode string — the "why" of the finding.
    pub detection_mode: String,
    /// Missing citation concepts — the evidence of erasure.
    pub missing_concepts: Vec<String>,
    /// Timestamp in seconds since Unix epoch.
    pub timestamp_secs: u64,
    /// CRC-16 of the evidence payload — structural integrity check.
    pub evidence_checksum: u16,
}

impl AuditJournalEntry {
    /// Create an immutable journal entry from a probe result.
    pub fn from_probe(result: &ProbeResult, timestamp_secs: u64) -> Self {
        let minter = KakiMinter::new(AUDIT_TRIBE);
        // Deterministic uuid_hash from (suspect, source, timestamp) via FNV-1a
        let uuid_hash = evidence_hash(result.suspect_uuid, result.source_uuid, timestamp_secs);
        let event_kaki = minter.mint_event(uuid_hash, KakiRole::Zikru);

        // CRC-16 over evidence payload for structural integrity.
        // citation_gap is computed once here and re-used in the payload so
        // verify() can reproduce the same bits without a lossy roundtrip.
        let citation_gap = 1.0 - result.citation_coverage;
        let payload = build_payload(
            result.suspect_uuid,
            result.source_uuid,
            result.knowledge_overlap,
            citation_gap,
            timestamp_secs,
            &result.detection_mode,
        );
        let evidence_checksum = crc16(&payload);

        Self {
            event_kaki,
            suspect_uuid: result.suspect_uuid,
            source_uuid: result.source_uuid,
            overlap_score: result.knowledge_overlap,
            citation_gap,
            idu_state: result.degraded_idu_state,
            detection_mode: result.detection_mode.clone(),
            missing_concepts: result.missing_concepts.clone(),
            timestamp_secs,
            evidence_checksum,
        }
    }

    /// Verify structural integrity of this journal entry.
    pub fn verify(&self) -> bool {
        let payload = build_payload(
            self.suspect_uuid,
            self.source_uuid,
            self.overlap_score,
            self.citation_gap,
            self.timestamp_secs,
            &self.detection_mode,
        );
        crc16(&payload) == self.evidence_checksum
    }

    /// Render a human-readable sovereign evidence tablet.
    pub fn to_markdown(&self) -> String {
        format!(
            "# 𒁾 Sovereign Audit Tablet\n\
             **Suspect:** `{:#010x}`  **Source:** `{:#010x}`\n\
             **Overlap:** {:.2}  **Citation Gap:** {:.2}  **State:** {}\n\
             **Mode:** {}\n\
             **Missing Concepts:** {}\n\
             **Evidence Seal:** `{:#06x}`  **Timestamp:** {}\n",
            self.suspect_uuid,
            self.source_uuid,
            self.overlap_score,
            self.citation_gap,
            self.idu_state.label(),
            self.detection_mode,
            if self.missing_concepts.is_empty() {
                "none".to_string()
            } else {
                self.missing_concepts.join(", ")
            },
            self.evidence_checksum,
            self.timestamp_secs,
        )
    }
}

/// Append-only audit journal.
#[derive(Debug, Default)]
pub struct AuditJournal {
    entries: Vec<AuditJournalEntry>,
}

impl AuditJournal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entry. Returns false if the evidence is a duplicate (same checksum).
    pub fn append(&mut self, entry: AuditJournalEntry) -> bool {
        if self
            .entries
            .iter()
            .any(|e| e.evidence_checksum == entry.evidence_checksum)
        {
            return false;
        }
        self.entries.push(entry);
        true
    }

    pub fn entries(&self) -> &[AuditJournalEntry] {
        &self.entries
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Filter entries by IDU state.
    pub fn by_idu_state(&self, state: IduState) -> Vec<&AuditJournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.idu_state == state)
            .collect()
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn evidence_hash(suspect: u32, source: u32, ts: u64) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME: u32 = 16_777_619;
    let mut h = OFFSET;
    for &b in &suspect.to_le_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    for &b in &source.to_le_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    for &b in &ts.to_le_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn build_payload(
    suspect_uuid: u32,
    source_uuid: u32,
    overlap: f32,
    citation_gap: f32,
    ts: u64,
    detection_mode: &str,
) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&suspect_uuid.to_le_bytes());
    v.extend_from_slice(&source_uuid.to_le_bytes());
    v.extend_from_slice(&overlap.to_bits().to_le_bytes());
    v.extend_from_slice(&citation_gap.to_bits().to_le_bytes());
    v.extend_from_slice(&ts.to_le_bytes());
    v.extend_from_slice(detection_mode.as_bytes());
    v
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::ProbeResult;
    use enkidullm_core::orbit::IduState;

    fn make_probe_result(suspect: u32, source: u32) -> ProbeResult {
        ProbeResult {
            suspect_uuid: suspect,
            source_uuid: source,
            knowledge_overlap: 0.92,
            citation_coverage: 0.05,
            missing_concepts: vec!["CAP_Theorem".into(), "ACID".into()],
            degraded_idu_state: IduState::Orange,
            plagiarism_signal: 0.87,
            is_flagged: true,
            detection_mode: "HIGH_OVERLAP_WITH_CONCEPT_ERASURE".into(),
        }
    }

    #[test]
    fn entry_from_probe_has_valid_kaki() {
        let result = make_probe_result(0xBBBB, 0xAAAA);
        let entry = AuditJournalEntry::from_probe(&result, 1_000_000);
        assert!(entry.event_kaki.verify_checksum());
    }

    #[test]
    fn entry_verify_integrity() {
        let result = make_probe_result(0xBBBB, 0xAAAA);
        let entry = AuditJournalEntry::from_probe(&result, 1_000_000);
        assert!(entry.verify(), "evidence integrity check failed");
    }

    #[test]
    fn journal_append_and_dedup() {
        let mut journal = AuditJournal::new();
        let result = make_probe_result(0xBBBB, 0xAAAA);
        let entry1 = AuditJournalEntry::from_probe(&result, 1_000_000);
        let entry2 = AuditJournalEntry::from_probe(&result, 1_000_000); // same timestamp = same checksum

        assert!(journal.append(entry1));
        assert!(!journal.append(entry2), "duplicate should be rejected");
        assert_eq!(journal.len(), 1);
    }

    #[test]
    fn journal_filter_by_idu_state() {
        let mut journal = AuditJournal::new();
        let r1 = make_probe_result(0xBB01, 0xAA01);
        let mut r2 = make_probe_result(0xBB02, 0xAA02);
        r2.degraded_idu_state = IduState::Gray;

        journal.append(AuditJournalEntry::from_probe(&r1, 1_000));
        journal.append(AuditJournalEntry::from_probe(&r2, 2_000));

        let orange = journal.by_idu_state(IduState::Orange);
        let gray = journal.by_idu_state(IduState::Gray);
        assert_eq!(orange.len(), 1);
        assert_eq!(gray.len(), 1);
    }

    #[test]
    fn markdown_contains_required_fields() {
        let result = make_probe_result(0xBBBB, 0xAAAA);
        let entry = AuditJournalEntry::from_probe(&result, 1_000_000);
        let md = entry.to_markdown();
        assert!(md.contains("ORANGE"));
        assert!(md.contains("CAP_Theorem"));
        assert!(md.contains("Sovereign Audit Tablet"));
    }

    #[test]
    fn different_timestamps_produce_different_entries() {
        let result = make_probe_result(0xBBBB, 0xAAAA);
        let e1 = AuditJournalEntry::from_probe(&result, 1_000);
        let e2 = AuditJournalEntry::from_probe(&result, 2_000);
        // Different timestamps → different uuid_hash → different KAKI
        assert_ne!(e1.event_kaki.uuid_hash(), e2.event_kaki.uuid_hash());
    }
}
