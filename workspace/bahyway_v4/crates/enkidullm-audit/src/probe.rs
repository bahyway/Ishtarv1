//! PlagiarismProbe — the sovereign truth protocol.
//!
//! "Plagiarism is not similarity. Plagiarism is similarity with deliberate
//! provenance erasure."
//!
//! Detection pipeline:
//!   1. concept_graph_overlap(A, B)          → knowledge similarity [0,1]
//!   2. citation_coverage(A, B)              → provenance acknowledgment [0,1]
//!   3. gap = overlap − coverage             → plagiarism signal [0,1]
//!   4. IDU degradation: Gold → Orange if gap > threshold
//!   5. Journal entry: immutable evidence record

use enkidullm_core::orbit::{IduState, KnowledgeTriple};
use crate::concept_graph::{concept_graph_overlap, missing_concepts};
use crate::citation::{analyze_citation_coverage, CitationAnalysis, SUSPICIOUS_THRESHOLD};

/// Thresholds for plagiarism classification.
pub const OVERLAP_SUSPICIOUS:     f32 = 0.75;  // concept overlap above this = suspicious
pub const OVERLAP_HIGH:           f32 = 0.90;  // concept overlap above this = high-confidence

/// Result of a plagiarism probe between two books.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// uuid_hash of the suspected plagiarist (B).
    pub suspect_uuid:           u32,
    /// uuid_hash of the original source (A).
    pub source_uuid:            u32,
    /// Jaccard concept graph overlap [0,1].
    pub knowledge_overlap:      f32,
    /// Citation coverage ratio [0,1].
    pub citation_coverage:      f32,
    /// Missing concepts: present in both A and B, but A not cited by B.
    pub missing_concepts:       Vec<String>,
    /// IDU state AFTER degradation (may have changed from input).
    pub degraded_idu_state:     IduState,
    /// Plagiarism confidence: gap between overlap and coverage.
    pub plagiarism_signal:      f32,
    /// True when evidence is strong enough to warrant audit journal entry.
    pub is_flagged:             bool,
    /// Detection mode string — for the journal evidence record.
    pub detection_mode:         String,
}

/// Run a plagiarism probe between suspect book B and source book A.
pub fn probe_plagiarism(
    suspect_uuid:         u32,
    source_uuid:          u32,
    suspect_triples:      &[KnowledgeTriple],
    source_triples:       &[KnowledgeTriple],
    suspect_cited_hashes: &[u32],
    suspect_idu_state:    IduState,
) -> ProbeResult {
    let knowledge_overlap = concept_graph_overlap(source_triples, suspect_triples);

    let citation_analysis = analyze_citation_coverage(
        source_uuid,
        &crate::concept_graph::extract_concepts(source_triples)
            .into_iter()
            .collect::<Vec<_>>(),
        suspect_cited_hashes,
        &[source_uuid],
    );

    let missing = missing_concepts(source_triples, suspect_triples, suspect_cited_hashes, source_uuid);

    let plagiarism_signal = (knowledge_overlap - citation_analysis.coverage_ratio).max(0.0);
    let is_flagged = knowledge_overlap >= OVERLAP_SUSPICIOUS && citation_analysis.is_suspicious;

    let degraded_idu_state = if is_flagged {
        suspect_idu_state.degrade()
    } else {
        suspect_idu_state
    };

    let detection_mode = classify_detection_mode(knowledge_overlap, &citation_analysis, &missing);

    ProbeResult {
        suspect_uuid, source_uuid,
        knowledge_overlap, citation_coverage: citation_analysis.coverage_ratio,
        missing_concepts: missing, degraded_idu_state,
        plagiarism_signal, is_flagged, detection_mode,
    }
}

fn classify_detection_mode(
    overlap:  f32,
    citation: &CitationAnalysis,
    missing:  &[String],
) -> String {
    if overlap >= OVERLAP_HIGH && citation.coverage_ratio < SUSPICIOUS_THRESHOLD {
        if !missing.is_empty() {
            format!("HIGH_OVERLAP_WITH_CONCEPT_ERASURE_θ₆ overlap={:.2} citation_gap={:.2}",
                    overlap, 1.0 - citation.coverage_ratio)
        } else {
            format!("HIGH_OVERLAP_CITATION_CHOPPING overlap={:.2}", overlap)
        }
    } else if overlap >= OVERLAP_SUSPICIOUS {
        format!("MODERATE_OVERLAP_LOW_CITATION overlap={:.2} coverage={:.2}",
                overlap, citation.coverage_ratio)
    } else {
        format!("BELOW_THRESHOLD overlap={:.2}", overlap)
    }
}

/// Batch probe: run all (suspect, source) pairs and return flagged results.
pub fn batch_probe(pairs: &[(ProbeInput, ProbeInput)]) -> Vec<ProbeResult> {
    pairs.iter()
        .map(|(suspect, source)| probe_plagiarism(
            suspect.uuid_hash,
            source.uuid_hash,
            &suspect.triples,
            &source.triples,
            &suspect.cited_hashes,
            suspect.idu_state,
        ))
        .collect()
}

/// Input to a batch probe.
pub struct ProbeInput {
    pub uuid_hash:    u32,
    pub triples:      Vec<KnowledgeTriple>,
    pub cited_hashes: Vec<u32>,
    pub idu_state:    IduState,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use enkidullm_core::orbit::KnowledgeTriple;

    fn t(s: &str, p: &str, o: &str) -> KnowledgeTriple { KnowledgeTriple::new(s, p, o) }

    fn cap_triples() -> Vec<KnowledgeTriple> {
        vec![
            t("ch1", "introduces", "CAP_Theorem"),
            t("ch1", "discusses",  "ACID_vs_BASE"),
            t("ch2", "introduces", "Event_Sourcing"),
            t("ch2", "depends_on", "Distributed_Log"),
        ]
    }

    fn renamed_triples() -> Vec<KnowledgeTriple> {
        // Suspect renames concepts but keeps structure
        vec![
            t("ch1", "introduces", "CAP_Theorem"),       // same
            t("ch1", "discusses",  "ACID_vs_BASE"),       // same
            t("ch2", "introduces", "Log_Driven_Arch"),    // renamed
            t("ch2", "depends_on", "Distributed_Log"),    // same
        ]
    }

    #[test]
    fn probe_identical_books_high_overlap() {
        let triples = cap_triples();
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA,
            &triples, &triples,
            &[],  // suspect does NOT cite source
            IduState::Gold,
        );
        assert!((result.knowledge_overlap - 1.0).abs() < 1e-5);
        assert!(result.is_flagged, "identical uncited book should be flagged");
        assert_eq!(result.degraded_idu_state, IduState::Orange); // Gold → Orange
    }

    #[test]
    fn probe_cited_book_not_flagged() {
        let triples = cap_triples();
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA,
            &triples, &triples,
            &[0xAAAA],  // suspect DOES cite source
            IduState::Gold,
        );
        // Citation coverage = 1/4 = 0.25 > SUSPICIOUS_THRESHOLD, so NOT suspicious
        assert!(!result.is_flagged, "cited source should not be flagged");
        assert_eq!(result.degraded_idu_state, IduState::Gold); // unchanged
    }

    #[test]
    fn probe_renamed_concepts_detected() {
        let source  = cap_triples();
        let suspect = renamed_triples();
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA,
            &suspect, &source,
            &[], IduState::Orange,
        );
        // Should have significant overlap despite renames
        assert!(result.knowledge_overlap > 0.5,
                "overlap={}", result.knowledge_overlap);
    }

    #[test]
    fn probe_disjoint_books_not_flagged() {
        let source  = vec![t("A", "b", "C"), t("D", "e", "F")];
        let suspect = vec![t("X", "y", "Z"), t("W", "v", "U")];
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA,
            &suspect, &source,
            &[], IduState::Orange,
        );
        assert!(!result.is_flagged);
        assert!((result.knowledge_overlap).abs() < 1e-5);
    }

    #[test]
    fn plagiarism_signal_gap_correct() {
        let triples = cap_triples();
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA,
            &triples, &triples,
            &[], IduState::Gold,
        );
        // signal = overlap − coverage = 1.0 − 0.0 = 1.0
        assert!(result.plagiarism_signal > 0.0);
    }

    #[test]
    fn detection_mode_is_populated() {
        let triples = cap_triples();
        let result = probe_plagiarism(
            0xBBBB, 0xAAAA, &triples, &triples, &[], IduState::Gold,
        );
        assert!(!result.detection_mode.is_empty());
    }
}
