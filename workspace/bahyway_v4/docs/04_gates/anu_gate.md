# ANU Gate — Authority Exclusion

> **DubSar Help** | `ANU` | Gates

## Akkadian Identity

ANU (An) — the Sky Father, Supreme Authority, source of all kingship. He sits
at the highest energy level and determines who has the "Right to Rule."

## Purpose

ANU is Gate 2. It governs multi-source authority: when a CRM, a Power Grid
sensor, and an ERP all claim the same Identity-KAKI, ANU determines which
source occupies the Master Position (n = 0) in the Jordan Block.

## Mechanism

- Each signal carries a `source_rank` (authority score).
- `ANU_AUTHORITY_RANK` defines the minimum rank required to occupy the Master
  Position.
- High-authority source: Active shell (Conduction Band).
- Low-authority source: Excluded to Valence Band (Historical/Backup orbit).
- CrossTribe-KAKI relationships are governed by ANU when two Tribes conflict
  over the same Identity-KAKI.

## Rust Gate Constant

```rust
pub const ANU_MIN_AUTHORITY: u32 = 100;
```

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
ANU never overwrites a higher-authority value with a lower-authority one.

## See Also

- `04_gates/high_council.md`
- `02_identity/crosstribe_kaki.md`
