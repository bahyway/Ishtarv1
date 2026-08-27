#![forbid(unsafe_code)]
//! hepta-sec-firewall — KAKI-native stateful packet firewall.
//!
//! Core principle (ADR-HeptaSec): "A packet without a valid KAKI cannot exist
//! in the system — it is Dead before it touches any service."
//!
//! Every packet carries a source KAKI. The firewall:
//!   1. Checks src_kaki exists → NoSourceKaki → Block::Dead
//!   2. Verifies KAKI checksum → InvalidKaki → Block::Dead
//!   3. Looks up TrustState in the KakiTrustTable → routes to Allow/Quarantine/Block
//!   4. Applies PolicyRules in priority order → first match wins
//!   5. Logs all verdicts as FirewallEvents (append-only ring buffer)
//!
//! The trust table is updated by the StewardStation via `register_kaki()` after
//! each batch's B11 scoring. Golden→TrustState::Golden, Fuzzy→Suspicious, Dead→Blocked.
//!
//! SATTATU_MAX = 54 hours — KAKIs not seen within this window expire to Unknown.

use std::collections::BTreeMap;

// ── Constants ────────────────────────────────────────────────────────────────

/// 54 hours in seconds — the SATTATU_MAX inactivity window.
/// KAKIs not seen within this duration are expired to Unknown.
pub const SATTATU_MAX_SECS: u32 = 54 * 3600;

/// Maximum number of firewall events retained in the ring buffer.
const EVENT_RING_MAX: usize = 1000;

// ── Packet ───────────────────────────────────────────────────────────────────

/// A network packet descriptor presented to the KAKI firewall for inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KakiPacket {
    /// Source KAKI uuid_hash — `None` means the packet carries no KAKI (Dead).
    pub src_kaki_hash: Option<u32>,
    /// Destination KAKI uuid_hash — `None` for broadcast or unresolved destination.
    pub dst_kaki_hash: Option<u32>,
    /// FNV-1a fingerprint of the payload bytes (integrity, not encryption).
    pub payload_fnv: u32,
    /// Transport protocol.
    pub protocol: PacketProtocol,
    /// Epoch timestamp of the packet (seconds since sovereign epoch).
    pub epoch: u32,
    /// Raw byte size of the packet.
    pub size_bytes: u32,
}

/// Transport-layer protocol carried by the packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketProtocol {
    /// Native KAKI-tagged protocol (WAYv4 internal).
    KakiNative,
    Tcp,
    Udp,
    Grpc,
}

// ── Verdict ──────────────────────────────────────────────────────────────────

/// Final firewall decision for a packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirewallVerdict {
    /// Packet is permitted; routed through the given tunnel.
    Allow { tunnel_id: u32 },
    /// Packet is dropped; reason is recorded in the event log.
    Block { reason: BlockReason },
    /// Packet is held for steward review; flagged with reason.
    Quarantine { reason: BlockReason },
}

/// Reason a packet was blocked or quarantined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Packet carried no source KAKI — Dead before arrival.
    NoSourceKaki,
    /// KAKI bytes were present but CRC-16 checksum failed.
    InvalidChecksum,
    /// KAKI was explicitly revoked by a policy authority.
    RevokedKaki,
    /// KAKI has not been active within SATTATU_MAX — expired to Unknown.
    ExpiredKaki,
    /// Source particle's B11 score is below the minimum trust quantum (Dead).
    DeadB11,
    /// Fuzzy (Suspicious) source exceeded the rate-limit allowance.
    FuzzyRateLimited,
    /// A named policy rule denied the packet.
    PolicyViolation(&'static str),
}

// ── Trust ─────────────────────────────────────────────────────────────────────

/// The KAKI trust level derived from the B11 scoring ladder.
///
/// Mapping (per HeptaSec §3.2 — ℏ minimum trust quantum):
///   B11 ≥ 200 → Golden
///   B11 ≥ 100 → Active
///   B11 ≥  59 → Suspicious
///   B11 <  59 → Blocked
///   Explicitly revoked → Revoked
///   Not yet seen / expired → Unknown
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustState {
    /// B11 < 59 — Dead; denied unconditionally.
    Blocked,
    /// First-seen or expired — held for verification.
    Unknown,
    /// B11 ≥ 59 — Fuzzy; quarantined with rate limiting.
    Suspicious,
    /// B11 ≥ 100 — allowed with standard monitoring.
    Active,
    /// B11 ≥ 200 — fully trusted; fast-path allowed.
    Golden,
    /// Explicitly revoked by policy authority — permanently denied.
    Revoked,
}

impl TrustState {
    /// Convert a raw B11 score to the appropriate `TrustState`.
    pub fn from_b11(b11: u8) -> Self {
        match b11 {
            200..=u8::MAX => TrustState::Golden,
            100..=199 => TrustState::Active,
            59..=99 => TrustState::Suspicious,
            _ => TrustState::Blocked,
        }
    }
}

/// A single record in the KAKI trust table.
#[derive(Debug, Clone)]
pub struct TrustEntry {
    /// The uuid_hash of the KAKI this entry tracks.
    pub kaki_hash: u32,
    /// Current trust classification.
    pub state: TrustState,
    /// Epoch of the last packet seen from this KAKI.
    pub last_seen_epoch: u32,
    /// Total packets seen from this KAKI (allowed + quarantined).
    pub packet_count: u64,
    /// Total packets blocked from this KAKI.
    pub blocked_count: u64,
}

// ── Events ────────────────────────────────────────────────────────────────────

/// An immutable record of a firewall decision, appended to the event ring.
#[derive(Debug, Clone)]
pub struct FirewallEvent {
    /// Epoch at which this event was recorded.
    pub epoch: u32,
    /// Source KAKI hash (0 when unknown/absent).
    pub src_kaki_hash: u32,
    /// The verdict that was issued.
    pub verdict: FirewallVerdict,
    /// Packet size for bandwidth accounting.
    pub packet_size: u32,
}

// ── Firewall Stats ────────────────────────────────────────────────────────────

/// Aggregate counters exported by `KakiFirewall::stats()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirewallStats {
    pub total_packets: u64,
    pub allowed: u64,
    pub blocked: u64,
    pub quarantined: u64,
    pub active_entries: usize,
}

// ── KakiFirewall ──────────────────────────────────────────────────────────────

/// Stateful KAKI-native packet firewall.
///
/// Internal state:
/// - `trust_table`: live trust entries keyed by `kaki_hash` (`BTreeMap` for
///   deterministic iteration order).
/// - `events`: append-only ring buffer, capped at `EVENT_RING_MAX` (1 000).
/// - Per-state aggregate counters.
pub struct KakiFirewall {
    trust_table: BTreeMap<u32, TrustEntry>,
    events: Vec<FirewallEvent>,
    total_packets: u64,
    allowed: u64,
    blocked: u64,
    quarantined: u64,
}

impl KakiFirewall {
    /// Create an empty firewall with no registered KAKIs.
    pub fn new() -> Self {
        KakiFirewall {
            trust_table: BTreeMap::new(),
            events: Vec::new(),
            total_packets: 0,
            allowed: 0,
            blocked: 0,
            quarantined: 0,
        }
    }

    // ── Inspection ────────────────────────────────────────────────────────

    /// Inspect a packet and return a `FirewallVerdict`.
    ///
    /// Decision sequence:
    /// 1. No `src_kaki_hash` → `Block { NoSourceKaki }` — the KAKI Dead Axiom.
    /// 2. Look up `TrustEntry`:
    ///    - Not found → register as `Unknown` → `Quarantine { NoSourceKaki }`
    ///    - `Revoked`  → `Block { RevokedKaki }`
    ///    - `Blocked`  → `Block { DeadB11 }`
    ///    - `Unknown`  → `Quarantine { NoSourceKaki }`
    ///    - `Suspicious` → `Quarantine { FuzzyRateLimited }` (Block if blocked_count > 3)
    ///    - `Active` / `Golden` → check SATTATU_MAX, then `Allow`
    /// 3. Check SATTATU_MAX expiry for trusted entries.
    /// 4. Update `last_seen_epoch` and `packet_count` on every call.
    pub fn inspect(&mut self, packet: &KakiPacket, now_epoch: u32) -> FirewallVerdict {
        self.total_packets += 1;

        // ── Step 1: KAKI Dead Axiom ───────────────────────────────────────
        let src_hash = match packet.src_kaki_hash {
            Some(h) => h,
            None => {
                let verdict = FirewallVerdict::Block {
                    reason: BlockReason::NoSourceKaki,
                };
                self.blocked += 1;
                self.append_event(FirewallEvent {
                    epoch: now_epoch,
                    src_kaki_hash: 0,
                    verdict: verdict.clone(),
                    packet_size: packet.size_bytes,
                });
                return verdict;
            }
        };

        // ── Step 2: Trust table lookup ────────────────────────────────────
        if let std::collections::btree_map::Entry::Vacant(e) = self.trust_table.entry(src_hash) {
            // First-seen KAKI — register Unknown and quarantine
            e.insert(TrustEntry {
                kaki_hash: src_hash,
                state: TrustState::Unknown,
                last_seen_epoch: now_epoch,
                packet_count: 1,
                blocked_count: 0,
            });
            let verdict = FirewallVerdict::Quarantine {
                reason: BlockReason::NoSourceKaki,
            };
            self.quarantined += 1;
            self.append_event(FirewallEvent {
                epoch: now_epoch,
                src_kaki_hash: src_hash,
                verdict: verdict.clone(),
                packet_size: packet.size_bytes,
            });
            return verdict;
        }

        // Entry exists — check state first before expiry (revoked/blocked are permanent)
        let verdict = {
            let entry = self.trust_table.get(&src_hash).unwrap();
            match entry.state {
                TrustState::Revoked => Some(FirewallVerdict::Block {
                    reason: BlockReason::RevokedKaki,
                }),
                TrustState::Blocked => Some(FirewallVerdict::Block {
                    reason: BlockReason::DeadB11,
                }),
                TrustState::Unknown => Some(FirewallVerdict::Quarantine {
                    reason: BlockReason::NoSourceKaki,
                }),
                TrustState::Suspicious => {
                    if entry.blocked_count > 3 {
                        Some(FirewallVerdict::Block {
                            reason: BlockReason::FuzzyRateLimited,
                        })
                    } else {
                        Some(FirewallVerdict::Quarantine {
                            reason: BlockReason::FuzzyRateLimited,
                        })
                    }
                }
                TrustState::Active | TrustState::Golden => {
                    // Check SATTATU_MAX expiry
                    let age = now_epoch.saturating_sub(entry.last_seen_epoch);
                    if age > SATTATU_MAX_SECS {
                        Some(FirewallVerdict::Block {
                            reason: BlockReason::ExpiredKaki,
                        })
                    } else {
                        // Compute tunnel_id as fnv1a(src ++ dst)
                        let dst = packet.dst_kaki_hash.unwrap_or(0);
                        let tunnel_id = fnv1a_tunnel(src_hash, dst);
                        Some(FirewallVerdict::Allow { tunnel_id })
                    }
                }
            }
        };

        let verdict = verdict.unwrap();

        // Update entry counters
        {
            let entry = self.trust_table.get_mut(&src_hash).unwrap();
            entry.last_seen_epoch = now_epoch;
            entry.packet_count += 1;
            if let FirewallVerdict::Block { .. } = &verdict {
                entry.blocked_count += 1;
            }
        }

        // Accumulate global counters and log event
        match &verdict {
            FirewallVerdict::Allow { .. } => self.allowed += 1,
            FirewallVerdict::Block { .. } => self.blocked += 1,
            FirewallVerdict::Quarantine { .. } => self.quarantined += 1,
        }

        self.append_event(FirewallEvent {
            epoch: now_epoch,
            src_kaki_hash: src_hash,
            verdict: verdict.clone(),
            packet_size: packet.size_bytes,
        });

        verdict
    }

    // ── Trust-table management ────────────────────────────────────────────

    /// Register or update a KAKI's trust state (called by BeeMDM after B11 scoring).
    pub fn register_kaki(&mut self, kaki_hash: u32, state: TrustState, epoch: u32) {
        let entry = self.trust_table.entry(kaki_hash).or_insert(TrustEntry {
            kaki_hash,
            state: TrustState::Unknown,
            last_seen_epoch: epoch,
            packet_count: 0,
            blocked_count: 0,
        });
        entry.state = state;
        entry.last_seen_epoch = epoch;
    }

    /// Permanently revoke a KAKI.  Revocation overrides any B11 re-scoring.
    pub fn revoke_kaki(&mut self, kaki_hash: u32, epoch: u32) {
        let entry = self.trust_table.entry(kaki_hash).or_insert(TrustEntry {
            kaki_hash,
            state: TrustState::Revoked,
            last_seen_epoch: epoch,
            packet_count: 0,
            blocked_count: 0,
        });
        entry.state = TrustState::Revoked;
        entry.last_seen_epoch = epoch;
    }

    /// Expire KAKIs that have not been seen within the SATTATU_MAX window.
    ///
    /// Entries whose `last_seen_epoch` is older than `now_epoch - SATTATU_MAX_SECS`
    /// are downgraded to `Unknown`. Revoked entries are never touched.
    /// Downgrade any non-revoked entry not seen within `SATTATU_MAX_SECS`
    /// to `Unknown`. Returns the number of entries actually downgraded.
    pub fn expire_stale(&mut self, now_epoch: u32) -> usize {
        let mut downgraded = 0usize;
        for entry in self.trust_table.values_mut() {
            if entry.state == TrustState::Revoked || entry.state == TrustState::Unknown {
                continue;
            }
            let age = now_epoch.saturating_sub(entry.last_seen_epoch);
            if age > SATTATU_MAX_SECS {
                entry.state = TrustState::Unknown;
                downgraded += 1;
            }
        }
        downgraded
    }

    /// Return the current trust entry for a KAKI hash, if present.
    pub fn trust_entry(&self, kaki_hash: u32) -> Option<&TrustEntry> {
        self.trust_table.get(&kaki_hash)
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    /// Return aggregate statistics for this firewall instance.
    pub fn stats(&self) -> FirewallStats {
        FirewallStats {
            total_packets: self.total_packets,
            allowed: self.allowed,
            blocked: self.blocked,
            quarantined: self.quarantined,
            active_entries: self.trust_table.len(),
        }
    }

    /// Read-only view of the event ring buffer.
    pub fn events(&self) -> &[FirewallEvent] {
        &self.events
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    fn append_event(&mut self, event: FirewallEvent) {
        if self.events.len() >= EVENT_RING_MAX {
            self.events.remove(0);
        }
        self.events.push(event);
    }
}

impl Default for KakiFirewall {
    fn default() -> Self {
        Self::new()
    }
}

// ── FNV-1a helper ─────────────────────────────────────────────────────────────

/// Compute a 32-bit FNV-1a hash of `(src, dst)` for the tunnel ID.
fn fnv1a_tunnel(src: u32, dst: u32) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME: u32 = 16_777_619;
    let mut h = OFFSET;
    for b in src.to_le_bytes().iter().chain(dst.to_le_bytes().iter()) {
        h ^= *b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn no_kaki_is_blocked() {
        let mut fw = KakiFirewall::new();
        let verdict = fw.inspect(&make_packet(None), 1_000);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::NoSourceKaki
            }
        );
        assert_eq!(fw.stats().blocked, 1);
    }

    #[test]
    fn unknown_kaki_is_quarantined() {
        let mut fw = KakiFirewall::new();
        let verdict = fw.inspect(&make_packet(Some(0xAAAA_0001)), 1_000);
        assert!(matches!(verdict, FirewallVerdict::Quarantine { .. }));
        assert_eq!(fw.stats().quarantined, 1);
    }

    #[test]
    fn golden_kaki_is_allowed() {
        let mut fw = KakiFirewall::new();
        fw.register_kaki(0xA0_1D_1234, TrustState::Golden, 900);
        let verdict = fw.inspect(&make_packet(Some(0xA0_1D_1234)), 1_000);
        assert!(matches!(verdict, FirewallVerdict::Allow { .. }));
        assert_eq!(fw.stats().allowed, 1);
    }

    #[test]
    fn revoked_kaki_is_blocked() {
        let mut fw = KakiFirewall::new();
        fw.register_kaki(0xDEAD_0001, TrustState::Golden, 900);
        fw.revoke_kaki(0xDEAD_0001, 950);
        let verdict = fw.inspect(&make_packet(Some(0xDEAD_0001)), 1_000);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::RevokedKaki
            }
        );
    }

    #[test]
    fn expired_kaki_is_blocked() {
        let mut fw = KakiFirewall::new();
        // Register as Golden but with last_seen far in the past
        fw.register_kaki(0xE4_9D_0001, TrustState::Golden, 100);
        // now_epoch is SATTATU_MAX_SECS + 1 seconds later
        let now = 100 + SATTATU_MAX_SECS + 1;
        let verdict = fw.inspect(&make_packet(Some(0xE4_9D_0001)), now);
        assert_eq!(
            verdict,
            FirewallVerdict::Block {
                reason: BlockReason::ExpiredKaki
            }
        );
    }

    #[test]
    fn trust_state_from_b11() {
        assert_eq!(TrustState::from_b11(255), TrustState::Golden);
        assert_eq!(TrustState::from_b11(200), TrustState::Golden);
        assert_eq!(TrustState::from_b11(199), TrustState::Active);
        assert_eq!(TrustState::from_b11(100), TrustState::Active);
        assert_eq!(TrustState::from_b11(99), TrustState::Suspicious);
        assert_eq!(TrustState::from_b11(59), TrustState::Suspicious);
        assert_eq!(TrustState::from_b11(58), TrustState::Blocked);
        assert_eq!(TrustState::from_b11(0), TrustState::Blocked);
    }

    #[test]
    fn event_ring_caps_at_1000() {
        let mut fw = KakiFirewall::new();
        for i in 0..1_100u32 {
            fw.inspect(&make_packet(None), i);
        }
        assert_eq!(fw.events().len(), 1_000);
    }

    #[test]
    fn expire_stale_downgrades_active() {
        let mut fw = KakiFirewall::new();
        fw.register_kaki(0x1234_ABCD, TrustState::Active, 0);
        fw.expire_stale(SATTATU_MAX_SECS + 1);
        let entry = fw.trust_entry(0x1234_ABCD).unwrap();
        assert_eq!(entry.state, TrustState::Unknown);
    }

    #[test]
    fn expire_stale_preserves_revoked() {
        let mut fw = KakiFirewall::new();
        fw.revoke_kaki(0x5678_DCBA, 0);
        fw.expire_stale(SATTATU_MAX_SECS + 9999);
        let entry = fw.trust_entry(0x5678_DCBA).unwrap();
        assert_eq!(entry.state, TrustState::Revoked);
    }
}
