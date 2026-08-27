#![forbid(unsafe_code)]
//! hepta-sec-sentinel — sovereign ZeroWay replacement sentinel.
//!
//! The HeptaSecSentinel integrates the firewall + policy engine into a
//! stateful inspection loop.  It receives packets (or packet descriptors),
//! applies the KAKI firewall + policy, and emits security events.
//!
//! Phase 1: in-process sentinel (library).
//! Phase 2: standalone UrOS process with NajafEngine topology feed.
//! Phase 3: kernel-level WPDEngine packet inspection on bare metal.

use hepta_sec_firewall::{BlockReason, FirewallVerdict, KakiFirewall, KakiPacket, TrustState};
use hepta_sec_policy::{PolicyAction, PolicyEngine};

/// Maximum number of security events retained in the event ring buffer.
const EVENT_RING_MAX: usize = 500;

// ── SecurityEvent ─────────────────────────────────────────────────────────────

/// An immutable record emitted by the sentinel for every security-relevant decision.
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// Epoch at which this event was recorded (seconds since sovereign epoch).
    pub epoch: u32,
    /// Classification of the event.
    pub event_kind: SecurityEventKind,
    /// Source KAKI hash (0 when absent or unknown).
    pub src_kaki_hash: u32,
    /// Human-readable detail string for audit logs.
    pub detail: &'static str,
}

/// Classification of a `SecurityEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEventKind {
    /// Packet was denied by the firewall with the given reason.
    PacketBlocked(BlockReason),
    /// Packet was quarantined for steward review.
    PacketQuarantined(BlockReason),
    /// A KAKI appeared that had never been registered — first contact.
    NewDeviceDetected,
    /// A previously-active KAKI expired past SATTATU_MAX.
    KakiExpired,
    /// A packet violated a named policy rule.
    PolicyViolation,
    /// Stale entries were purged; contains the count removed.
    StalePurged(usize),
}

// ── SentinelStats ─────────────────────────────────────────────────────────────

/// Aggregate counters exported by `HeptaSecSentinel::stats()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SentinelStats {
    pub packets_inspected: u64,
    pub blocked: u64,
    pub quarantined: u64,
    pub allowed: u64,
    pub events_emitted: u64,
}

// ── HeptaSecSentinel ──────────────────────────────────────────────────────────

/// Sovereign stateful inspection sentinel.
///
/// Integrates `KakiFirewall` (trust table + verdict) with `PolicyEngine`
/// (rule evaluation) and emits `SecurityEvent`s for every Block/Quarantine
/// decision.  All state is in-process; Phase 2 will extract this into a
/// standalone UrOS service.
pub struct HeptaSecSentinel {
    firewall: KakiFirewall,
    policy: PolicyEngine,
    events: Vec<SecurityEvent>,
    stats: SentinelStats,
}

impl HeptaSecSentinel {
    /// Create a new sentinel with the 7 HeptaSec sovereign default policy rules.
    pub fn new() -> Self {
        HeptaSecSentinel {
            firewall: KakiFirewall::new(),
            policy: PolicyEngine::default_rules(),
            events: Vec::new(),
            stats: SentinelStats::default(),
        }
    }

    // ── Core inspection ───────────────────────────────────────────────────────

    /// Inspect a packet through the firewall + policy pipeline.
    ///
    /// Pipeline:
    /// 1. `firewall.inspect()` — fast path: KAKI presence, trust table, expiry.
    /// 2. For non-trivial verdicts, `policy.evaluate()` may upgrade/downgrade.
    /// 3. `SecurityEvent` is emitted for every Block and Quarantine verdict.
    /// 4. Final verdict is returned.
    pub fn inspect_packet(&mut self, packet: &KakiPacket, now_epoch: u32) -> FirewallVerdict {
        self.stats.packets_inspected += 1;

        // ── Firewall fast-path ────────────────────────────────────────────────
        let fw_verdict = self.firewall.inspect(packet, now_epoch);

        // ── Policy secondary check (for Allow verdicts — policy may restrict) ─
        // If the firewall allows the packet, run it through the policy engine for
        // any additional rule-based restrictions (e.g. protocol/size rules).
        let fw_verdict = if let FirewallVerdict::Allow { tunnel_id } = &fw_verdict {
            let trust = packet
                .src_kaki_hash
                .and_then(|h| self.firewall.trust_entry(h));
            match self.policy.evaluate(packet, trust) {
                Some(PolicyAction::Deny(reason)) => FirewallVerdict::Block { reason },
                Some(PolicyAction::QuarantineAndAlert(_msg)) => FirewallVerdict::Quarantine {
                    reason: BlockReason::PolicyViolation("policy_engine"),
                },
                Some(PolicyAction::Permit) | None => FirewallVerdict::Allow {
                    tunnel_id: *tunnel_id,
                },
            }
        } else {
            fw_verdict
        };

        // ── Accumulate global counters and emit events ────────────────────────
        match &fw_verdict {
            FirewallVerdict::Allow { .. } => {
                self.stats.allowed += 1;
            }
            FirewallVerdict::Block { reason } => {
                self.stats.blocked += 1;
                let (detail, kind) = match reason {
                    BlockReason::NoSourceKaki => (
                        "blocked: no source KAKI",
                        SecurityEventKind::PacketBlocked(*reason),
                    ),
                    BlockReason::InvalidChecksum => (
                        "blocked: invalid KAKI checksum",
                        SecurityEventKind::PacketBlocked(*reason),
                    ),
                    BlockReason::RevokedKaki => (
                        "blocked: KAKI revoked",
                        SecurityEventKind::PacketBlocked(*reason),
                    ),
                    BlockReason::ExpiredKaki => (
                        "blocked: KAKI expired (SATTATU_MAX)",
                        SecurityEventKind::KakiExpired,
                    ),
                    BlockReason::DeadB11 => (
                        "blocked: B11 below minimum trust quantum",
                        SecurityEventKind::PacketBlocked(*reason),
                    ),
                    BlockReason::FuzzyRateLimited => (
                        "blocked: Fuzzy rate limit exceeded",
                        SecurityEventKind::PacketBlocked(*reason),
                    ),
                    BlockReason::PolicyViolation(_) => (
                        "blocked: policy violation",
                        SecurityEventKind::PolicyViolation,
                    ),
                };
                self.emit_event(SecurityEvent {
                    epoch: now_epoch,
                    event_kind: kind,
                    src_kaki_hash: packet.src_kaki_hash.unwrap_or(0),
                    detail,
                });
            }
            FirewallVerdict::Quarantine { reason } => {
                self.stats.quarantined += 1;
                let (detail, kind) = match reason {
                    BlockReason::NoSourceKaki => {
                        // Could be first-seen device
                        (
                            "quarantined: unknown source KAKI — first contact",
                            SecurityEventKind::NewDeviceDetected,
                        )
                    }
                    BlockReason::FuzzyRateLimited => (
                        "quarantined: suspicious Fuzzy B11 source",
                        SecurityEventKind::PacketQuarantined(*reason),
                    ),
                    _ => (
                        "quarantined: policy hold",
                        SecurityEventKind::PacketQuarantined(*reason),
                    ),
                };
                self.emit_event(SecurityEvent {
                    epoch: now_epoch,
                    event_kind: kind,
                    src_kaki_hash: packet.src_kaki_hash.unwrap_or(0),
                    detail,
                });
            }
        }

        fw_verdict
    }

    // ── Trust registration ────────────────────────────────────────────────────

    /// Register a KAKI with the firewall after BeeMDM B11 scoring.
    ///
    /// Converts the raw `b11` score to a `TrustState` and delegates to
    /// `KakiFirewall::register_kaki()`.
    pub fn register_kaki_b11(&mut self, kaki_hash: u32, b11: u8, epoch: u32) {
        let state = TrustState::from_b11(b11);
        self.firewall.register_kaki(kaki_hash, state, epoch);
    }

    /// Revoke a KAKI — permanent denial regardless of future B11 rescoring.
    pub fn revoke_kaki(&mut self, kaki_hash: u32, epoch: u32) {
        self.firewall.revoke_kaki(kaki_hash, epoch);
    }

    // ── Maintenance ───────────────────────────────────────────────────────────

    /// Expire stale entries (not seen within SATTATU_MAX) and emit a
    /// `StalePurged` event.
    ///
    /// Returns the number of entries that were downgraded to `Unknown`.
    pub fn purge_stale(&mut self, now_epoch: u32) -> usize {
        let purged = self.firewall.expire_stale(now_epoch);

        let epoch = now_epoch;
        self.emit_event(SecurityEvent {
            epoch,
            event_kind: SecurityEventKind::StalePurged(purged),
            src_kaki_hash: 0,
            detail: "stale_purge: SATTATU_MAX window sweep completed",
        });

        purged
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Return the current aggregate statistics.
    pub fn stats(&self) -> &SentinelStats {
        &self.stats
    }

    /// Read-only view of the event ring buffer.
    pub fn events(&self) -> &[SecurityEvent] {
        &self.events
    }

    /// Read-only access to the underlying firewall (for inspection/testing).
    pub fn firewall(&self) -> &KakiFirewall {
        &self.firewall
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn emit_event(&mut self, event: SecurityEvent) {
        self.stats.events_emitted += 1;
        if self.events.len() >= EVENT_RING_MAX {
            self.events.remove(0);
        }
        self.events.push(event);
    }
}

impl Default for HeptaSecSentinel {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use hepta_sec_firewall::{BlockReason, FirewallVerdict, KakiPacket, PacketProtocol};

    fn make_packet(src: Option<u32>) -> KakiPacket {
        KakiPacket {
            src_kaki_hash: src,
            dst_kaki_hash: Some(0xBEEF_0001),
            payload_fnv: 0,
            protocol: PacketProtocol::KakiNative,
            epoch: 1_000,
            size_bytes: 128,
        }
    }

    // ── End-to-end: register Golden KAKI → inspect → Allow ───────────────────

    #[test]
    fn golden_kaki_end_to_end_allow() {
        let mut sentinel = HeptaSecSentinel::new();
        // B11 = 220 → Golden
        sentinel.register_kaki_b11(0xAAAA_0001, 220, 900);
        let verdict = sentinel.inspect_packet(&make_packet(Some(0xAAAA_0001)), 1_000);
        assert!(matches!(verdict, FirewallVerdict::Allow { .. }));
        assert_eq!(sentinel.stats().allowed, 1);
        assert_eq!(sentinel.stats().blocked, 0);
        // No security event emitted for Allow
        assert_eq!(sentinel.events().len(), 0);
    }

    // ── Unknown KAKI → Quarantine ─────────────────────────────────────────────

    #[test]
    fn unknown_kaki_quarantined() {
        let mut sentinel = HeptaSecSentinel::new();
        let verdict = sentinel.inspect_packet(&make_packet(Some(0xBBBB_0002)), 1_000);
        assert!(matches!(verdict, FirewallVerdict::Quarantine { .. }));
        assert_eq!(sentinel.stats().quarantined, 1);
        assert_eq!(sentinel.events().len(), 1);
        assert!(matches!(
            sentinel.events()[0].event_kind,
            SecurityEventKind::NewDeviceDetected,
        ));
    }

    // ── B11=50 → Blocked trust → inspect → Block ─────────────────────────────

    #[test]
    fn dead_b11_blocked() {
        let mut sentinel = HeptaSecSentinel::new();
        // B11 = 50 < 59 → Blocked
        sentinel.register_kaki_b11(0xCCCC_0003, 50, 900);
        let verdict = sentinel.inspect_packet(&make_packet(Some(0xCCCC_0003)), 1_000);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::DeadB11
            }
        );
        assert_eq!(sentinel.stats().blocked, 1);
        assert_eq!(sentinel.events().len(), 1);
        assert!(matches!(
            sentinel.events()[0].event_kind,
            SecurityEventKind::PacketBlocked(BlockReason::DeadB11),
        ));
    }

    // ── purge_stale clears expired entries ───────────────────────────────────

    #[test]
    fn purge_stale_emits_event() {
        let mut sentinel = HeptaSecSentinel::new();
        // Register a Golden KAKI at epoch 0
        sentinel.register_kaki_b11(0xDDDD_0004, 200, 0);
        // Sweep well past SATTATU_MAX
        sentinel.purge_stale(200_000);
        // A StalePurged event should appear
        let purge_events: Vec<_> = sentinel
            .events()
            .iter()
            .filter(|e| matches!(e.event_kind, SecurityEventKind::StalePurged(_)))
            .collect();
        assert_eq!(purge_events.len(), 1);
    }

    // ── Revoked KAKI → Block ──────────────────────────────────────────────────

    #[test]
    fn revoked_kaki_blocked() {
        let mut sentinel = HeptaSecSentinel::new();
        sentinel.register_kaki_b11(0xEEEE_0005, 200, 900);
        sentinel.revoke_kaki(0xEEEE_0005, 950);
        let verdict = sentinel.inspect_packet(&make_packet(Some(0xEEEE_0005)), 1_000);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::RevokedKaki
            }
        );
        assert_eq!(sentinel.stats().blocked, 1);
    }

    // ── No KAKI → Block ───────────────────────────────────────────────────────

    #[test]
    fn no_kaki_blocked() {
        let mut sentinel = HeptaSecSentinel::new();
        let verdict = sentinel.inspect_packet(&make_packet(None), 1_000);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::NoSourceKaki
            }
        );
    }

    // ── Event ring caps at 500 ────────────────────────────────────────────────

    #[test]
    fn event_ring_caps_at_500() {
        let mut sentinel = HeptaSecSentinel::new();
        for i in 0..600u32 {
            // Each packet has no KAKI → Block event
            sentinel.inspect_packet(&make_packet(None), i);
        }
        assert_eq!(sentinel.events().len(), 500);
    }
}
