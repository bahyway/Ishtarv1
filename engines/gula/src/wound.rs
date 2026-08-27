//! GL-LSN-001 §1 — The Wound Trigger.
//! Three distinct Ḫubullu glosses on one structural signature = a wound.
//! Two independent engine witnesses required before the wound may TRIGGER.

use crate::{PatternWitness, Signature, Verdict};
use std::collections::{BTreeMap, BTreeSet};

pub const GLOSS_TRIGGER: usize = 3; // §1.2 — three-gloss rule
pub const WITNESS_TRIGGER: usize = 2; // §1.3 — two-witness rule

#[derive(Debug, Clone)]
pub struct Wound {
    pub signature: Signature,
    pub glosses: BTreeSet<String>,
    pub engines: BTreeSet<String>,
}

impl Wound {
    pub fn status(&self) -> Verdict {
        if self.glosses.len() >= GLOSS_TRIGGER && self.engines.len() >= WITNESS_TRIGGER {
            Verdict::Triggered
        } else {
            Verdict::Observed
        }
    }
}

/// Append-only registry of the corpus's wounds.
#[derive(Debug, Default)]
pub struct WoundRegistry {
    wounds: BTreeMap<String, Wound>,
}

impl WoundRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// WITNESS phase: ingest one testimony.
    pub fn witness(&mut self, w: &PatternWitness) -> Verdict {
        let key = w.signature.key();
        let entry = self.wounds.entry(key).or_insert_with(|| Wound {
            signature: w.signature.clone(),
            glosses: BTreeSet::new(),
            engines: BTreeSet::new(),
        });
        entry.glosses.insert(w.hubullu_gloss.trim().to_lowercase());
        entry.engines.insert(w.engine.trim().to_lowercase());
        entry.status()
    }

    /// JUDGE phase: all wounds currently past the trigger.
    pub fn triggered(&self) -> Vec<&Wound> {
        self.wounds
            .values()
            .filter(|w| w.status() == Verdict::Triggered)
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Wound> {
        self.wounds.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(engine: &str, gloss: &str) -> PatternWitness {
        PatternWitness {
            engine: engine.into(),
            signature: Signature::new("state-change", "limit-cycle", "golden-tribe"),
            hubullu_gloss: gloss.into(),
        }
    }

    #[test]
    fn three_glosses_one_engine_stays_observed() {
        let mut reg = WoundRegistry::new();
        reg.witness(&w("nanshe", "a ripening"));
        reg.witness(&w("nanshe", "a maturation"));
        let v = reg.witness(&w("nanshe", "a settling"));
        assert_eq!(v, Verdict::Observed, "one witness must never trigger (§1.3)");
    }

    #[test]
    fn two_glosses_two_engines_stays_observed() {
        let mut reg = WoundRegistry::new();
        reg.witness(&w("nanshe", "a ripening"));
        let v = reg.witness(&w("karanu", "a maturation"));
        assert_eq!(v, Verdict::Observed, "two glosses is not yet a wound (§1.2)");
    }

    #[test]
    fn three_glosses_two_engines_triggers() {
        let mut reg = WoundRegistry::new();
        reg.witness(&w("nanshe", "a ripening"));
        reg.witness(&w("karanu", "a maturation"));
        let v = reg.witness(&w("nanshe", "a settling"));
        assert_eq!(v, Verdict::Triggered);
        assert_eq!(reg.triggered().len(), 1);
    }
}
