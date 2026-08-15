//! Deterministic Rule Engine — boolean constraint evaluation for Validity scoring.
//!
//! Rules are typed closures that accept a record's attribute map and return
//! `RuleVerdict`.  All rules fire; violations are accumulated.  The Validity
//! score = passing_rules / total_rules.
//!
//! Built-in rules cover the most common enterprise validation patterns.
//! Custom rules are registered via `RuleEngine::add_rule()`.

/// Outcome of a single rule evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleVerdict {
    /// Record passes this rule.
    Pass,
    /// Record violates this rule.
    Fail { reason: &'static str },
    /// Rule was not applicable (field absent — not a violation).
    NotApplicable,
}

impl RuleVerdict {
    pub fn is_pass(&self) -> bool { matches!(self, RuleVerdict::Pass) }
    pub fn is_fail(&self) -> bool { matches!(self, RuleVerdict::Fail { .. }) }
}

/// A named validation rule.
pub struct Rule {
    pub name: &'static str,
    /// Closure: (attr_hash, value_bytes) pairs → RuleVerdict
    check: Box<dyn Fn(&[(u32, Vec<u8>)]) -> RuleVerdict + Send + Sync>,
}

impl Rule {
    pub fn new(
        name: &'static str,
        check: impl Fn(&[(u32, Vec<u8>)]) -> RuleVerdict + Send + Sync + 'static,
    ) -> Self {
        Rule { name, check: Box::new(check) }
    }

    pub fn evaluate(&self, record: &[(u32, Vec<u8>)]) -> RuleVerdict {
        (self.check)(record)
    }
}

/// Result of evaluating all rules against one record.
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub passed:  usize,
    pub failed:  usize,
    pub skipped: usize,
    pub violations: Vec<(&'static str, &'static str)>,  // (rule_name, reason)
}

impl RuleResult {
    /// Validity score = passing / (passing + failing).  Skipped rules are neutral.
    pub fn validity_score(&self) -> f32 {
        let total = self.passed + self.failed;
        if total == 0 { return 1.0; }
        self.passed as f32 / total as f32
    }
}

/// The deterministic rule engine.
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> Self { RuleEngine { rules: Vec::new() } }

    /// Register a custom rule.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Evaluate all rules against a record; returns aggregated result.
    pub fn evaluate(&self, record: &[(u32, Vec<u8>)]) -> RuleResult {
        let mut passed  = 0;
        let mut failed  = 0;
        let mut skipped = 0;
        let mut violations = Vec::new();

        for rule in &self.rules {
            match rule.evaluate(record) {
                RuleVerdict::Pass                => { passed += 1; }
                RuleVerdict::Fail { reason }    => {
                    failed += 1;
                    violations.push((rule.name, reason));
                }
                RuleVerdict::NotApplicable       => { skipped += 1; }
            }
        }
        RuleResult { passed, failed, skipped, violations }
    }

    pub fn rule_count(&self) -> usize { self.rules.len() }
}

impl Default for RuleEngine { fn default() -> Self { Self::new() } }

// ── Built-in Rule Constructors ────────────────────────────────────────────────

/// Rule: field `attr_hash` must be present and non-empty.
pub fn rule_not_empty(name: &'static str, attr_hash: u32) -> Rule {
    Rule::new(name, move |record| {
        match record.iter().find(|(h, _)| *h == attr_hash) {
            None               => RuleVerdict::Fail { reason: "field absent" },
            Some((_, v)) if v.is_empty() => RuleVerdict::Fail { reason: "field empty" },
            _                  => RuleVerdict::Pass,
        }
    })
}

/// Rule: UTF-8 text field must contain the character `ch`.
pub fn rule_contains_char(name: &'static str, attr_hash: u32, ch: char) -> Rule {
    Rule::new(name, move |record| {
        match record.iter().find(|(h, _)| *h == attr_hash) {
            None               => RuleVerdict::NotApplicable,
            Some((_, v)) => {
                let text = std::str::from_utf8(v).unwrap_or("");
                if text.contains(ch) { RuleVerdict::Pass }
                else { RuleVerdict::Fail { reason: "required character absent" } }
            }
        }
    })
}

/// Rule: numeric field (stored as ASCII decimal) must be within [min, max].
pub fn rule_numeric_range(name: &'static str, attr_hash: u32, min: f64, max: f64) -> Rule {
    Rule::new(name, move |record| {
        match record.iter().find(|(h, _)| *h == attr_hash) {
            None => RuleVerdict::NotApplicable,
            Some((_, v)) => {
                let text = std::str::from_utf8(v).unwrap_or("");
                match text.trim().parse::<f64>() {
                    Err(_)                       => RuleVerdict::Fail { reason: "not a number" },
                    Ok(n) if n < min || n > max => RuleVerdict::Fail { reason: "out of range" },
                    Ok(_)                        => RuleVerdict::Pass,
                }
            }
        }
    })
}

/// Rule: text field length must be at least `min_len` bytes.
pub fn rule_min_length(name: &'static str, attr_hash: u32, min_len: usize) -> Rule {
    Rule::new(name, move |record| {
        match record.iter().find(|(h, _)| *h == attr_hash) {
            None           => RuleVerdict::NotApplicable,
            Some((_, v)) => {
                if v.len() >= min_len { RuleVerdict::Pass }
                else { RuleVerdict::Fail { reason: "value too short" } }
            }
        }
    })
}

/// Rule: text field must be all ASCII digits.
pub fn rule_digits_only(name: &'static str, attr_hash: u32) -> Rule {
    Rule::new(name, move |record| {
        match record.iter().find(|(h, _)| *h == attr_hash) {
            None           => RuleVerdict::NotApplicable,
            Some((_, v)) => {
                if v.iter().all(|b| b.is_ascii_digit()) { RuleVerdict::Pass }
                else { RuleVerdict::Fail { reason: "non-digit characters present" } }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(pairs: &[(u32, &[u8])]) -> Vec<(u32, Vec<u8>)> {
        pairs.iter().map(|(h, v)| (*h, v.to_vec())).collect()
    }

    #[test]
    fn not_empty_passes_for_populated_field() {
        let rule = rule_not_empty("id_present", 0x01);
        let r = rec(&[(0x01, b"42")]);
        assert!(rule.evaluate(&r).is_pass());
    }

    #[test]
    fn not_empty_fails_for_empty_field() {
        let rule = rule_not_empty("id_present", 0x01);
        let r = rec(&[(0x01, b"")]);
        assert!(rule.evaluate(&r).is_fail());
    }

    #[test]
    fn not_empty_fails_when_field_absent() {
        let rule = rule_not_empty("id_present", 0x01);
        let r = rec(&[(0x02, b"other")]);
        assert!(rule.evaluate(&r).is_fail());
    }

    #[test]
    fn contains_char_passes_for_at_sign_in_email() {
        let rule = rule_contains_char("email_at", 0x10, '@');
        let r = rec(&[(0x10, b"user@example.com")]);
        assert!(rule.evaluate(&r).is_pass());
    }

    #[test]
    fn contains_char_fails_when_missing() {
        let rule = rule_contains_char("email_at", 0x10, '@');
        let r = rec(&[(0x10, b"not_an_email")]);
        assert!(rule.evaluate(&r).is_fail());
    }

    #[test]
    fn numeric_range_passes_within_bounds() {
        let rule = rule_numeric_range("age_range", 0x20, 0.0, 150.0);
        let r = rec(&[(0x20, b"42")]);
        assert!(rule.evaluate(&r).is_pass());
    }

    #[test]
    fn numeric_range_fails_above_max() {
        let rule = rule_numeric_range("age_range", 0x20, 0.0, 150.0);
        let r = rec(&[(0x20, b"200")]);
        assert!(rule.evaluate(&r).is_fail());
    }

    #[test]
    fn digits_only_passes_for_numeric_id() {
        let rule = rule_digits_only("phone_digits", 0x30);
        let r = rec(&[(0x30, b"07901234567")]);
        assert!(rule.evaluate(&r).is_pass());
    }

    #[test]
    fn digits_only_fails_with_letters() {
        let rule = rule_digits_only("phone_digits", 0x30);
        let r = rec(&[(0x30, b"079-ABC")]);
        assert!(rule.evaluate(&r).is_fail());
    }

    #[test]
    fn engine_accumulates_results() {
        let mut engine = RuleEngine::new();
        engine.add_rule(rule_not_empty("f1", 0x01));
        engine.add_rule(rule_contains_char("email_at", 0x02, '@'));
        engine.add_rule(rule_numeric_range("age", 0x03, 0.0, 120.0));

        let r = rec(&[
            (0x01, b"present"),
            (0x02, b"bad_email"),      // fails
            (0x03, b"25"),
        ]);
        let result = engine.evaluate(&r);
        assert_eq!(result.passed, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.violations[0].0, "email_at");
    }

    #[test]
    fn validity_score_all_pass_is_one() {
        let mut engine = RuleEngine::new();
        engine.add_rule(rule_not_empty("f1", 0x01));
        engine.add_rule(rule_not_empty("f2", 0x02));
        let r = rec(&[(0x01, b"a"), (0x02, b"b")]);
        let result = engine.evaluate(&r);
        assert!((result.validity_score() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn validity_score_no_rules_is_one() {
        let engine = RuleEngine::new();
        let r = rec(&[(0x01, b"x")]);
        assert!((engine.evaluate(&r).validity_score() - 1.0).abs() < f32::EPSILON);
    }
}
