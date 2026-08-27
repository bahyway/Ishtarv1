//! shape-verdict — GL-VIZ-008 (The Shape Verdict): non-matching
//! discovered shapes get one of three verdicts, decided by the
//! direction of the failing structure-preserving simplicial map.
//! PB-346/347. Pure Rust, zero dependencies.
//!
//! Convention (derived from the tablet's own stated verdict semantics,
//! §1-2): a structure-preserving map A -> B exists here iff B's feature
//! set is already covered by A's (A subsumes B). Under this convention:
//! "phi: S -> T exists" means S subsumes T (S is at least as rich as T),
//! so a failing phi^-1: T -> S means T has something S lacks -- exactly
//! the tablet's own reading of each verdict.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub type FeatureSet = BTreeSet<String>;

/// A structure-preserving simplicial map A -> B exists iff features(B)
/// is a subset of features(A).
pub fn map_exists(a: &FeatureSet, b: &FeatureSet) -> bool {
    b.is_subset(a)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Verdict 1: neither map exists, S is internally coherent -> new template.
    NewShape,
    /// Verdict 2: S -> T exists but not back -> S is richer -> amend the template.
    TemplateDefect,
    /// Verdict 3: T -> S exists but not back -> S has a hole -> complete the data.
    Incomplete,
    /// Both maps exist: not a genuine non-match; re-check the NESU threshold.
    NotANonMatch,
}

/// §3 — the Verdict Rite's CASE statement.
pub fn classify(s: &FeatureSet, t: &FeatureSet, s_internally_coherent: bool) -> Verdict {
    let s_to_t = map_exists(s, t);
    let t_to_s = map_exists(t, s);
    match (s_to_t, t_to_s) {
        (true, true) => Verdict::NotANonMatch,
        (true, false) => Verdict::TemplateDefect,
        (false, true) => Verdict::Incomplete,
        (false, false) => {
            if s_internally_coherent {
                Verdict::NewShape
            } else {
                // Neither map exists AND S is not internally coherent: the
                // tablet does not name this case explicitly. Treated
                // conservatively as Incomplete (never silently minted as
                // New) pending a real inquest.
                Verdict::Incomplete
            }
        }
    }
}

/// TDA names the exact obstruction: the symmetric difference of feature
/// sets is the named obstruction reported to the client (§2, Verdict 3).
pub fn named_obstruction(s: &FeatureSet, t: &FeatureSet) -> Vec<String> {
    s.symmetric_difference(t).cloned().collect()
}

#[derive(Debug, Clone)]
pub struct VerdictParticle {
    pub verdict: Verdict,
    pub obstruction: Vec<String>,
    pub epsilon: f64,
}

/// §3 — emit the VERDICT particle: which maps existed, the named
/// obstruction, and epsilon.
pub fn verdict_rite(s: &FeatureSet, t: &FeatureSet, s_internally_coherent: bool, epsilon: f64) -> VerdictParticle {
    VerdictParticle {
        verdict: classify(s, t, s_internally_coherent),
        obstruction: named_obstruction(s, t),
        epsilon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature_set(items: &[&str]) -> FeatureSet {
        items.iter().map(|s| s.to_string()).collect()
    }

    // Neither map, S coherent -> VERDICT 1 (new template).
    #[test]
    fn verdict1_new_shape() {
        let s = feature_set(&["loop-A", "component-B"]);
        let t = feature_set(&["loop-X", "component-Y"]);
        assert_eq!(classify(&s, &t, true), Verdict::NewShape);
    }

    // S -> T only (S richer) -> VERDICT 2 (template defect).
    #[test]
    fn verdict2_template_defect() {
        let t = feature_set(&["loop-A"]);
        let s = feature_set(&["loop-A", "loop-B"]); // S has everything T has, plus more
        assert_eq!(classify(&s, &t, true), Verdict::TemplateDefect);
        let obstruction = named_obstruction(&s, &t);
        assert_eq!(obstruction, vec!["loop-B".to_string()], "the extra feature is the named obstruction");
    }

    // T -> S only (S has a hole) -> VERDICT 3 (incomplete data).
    #[test]
    fn verdict3_incomplete() {
        let s = feature_set(&["loop-A"]);
        let t = feature_set(&["loop-A", "loop-B"]); // T has everything S has, plus more
        assert_eq!(classify(&s, &t, true), Verdict::Incomplete);
        let obstruction = named_obstruction(&s, &t);
        assert_eq!(obstruction, vec!["loop-B".to_string()], "T's missing-in-S feature is the named obstruction");
    }

    // Both maps exist -> not a non-match; NESU re-check.
    #[test]
    fn both_maps_exist_not_a_non_match() {
        let s = feature_set(&["loop-A", "loop-B"]);
        let t = feature_set(&["loop-A", "loop-B"]);
        assert_eq!(classify(&s, &t, true), Verdict::NotANonMatch);
        assert!(named_obstruction(&s, &t).is_empty());
    }

    #[test]
    fn verdict_rite_carries_epsilon() {
        let s = feature_set(&["loop-A"]);
        let t = feature_set(&["loop-X"]);
        let particle = verdict_rite(&s, &t, true, 0.05);
        assert_eq!(particle.verdict, Verdict::NewShape);
        assert_eq!(particle.epsilon, 0.05);
    }
}
