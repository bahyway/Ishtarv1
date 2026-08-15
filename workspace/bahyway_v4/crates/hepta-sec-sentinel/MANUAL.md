# HeptaSecSentinel — Developer Manual

> **DubSar Help** | `Manuals > HeptaSecSentinel` | Crate Reference

## Overview

`hepta-sec-sentinel` is the sovereign stateful inspection sentinel.  It integrates
`KakiFirewall` and `PolicyEngine` into a single inspection loop and emits
`SecurityEvent`s for every blocked or quarantined decision.

This is the **single integration point** for production code — you do not need
to call `KakiFirewall` or `PolicyEngine` directly.

**Phases:**
- Phase 1 (current) — in-process library.
- Phase 2 — standalone EriduOS service with NajafEngine topology feed.
- Phase 3 — kernel-level WPDEngine packet inspection on bare metal.

---

## HeptaSecSentinel — API

```rust
use hepta_sec_sentinel::HeptaSecSentinel;

let mut sentinel = HeptaSecSentinel::new();

// Register a KAKI after BeeMDM B11 scoring
sentinel.register_kaki_b11(kaki_hash, b11_score, now_epoch);

// Permanently revoke a KAKI
sentinel.revoke_kaki(kaki_hash, now_epoch);

// Inspect a packet through the firewall + policy pipeline
let verdict: FirewallVerdict = sentinel.inspect_packet(&packet, now_epoch);

// Sweep SATTATU_MAX expiry (call periodically — every hour is sufficient)
sentinel.purge_stale(now_epoch);

// Read statistics
let stats: &SentinelStats = sentinel.stats();

// Read security event ring (capped at 500 entries)
let events: &[SecurityEvent] = sentinel.events();

// Access underlying firewall for test inspection
let fw: &KakiFirewall = sentinel.firewall();
```

---

## Inspection Pipeline

```
KakiPacket → KakiFirewall.inspect()
                 ↓
             Allow?  →  PolicyEngine.evaluate()
                            ↓ Deny?  → Block { reason }
                            ↓ Quarantine? → Quarantine
                            ↓ Permit / None → Allow (pass-through)
                 ↓
             Block / Quarantine → emit SecurityEvent
```

- Firewall verdicts for `Block` and `Quarantine` emit events immediately.
- `Allow` verdicts go through a secondary policy check — custom rules may
  downgrade an `Allow` to `Block` or `Quarantine`.
- No `SecurityEvent` is emitted for clean `Allow` verdicts.

---

## SecurityEventKind

| Variant                     | Triggered when                                     |
|-----------------------------|-----------------------------------------------------|
| `PacketBlocked(BlockReason)`| Firewall issued `Block` verdict                    |
| `PacketQuarantined(BlockReason)` | Firewall issued `Quarantine` verdict          |
| `NewDeviceDetected`         | First packet from an unknown KAKI                  |
| `KakiExpired`               | KAKI expired past SATTATU_MAX                      |
| `PolicyViolation`           | Named policy rule caused a block                   |
| `StalePurged(usize)`        | `purge_stale()` completed (count=0 in Phase 1)     |

---

## SentinelStats

```rust
pub struct SentinelStats {
    pub packets_inspected: u64,
    pub blocked:           u64,
    pub quarantined:       u64,
    pub allowed:           u64,
    pub events_emitted:    u64,
}
```

---

## register_kaki_b11

```rust
// B11 score → TrustState mapping
//   ≥ 200 → Golden    → fast-path allowed
//   ≥ 100 → Active    → standard monitoring
//   ≥  59 → Suspicious → quarantined
//   <  59 → Blocked   → denied

sentinel.register_kaki_b11(0xAAAA_0001, 220, now_epoch);  // Golden
sentinel.register_kaki_b11(0xBBBB_0002, 75,  now_epoch);  // Suspicious
sentinel.register_kaki_b11(0xCCCC_0003, 45,  now_epoch);  // Blocked
```

---

## purge_stale

Call `purge_stale(now_epoch)` periodically.  All entries not seen within
`SATTATU_MAX_SECS` (54 hours = 194 400 s) are downgraded from `Active`/`Golden`
to `Unknown`.  Revoked entries are never touched.

A `StalePurged` `SecurityEvent` is always emitted when `purge_stale()` runs.

---

## Event Ring

The event ring holds **500** entries.  Oldest entries are evicted when full.
Access via `sentinel.events()` — returns `&[SecurityEvent]`.

---

## Dependencies

- `hepta-sec-firewall` — `KakiFirewall`, `KakiPacket`, `FirewallVerdict`, `BlockReason`, `TrustState`
- `hepta-sec-policy` — `PolicyEngine`, `PolicyAction`

---

## See Also

- `crates/hepta-sec-firewall/MANUAL.md` — KakiFirewall, TrustState, SATTATU_MAX
- `crates/hepta-sec-policy/MANUAL.md`   — PolicyEngine, RateTracker, default sovereign rules
- `crates/hepta-sec-web/MANUAL.md`      — HTTP boundary adapter (WebSentinelGuard)
- `crates/enkidb-kaki/MANUAL.md`        — uuid_hash() contract for KakiPacket construction
