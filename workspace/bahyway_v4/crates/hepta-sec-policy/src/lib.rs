#![forbid(unsafe_code)]
//! hepta-sec-policy — WAYv4.0 security policy engine.
//!
//! Security policies are written in `.akk` HeptaScript and compiled to
//! `PolicyRule` structs.  The `PolicyEngine` evaluates rules in priority order
//! (lower priority number = higher precedence) against each `KakiPacket`.
//!
//! Example policy (in future .akk format):
//!   GUARD block_unknown_sources {
//!     REQUIRE: particle.kaki_state = Unknown
//!     ALLOW:   [Quarantine]
//!   }

use std::collections::HashMap;

use hepta_sec_firewall::{BlockReason, KakiPacket, PacketProtocol, TrustEntry, TrustState};

// ── RateTracker ───────────────────────────────────────────────────────────────

/// One-second sliding window for a single KAKI.
struct RateWindow {
    epoch: u32,
    count: u32,
}

/// Per-KAKI packet-rate tracker used by `PolicyCondition::RateExceeds`.
///
/// Records one packet per call to `record_and_check()`; returns `true`
/// when the rate within the current one-second window exceeds `threshold`.
pub struct RateTracker {
    windows: HashMap<u32, RateWindow>,
}

impl RateTracker {
    pub fn new() -> Self {
        RateTracker {
            windows: HashMap::new(),
        }
    }

    /// Record one packet for `kaki_hash` at `now_epoch` and return whether
    /// the per-second count now exceeds `threshold`.
    pub fn record_and_check(&mut self, kaki_hash: u32, threshold: u32, now_epoch: u32) -> bool {
        let w = self.windows.entry(kaki_hash).or_insert(RateWindow {
            epoch: now_epoch,
            count: 0,
        });
        if w.epoch != now_epoch {
            w.epoch = now_epoch;
            w.count = 0;
        }
        w.count += 1;
        w.count > threshold
    }

    /// Remove windows older than `now_epoch` to bound memory use.
    pub fn purge_stale(&mut self, now_epoch: u32) {
        self.windows.retain(|_, w| w.epoch == now_epoch);
    }
}

impl Default for RateTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ── PolicyPriority ────────────────────────────────────────────────────────────

/// Execution priority of a policy rule.  Lower numeric value = higher precedence.
/// Rules are sorted by priority before evaluation; the first match wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyPriority {
    /// Priority 1 — highest; absolute deny/allow (e.g. KAKI absent, revoked).
    Critical = 1,
    /// Priority 2 — high; identity-verified denials (dead B11, unknown source).
    High = 2,
    /// Priority 3 — normal; conditional quarantine / permit rules.
    Normal = 3,
    /// Priority 4 — lowest; catch-all default-deny.
    Low = 4,
}

impl PolicyPriority {
    /// Numeric precedence value (1 = most urgent).
    pub fn value(self) -> u8 {
        self as u8
    }
}

// ── PolicyCondition ───────────────────────────────────────────────────────────

/// A predicate evaluated against a packet + optional trust entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyCondition {
    /// Packet carries no source KAKI hash.
    SourceKakiAbsent,
    /// Source KAKI is in the given trust state.
    SourceTrustState(TrustState),
    /// Destination KAKI is in the given trust state.
    DestTrustState(TrustState),
    /// Packet uses the specified protocol.
    ProtocolIs(PacketProtocol),
    /// Packet size (in bytes) exceeds the threshold.
    PacketSizeExceeds(u32),
    /// Rate exceeds N packets/second — requires an external counter (Phase 2).
    RateExceeds { packets_per_second: u32 },
}

// ── PolicyAction ──────────────────────────────────────────────────────────────

/// The action taken when a policy rule matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyAction {
    /// Drop the packet with the given reason.
    Deny(BlockReason),
    /// Allow the packet through.
    Permit,
    /// Hold the packet and trigger an alert with the given message.
    QuarantineAndAlert(&'static str),
}

// ── PolicyRule ────────────────────────────────────────────────────────────────

/// A single security rule: name + priority + conditions (AND logic) + action.
#[derive(Debug, Clone)]
pub struct PolicyRule {
    /// Human-readable rule identifier (used in audit logs).
    pub name: &'static str,
    /// Evaluation precedence — lower value = first evaluated.
    pub priority: PolicyPriority,
    /// All conditions must be satisfied (logical AND).
    pub conditions: Vec<PolicyCondition>,
    /// Action taken when all conditions match.
    pub action: PolicyAction,
}

// ── PolicyEngine ──────────────────────────────────────────────────────────────

/// Evaluates an ordered set of `PolicyRule`s against incoming packets.
///
/// Rules are stored sorted by `priority` (ascending).  `evaluate()` returns
/// the action of the first rule whose conditions all match, or `None` when
/// no rule matches (caller should treat this as implicit permit or default-deny
/// depending on posture).
pub struct PolicyEngine {
    /// Rules sorted by priority value (ascending = highest precedence first).
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    /// Create an empty engine with no rules.
    pub fn new() -> Self {
        PolicyEngine { rules: Vec::new() }
    }

    /// Add a rule.  The rules list is re-sorted after insertion.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // Sort by priority value ascending (Critical=1 first, Low=4 last)
        self.rules.sort_by_key(|r| r.priority.value());
    }

    /// Evaluate rules against `packet` + optional trust entry.
    ///
    /// Returns the first matching rule's action, or `None` if no rule matches.
    pub fn evaluate(
        &self,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
    ) -> Option<PolicyAction> {
        for rule in &self.rules {
            if self.rule_matches(rule, packet, trust) {
                return Some(rule.action.clone());
            }
        }
        None
    }

    /// Read-only access to the loaded rules.
    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    /// Evaluate rules with rate-limiting support.
    ///
    /// Identical to `evaluate()` except `PolicyCondition::RateExceeds` is live:
    /// it records the packet in `tracker` and returns `true` when the per-second
    /// count for `packet.src_kaki_hash` exceeds the configured threshold.
    pub fn evaluate_with_rate(
        &self,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
        tracker: &mut RateTracker,
        now_epoch: u32,
    ) -> Option<PolicyAction> {
        for rule in &self.rules {
            if self.rule_matches_with_rate(rule, packet, trust, tracker, now_epoch) {
                return Some(rule.action.clone());
            }
        }
        None
    }

    /// Build a `PolicyEngine` pre-loaded with the 7 HeptaSec sovereign rules.
    ///
    /// Rule table (priority order):
    ///   1. Critical — deny: packet carries no KAKI                 → NoSourceKaki
    ///   2. Critical — deny: KAKI is explicitly revoked             → RevokedKaki
    ///   3. High     — deny: KAKI trust = Blocked (Dead B11 < 59)   → DeadB11
    ///   4. High     — quarantine+alert: trust = Unknown (first seen)
    ///   5. Normal   — quarantine+alert: trust = Suspicious (Fuzzy B11)
    ///   6. Normal   — permit: trust = Active or Golden
    ///   7. Low      — deny all (default-deny catch-all)            → PolicyViolation
    pub fn default_rules() -> Self {
        let mut engine = PolicyEngine::new();

        // Rule 1 — Critical: block packets with no source KAKI
        engine.add_rule(PolicyRule {
            name: "block_no_kaki",
            priority: PolicyPriority::Critical,
            conditions: vec![PolicyCondition::SourceKakiAbsent],
            action: PolicyAction::Deny(BlockReason::NoSourceKaki),
        });

        // Rule 2 — Critical: block revoked KAKIs
        engine.add_rule(PolicyRule {
            name: "block_revoked_kaki",
            priority: PolicyPriority::Critical,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Revoked)],
            action: PolicyAction::Deny(BlockReason::RevokedKaki),
        });

        // Rule 3 — High: block Blocked trust state (Dead B11)
        engine.add_rule(PolicyRule {
            name: "block_dead_b11",
            priority: PolicyPriority::High,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Blocked)],
            action: PolicyAction::Deny(BlockReason::DeadB11),
        });

        // Rule 4 — High: quarantine first-seen / unknown sources
        engine.add_rule(PolicyRule {
            name: "quarantine_unknown_source",
            priority: PolicyPriority::High,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Unknown)],
            action: PolicyAction::QuarantineAndAlert(
                "unknown_source: first-seen KAKI held for verification",
            ),
        });

        // Rule 5 — Normal: quarantine suspicious (Fuzzy B11) sources
        engine.add_rule(PolicyRule {
            name: "quarantine_suspicious_source",
            priority: PolicyPriority::Normal,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Suspicious)],
            action: PolicyAction::QuarantineAndAlert(
                "suspicious_source: Fuzzy B11 held for steward review",
            ),
        });

        // Rule 6a — Normal: permit Active sources
        engine.add_rule(PolicyRule {
            name: "permit_active_source",
            priority: PolicyPriority::Normal,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Active)],
            action: PolicyAction::Permit,
        });

        // Rule 6b — Normal: permit Golden sources (counted as one of the 7 sovereign rules)
        engine.add_rule(PolicyRule {
            name: "permit_golden_source",
            priority: PolicyPriority::Normal,
            conditions: vec![PolicyCondition::SourceTrustState(TrustState::Golden)],
            action: PolicyAction::Permit,
        });

        // Rule 7 — Low: catch-all default deny
        engine.add_rule(PolicyRule {
            name: "default_deny_all",
            priority: PolicyPriority::Low,
            conditions: vec![], // empty = always matches
            action: PolicyAction::Deny(BlockReason::PolicyViolation(
                "default_deny: no rule permitted this packet",
            )),
        });

        engine
    }

    // ── Condition evaluation ──────────────────────────────────────────────────

    fn rule_matches(
        &self,
        rule: &PolicyRule,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
    ) -> bool {
        for cond in &rule.conditions {
            if !self.condition_matches(cond, packet, trust) {
                return false;
            }
        }
        true
    }

    fn condition_matches(
        &self,
        condition: &PolicyCondition,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
    ) -> bool {
        match condition {
            PolicyCondition::SourceKakiAbsent => packet.src_kaki_hash.is_none(),
            PolicyCondition::SourceTrustState(expected) => {
                trust.is_some_and(|e| &e.state == expected)
            }
            PolicyCondition::DestTrustState(expected) => {
                // For now: only matchable if we have a trust entry and the
                // packet dst matches its hash.  Phase 2 will add a lookup.
                trust.is_some_and(|e| {
                    packet.dst_kaki_hash == Some(e.kaki_hash) && &e.state == expected
                })
            }
            PolicyCondition::ProtocolIs(proto) => &packet.protocol == proto,
            PolicyCondition::PacketSizeExceeds(threshold) => packet.size_bytes > *threshold,
            PolicyCondition::RateExceeds { .. } => {
                // Stateless evaluate() cannot track rates — always false.
                // Use evaluate_with_rate() to get live rate enforcement.
                false
            }
        }
    }

    fn rule_matches_with_rate(
        &self,
        rule: &PolicyRule,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
        tracker: &mut RateTracker,
        now_epoch: u32,
    ) -> bool {
        for cond in &rule.conditions {
            if !self.condition_matches_with_rate(cond, packet, trust, tracker, now_epoch) {
                return false;
            }
        }
        true
    }

    fn condition_matches_with_rate(
        &self,
        condition: &PolicyCondition,
        packet: &KakiPacket,
        trust: Option<&TrustEntry>,
        tracker: &mut RateTracker,
        now_epoch: u32,
    ) -> bool {
        match condition {
            PolicyCondition::RateExceeds { packets_per_second } => {
                if let Some(hash) = packet.src_kaki_hash {
                    tracker.record_and_check(hash, *packets_per_second, now_epoch)
                } else {
                    false
                }
            }
            other => self.condition_matches(other, packet, trust),
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use hepta_sec_firewall::{BlockReason, KakiPacket, PacketProtocol, TrustEntry, TrustState};

    fn make_packet(src: Option<u32>, size: u32) -> KakiPacket {
        KakiPacket {
            src_kaki_hash: src,
            dst_kaki_hash: Some(0xBEEF_0001),
            payload_fnv: 0,
            protocol: PacketProtocol::KakiNative,
            epoch: 1_000,
            size_bytes: size,
        }
    }

    fn make_trust(hash: u32, state: TrustState) -> TrustEntry {
        TrustEntry {
            kaki_hash: hash,
            state,
            last_seen_epoch: 900,
            packet_count: 0,
            blocked_count: 0,
        }
    }

    #[test]
    fn condition_source_kaki_absent() {
        let engine = PolicyEngine::new();
        let packet = make_packet(None, 128);
        assert!(engine.condition_matches(&PolicyCondition::SourceKakiAbsent, &packet, None,));
        let packet2 = make_packet(Some(0x1234), 128);
        assert!(!engine.condition_matches(&PolicyCondition::SourceKakiAbsent, &packet2, None,));
    }

    #[test]
    fn condition_source_trust_state_matches() {
        let engine = PolicyEngine::new();
        let packet = make_packet(Some(0x1111), 128);
        let trust = make_trust(0x1111, TrustState::Golden);
        assert!(engine.condition_matches(
            &PolicyCondition::SourceTrustState(TrustState::Golden),
            &packet,
            Some(&trust),
        ));
        assert!(!engine.condition_matches(
            &PolicyCondition::SourceTrustState(TrustState::Suspicious),
            &packet,
            Some(&trust),
        ));
    }

    #[test]
    fn condition_protocol_matches() {
        let engine = PolicyEngine::new();
        let packet = make_packet(Some(0x2222), 128);
        assert!(engine.condition_matches(
            &PolicyCondition::ProtocolIs(PacketProtocol::KakiNative),
            &packet,
            None,
        ));
        assert!(!engine.condition_matches(
            &PolicyCondition::ProtocolIs(PacketProtocol::Tcp),
            &packet,
            None,
        ));
    }

    #[test]
    fn condition_packet_size_exceeds() {
        let engine = PolicyEngine::new();
        let big_packet = make_packet(Some(0x3333), 1024);
        let small_packet = make_packet(Some(0x3333), 64);
        assert!(engine.condition_matches(
            &PolicyCondition::PacketSizeExceeds(512),
            &big_packet,
            None,
        ));
        assert!(!engine.condition_matches(
            &PolicyCondition::PacketSizeExceeds(512),
            &small_packet,
            None,
        ));
    }

    #[test]
    fn priority_ordering() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            name: "low_rule",
            priority: PolicyPriority::Low,
            conditions: vec![],
            action: PolicyAction::Deny(BlockReason::PolicyViolation("low")),
        });
        engine.add_rule(PolicyRule {
            name: "critical_rule",
            priority: PolicyPriority::Critical,
            conditions: vec![],
            action: PolicyAction::Deny(BlockReason::NoSourceKaki),
        });
        // Critical (1) should come before Low (4)
        assert_eq!(engine.rules()[0].name, "critical_rule");
        assert_eq!(engine.rules()[1].name, "low_rule");
    }

    #[test]
    fn evaluate_first_match_wins() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            name: "deny_no_kaki",
            priority: PolicyPriority::Critical,
            conditions: vec![PolicyCondition::SourceKakiAbsent],
            action: PolicyAction::Deny(BlockReason::NoSourceKaki),
        });
        engine.add_rule(PolicyRule {
            name: "catch_all",
            priority: PolicyPriority::Low,
            conditions: vec![],
            action: PolicyAction::Deny(BlockReason::PolicyViolation("catch-all")),
        });
        let packet = make_packet(None, 128);
        let action = engine.evaluate(&packet, None);
        assert_eq!(action, Some(PolicyAction::Deny(BlockReason::NoSourceKaki)));
    }

    #[test]
    fn default_rules_produces_expected_count() {
        let engine = PolicyEngine::default_rules();
        // 7 sovereign rules: 2 Critical + 2 High + 3 Normal(quarantine+permit+permit) + 1 Low
        assert_eq!(engine.rules().len(), 8);
    }

    #[test]
    fn default_rules_catch_all_fires_last() {
        let engine = PolicyEngine::default_rules();
        // A packet with a known Active trust — should hit permit_active_source before catch-all
        let packet = make_packet(Some(0x4444), 128);
        let trust = make_trust(0x4444, TrustState::Active);
        let action = engine.evaluate(&packet, Some(&trust));
        assert_eq!(action, Some(PolicyAction::Permit));
    }

    #[test]
    fn default_rules_no_kaki_denied_first() {
        let engine = PolicyEngine::default_rules();
        let packet = make_packet(None, 128);
        let action = engine.evaluate(&packet, None);
        assert_eq!(action, Some(PolicyAction::Deny(BlockReason::NoSourceKaki)));
    }

    #[test]
    fn default_rules_unknown_quarantined() {
        let engine = PolicyEngine::default_rules();
        let packet = make_packet(Some(0x5555), 128);
        let trust = make_trust(0x5555, TrustState::Unknown);
        let action = engine.evaluate(&packet, Some(&trust));
        assert!(matches!(action, Some(PolicyAction::QuarantineAndAlert(_))));
    }

    // ── RateTracker tests ─────────────────────────────────────────────────────

    #[test]
    fn rate_tracker_under_threshold() {
        let mut tracker = RateTracker::new();
        // threshold = 5 packets/second; 5 packets should NOT exceed
        for _ in 0..5 {
            let exceeded = tracker.record_and_check(0xAAAA_0001, 5, 1_000);
            assert!(!exceeded, "should not exceed at or below threshold");
        }
    }

    #[test]
    fn rate_tracker_exceeds_threshold() {
        let mut tracker = RateTracker::new();
        // threshold = 3; 4th packet crosses the limit
        for _ in 0..3 {
            tracker.record_and_check(0xBBBB_0001, 3, 2_000);
        }
        let exceeded = tracker.record_and_check(0xBBBB_0001, 3, 2_000);
        assert!(
            exceeded,
            "4th packet in same second should exceed threshold of 3"
        );
    }

    #[test]
    fn rate_tracker_resets_on_new_epoch() {
        let mut tracker = RateTracker::new();
        for _ in 0..10 {
            tracker.record_and_check(0xCCCC_0001, 5, 3_000);
        }
        // Next second — count resets
        let exceeded = tracker.record_and_check(0xCCCC_0001, 5, 3_001);
        assert!(!exceeded, "first packet in new second should never exceed");
    }

    #[test]
    fn evaluate_with_rate_triggers_on_flood() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            name: "rate_limit_rule",
            priority: PolicyPriority::Critical,
            conditions: vec![PolicyCondition::RateExceeds {
                packets_per_second: 2,
            }],
            action: PolicyAction::Deny(BlockReason::PolicyViolation("rate_exceeded")),
        });
        engine.add_rule(PolicyRule {
            name: "catch_all_permit",
            priority: PolicyPriority::Low,
            conditions: vec![],
            action: PolicyAction::Permit,
        });

        let packet = make_packet(Some(0xDDDD_0001), 64);
        let mut tracker = RateTracker::new();

        // First 2 packets: below/at threshold → permit
        assert_eq!(
            engine.evaluate_with_rate(&packet, None, &mut tracker, 5_000),
            Some(PolicyAction::Permit)
        );
        assert_eq!(
            engine.evaluate_with_rate(&packet, None, &mut tracker, 5_000),
            Some(PolicyAction::Permit)
        );
        // 3rd packet: threshold exceeded → deny
        let action = engine.evaluate_with_rate(&packet, None, &mut tracker, 5_000);
        assert_eq!(
            action,
            Some(PolicyAction::Deny(BlockReason::PolicyViolation(
                "rate_exceeded"
            )))
        );
    }
}
