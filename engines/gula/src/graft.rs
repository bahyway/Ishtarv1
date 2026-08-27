//! GL-LSN-001 §2, §5 — The Graft.
//! One dialect candidate per wound; ancestry from sealed flesh only (§5.2).
//! Productions are cut from the wound's own signature — the autograft
//! principle: same genome, no foreign tissue.

use crate::wound::Wound;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct DialectCandidate {
    /// Deterministic candidate id, e.g. "LSN-CAND-P[x]|O[y]|T[z]".
    pub id: String,
    /// Grammar-sketch productions, AkkadianAOL sub-dialect (never a DSL).
    pub productions: Vec<String>,
    /// Mandatory plain-language gloss at minting time (GL-NAM-002).
    pub hubullu_gloss: String,
    /// Sealed ancestors only (§5.2) — law/theorem ids this graft grows from.
    pub sealed_ancestry: Vec<String>,
    /// The glosses whose collision proved the notation missing.
    pub wound_glosses: Vec<String>,
}

/// Mints at most one candidate per signature key, ever (§5.2 regulatory layer).
#[derive(Debug, Default)]
pub struct GraftMint {
    minted_keys: BTreeSet<String>,
}

impl GraftMint {
    pub fn new() -> Self {
        Self::default()
    }

    /// MOVE phase: cut a graft from a triggered wound.
    /// Returns None if this wound already received its one candidate.
    pub fn mint(&mut self, wound: &Wound) -> Option<DialectCandidate> {
        let key = wound.signature.key();
        if !self.minted_keys.insert(key.clone()) {
            return None; // one graft per wound, no self-similar overgrowth
        }
        let sig = &wound.signature;
        let productions = vec![
            // Sovereign constructs only: ORBIT / EMIT / PROVE / SYNC / WITNESS,
            // PRESENT clause, W5H2, PHYSICS/META predicates.
            format!("dialect      ::= ORBIT {} PRESENT particle-clause", sig.orbit_face),
            format!("particle-clause ::= WITNESS {} EMIT verdict-line", sig.particle_face),
            format!("verdict-line ::= PROVE tribe({}) SYNC naru-journal", sig.tribe_face),
            "naru-journal ::= META(w5h2) PHYSICS(membrane) ;".to_string(),
        ];
        Some(DialectCandidate {
            id: format!("LSN-CAND-{}", key),
            productions,
            hubullu_gloss: format!(
                "A small shared notation for the pattern shaped '{}', which until now \
                 needed {} different explanations across {} engines.",
                key,
                wound.glosses.len(),
                wound.engines.len()
            ),
            sealed_ancestry: vec!["GL-ALG-002".into(), "GL-LSN-001".into()],
            wound_glosses: wound.glosses.iter().cloned().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signature;
    use std::collections::BTreeSet;

    fn triggered_wound() -> Wound {
        let mut g = BTreeSet::new();
        g.insert("a".into());
        g.insert("b".into());
        g.insert("c".into());
        let mut e = BTreeSet::new();
        e.insert("nanshe".into());
        e.insert("karanu".into());
        Wound { signature: Signature::new("p", "o", "t"), glosses: g, engines: e }
    }

    #[test]
    fn one_graft_per_wound() {
        let mut mint = GraftMint::new();
        let w = triggered_wound();
        assert!(mint.mint(&w).is_some());
        assert!(mint.mint(&w).is_none(), "second mint on same wound must refuse (§5.2)");
    }

    #[test]
    fn ancestry_is_sealed_only() {
        let mut mint = GraftMint::new();
        let c = mint.mint(&triggered_wound()).unwrap();
        assert!(c.sealed_ancestry.iter().all(|a| a.starts_with("GL-")));
        assert!(!c.id.is_empty() && c.productions.len() == 4);
    }
}
