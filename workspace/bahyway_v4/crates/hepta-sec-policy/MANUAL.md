# HeptaSecPolicy — Developer Manual

> **DubSar Help** | `Manuals > HeptaSecPolicy` | Crate Reference

## Overview

`hepta-sec-policy` is the WAYv4.0 security policy engine.  It evaluates an ordered
set of `PolicyRule` structs against `KakiPacket` + optional `TrustEntry` pairs and
returns the first matching rule's `PolicyAction`.

Policy rules are intended to be generated from `.akk` HeptaScript GUARD blocks in
Phase 2.  Phase 1 provides a hardcoded `default_rules()` set.

---

## PolicyPriority

| Value      | Numeric | Precedence                                          |
|------------|---------|-----------------------------------------------------|
| `Critical` | 1       | Highest — absolute deny/allow (no-KAKI, revoked)    |
| `High`     | 2       | Identity-verified denials (Dead B11, unknown source)|
| `Normal`   | 3       | Conditional quarantine / permit rules               |
| `Low`      | 4       | Lowest — catch-all default-deny                     |

Rules are sorted ascending by `priority.value()` before evaluation.
**First match wins.**

---

## PolicyCondition

| Variant                              | Description                                   |
|--------------------------------------|-----------------------------------------------|
| `SourceKakiAbsent`                   | Packet carries no `src_kaki_hash`             |
| `SourceTrustState(TrustState)`       | Source KAKI in specified trust state          |
| `DestTrustState(TrustState)`         | Destination KAKI in specified trust state     |
| `ProtocolIs(PacketProtocol)`         | Packet uses given protocol                    |
| `PacketSizeExceeds(u32)`             | Packet size (bytes) exceeds threshold         |
| `RateExceeds { packets_per_second }` | Rate exceeds N pkt/s — live via `evaluate_with_rate()` |

Multiple conditions in a rule are evaluated as **logical AND**.

---

## PolicyAction

| Variant                           | Effect                                       |
|-----------------------------------|----------------------------------------------|
| `Deny(BlockReason)`               | Drop packet with reason                      |
| `Permit`                          | Allow packet through                         |
| `QuarantineAndAlert(&'static str)`| Hold packet and emit audit alert             |

---

## PolicyEngine — API

```rust
use hepta_sec_policy::{PolicyEngine, PolicyRule, PolicyCondition, PolicyAction, PolicyPriority,
                       RateTracker};
use hepta_sec_firewall::{BlockReason, TrustState};

// Empty engine
let mut engine = PolicyEngine::new();

// Add a rule — list is re-sorted after each add
engine.add_rule(PolicyRule {
    name:       "block_large_packets",
    priority:   PolicyPriority::Normal,
    conditions: vec![PolicyCondition::PacketSizeExceeds(65_535)],
    action:     PolicyAction::Deny(BlockReason::PolicyViolation("oversized_packet")),
});

// Stateless evaluate (RateExceeds always false)
let trust = firewall.trust_entry(src_hash);
match engine.evaluate(&packet, trust) {
    Some(PolicyAction::Permit)                  => { /* allow */ }
    Some(PolicyAction::Deny(reason))            => { /* block */ }
    Some(PolicyAction::QuarantineAndAlert(msg)) => { /* alert */ }
    None                                        => { /* no rule matched — default-deny posture */ }
}

// Stateful evaluate — RateExceeds is live
let mut tracker = RateTracker::new();
match engine.evaluate_with_rate(&packet, trust, &mut tracker, now_epoch) {
    Some(PolicyAction::Deny(_)) => { /* rate exceeded or other deny */ }
    Some(PolicyAction::Permit)  => { /* allowed */ }
    _                           => {}
}
```

---

## Default Sovereign Rules

`PolicyEngine::default_rules()` loads all 8 sovereign rules in priority order:

| # | Name                        | Priority | Condition                     | Action                        |
|---|------------------------------|----------|-------------------------------|-------------------------------|
| 1 | `block_no_kaki`             | Critical | SourceKakiAbsent              | Deny(NoSourceKaki)            |
| 2 | `block_revoked_kaki`        | Critical | SourceTrustState(Revoked)     | Deny(RevokedKaki)             |
| 3 | `block_dead_b11`            | High     | SourceTrustState(Blocked)     | Deny(DeadB11)                 |
| 4 | `quarantine_unknown_source` | High     | SourceTrustState(Unknown)     | QuarantineAndAlert(...)       |
| 5 | `quarantine_suspicious`     | Normal   | SourceTrustState(Suspicious)  | QuarantineAndAlert(...)       |
| 6 | `permit_active_source`      | Normal   | SourceTrustState(Active)      | Permit                        |
| 7 | `permit_golden_source`      | Normal   | SourceTrustState(Golden)      | Permit                        |
| 8 | `default_deny_all`          | Low      | *(empty — always matches)*    | Deny(PolicyViolation(...))    |

**Default posture: deny-all unless a rule explicitly permits.**

---

## RateTracker — Per-KAKI Rate Limiting

`RateTracker` implements a one-second sliding window counter keyed by
`kaki_hash`.  Pass one shared instance to `evaluate_with_rate()` on every
packet inspection call.

```rust
use hepta_sec_policy::RateTracker;

let mut tracker = RateTracker::new();

// Direct usage (without PolicyEngine):
let exceeded = tracker.record_and_check(
    kaki_hash,          // u32 — source KAKI hash
    100,                // u32 — threshold: packets per second
    now_epoch,          // u32 — current epoch (seconds)
);

// Purge windows for epochs other than now_epoch (bound memory use):
tracker.purge_stale(now_epoch);
```

**Window semantics:** All packets received within the same `now_epoch` second
are counted together.  When `now_epoch` advances, the counter resets.  A
packet is rate-exceeded when its count *exceeds* (not equals) the threshold.

### Adding a Rate Limit Rule

```rust
use hepta_sec_policy::{PolicyRule, PolicyCondition, PolicyAction, PolicyPriority};
use hepta_sec_firewall::BlockReason;

engine.add_rule(PolicyRule {
    name:       "rate_limit_web_api",
    priority:   PolicyPriority::Critical,
    conditions: vec![PolicyCondition::RateExceeds { packets_per_second: 120 }],
    action:     PolicyAction::Deny(BlockReason::PolicyViolation("rate_exceeded")),
});
```

---

## Custom Rules

Rules can be added on top of `default_rules()` or composed from scratch.
Insert at any priority; the engine re-sorts on `add_rule()`.

```rust
let mut engine = PolicyEngine::default_rules();

// Override: allow Grpc protocol from Golden sources even if oversized
engine.add_rule(PolicyRule {
    name:       "permit_grpc_golden",
    priority:   PolicyPriority::Critical,
    conditions: vec![
        PolicyCondition::ProtocolIs(PacketProtocol::Grpc),
        PolicyCondition::SourceTrustState(TrustState::Golden),
    ],
    action: PolicyAction::Permit,
});
```

---

## evaluate() vs evaluate_with_rate()

| Method                  | `RateExceeds` condition | Requires             |
|-------------------------|-------------------------|----------------------|
| `evaluate()`            | Always `false`          | Nothing extra        |
| `evaluate_with_rate()`  | Live (1-second window)  | `&mut RateTracker`   |

Use `evaluate()` when rate limiting is handled externally (nginx, proxy).
Use `evaluate_with_rate()` when rate limiting must be enforced in-process.

---

## Dependencies

- `hepta-sec-firewall` — `KakiPacket`, `TrustEntry`, `TrustState`, `BlockReason`, `PacketProtocol`

---

## See Also

- `crates/hepta-sec-firewall/MANUAL.md`  — KakiFirewall, TrustState, SATTATU_MAX
- `crates/hepta-sec-sentinel/MANUAL.md`  — HeptaSecSentinel integrated pipeline
- `crates/hepta-sec-web/MANUAL.md`       — HTTP adapter using WebSentinelGuard
- `policies/heptasec_web_access.akk`     — WAYv4.0 rate-limit policy template
