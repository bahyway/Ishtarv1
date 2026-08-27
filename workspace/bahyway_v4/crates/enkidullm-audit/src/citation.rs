//! CitationGraph — citation coverage analysis for provenance auditing.
//!
//! The "citation gap" is the key plagiarism signal:
//!   knowledge_graph_overlap is HIGH (stolen concepts)
//!   citation_coverage_ratio is LOW  (source not acknowledged)
//!
//! citation_coverage_ratio = |cited ISBNs from source corpus| / |shared concepts|
//! A ratio near 0 with high overlap = citation chopping / provenance erasure.

/// Citation graph analysis result.
#[derive(Debug, Clone)]
pub struct CitationAnalysis {
    /// How many shared concepts the suspect covers from the source.
    pub shared_concept_count: u32,
    /// How many of the shared concepts' sources the suspect actually cites.
    pub cited_source_count: u32,
    /// Ratio: cited_sources / shared_concepts. Low = suspicious.
    pub coverage_ratio: f32,
    /// True when coverage_ratio < SUSPICIOUS_THRESHOLD.
    pub is_suspicious: bool,
}

/// Coverage ratio below which a book pair is flagged as suspicious.
pub const SUSPICIOUS_THRESHOLD: f32 = 0.15;

/// Compute citation coverage for a (source, suspect) pair.
///
/// Arguments:
///   source_uuid_hash      — uuid_hash of the original book (A)
///   shared_concepts       — concept identifiers shared between A and B
///   suspect_cited_hashes  — isbn hashes cited by the suspect book (B)
///   _all_source_hashes    — reserved for corpus-depth checks (future)
pub fn analyze_citation_coverage(
    source_uuid_hash: u32,
    shared_concepts: &[String],
    suspect_cited_hashes: &[u32],
    _all_source_hashes: &[u32],
) -> CitationAnalysis {
    let shared_n = shared_concepts.len() as u32;
    if shared_n == 0 {
        return CitationAnalysis {
            shared_concept_count: 0,
            cited_source_count: 0,
            coverage_ratio: 1.0,
            is_suspicious: false,
        };
    }

    let cited = if suspect_cited_hashes.contains(&source_uuid_hash) {
        1u32
    } else {
        0u32
    };
    let ratio = cited as f32 / shared_n as f32;

    CitationAnalysis {
        shared_concept_count: shared_n,
        cited_source_count: cited,
        coverage_ratio: ratio,
        is_suspicious: ratio < SUSPICIOUS_THRESHOLD,
    }
}

/// BFS depth from any of `suspect_hashes` to `source_hash` through citation edges.
/// Returns None if the source is not reachable.
pub fn citation_depth(
    source_hash: u32,
    suspect_hashes: &[u32],
    citation_map: &[(u32, u32)], // (from_hash, to_hash) citation edges
) -> Option<u32> {
    use std::collections::{HashSet, VecDeque};
    let mut visited: HashSet<u32> = HashSet::new();
    let mut queue: VecDeque<(u32, u32)> = suspect_hashes.iter().map(|&h| (h, 0)).collect();

    while let Some((current, depth)) = queue.pop_front() {
        if current == source_hash {
            return Some(depth);
        }
        if !visited.insert(current) {
            continue;
        }
        for &(from, to) in citation_map {
            if from == current && !visited.contains(&to) {
                queue.push_back((to, depth + 1));
            }
        }
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SRC_HASH: u32 = 0xAAAA_0001;
    const SUSPECT_HASH: u32 = 0xBBBB_0002;
    const OTHER_HASH: u32 = 0xCCCC_0003;

    fn concepts(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("concept_{}", i)).collect()
    }

    #[test]
    fn coverage_zero_when_not_cited() {
        let ca = analyze_citation_coverage(SRC_HASH, &concepts(5), &[], &[]);
        assert_eq!(ca.cited_source_count, 0);
        assert_eq!(ca.coverage_ratio, 0.0);
        assert!(ca.is_suspicious);
    }

    #[test]
    fn coverage_full_when_cited_one_concept() {
        let ca = analyze_citation_coverage(SRC_HASH, &concepts(1), &[SRC_HASH], &[]);
        assert_eq!(ca.cited_source_count, 1);
        assert!((ca.coverage_ratio - 1.0).abs() < 1e-5);
        assert!(!ca.is_suspicious);
    }

    #[test]
    fn suspicious_below_threshold() {
        // 20 shared concepts, source cited once → ratio = 1/20 = 0.05 < 0.15
        let ca = analyze_citation_coverage(SRC_HASH, &concepts(20), &[SRC_HASH], &[]);
        assert!(ca.is_suspicious, "ratio={}", ca.coverage_ratio);
    }

    #[test]
    fn not_suspicious_one_concept_one_citation() {
        let ca = analyze_citation_coverage(SRC_HASH, &concepts(1), &[SRC_HASH], &[]);
        assert!(!ca.is_suspicious);
    }

    #[test]
    fn no_shared_concepts_not_suspicious() {
        let ca = analyze_citation_coverage(SRC_HASH, &[], &[], &[]);
        assert!(!ca.is_suspicious);
        assert!((ca.coverage_ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn citation_depth_direct_hit() {
        let depth = citation_depth(SRC_HASH, &[SRC_HASH], &[]);
        assert_eq!(depth, Some(0));
    }

    #[test]
    fn citation_depth_one_hop() {
        let map = vec![(SUSPECT_HASH, SRC_HASH)];
        let depth = citation_depth(SRC_HASH, &[SUSPECT_HASH], &map);
        assert_eq!(depth, Some(1));
    }

    #[test]
    fn citation_depth_two_hops() {
        let map = vec![(SUSPECT_HASH, OTHER_HASH), (OTHER_HASH, SRC_HASH)];
        let depth = citation_depth(SRC_HASH, &[SUSPECT_HASH], &map);
        assert_eq!(depth, Some(2));
    }

    #[test]
    fn citation_depth_unreachable() {
        let depth = citation_depth(SRC_HASH, &[OTHER_HASH], &[]);
        assert_eq!(depth, None);
    }

    #[test]
    fn citation_depth_no_cycle() {
        // Cycle: SUSPECT → OTHER → SUSPECT (no path to SRC)
        let map = vec![(SUSPECT_HASH, OTHER_HASH), (OTHER_HASH, SUSPECT_HASH)];
        let depth = citation_depth(SRC_HASH, &[SUSPECT_HASH], &map);
        assert_eq!(depth, None);
    }
}
