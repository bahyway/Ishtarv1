# HeptaSecFirewall — Developer Manual

> **DubSar Help** | `Manuals > HeptaSecFirewall` | Crate Reference

## Overview

`hepta-sec-firewall` is the KAKI-native stateful packet firewall for the WAYv4.0
sovereignty layer.  Every packet that enters the system must carry a valid source KAKI;
any packet without one is **Dead before it touches any service**.

Sovereignty principle (ADR-HeptaSec): *"A packet without a valid KAKI cannot exist
in the system — it is Dead before it touches any service."*

---

## Core Types

### KakiPacket

```rust
pub struct KakiPacket {
    pub src_kaki_hash: Option<u32>,  // None = Dead — blocked unconditionally
    pub dst_kaki_hash: Option<u32>,
    pub payload_fnv:   u32,          // FNV-1a payload fingerprint
    pub protocol:      PacketProtocol,
    pub epoch:         u32,          // seconds since sovereign epoch
    pub size_bytes:    u32,
}
```

### PacketProtocol

| Value          | Description                         |
|----------------|-------------------------------------|
| `KakiNative`   | WAYv4 internal protocol (preferred) |
| `Tcp`          | TCP transport                       |
| `Udp`          | UDP transport                       |
| `Grpc`         | gRPC (future WpdEngine bridge)      |

---

## TrustState — B11 Ladder

| TrustState    | B11 Score | Behaviour                                         |
|---------------|-----------|---------------------------------------------------|
| `Golden`      | ≥ 200     | Fast-path allowed, full trust                     |
| `Active`      | ≥ 100     | Allowed with standard monitoring                  |
| `Suspicious`  | ≥ 59      | Quarantined with rate limiting (blocked after 3)  |
| `Blocked`     | < 59      | Denied unconditionally — Dead B11                 |
| `Unknown`     | —         | First-seen or SATTATU_MAX expired — quarantined   |
| `Revoked`     | —         | Permanently denied — overrides B11 rescoring      |

```rust
let state = TrustState::from_b11(220);  // → TrustState::Golden
let state = TrustState::from_b11(50);   // → TrustState::Blocked
```

---

## SATTATU_MAX

```rust
pub const SATTATU_MAX_SECS: u32 = 54 * 3600;  // 194,400 seconds = 54 hours
```

`Active` and `Golden` KAKIs that have not been seen for more than 54 hours are
downgraded to `Unknown` by `expire_stale()`.  Revoked entries are never touched.

---

## FirewallVerdict

```rust
pub enum FirewallVerdict {
    Allow     { tunnel_id: u32 },      // fast-path: FNV-1a(src ++ dst)
    Block     { reason: BlockReason },
    Quarantine{ reason: BlockReason },
}
```

### BlockReason

| Reason                          | Triggered by                               |
|---------------------------------|--------------------------------------------|
| `NoSourceKaki`                  | Packet carries no `src_kaki_hash`          |
| `InvalidChecksum`               | KAKI CRC-16 verification failed            |
| `RevokedKaki`                   | `revoke_kaki()` was called                 |
| `ExpiredKaki`                   | Not seen within `SATTATU_MAX`              |
| `DeadB11`                       | B11 < 59 → TrustState::Blocked             |
| `FuzzyRateLimited`              | Suspicious source exceeded 3 blocks        |
| `PolicyViolation(&'static str)` | Named policy rule denied the packet        |

---

## KakiFirewall — API

```rust
let mut fw = KakiFirewall::new();

// Register after BeeMDM B11 scoring
fw.register_kaki(kaki_hash, TrustState::Golden, now_epoch);

// Permanently revoke (cannot be un-revoked)
fw.revoke_kaki(kaki_hash, now_epoch);

// Inspect a packet
let verdict = fw.inspect(&packet, now_epoch);

// Sweep SATTATU_MAX expiry
fw.expire_stale(now_epoch);

// Read statistics
let stats: FirewallStats = fw.stats();

// Read event ring (capped at 1 000 entries)
let events: &[FirewallEvent] = fw.events();

// Lookup a trust entry
let entry: Option<&TrustEntry> = fw.trust_entry(kaki_hash);
```

### Inspection Decision Sequence

1. `src_kaki_hash = None` → `Block { NoSourceKaki }` — KAKI Dead Axiom
2. `src_kaki_hash` not in trust table → register as `Unknown` → `Quarantine`
3. `Revoked` → `Block { RevokedKaki }` (permanent)
4. `Blocked` → `Block { DeadB11 }`
5. `Unknown` → `Quarantine { NoSourceKaki }`
6. `Suspicious` → `Quarantine { FuzzyRateLimited }` (→ Block if `blocked_count > 3`)
7. `Active` / `Golden` → check SATTATU_MAX → `Allow { tunnel_id }`

---

## FirewallStats

```rust
pub struct FirewallStats {
    pub total_packets:  u64,
    pub allowed:        u64,
    pub blocked:        u64,
    pub quarantined:    u64,
    pub active_entries: usize,  // current trust table size
}
```

---

## Event Ring Buffer

`FirewallEvent` records are appended for every verdict.  The ring is capped at
**1 000 events** — oldest entries are evicted when full.

```rust
pub struct FirewallEvent {
    pub epoch:         u32,
    pub src_kaki_hash: u32,
    pub verdict:       FirewallVerdict,
    pub packet_size:   u32,
}
```

---

## See Also

- `crates/hepta-sec-policy/MANUAL.md`   — PolicyEngine, RateTracker, 8 sovereign default rules
- `crates/hepta-sec-sentinel/MANUAL.md` — HeptaSecSentinel, integrated inspection loop
- `crates/hepta-sec-web/MANUAL.md`      — HTTP boundary adapter (WebSentinelGuard)
- `crates/enkidb-kaki/MANUAL.md`        — uuid_hash() vs minted_id() security contract
