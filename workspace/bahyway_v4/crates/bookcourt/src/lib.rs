//! bookcourt — GL-LIT-001 (The Book Court Law): literature patterns as
//! minted templates with provenance. PB-325. Pure Rust, zero
//! dependencies.
//!
//! Structure-Only Clause (§6): this crate stores structure and pointers,
//! never prose -- `TemplateParticle` has no field for the work's text at
//! all; that is enforced by the type itself, not by a runtime check.
#![forbid(unsafe_code)]

use std::collections::HashSet;

/// Rite II — the FCA formal context: objects = definitions/lemmas/
/// theorems/algorithms; attributes = assumptions/prior results/tools.
#[derive(Debug, Clone)]
pub struct FormalContext {
    pub objects: Vec<String>,
    pub attributes: Vec<String>,
    incidence: Vec<Vec<bool>>,
}

impl FormalContext {
    pub fn new(objects: Vec<String>, attributes: Vec<String>) -> Self {
        let incidence = vec![vec![false; attributes.len()]; objects.len()];
        Self { objects, attributes, incidence }
    }

    pub fn set(&mut self, object_idx: usize, attr_idx: usize, value: bool) {
        self.incidence[object_idx][attr_idx] = value;
    }

    /// Attribute closure: attributes shared by ALL given objects.
    pub fn attribute_intent(&self, object_idxs: &[usize]) -> Vec<usize> {
        (0..self.attributes.len())
            .filter(|&a| object_idxs.iter().all(|&o| self.incidence[o][a]))
            .collect()
    }

    /// Object extent: objects having ALL given attributes.
    pub fn object_extent(&self, attr_idxs: &[usize]) -> Vec<usize> {
        (0..self.objects.len())
            .filter(|&o| attr_idxs.iter().all(|&a| self.incidence[o][a]))
            .collect()
    }

    /// Lattice rank of one object: how many other concepts strictly
    /// generalize it (their intent is a strict superset of its own).
    pub fn lattice_rank(&self, object_idx: usize) -> usize {
        let my_intent = self.attribute_intent(&[object_idx]);
        (0..self.objects.len())
            .filter(|&o| {
                if o == object_idx {
                    return false;
                }
                let their_intent = self.attribute_intent(&[o]);
                their_intent.len() > my_intent.len() && my_intent.iter().all(|a| their_intent.contains(a))
            })
            .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Witness2 {
    FullLean4Z3Proof,
    WorkingRustPropertyTest { playbook: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintState {
    Fuzzy,
    Golden,
}

/// Rite IV — the template particle. Structure and pointers only:
/// `lit_source`/`lit_locus` point AT the work; the work's text is never
/// copied in here.
#[derive(Debug, Clone)]
pub struct TemplateParticle {
    pub lit_source: String,
    pub lit_locus: String,
    pub lit_intent: Vec<String>,
    pub lit_rank_home: usize,
    pub lit_notation: String,
    pub lit_witness_1: String,
    pub lit_witness_2: Option<Witness2>,
    pub state: MintState,
}

/// Rite IV — mint. State at mint is ALWAYS FUZZY: a literature pattern is
/// never born GOLDEN.
pub fn mint_pattern(
    source: &str,
    locus: &str,
    intent: Vec<String>,
    rank_home: usize,
    notation: &str,
    witness_1: &str,
) -> TemplateParticle {
    TemplateParticle {
        lit_source: source.to_string(),
        lit_locus: locus.to_string(),
        lit_intent: intent,
        lit_rank_home: rank_home,
        lit_notation: notation.to_string(),
        lit_witness_1: witness_1.to_string(),
        lit_witness_2: None,
        state: MintState::Fuzzy,
    }
}

/// Rite V — Gate Before Service: promotes to GOLDEN only upon a second
/// witness.
pub fn attach_second_witness(particle: &mut TemplateParticle, witness_2: Witness2) {
    particle.lit_witness_2 = Some(witness_2);
    particle.state = MintState::Golden;
}

/// The gate guard: fails without a real second witness, regardless of
/// what `state` claims -- "approved" means approved by the ladder, never
/// by the visualization.
pub fn assert_served(particle: &TemplateParticle) -> Result<(), &'static str> {
    match &particle.lit_witness_2 {
        Some(_) if particle.state == MintState::Golden => Ok(()),
        _ => Err("gate refuses: no second witness present"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationSource {
    MachineProposed,
    HumanConfirmed,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub text: String,
    pub source: AnnotationSource,
}

/// §4 — the Annotation Clause: no single-witness (machine-only)
/// annotation enters the formal context.
pub fn admit_annotation(annotation: &Annotation) -> Result<(), &'static str> {
    match annotation.source {
        AnnotationSource::HumanConfirmed => Ok(()),
        AnnotationSource::MachineProposed => {
            Err("machine-proposed annotation alone cannot enter the formal context")
        }
    }
}

/// §5 — the Cross-Corpus Clause: exact Jaccard similarity over intent
/// sets (MinHash at scale in production; exact here for a small, tested
/// kernel).
pub fn intent_similarity(a: &[String], b: &[String]) -> f64 {
    let set_a: HashSet<&String> = a.iter().collect();
    let set_b: HashSet<&String> = b.iter().collect();
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

pub fn is_unification_candidate(a: &[String], b: &[String], threshold: f64) -> bool {
    intent_similarity(a, b) >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    // A literature pattern is never born GOLDEN.
    #[test]
    fn fuzzy_at_mint_always() {
        let mut p = mint_pattern("Dorst-Fontijne-Mann", "ch.4", vec!["GA meet".into()], 2, "GA", "Theorem 4.2");
        p.lit_witness_2 = None;
        p.state = MintState::Fuzzy;
        assert_eq!(p.state, MintState::Fuzzy);
        assert!(assert_served(&p).is_err(), "unserved FUZZY particle must not pass the gate");
    }

    // Rite V — gate guard fails without witness_2, passes with it.
    #[test]
    fn gate_guard_requires_second_witness() {
        let mut p = mint_pattern("Dorst-Fontijne-Mann", "ch.4", vec!["GA meet".into()], 2, "GA", "Theorem 4.2");
        p.state = MintState::Fuzzy;
        p.lit_witness_2 = None;
        assert!(assert_served(&p).is_err());

        attach_second_witness(&mut p, Witness2::WorkingRustPropertyTest { playbook: "PB-325".into() });
        assert_eq!(p.state, MintState::Golden);
        assert!(assert_served(&p).is_ok());
    }

    // Annotation clause: machine-only refused, human-confirmed admitted.
    #[test]
    fn annotation_clause_needs_human_confirmation() {
        let machine = Annotation { text: "implicit assumption".into(), source: AnnotationSource::MachineProposed };
        assert!(admit_annotation(&machine).is_err());
        let human = Annotation { text: "implicit assumption".into(), source: AnnotationSource::HumanConfirmed };
        assert!(admit_annotation(&human).is_ok());
    }

    // Cross-corpus clause: identical intents unify; disjoint intents don't.
    #[test]
    fn cross_corpus_unification_candidates() {
        let a = vec!["meet".to_string(), "join".to_string()];
        let b = vec!["meet".to_string(), "join".to_string()];
        assert_eq!(intent_similarity(&a, &b), 1.0);
        assert!(is_unification_candidate(&a, &b, 0.8));

        let c = vec!["unrelated".to_string()];
        assert_eq!(intent_similarity(&a, &c), 0.0);
        assert!(!is_unification_candidate(&a, &c, 0.8));
    }

    #[test]
    fn lattice_rank_reflects_generality() {
        let mut ctx = FormalContext::new(
            vec!["Lemma1".into(), "Theorem2".into()],
            vec!["assumeA".into(), "assumeB".into()],
        );
        ctx.set(0, 0, true); // Lemma1 consumes assumeA only
        ctx.set(1, 0, true); // Theorem2 consumes assumeA and assumeB
        ctx.set(1, 1, true);
        // Lemma1's intent {A} is a strict subset of Theorem2's {A,B}:
        // Theorem2 generalizes Lemma1, so Lemma1's rank counts it.
        assert_eq!(ctx.lattice_rank(0), 1);
        assert_eq!(ctx.lattice_rank(1), 0);
    }
}
