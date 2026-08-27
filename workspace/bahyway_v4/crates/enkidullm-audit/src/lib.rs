//! enkidullm-audit — Sovereign plagiarism probe and provenance audit.
//!
//! "The system does not accuse. It reveals the gap between what is known
//! and what is acknowledged." — Triple-O Provenance Axiom
//!
//! Pipeline:
//!   concept_graph.rs  — Jaccard overlap, concept extraction, missing citations
//!   citation.rs       — Coverage ratio, citation depth, suspicious threshold
//!   probe.rs          — PlagiarismProbe combining overlap + citation + IDU degradation
//!   journal.rs        — Immutable AuditJournal, sovereign evidence tablets
//!
//! §8.3 compliance: CrossTribe state (plagiarism relationship) is COMPUTED on demand,
//! never stored as a persistent filament. Journal entries ARE stored, but as
//! Event-KAKIs (type=0x02) — they record the computation, not the filament.

#![forbid(unsafe_code)]

pub mod citation;
pub mod concept_graph;
pub mod journal;
pub mod probe;

pub use citation::{
    analyze_citation_coverage, citation_depth, CitationAnalysis, SUSPICIOUS_THRESHOLD,
};
pub use concept_graph::{
    concept_graph_overlap, extract_concepts, jaccard_similarity, missing_concepts,
};
pub use journal::{AuditJournal, AuditJournalEntry};
pub use probe::{
    batch_probe, probe_plagiarism, ProbeInput, ProbeResult, OVERLAP_HIGH, OVERLAP_SUSPICIOUS,
};

/// Run a full audit and write flagged results to the journal.
/// Returns the number of entries written.
pub fn audit_and_journal(
    pairs: &[(ProbeInput, ProbeInput)],
    journal: &mut AuditJournal,
    timestamp_secs: u64,
) -> usize {
    let results = batch_probe(pairs);
    let mut written = 0;
    for result in &results {
        if result.is_flagged {
            let entry = AuditJournalEntry::from_probe(result, timestamp_secs);
            if journal.append(entry) {
                written += 1;
            }
        }
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidullm_core::orbit::{IduState, KnowledgeTriple};

    fn t(s: &str, p: &str, o: &str) -> KnowledgeTriple {
        KnowledgeTriple::new(s, p, o)
    }

    #[test]
    fn full_audit_pipeline() {
        let source_triples = vec![
            t("ch1", "introduces", "CAP_Theorem"),
            t("ch1", "discusses", "ACID_vs_BASE"),
            t("ch2", "depends_on", "Paxos"),
        ];
        let suspect_triples = source_triples.clone(); // exact copy, no citation

        let pairs = vec![(
            ProbeInput {
                uuid_hash: 0xBBBB_0000,
                triples: suspect_triples,
                cited_hashes: vec![], // does NOT cite source
                idu_state: IduState::Gold,
            },
            ProbeInput {
                uuid_hash: 0xAAAA_0000,
                triples: source_triples,
                cited_hashes: vec![],
                idu_state: IduState::Gold,
            },
        )];

        let mut journal = AuditJournal::new();
        let written = audit_and_journal(&pairs, &mut journal, 9_999_999);

        assert_eq!(
            written, 1,
            "one flagged pair should produce one journal entry"
        );
        assert_eq!(journal.len(), 1);
        let entry = &journal.entries()[0];
        assert!(entry.verify());
        assert_eq!(entry.idu_state, IduState::Orange); // Gold degraded to Orange
    }

    #[test]
    fn audit_clean_books_no_journal_entries() {
        let pairs = vec![(
            ProbeInput {
                uuid_hash: 0xBBBB,
                triples: vec![t("x", "y", "z")],
                cited_hashes: vec![],
                idu_state: IduState::Gold,
            },
            ProbeInput {
                uuid_hash: 0xAAAA,
                triples: vec![t("a", "b", "c")],
                cited_hashes: vec![],
                idu_state: IduState::Gold,
            },
        )];

        let mut journal = AuditJournal::new();
        let written = audit_and_journal(&pairs, &mut journal, 1_000);
        assert_eq!(written, 0);
        assert!(journal.is_empty());
    }
}
