//! GL-LSN-001 §4, §5 — The Immune Clause and the Genome Check.
//! Foreign tissue (SQL, embedded foreign DSL directives) is rejected before
//! staging. A graft that fails the triality gate τ + ε ≥ ε(Θ) is a tumor.

use crate::graft::DialectCandidate;
use crate::Verdict;

/// §4.1 — non-sovereign constructs. Matched as whole uppercase words.
const FOREIGN_TISSUE: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "EXISTS", "INSERT", "UPDATE", "DELETE",
    "GROUP BY", "ORDER BY",
];

/// Witnessed inputs to the triality gate. The proxy formula that produces
/// these from wound statistics is documented at the call site (arc.rs) and
/// remains subject to court calibration under the Fadam functional (§6.1).
#[derive(Debug, Clone, Copy)]
pub struct TrialityWitness {
    pub tau: f64,           // transparency deficit carried by the wound
    pub epsilon: f64,       // uncertainty the graft admits
    pub epsilon_theta: f64, // stakeholder-template floor ε(Θ)
}

pub fn triality_holds(t: TrialityWitness) -> bool {
    t.tau + t.epsilon >= t.epsilon_theta
}

/// The judging bench. Order of rites: immune check first, genome second.
pub fn judge(candidate: &DialectCandidate, t: TrialityWitness) -> Verdict {
    // §4 — immune clause
    for prod in &candidate.productions {
        let upper = prod.to_uppercase();
        for kw in FOREIGN_TISSUE {
            if contains_word(&upper, kw) {
                return Verdict::RejectedForeign(format!(
                    "foreign tissue '{}' in production '{}'",
                    kw, prod
                ));
            }
        }
        if upper.contains("DSL") {
            return Verdict::RejectedForeign(
                "candidate designates itself a DSL; AkkadianAOL dialects are not DSLs".into(),
            );
        }
    }
    // §5 — genome check
    if candidate.sealed_ancestry.is_empty()
        || !candidate.sealed_ancestry.iter().any(|a| a == "GL-ALG-002")
    {
        return Verdict::RejectedTumor("graft does not grow from GL-ALG-002 flesh".into());
    }
    if !triality_holds(t) {
        return Verdict::RejectedTumor(format!(
            "triality gate failed: τ({}) + ε({}) < ε(Θ)({})",
            t.tau, t.epsilon, t.epsilon_theta
        ));
    }
    Verdict::Minted
}

/// Whole-word match on already-uppercased haystack (handles "GROUP BY" too).
fn contains_word(hay: &str, needle: &str) -> bool {
    let bytes = hay.as_bytes();
    let mut start = 0;
    while let Some(pos) = hay[start..].find(needle) {
        let i = start + pos;
        let j = i + needle.len();
        let left_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        let right_ok = j >= hay.len() || !bytes[j].is_ascii_alphanumeric();
        if left_ok && right_ok {
            return true;
        }
        start = i + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(productions: Vec<String>, ancestry: Vec<String>) -> DialectCandidate {
        DialectCandidate {
            id: "LSN-CAND-TEST".into(),
            productions,
            hubullu_gloss: "test gloss".into(),
            sealed_ancestry: ancestry,
            wound_glosses: vec!["a".into(), "b".into(), "c".into()],
        }
    }

    fn ok_triality() -> TrialityWitness {
        TrialityWitness { tau: 2.0, epsilon: 0.5, epsilon_theta: 1.0 }
    }

    #[test]
    fn sql_is_foreign_tissue() {
        let c = cand(
            vec!["clause ::= SELECT particle FROM tribe".into()],
            vec!["GL-ALG-002".into()],
        );
        assert_eq!(judge(&c, ok_triality()).as_str(), "REJECTED-FOREIGN");
    }

    #[test]
    fn word_boundary_does_not_false_positive() {
        // "WHEREABOUTS" must not match WHERE; "FROMAGE" must not match FROM.
        let c = cand(
            vec!["clause ::= WITNESS whereabouts EMIT fromage".into()],
            vec!["GL-ALG-002".into()],
        );
        assert_eq!(judge(&c, ok_triality()), Verdict::Minted);
    }

    #[test]
    fn tumor_without_algebra_ancestry() {
        let c = cand(vec!["clause ::= ORBIT x".into()], vec![]);
        assert_eq!(judge(&c, ok_triality()).as_str(), "REJECTED-TUMOR");
    }

    #[test]
    fn triality_gate_rejects() {
        let c = cand(vec!["clause ::= ORBIT x".into()], vec!["GL-ALG-002".into()]);
        let bad = TrialityWitness { tau: 0.1, epsilon: 0.1, epsilon_theta: 1.0 };
        assert_eq!(judge(&c, bad).as_str(), "REJECTED-TUMOR");
    }

    #[test]
    fn sovereign_candidate_is_minted() {
        let c = cand(
            vec!["dialect ::= ORBIT o PRESENT p WITNESS t".into()],
            vec!["GL-ALG-002".into(), "GL-LSN-001".into()],
        );
        assert_eq!(judge(&c, ok_triality()), Verdict::Minted);
    }
}
