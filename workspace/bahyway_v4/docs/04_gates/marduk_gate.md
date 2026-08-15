# MARDUK Gate — Structural Exclusion

> **DubSar Help** | `MARDUK` | Gates

## Akkadian Identity

MARDUK — The Architect of Order, who defeated Tiamat (chaos) and structured the
universe. Master of the "Order of Things."

## Purpose

MARDUK is Gate 3. It manages transformation locks: if a particle is currently
being cleansed by the VGCA Delta engine, MARDUK prevents simultaneous manual
edits or automated updates from creating a race condition.

## Mechanism

- **Governance Lock**: when a particle enters the Cleansing State, MARDUK sets
  an exclusive lock on that (Identity-KAKI, Orbit-Level) coordinate.
- **Human authority wins**: if a Data Steward edits a particle already locked
  by the automated pipeline, MARDUK pauses the pipeline (lower quantum spin)
  and elevates the Steward's edit (higher quantum spin).
- **Noise threshold**: `MARDUK_ORDER_INDEX` — if the VGCA delta exceeds this
  threshold, the particle is sent to the Stewardship Gap instead of being
  passed through.

## Rust Gate Constant

```rust
pub const MARDUK_ORDER_INDEX: f64 = 0.35;
```

## Sovereign Constraints

While MARDUK holds a lock, no other process may modify the locked particle.
Locks are held for the minimum duration of the transformation pipeline stage.

## See Also

- `04_gates/high_council.md`
- `04_gates/shamash_gate.md`
- `06_governance_parzu/parzu_laws.md`
