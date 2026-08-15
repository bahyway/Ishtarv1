# istar — Sovereign Access Control Engine
𒀭𒄑𒆳 *Ištar* — "divine gate / sovereign arbiter"

> **Layer 9.1 · Access Control** | 5 Constitutional Meta-Rules | ABAC | Pure Rust

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Crate

| Persona | Role |
|---|---|
| **musaru-security** (`crates/musaru-security`) | Runs firewall on every inbound pipeline packet |
| **adad-gate** (`crates/adad-gate`) | Validates `SargonPassport` scope before KAKI write |
| **permanent-storage** (`crates/permanent-storage`) | Blocks unauthorized cross-domain writes |
| **data-steward-station** | Enforces gem-quality requirement for admin operations |
| **AkkadianAOL runtime** (future) | Compiles `POLICY` / `FIREWALL` directives to `AkkRule` closures |
| **Akkadi CLI** (`bin/akkadi-cli`) | Evaluates trial access decisions in developer mode |

---

### WHAT — 𒁾 What This Tablet Contains

`istar` is the **sovereign ABAC (Attribute-Based Access Control) engine**.  
It compiles firewall rules from `.akk` policy files into Rust closures, evaluates them in  
priority order, and returns a `FirewallVerdict` before any data operation is executed.

#### FirewallVerdict

```
Allow     — operation permitted
Deny      — operation blocked
Escalate  — pass to human review (clearance threshold exceeded)
Redact    — allow access but remove sensitive fields
Audit     — permit AND write to sovereign audit trail
```

#### AkkContext — the full decision context

```
AkkContext {
    subject:   AkkSubject { id, domain, quality, tribe, clearance }
    resource:  AkkResource { path, domain, sensitivity }
    operation: AkkOperation { Read | Write | Execute | Delete | Admin }
    timestamp: u64  (sovereign_now())
}
```

#### Five Constitutional Meta-Rules (immutable, always evaluated first)

| Priority | Name | Law |
|---|---|---|
| 255 | `META-1:DeadQualityDeny` | `quality < 59` → **Deny** |
| 254 | `META-2:CrossDomainWrite` | Cross-domain Write + clearance < 0xA0 → **Deny** |
| 253 | `META-3:AdminGemOnly` | Admin operation + quality < 200 → **Deny** |
| 252 | `META-4:ClearanceEscalate` | `sensitivity > clearance` → **Escalate** |
| 1 | `META-5:AuditDestructive` | Delete or Admin → **Audit** (informational, continues) |

Meta-rules cannot be overridden or removed. User rules are evaluated after them (priority 0–251).

#### Gate builders

```rust
// Zero Trust gate — deny if quality below threshold
gates::zero_trust_gate(threshold: u8) -> AkkRule

// Sho (show) gate — redact if quality below threshold
gates::sho_redact_gate(threshold: u8) -> AkkRule

// Standard firewall with both gates
gates::standard_firewall(zero_threshold: u8, sho_threshold: u8) -> AkkFirewall
```

---

### WHEN — 𒌓 When Is This Invoked

```
Every data operation follows this sequence:

1. Caller assembles AkkContext (subject + resource + operation)
   │
   ▼
2. AkkFirewall::evaluate(ctx)
   │  — Rules sorted by priority (highest first)
   │  — META-1 through META-5 checked unconditionally
   │  — First non-None non-Audit verdict wins
   │  — If no rule matches → default Allow
   │
   ▼
3. Match FirewallVerdict
   ├── Allow     → proceed with operation
   ├── Deny      → return Err(IstarError::AccessDenied)
   ├── Escalate  → return Err(IstarError::EscalationRequired)
   ├── Redact    → proceed, but strip sensitive fields
   └── Audit     → proceed + write to sovereign audit trail
```

All pipeline writes go through `istar` evaluation.  
No write operation may bypass the firewall — this is enforced architecturally, not by convention.

---

### WHERE — 𒆳 Architectural Position

```
                        istar
                    ┌───────────────────────────────┐
                    │  5 Meta-Rules                 │
                    │  Composable AkkRule closures  │
                    │  Priority-ordered evaluation  │
                    └───────────────┬───────────────┘
                                    │ used by
         ┌──────────────────────────┼──────────────────────────┐
         │                          │                          │
   adad-gate                musaru-security            permanent-storage
(KAKI validation)          (pipeline gate)             (write guard)
         │
   [SargonPassport from kupru]
```

`istar` imports from `kupru` (for `SargonPassport` type awareness) but **does not perform crypto**  
— cryptographic verification is kupru's responsibility.

---

### WHY — 𒀊 Why This Exists

**The problem in v3.5:**  
Access control was scattered as `if quality > 100 { … }` blocks in 23 different crates.  
There was no central audit trail and no way to inspect the full policy set.

**The sovereign solution:**  
One engine, one evaluation loop, one trait. Every access decision passes through `istar`.  
The five Meta-Rules are constitutional — they protect the ecosystem even when a misconfigured  
user rule accidentally tries to grant too-broad access.

**Why closures instead of a DSL interpreter?**  
AkkadianAOL policy files compile to `AkkRule` Rust closures at startup.  
At evaluation time, there is no interpreter overhead — just a sorted `Vec` of function calls.

**Why "Zero" and "Sho" gate names?**  
"Zero Trust" → `Zero` gate (v4.0 — no "Way" suffix per ADR-002).  
"Show/Display" → `Sho` gate (Akkadian root for "to reveal").  
The gates are composable building blocks, not fixed policy.

**Why default Allow when no rule matches?**  
The five Meta-Rules already cover the most dangerous operations.  
A default Deny would block all operations until rules are explicitly written —  
impractical during development. Production systems add explicit Deny rules.

---

### HOW — 𒅗 How It Works

#### Basic usage

```rust
use istar::{AkkFirewall, AkkContext, AkkSubject, AkkResource, AkkOperation};

let fw = AkkFirewall::new();  // includes 5 Meta-Rules

let ctx = AkkContext {
    subject:   AkkSubject { id: "adad-gate".into(), domain: 0x03, quality: 185,
                            tribe: Some("water".into()), clearance: 0x50 },
    resource:  AkkResource { path: "enki/pm2_5".into(), domain: 0x03, sensitivity: 0x40 },
    operation: AkkOperation::Write,
    timestamp: istar::sovereign_now(),  // not exported — use SargonKdf::sovereign_now()
};

let verdict = fw.evaluate(&ctx);
// → FirewallVerdict::Allow  (quality=185 ≥ 59; same domain; clearance 0x50 ≥ sensitivity 0x40)
```

#### Adding custom rules

```rust
let mut fw = istar::gates::standard_firewall(100, 140);

fw.add_rule(AkkRule::new(
    "CustomRule:OilDomainReadOnly",
    200,  // priority (higher = checked first, after meta-rules)
    |ctx| {
        if ctx.resource.domain == 0x03 && ctx.operation == AkkOperation::Write {
            Some(FirewallVerdict::Deny)
        } else {
            None  // None = "I don't match, continue to next rule"
        }
    },
));
```

#### Handling verdicts

```rust
match fw.evaluate(&ctx) {
    FirewallVerdict::Allow    => proceed_with_write(),
    FirewallVerdict::Deny     => return Err(IstarError::AccessDenied("oil domain is read-only".into())),
    FirewallVerdict::Escalate => queue_for_human_review(ctx),
    FirewallVerdict::Redact   => proceed_with_field_stripping(),
    FirewallVerdict::Audit    => { proceed_with_write(); write_audit_trail(ctx); }
}
```

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 2 |
| Lines of Rust | ~280 |
| Constitutional meta-rules | **5** (immutable) |
| Verdict types | **5** |
| Rule evaluation | Priority-ordered, O(n rules) |
| Rule storage | Sorted `Vec<AkkRule>` |
| Context fields | Subject + Resource + Operation + Timestamp |
| External dependencies | serde, tracing |

---

## Sovereign Constraints

- `#![forbid(unsafe_code)]`
- Meta-rules are **always present** and cannot be removed — `AkkFirewall::new()` always includes them
- Rule evaluation is **deterministic**: same context + same rules = same verdict always
- `AkkRule` closures are `Send + Sync` — the firewall is safe across async boundaries
- `Audit` verdict is **informational only** — it does not block the operation; a real deny requires `Deny`
- "Way" suffix removed from all gate names: `Zero` (not `ZeroWay`), `Sho` (not `ShoWay`) — ADR-002

---

## Files

```
crates/istar/
├── Cargo.toml          (deps: serde, tracing)
└── src/
    ├── lib.rs           — crate root, IstarError, IstarResult, all re-exports
    └── akk_firewall.rs  — AkkFirewall, AkkRule, AkkContext, AkkSubject,
                           AkkResource, AkkOperation, FirewallVerdict,
                           meta_rules(), gates::{ zero_trust_gate, sho_redact_gate,
                           standard_firewall }
```
