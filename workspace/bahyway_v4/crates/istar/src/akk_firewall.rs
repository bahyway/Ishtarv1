//! AkkFirewall — Sovereign access control engine.
//! Rules compiled from AkkadianAOL to Rust closures.
//! 𒀭𒄑𒆳 *Ištar* — "sovereign gate / access arbiter".

use serde::{Deserialize, Serialize};

// ── Verdict ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallVerdict {
    Allow,
    Deny,
    Escalate,
    Redact,
    Audit,
}

impl std::fmt::Display for FirewallVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow    => write!(f, "ALLOW"),
            Self::Deny     => write!(f, "DENY"),
            Self::Escalate => write!(f, "ESCALATE"),
            Self::Redact   => write!(f, "REDACT"),
            Self::Audit    => write!(f, "AUDIT"),
        }
    }
}

// ── Subject / Context ─────────────────────────────────────────────────────

/// Sovereign subject of an access request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkSubject {
    pub id:         String,
    pub domain:     u8,
    pub quality:    u8,     // KAKI B11 (0–240)
    pub tribe:      Option<String>,
    pub clearance:  u8,     // 0x00 = none … 0xFF = sovereign
}

/// Resource being accessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkResource {
    pub path:       String,
    pub domain:     u8,
    pub sensitivity: u8,    // 0 = public … 0xFF = sealed
}

/// Full access-check context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkContext {
    pub subject:   AkkSubject,
    pub resource:  AkkResource,
    pub operation: AkkOperation,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AkkOperation {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
}

// ── Rule ──────────────────────────────────────────────────────────────────

/// A compiled firewall rule.
pub struct AkkRule {
    pub name:    String,
    pub priority: u8,
    pub eval:    Box<dyn Fn(&AkkContext) -> Option<FirewallVerdict> + Send + Sync>,
}

impl std::fmt::Debug for AkkRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AkkRule")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .finish()
    }
}

impl AkkRule {
    pub fn new(
        name:     impl Into<String>,
        priority: u8,
        eval:     impl Fn(&AkkContext) -> Option<FirewallVerdict> + Send + Sync + 'static,
    ) -> Self {
        Self { name: name.into(), priority, eval: Box::new(eval) }
    }
}

// ── Five Meta Rules ────────────────────────────────────────────────────────
// Sovereign constitutional rules that cannot be overridden.

pub fn meta_rules() -> Vec<AkkRule> {
    vec![
        // Meta-1: Dead quality blocked at gate.
        AkkRule::new("META-1:DeadQualityDeny", 255, |ctx| {
            if ctx.subject.quality < 59 {
                Some(FirewallVerdict::Deny)
            } else {
                None
            }
        }),
        // Meta-2: Cross-domain writes require sovereign clearance.
        AkkRule::new("META-2:CrossDomainWrite", 254, |ctx| {
            if ctx.subject.domain != ctx.resource.domain
                && ctx.operation == AkkOperation::Write
                && ctx.subject.clearance < 0xA0
            {
                Some(FirewallVerdict::Deny)
            } else {
                None
            }
        }),
        // Meta-3: Admin operations require gem-quality subject (q≥200).
        AkkRule::new("META-3:AdminGemOnly", 253, |ctx| {
            if ctx.operation == AkkOperation::Admin && ctx.subject.quality < 200 {
                Some(FirewallVerdict::Deny)
            } else {
                None
            }
        }),
        // Meta-4: Sensitivity > clearance → Escalate.
        AkkRule::new("META-4:ClearanceEscalate", 252, |ctx| {
            if ctx.resource.sensitivity > ctx.subject.clearance {
                Some(FirewallVerdict::Escalate)
            } else {
                None
            }
        }),
        // Meta-5: All audit-sensitive operations (Delete/Admin) are always audited.
        AkkRule::new("META-5:AuditDestructive", 1, |ctx| {
            if matches!(ctx.operation, AkkOperation::Delete | AkkOperation::Admin) {
                Some(FirewallVerdict::Audit)
            } else {
                None
            }
        }),
    ]
}

// ── Firewall Engine ────────────────────────────────────────────────────────

/// Sovereign firewall — evaluates rules in priority order (highest first).
pub struct AkkFirewall {
    rules: Vec<AkkRule>,
}

impl AkkFirewall {
    pub fn new() -> Self {
        let mut fw = Self { rules: meta_rules() };
        fw.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        fw
    }

    pub fn add_rule(&mut self, rule: AkkRule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluate context. First non-None verdict wins (by priority).
    /// Falls back to `Allow` if no rule matches.
    pub fn evaluate(&self, ctx: &AkkContext) -> FirewallVerdict {
        for rule in &self.rules {
            if let Some(v) = (rule.eval)(ctx) {
                tracing::debug!(rule = %rule.name, verdict = %v, "firewall verdict");
                // Audit is informational — continue to find a real verdict.
                if v != FirewallVerdict::Audit {
                    return v;
                }
            }
        }
        FirewallVerdict::Allow
    }

    pub fn rule_count(&self) -> usize { self.rules.len() }
}

impl Default for AkkFirewall { fn default() -> Self { Self::new() } }

impl std::fmt::Debug for AkkFirewall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AkkFirewall")
            .field("rules", &self.rules.len())
            .finish()
    }
}

// ── QualityLane gate builders ─────────────────────────────────────────────
// Sovereign lane names (no "Way" suffix per ADR-002).

pub mod gates {
    use super::{AkkFirewall, AkkRule, FirewallVerdict};

    /// Zero trust: deny unless quality ≥ threshold.
    pub fn zero_trust_gate(threshold: u8) -> AkkRule {
        AkkRule::new(
            format!("Zero:QualityGate({threshold})"),
            200,
            move |ctx| {
                if ctx.subject.quality < threshold {
                    Some(FirewallVerdict::Deny)
                } else {
                    None
                }
            },
        )
    }

    /// Sho (show) gate: redact if quality below threshold.
    pub fn sho_redact_gate(threshold: u8) -> AkkRule {
        AkkRule::new(
            format!("Sho:RedactGate({threshold})"),
            150,
            move |ctx| {
                if ctx.subject.quality < threshold {
                    Some(FirewallVerdict::Redact)
                } else {
                    None
                }
            },
        )
    }

    /// Build a standard firewall with Zero + Sho gates.
    pub fn standard_firewall(zero_threshold: u8, sho_threshold: u8) -> AkkFirewall {
        let mut fw = AkkFirewall::new();
        fw.add_rule(zero_trust_gate(zero_threshold));
        fw.add_rule(sho_redact_gate(sho_threshold));
        fw
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────
//
// This module (the whole istar access-control engine — 5 constitutional
// meta rules plus the QualityLane gate builders) had zero test coverage
// before playbook_251, despite being istar's actual security logic. istar
// is also currently orphaned (no binary in the workspace consumes it yet),
// so unlike hepta-sec-* or ea-agent-chat there's no live service to hit —
// these are the first real correctness tests this engine has ever had.

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(
        quality: u8,
        subject_domain: u8,
        resource_domain: u8,
        clearance: u8,
        sensitivity: u8,
        op: AkkOperation,
    ) -> AkkContext {
        AkkContext {
            subject: AkkSubject {
                id: "test-subject".into(),
                domain: subject_domain,
                quality,
                tribe: None,
                clearance,
            },
            resource: AkkResource {
                path: "test/resource".into(),
                domain: resource_domain,
                sensitivity,
            },
            operation: op,
            timestamp: 0,
        }
    }

    #[test]
    fn default_allow_when_nothing_matches() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn meta1_dead_quality_denied() {
        let fw = AkkFirewall::new();
        let c = ctx(10, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn meta1_boundary_quality_59_is_allowed() {
        // Rule fires only for quality < 59.
        let fw = AkkFirewall::new();
        let c = ctx(59, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn meta2_cross_domain_write_without_clearance_denied() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 2, 0x50, 0, AkkOperation::Write);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn meta2_cross_domain_write_with_sovereign_clearance_allowed() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 2, 0xA0, 0, AkkOperation::Write);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn meta2_same_domain_write_without_clearance_allowed() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 1, 0x00, 0, AkkOperation::Write);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn meta3_admin_below_gem_quality_denied() {
        let fw = AkkFirewall::new();
        let c = ctx(199, 1, 1, 0xFF, 0, AkkOperation::Admin);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn meta3_admin_at_gem_quality_falls_through_to_audit_then_allow() {
        let fw = AkkFirewall::new();
        let c = ctx(200, 1, 1, 0xFF, 0, AkkOperation::Admin);
        // Meta-3 doesn't deny at q=200; Meta-5 fires Audit (informational,
        // doesn't block) so evaluate() continues past it to Allow.
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn meta4_sensitivity_exceeds_clearance_escalates() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 1, 0x10, 0x50, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Escalate);
    }

    #[test]
    fn meta5_delete_is_audited_but_allowed_absent_other_denials() {
        let fw = AkkFirewall::new();
        let c = ctx(240, 1, 1, 0xFF, 0, AkkOperation::Delete);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn priority_order_meta1_outranks_meta4() {
        // Dead quality (Meta-1, priority 255) AND sensitivity > clearance
        // (Meta-4, priority 252) both apply -- Meta-1 must win since it's
        // checked first in priority order, proving evaluate() actually
        // respects the sorted rule list rather than rule-definition order.
        let fw = AkkFirewall::new();
        let c = ctx(10, 1, 1, 0x00, 0x50, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn rule_count_reflects_five_meta_rules() {
        let fw = AkkFirewall::new();
        assert_eq!(fw.rule_count(), 5);
    }

    #[test]
    fn add_rule_increases_count_and_respects_priority() {
        let mut fw = AkkFirewall::new();
        fw.add_rule(AkkRule::new("Custom:AlwaysDeny", 255, |_| Some(FirewallVerdict::Deny)));
        assert_eq!(fw.rule_count(), 6);
        // A high-quality, low-risk context that would otherwise Allow now
        // hits the custom top-priority Deny rule.
        let c = ctx(240, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn gates_zero_trust_denies_below_threshold() {
        let fw = gates::standard_firewall(100, 50);
        // quality=70 clears META-1's dead-quality floor (59) so this
        // actually exercises the zero_trust_gate's own Deny, not META-1's.
        let c = ctx(70, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Deny);
    }

    #[test]
    fn gates_sho_redacts_between_thresholds() {
        let fw = gates::standard_firewall(30, 80);
        // quality=65: clears META-1's dead-quality floor (59) so the
        // constitutional meta rules (still active -- standard_firewall
        // adds gates on top of them, it doesn't replace them) stay quiet;
        // above zero_trust threshold (30) but below sho threshold (80).
        let c = ctx(65, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Redact);
    }

    #[test]
    fn gates_standard_firewall_allows_above_both_thresholds() {
        let fw = gates::standard_firewall(30, 80);
        let c = ctx(240, 1, 1, 0xFF, 0, AkkOperation::Read);
        assert_eq!(fw.evaluate(&c), FirewallVerdict::Allow);
    }

    #[test]
    fn firewall_verdict_display_matches_expected_strings() {
        assert_eq!(FirewallVerdict::Allow.to_string(), "ALLOW");
        assert_eq!(FirewallVerdict::Deny.to_string(), "DENY");
        assert_eq!(FirewallVerdict::Escalate.to_string(), "ESCALATE");
        assert_eq!(FirewallVerdict::Redact.to_string(), "REDACT");
        assert_eq!(FirewallVerdict::Audit.to_string(), "AUDIT");
    }
}
