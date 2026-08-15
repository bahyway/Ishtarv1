//! ConceptGraph — knowledge graph overlap computation for plagiarism detection.
//!
//! Two books are compared at the concept graph level.
//! overlap_score = |concepts(A) ∩ concepts(B)| / |concepts(A) ∪ concepts(B)|
//! (Jaccard similarity over normalized concept identifiers)
//!
//! The θ₆ (abstract) sector similarity from Zikru-Embed is a complementary
//! signal — high θ₆ similarity + low citation overlap = structural plagiarism.

use std::collections::HashSet;
use enkidullm_core::orbit::KnowledgeTriple;

/// Extract normalized concept identifiers from a set of knowledge triples.
/// Subjects and objects are included; predicates are excluded (they're structural).
pub fn extract_concepts(triples: &[KnowledgeTriple]) -> HashSet<String> {
    triples.iter()
        .flat_map(|t| [normalize_concept(&t.subject), normalize_concept(&t.object)])
        .filter(|c| !c.is_empty())
        .collect()
}

/// Normalize a concept identifier: lowercase, strip punctuation, trim.
fn normalize_concept(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Jaccard similarity between two concept sets: |A ∩ B| / |A ∪ B|.
/// Returns 0.0 for empty sets (no concepts = no overlap assertion).
pub fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() { return 0.0; }
    let intersection = a.intersection(b).count() as f32;
    let union        = a.union(b).count() as f32;
    if union == 0.0 { 0.0 } else { intersection / union }
}

/// Knowledge graph overlap between two books' triple sets.
pub fn concept_graph_overlap(a: &[KnowledgeTriple], b: &[KnowledgeTriple]) -> f32 {
    let ca = extract_concepts(a);
    let cb = extract_concepts(b);
    jaccard_similarity(&ca, &cb)
}

/// Find concepts present in B but NOT attributed to A in B's citation graph.
/// These are the "missing acknowledgments" — the negative-space of plagiarism.
pub fn missing_concepts(
    source_triples:  &[KnowledgeTriple],  // original (A)
    suspect_triples: &[KnowledgeTriple],  // suspect (B)
    suspect_cited_hashes: &[u32],         // what B actually cites (isbn hashes)
    source_uuid_hash: u32,                // A's book hash
) -> Vec<String> {
    let source_concepts  = extract_concepts(source_triples);
    let suspect_concepts = extract_concepts(suspect_triples);

    // Concepts shared between A and B
    let shared: HashSet<&String> = source_concepts.intersection(&suspect_concepts).collect();

    // If A is cited by B (uuid_hash in cited list), there's no plagiarism issue
    if suspect_cited_hashes.contains(&source_uuid_hash) { return Vec::new(); }

    // Missing: concepts from A that appear in B, but A is not cited by B
    shared.into_iter().cloned().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use enkidullm_core::orbit::KnowledgeTriple;

    fn t(s: &str, p: &str, o: &str) -> KnowledgeTriple { KnowledgeTriple::new(s, p, o) }

    #[test]
    fn jaccard_identical_sets() {
        let a: HashSet<String> = ["x", "y", "z"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &a) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn jaccard_disjoint_sets() {
        let a: HashSet<String> = ["a"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["b"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &b)).abs() < 1e-5);
    }

    #[test]
    fn concept_overlap_same_triples() {
        let triples = vec![t("ch1", "introduces", "CAP"), t("ch2", "depends_on", "ACID")];
        let sim = concept_graph_overlap(&triples, &triples);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn concept_overlap_renamed_concepts() {
        // Suspect renames "CAP_Theorem" → "Consistency_Trilemma" but keeps ACID
        let source  = vec![t("ch1", "introduces", "CAP_Theorem"), t("ch1", "discusses", "ACID")];
        let suspect = vec![t("ch1", "introduces", "Consistency_Trilemma"), t("ch1", "discusses", "ACID")];
        let sim = concept_graph_overlap(&source, &suspect);
        // ACID overlaps, CAP_Theorem vs Consistency_Trilemma don't
        // |intersection|=1 ("acid"), |union|=3 → 1/3 ≈ 0.333
        assert!(sim > 0.0 && sim < 1.0, "sim={}", sim);
    }

    #[test]
    fn missing_concepts_when_not_cited() {
        let source  = vec![t("ch1", "introduces", "CAP_Theorem")];
        let suspect = vec![t("ch1", "introduces", "CAP_Theorem")];
        let missing = missing_concepts(&source, &suspect, &[], 0xABCDEF);
        assert!(!missing.is_empty(), "should find missing citations");
    }

    #[test]
    fn no_missing_when_cited() {
        let source  = vec![t("ch1", "introduces", "CAP_Theorem")];
        let suspect = vec![t("ch1", "introduces", "CAP_Theorem")];
        // Suspect does cite the source (source_uuid_hash in cited list)
        let missing = missing_concepts(&source, &suspect, &[0xABCDEF], 0xABCDEF);
        assert!(missing.is_empty(), "no missing citations when source is cited");
    }

    #[test]
    fn normalize_concept_lowercases() {
        let c = normalize_concept("CAP_Theorem");
        assert_eq!(c, "cap_theorem");
    }
}
