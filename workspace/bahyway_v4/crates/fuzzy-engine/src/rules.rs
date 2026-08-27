// ============================================================================
// fuzzy-engine/src/rules.rs
// Sovereign fuzzy rule set — 5 default rules SR-01 through SR-05
// Mamdani: IF antecedent THEN consequent. MAX aggregation.
// ============================================================================
#![allow(missing_docs)]

use crate::dimensions::FuzzifiedDimensions;

/// Output quality tiers (consequents).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier {
    /// B11 < 100 — Dead lane.
    NonActive,
    /// B11 100–139 — Active.
    Active,
    /// B11 140–199 — Tribe.
    Tribe,
    /// B11 >= 200 — Golden Record.
    Gem,
}

/// Fired rule output: which tier and with what strength.
#[derive(Debug, Clone)]
pub struct RuleFiring {
    pub tier: QualityTier,
    pub strength: f32,
}

/// Aggregated output memberships after MAX across all rules.
#[derive(Debug, Clone, Default)]
pub struct AggregatedOutput {
    pub non_active: f32,
    pub active: f32,
    pub tribe: f32,
    pub gem: f32,
}

impl AggregatedOutput {
    /// Apply a rule firing via MAX aggregation.
    pub fn apply(&mut self, firing: RuleFiring) {
        match firing.tier {
            QualityTier::NonActive => self.non_active = self.non_active.max(firing.strength),
            QualityTier::Active => self.active = self.active.max(firing.strength),
            QualityTier::Tribe => self.tribe = self.tribe.max(firing.strength),
            QualityTier::Gem => self.gem = self.gem.max(firing.strength),
        }
    }
}

/// Evaluate all 5 sovereign rules against fuzzified dimensions.
pub fn evaluate_rules(f: &FuzzifiedDimensions) -> AggregatedOutput {
    let mut out = AggregatedOutput::default();

    // SR-01: Perfect particle → Gem
    // IF d1 HIGH AND d3 HIGH AND d6 HIGH AND d7 LOW (no duplicates)
    let sr01 = f.d1.2.min(f.d3.2).min(f.d6.2).min(f.d7.0);
    out.apply(RuleFiring {
        tier: QualityTier::Gem,
        strength: sr01 * 1.0,
    });

    // SR-02: Good particle → Tribe
    // IF d1 HIGH AND d2 MEDIUM OR HIGH AND d6 MEDIUM OR HIGH AND d7 LOW
    let d2_ok = f.d2.1.max(f.d2.2);
    let d6_ok = f.d6.1.max(f.d6.2);
    let sr02 = f.d1.2.min(d2_ok).min(d6_ok).min(f.d7.0);
    out.apply(RuleFiring {
        tier: QualityTier::Tribe,
        strength: sr02 * 0.8,
    });

    // SR-03: File integrity failure → NonActive
    // IF d3 LOW (ID invalid) AND d4 LOW (date invalid)
    let sr03 = f.d3.0.min(f.d4.0);
    out.apply(RuleFiring {
        tier: QualityTier::NonActive,
        strength: sr03 * 1.0,
    });

    // SR-04: High duplication → Active
    // IF d7 HIGH (near-duplicate)
    let sr04 = f.d7.2;
    out.apply(RuleFiring {
        tier: QualityTier::Active,
        strength: sr04 * 0.7,
    });

    // SR-05: Incomplete data → NonActive
    // IF d1 LOW AND d8 LOW (no Arabic content = likely wrong record)
    let sr05 = f.d1.0.min(f.d8.0);
    out.apply(RuleFiring {
        tier: QualityTier::NonActive,
        strength: sr05 * 0.9,
    });

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dimensions::{FuzzyDimensions, SourceTrust};

    fn fuzzify(d: &FuzzyDimensions) -> FuzzifiedDimensions {
        d.fuzzify()
    }

    #[test]
    fn perfect_record_fires_gem_strongly() {
        let f = fuzzify(&FuzzyDimensions::perfect());
        let out = evaluate_rules(&f);
        assert!(
            out.gem > 0.8,
            "perfect record must fire Gem > 0.8, got {}",
            out.gem
        );
    }

    #[test]
    fn near_duplicate_fires_active() {
        let mut d = FuzzyDimensions::perfect();
        d.d7_duplicate_proximity = 0.95;
        let f = fuzzify(&d);
        let out = evaluate_rules(&f);
        assert!(
            out.active > 0.5,
            "near-duplicate must fire Active > 0.5, got {}",
            out.active
        );
    }

    #[test]
    fn invalid_id_and_date_fires_non_active() {
        let mut d = FuzzyDimensions::perfect();
        d.d3_id_validity = 0.0;
        d.d4_date_validity = 0.0;
        let f = fuzzify(&d);
        let out = evaluate_rules(&f);
        assert!(
            out.non_active > 0.5,
            "invalid id+date must fire NonActive, got {}",
            out.non_active
        );
    }

    #[test]
    fn empty_record_fires_non_active() {
        let mut d = FuzzyDimensions::perfect();
        d.d1_name_completeness = 0.0;
        d.d8_arabic_density = 0.0;
        let f = fuzzify(&d);
        let out = evaluate_rules(&f);
        assert!(
            out.non_active > 0.5,
            "empty record must fire NonActive, got {}",
            out.non_active
        );
    }

    #[test]
    fn good_but_not_perfect_fires_tribe() {
        let mut d = FuzzyDimensions::perfect();
        d.d6_source_trust = SourceTrust::Official;
        d.d2_name_consistency = 0.72;
        let f = fuzzify(&d);
        let out = evaluate_rules(&f);
        assert!(
            out.tribe > 0.1 || out.gem > 0.1,
            "good record must fire Tribe or Gem"
        );
    }

    #[test]
    fn aggregated_output_max_aggregation() {
        let mut agg = AggregatedOutput::default();
        agg.apply(RuleFiring {
            tier: QualityTier::Gem,
            strength: 0.4,
        });
        agg.apply(RuleFiring {
            tier: QualityTier::Gem,
            strength: 0.9,
        });
        agg.apply(RuleFiring {
            tier: QualityTier::Gem,
            strength: 0.6,
        });
        assert!((agg.gem - 0.9).abs() < 0.001, "MAX should keep 0.9");
    }
}
