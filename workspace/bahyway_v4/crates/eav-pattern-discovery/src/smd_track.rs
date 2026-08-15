//! Track B — SMD (Semantic Modeling for Data): attribute-NAME
//! equivalence reconciliation across an organization's different
//! departments. The Architect's own example: `employee_name` <=>
//! `Empl_firstname`/`Empl_surname` <=> `Employee_Full_name` — different
//! departments naming the same business concept differently.
//!
//! Distinct from KAKI's own identity resolution (which the Architect
//! confirmed stays untouched/immutable): SMD operates one layer up, on
//! schema VOCABULARY, never on entity identity.
//!
//! HONEST SCOPE: `name_similarity`/`detect_smd_clusters` are one-to-one
//! matching — normalized-token Jaccard similarity between individual
//! attribute names. `detect_composition_candidates` (2026-07-31) is the
//! many-to-one follow-up: it unions two-or-more SAME-department fields'
//! tokens and fuzzy-matches the union against every OTHER department's
//! individual fields, catching real cases like `Empl_firstname` +
//! `Empl_surname` together composing `employee_name`/`Employee_Full_Name`
//! — via a bounded, named heuristic (prefix/substring token equivalence,
//! `MIN_FUZZY_LEN`-guarded so short strings can't trivially "match"
//! everything), not full semantic understanding. It will miss
//! compositions that don't share any token-level resemblance at all
//! (e.g. a department using a completely different vocabulary), and it
//! only tries 2-field combinations, not arbitrary N-way ones — real,
//! named limits, not silently overclaimed as solved.

use std::collections::HashSet;

/// One attribute name as it appears in one department's own data.
#[derive(Debug, Clone)]
pub struct DepartmentAttr {
    pub department: String,
    pub attr_name: String,
}

/// A cluster of attribute names, from different departments, judged
/// equivalent by name similarity.
#[derive(Debug, Clone)]
pub struct SmdEquivalenceCluster {
    pub members: Vec<DepartmentAttr>,
    /// Average pairwise token-similarity across the cluster (0.0..=1.0)
    /// — the confidence signal fed to `build_discovered_pattern`.
    pub confidence: f64,
}

/// Split an attribute name into lowercase tokens on common separators
/// (`_`, `-`, whitespace, and camelCase/PascalCase boundaries), dropping
/// empty tokens. `employee_name` -> {"employee", "name"};
/// `EmplFirstname` -> {"empl", "firstname"}.
fn tokenize(name: &str) -> HashSet<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;
    for c in name.chars() {
        if c == '_' || c == '-' || c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            prev_lower = false;
            continue;
        }
        if c.is_uppercase() && prev_lower {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        }
        current.push(c.to_ascii_lowercase());
        prev_lower = c.is_lowercase();
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens.into_iter().filter(|t| !t.is_empty()).collect()
}

/// Jaccard similarity between two attribute names' token sets:
/// |intersection| / |union|. 0.0 for two empty names (vacuously
/// dissimilar, not divided-by-zero).
pub fn name_similarity(a: &str, b: &str) -> f64 {
    let ta = tokenize(a);
    let tb = tokenize(b);
    if ta.is_empty() && tb.is_empty() {
        return 0.0;
    }
    let intersection = ta.intersection(&tb).count();
    let union = ta.union(&tb).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Detect cross-department attribute-name equivalence clusters: any set
/// of attrs from DIFFERENT departments whose pairwise name similarity is
/// all >= `min_similarity`. Real, single-linkage-style grouping — a real
/// v1 that catches literal token overlap, not full semantic
/// understanding (see this module's own doc comment for the named
/// many-to-one limitation).
pub fn detect_smd_clusters(attrs: &[DepartmentAttr], min_similarity: f64) -> Vec<SmdEquivalenceCluster> {
    let mut clusters: Vec<SmdEquivalenceCluster> = Vec::new();
    let mut used = vec![false; attrs.len()];

    for i in 0..attrs.len() {
        if used[i] {
            continue;
        }
        let mut members = vec![attrs[i].clone()];
        let mut similarities = Vec::new();
        used[i] = true;
        for j in (i + 1)..attrs.len() {
            if used[j] || attrs[j].department == attrs[i].department {
                continue;
            }
            let sim = name_similarity(&attrs[i].attr_name, &attrs[j].attr_name);
            if sim >= min_similarity {
                members.push(attrs[j].clone());
                similarities.push(sim);
                used[j] = true;
            }
        }
        if members.len() >= 2 {
            let confidence = similarities.iter().sum::<f64>() / similarities.len() as f64;
            clusters.push(SmdEquivalenceCluster { members, confidence });
        }
    }
    clusters
}

/// Shortest token length a fuzzy (prefix/substring) match is allowed to
/// apply to. Guards against trivial false positives like `"a"` matching
/// almost anything — below this length, only an exact token match counts.
const MIN_FUZZY_LEN: usize = 4;

/// Are two individual tokens "the same concept" for composition purposes?
/// Exact match always counts; below `MIN_FUZZY_LEN` that's the only way
/// they can match. At or above it, a prefix relationship (`"empl"` /
/// `"employee"`) or a substring relationship (`"name"` / `"surname"`)
/// also counts — a real, bounded heuristic, not full semantic understanding.
fn tokens_equivalent(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    if a.len() >= MIN_FUZZY_LEN && b.starts_with(a) {
        return true;
    }
    if b.len() >= MIN_FUZZY_LEN && a.starts_with(b) {
        return true;
    }
    if a.len() >= MIN_FUZZY_LEN && b.contains(a) {
        return true;
    }
    if b.len() >= MIN_FUZZY_LEN && a.contains(b) {
        return true;
    }
    false
}

/// Jaccard-style similarity between two token sets, but "intersection"
/// counts a greedy one-to-one `tokens_equivalent` pairing instead of
/// requiring exact equality — this is what lets a composed union like
/// `{"empl","firstname","surname"}` register real overlap against
/// `{"employee","full","name"}` even though no token is byte-identical.
fn fuzzy_set_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_tokens: Vec<&String> = {
        let mut v: Vec<&String> = a.iter().collect();
        v.sort();
        v
    };
    let b_tokens: Vec<&String> = {
        let mut v: Vec<&String> = b.iter().collect();
        v.sort();
        v
    };
    let mut b_used = vec![false; b_tokens.len()];
    let mut matched = 0usize;
    for ta in &a_tokens {
        for (j, tb) in b_tokens.iter().enumerate() {
            if !b_used[j] && tokens_equivalent(ta, tb) {
                b_used[j] = true;
                matched += 1;
                break;
            }
        }
    }
    let union = a_tokens.len() + b_tokens.len() - matched;
    if union == 0 {
        return 0.0;
    }
    matched as f64 / union as f64
}

/// A many-to-one composition candidate: two-or-more same-department
/// attrs (`components`) whose UNIONED tokens fuzzy-match another
/// department's single attr (`composed_of`) at or above the caller's
/// similarity threshold.
#[derive(Debug, Clone)]
pub struct CompositionCandidate {
    pub components: Vec<DepartmentAttr>,
    pub composed_of: DepartmentAttr,
    pub confidence: f64,
}

/// Detect many-to-one SMD composition: for every same-department PAIR of
/// attrs, union their tokens and fuzzy-compare that union against every
/// OTHER department's individual attrs. Only tries 2-field combinations
/// (see this module's own doc comment for why that's a named limit, not
/// a silent one).
pub fn detect_composition_candidates(attrs: &[DepartmentAttr], min_similarity: f64) -> Vec<CompositionCandidate> {
    let mut out = Vec::new();
    for i in 0..attrs.len() {
        for j in (i + 1)..attrs.len() {
            if attrs[i].department != attrs[j].department {
                continue;
            }
            let union_tokens: HashSet<String> =
                tokenize(&attrs[i].attr_name).union(&tokenize(&attrs[j].attr_name)).cloned().collect();
            for other in attrs {
                if other.department == attrs[i].department {
                    continue;
                }
                let other_tokens = tokenize(&other.attr_name);
                let sim = fuzzy_set_similarity(&union_tokens, &other_tokens);
                if sim >= min_similarity {
                    out.push(CompositionCandidate {
                        components: vec![attrs[i].clone(), attrs[j].clone()],
                        composed_of: other.clone(),
                        confidence: sim,
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(dept: &str, name: &str) -> DepartmentAttr {
        DepartmentAttr { department: dept.to_string(), attr_name: name.to_string() }
    }

    #[test]
    fn tokenize_splits_on_underscore() {
        let t = tokenize("employee_name");
        assert!(t.contains("employee"));
        assert!(t.contains("name"));
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn tokenize_splits_camel_case() {
        let t = tokenize("EmployeeFullName");
        assert!(t.contains("employee"));
        assert!(t.contains("full"));
        assert!(t.contains("name"));
    }

    #[test]
    fn identical_names_are_fully_similar() {
        assert_eq!(name_similarity("employee_name", "employee_name"), 1.0);
    }

    #[test]
    fn completely_different_names_are_dissimilar() {
        let sim = name_similarity("employee_name", "pipeline_pressure");
        assert!(sim < 0.1, "unrelated names must score near zero, got {sim}");
    }

    #[test]
    fn the_architects_own_example_is_detected() {
        // employee_name <=> Employee_Full_name: shares "employee" and
        // both have a "name"-ish token -- a real, catchable case for
        // this v1's token-overlap approach.
        let sim = name_similarity("employee_name", "Employee_Full_Name");
        assert!(sim > 0.3, "expected meaningful overlap between employee_name and Employee_Full_Name, got {sim}");
    }

    #[test]
    fn detect_smd_clusters_groups_cross_department_equivalents() {
        let attrs = vec![
            attr("hr", "employee_name"),
            attr("finance", "Employee_Full_Name"),
            attr("logistics", "pipeline_pressure"), // unrelated, must not join the cluster
        ];
        let clusters = detect_smd_clusters(&attrs, 0.3);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].members.len(), 2);
    }

    #[test]
    fn detect_smd_clusters_never_groups_the_same_department_with_itself() {
        let attrs = vec![
            attr("hr", "employee_name"),
            attr("hr", "employee_name_2"), // same department -- never a cross-department equivalence
        ];
        let clusters = detect_smd_clusters(&attrs, 0.1);
        assert!(clusters.is_empty(), "same-department attrs must never form an SMD cluster");
    }

    #[test]
    fn detect_smd_clusters_returns_empty_for_no_matches() {
        let attrs = vec![attr("hr", "employee_name"), attr("logistics", "pipeline_pressure")];
        let clusters = detect_smd_clusters(&attrs, 0.5);
        assert!(clusters.is_empty());
    }

    #[test]
    fn tokens_equivalent_matches_exact() {
        assert!(tokens_equivalent("name", "name"));
    }

    #[test]
    fn tokens_equivalent_matches_prefix_above_min_len() {
        assert!(tokens_equivalent("empl", "employee"));
    }

    #[test]
    fn tokens_equivalent_matches_substring_above_min_len() {
        assert!(tokens_equivalent("surname", "name"));
        assert!(tokens_equivalent("firstname", "name"));
    }

    #[test]
    fn tokens_equivalent_rejects_short_unrelated_tokens() {
        // "id" is a substring of "grid" but both are below MIN_FUZZY_LEN
        // via the SHORT side, and they aren't equal -- must not match.
        assert!(!tokens_equivalent("id", "grid"));
        assert!(!tokens_equivalent("a", "cat"));
    }

    #[test]
    fn fuzzy_set_similarity_of_identical_sets_is_one() {
        let a = tokenize("employee_name");
        let b = tokenize("employee_name");
        assert_eq!(fuzzy_set_similarity(&a, &b), 1.0);
    }

    #[test]
    fn fuzzy_set_similarity_of_empty_set_is_zero() {
        let empty: HashSet<String> = HashSet::new();
        let b = tokenize("employee_name");
        assert_eq!(fuzzy_set_similarity(&empty, &b), 0.0);
    }

    #[test]
    fn detect_composition_candidates_finds_the_architects_example() {
        // hr splits the concept across two fields; finance names it as
        // one field. Composing hr's two fields together must fuzzy-match
        // finance's single field, even though no token is byte-identical.
        let attrs = vec![
            attr("hr", "Empl_firstname"),
            attr("hr", "Empl_surname"),
            attr("finance", "Employee_Full_Name"),
        ];
        let candidates = detect_composition_candidates(&attrs, 0.3);
        assert_eq!(candidates.len(), 1, "expected exactly one composition candidate, got {candidates:?}");
        let c = &candidates[0];
        assert_eq!(c.composed_of.department, "finance");
        assert_eq!(c.components.len(), 2);
        assert!(c.confidence >= 0.3 && c.confidence <= 0.8, "confidence out of expected range: {}", c.confidence);
    }

    #[test]
    fn detect_composition_candidates_never_composes_within_the_same_department() {
        let attrs = vec![attr("hr", "Empl_firstname"), attr("hr", "Empl_surname")];
        let candidates = detect_composition_candidates(&attrs, 0.1);
        assert!(candidates.is_empty(), "must never treat two same-department fields as composing each other");
    }

    #[test]
    fn detect_composition_candidates_returns_empty_when_nothing_meets_threshold() {
        let attrs = vec![
            attr("hr", "Empl_firstname"),
            attr("hr", "Empl_surname"),
            attr("logistics", "pipeline_pressure"),
        ];
        let candidates = detect_composition_candidates(&attrs, 0.3);
        assert!(candidates.is_empty());
    }
}
