//! Nusku sovereign disambiguation rule base.
//!
//! Rule format: IF <antecedent(s)> THEN <consequent> [weight]
//! Antecedents combined with T-norm (min) for AND, S-norm (max) for OR.
//! These rules are the compiled output of AkkadianAOL .akk policies.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

// ── Antecedent ────────────────────────────────────────────────────────────────

/// IF-part of a fuzzy rule.
#[derive(Debug, Clone)]
pub struct Antecedent {
    pub variable: String,
    pub term: String,
    pub negated: bool,
}

impl Antecedent {
    pub fn new(variable: &str, term: &str) -> Self {
        Self {
            variable: variable.into(),
            term: term.into(),
            negated: false,
        }
    }

    pub fn not(variable: &str, term: &str) -> Self {
        Self {
            variable: variable.into(),
            term: term.into(),
            negated: true,
        }
    }
}

// ── Consequent ────────────────────────────────────────────────────────────────

/// THEN-part of a fuzzy rule.
#[derive(Debug, Clone)]
pub struct Consequent {
    pub variable: String,
    pub term: String,
}

impl Consequent {
    pub fn new(variable: &str, term: &str) -> Self {
        Self {
            variable: variable.into(),
            term: term.into(),
        }
    }
}

// ── Connective ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Connective {
    And,
    Or,
}

// ── FuzzyRule ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FuzzyRule {
    pub id: u32,
    pub description: String,
    pub antecedents: Vec<Antecedent>,
    pub connective: Connective,
    pub consequent: Consequent,
    pub weight: f32,
}

impl FuzzyRule {
    pub fn and(id: u32, desc: &str, ants: Vec<Antecedent>, cons: Consequent) -> Self {
        Self {
            id,
            description: desc.into(),
            antecedents: ants,
            connective: Connective::And,
            consequent: cons,
            weight: 1.0,
        }
    }

    pub fn weighted(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
}

// ── Nusku Rule Base ───────────────────────────────────────────────────────────

/// The 10 sovereign disambiguation rules — compiled from AkkadianAOL policies.
pub fn nusku_rule_base() -> Vec<FuzzyRule> {
    vec![
        // Rule 01: Classic security threat
        // IF torso=cold_shadow AND arm=hot_wiring AND inversion=strong
        // THEN state=security_threat [0.95]
        FuzzyRule::and(
            1,
            "Classic cold-shadow + detonator heat pattern",
            vec![
                Antecedent::new("torso_deviation", "cold_shadow"),
                Antecedent::new("arm_deviation", "hot_wiring"),
                Antecedent::new("torso_arm_inversion", "strong"),
            ],
            Consequent::new("state", "security_threat"),
        )
        .weighted(0.95),
        // Rule 02: Security threat with stress
        // IF torso=cold_shadow AND head=stress_elevated
        // THEN state=security_threat [0.88]
        FuzzyRule::and(
            2,
            "Cold-shadow torso + elevated head (stress carrying device)",
            vec![
                Antecedent::new("torso_deviation", "cold_shadow"),
                Antecedent::new("head_deviation", "stress_elevated"),
            ],
            Consequent::new("state", "security_threat"),
        )
        .weighted(0.88),
        // Rule 03: Systemic fever
        // IF torso=hot_fever AND head=fever_elevated AND symmetry=symmetric
        // THEN state=fever [0.92]
        FuzzyRule::and(
            3,
            "Symmetric whole-body elevation — systemic fever",
            vec![
                Antecedent::new("torso_deviation", "hot_fever"),
                Antecedent::new("head_deviation", "fever_elevated"),
                Antecedent::new("symmetry", "symmetric"),
            ],
            Consequent::new("state", "fever"),
        )
        .weighted(0.92),
        // Rule 04: Fever without cold shadow
        // IF torso=warm AND head=fever_elevated AND NOT torso=cold_shadow
        // THEN state=fever [0.85]
        FuzzyRule::and(
            4,
            "Warm torso + elevated head, no cold shadow — fever",
            vec![
                Antecedent::new("torso_deviation", "warm"),
                Antecedent::new("head_deviation", "fever_elevated"),
                Antecedent::not("torso_deviation", "cold_shadow"),
            ],
            Consequent::new("state", "fever"),
        )
        .weighted(0.85),
        // Rule 05: Anxiety / Stress (no threat)
        // IF head=stress_elevated AND arm=cold_hands AND torso=normal
        // THEN state=stress [0.82]
        FuzzyRule::and(
            5,
            "Elevated head + cold hands + normal torso — anxiety response",
            vec![
                Antecedent::new("head_deviation", "stress_elevated"),
                Antecedent::new("arm_deviation", "cold_hands"),
                Antecedent::new("torso_deviation", "normal"),
            ],
            Consequent::new("state", "stress"),
        )
        .weighted(0.82),
        // Rule 06: Adrenaline only (fear but no device)
        // IF head=stress_elevated AND torso=normal AND NOT arm=hot_wiring
        // THEN state=stress [0.75]
        FuzzyRule::and(
            6,
            "Elevated head + normal torso + no hot arms — pure adrenaline",
            vec![
                Antecedent::new("head_deviation", "stress_elevated"),
                Antecedent::new("torso_deviation", "normal"),
                Antecedent::not("arm_deviation", "hot_wiring"),
            ],
            Consequent::new("state", "stress"),
        )
        .weighted(0.75),
        // Rule 07: Ambiguous — cold torso + no hot arms
        // IF torso=cold_shadow AND NOT arm=hot_wiring
        // THEN state=ambiguous [0.70]
        FuzzyRule::and(
            7,
            "Cold torso but no hot arms — clothing or ambiguous",
            vec![
                Antecedent::new("torso_deviation", "cold_shadow"),
                Antecedent::not("arm_deviation", "hot_wiring"),
            ],
            Consequent::new("state", "ambiguous"),
        )
        .weighted(0.70),
        // Rule 08: Clear — all zones normal
        FuzzyRule::and(
            8,
            "All zones within normal range",
            vec![
                Antecedent::new("torso_deviation", "normal"),
                Antecedent::new("head_deviation", "normal"),
                Antecedent::new("arm_deviation", "normal"),
            ],
            Consequent::new("state", "clear"),
        )
        .weighted(0.95),
        // Rule 09: Medical — fever with asymmetry
        // IF head=fever_elevated AND symmetry=asymmetric
        // THEN state=medical_other [0.72]
        FuzzyRule::and(
            9,
            "Fever with asymmetry — localized infection or one-sided condition",
            vec![
                Antecedent::new("head_deviation", "fever_elevated"),
                Antecedent::new("symmetry", "asymmetric"),
            ],
            Consequent::new("state", "medical_other"),
        )
        .weighted(0.72),
        // Rule 10: Iraq high-alert — partial cold shadow (elevated sensitivity)
        // IF torso=slightly_cool AND arm=warm AND inversion=partial
        // THEN state=ambiguous [0.80]
        FuzzyRule::and(
            10,
            "Partial torso cooling + arm warmth — elevated watch (Iraq protocol)",
            vec![
                Antecedent::new("torso_deviation", "slightly_cool"),
                Antecedent::new("arm_deviation", "warm"),
                Antecedent::new("torso_arm_inversion", "partial"),
            ],
            Consequent::new("state", "ambiguous"),
        )
        .weighted(0.80),
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_base_has_ten_rules() {
        assert_eq!(nusku_rule_base().len(), 10);
    }

    #[test]
    fn rule_ids_are_unique_and_sequential() {
        let rules = nusku_rule_base();
        let ids: Vec<u32> = rules.iter().map(|r| r.id).collect();
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id as usize, i + 1, "rule at index {i} has id {id}");
        }
    }

    #[test]
    fn rule_weights_are_valid() {
        for r in nusku_rule_base() {
            assert!(
                r.weight > 0.0 && r.weight <= 1.0,
                "rule {} weight={}",
                r.id,
                r.weight
            );
        }
    }

    #[test]
    fn all_rules_have_descriptions() {
        for r in nusku_rule_base() {
            assert!(
                !r.description.is_empty(),
                "rule {} has empty description",
                r.id
            );
        }
    }

    #[test]
    fn rule_1_is_security_threat_weighted_095() {
        let r = &nusku_rule_base()[0];
        assert_eq!(r.consequent.term, "security_threat");
        assert!((r.weight - 0.95).abs() < 1e-5);
    }

    #[test]
    fn rule_3_is_fever_weighted_092() {
        let r = &nusku_rule_base()[2];
        assert_eq!(r.consequent.term, "fever");
        assert!((r.weight - 0.92).abs() < 1e-5);
    }

    #[test]
    fn all_antecedents_have_non_empty_variable_and_term() {
        for r in nusku_rule_base() {
            for ant in &r.antecedents {
                assert!(!ant.variable.is_empty(), "rule {} ant variable empty", r.id);
                assert!(!ant.term.is_empty(), "rule {} ant term empty", r.id);
            }
        }
    }
}
