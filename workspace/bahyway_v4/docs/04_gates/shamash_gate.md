# SHAMASH Gate — State Exclusion

> **DubSar Help** | `SHAMASH` | Gates

## Akkadian Identity

SHAMASH (Šamaš) — The Sun God and Judge of Truth. He sees all, brings hidden
things to light, and judges both the living and the dead.

## Purpose

SHAMASH is Gate 4. It guards the boundary between the Active Orbit and the Dead
State. It prevents "Zombie" data — a signal for a Dead particle that should not
be reactivated — from corrupting the Active Orbit.

## Mechanism

- **Dead particle detection**: if an incoming signal's Identity-KAKI matches a
  record in the Dead State, SHAMASH intercepts it.
- **Steward approval required**: the signal is placed in the Stewardship Gap.
  The Data Steward must decide:
  - **Zombie**: the signal is a mistake → reject.
  - **Reincarnation**: the sensor/entity was replaced → approve and re-mint
    a new particle with a new Events-KAKI lineage.
- **Judgment score**: `SHAMASH_JUDGMENT_SCORE` — the confidence threshold
  required for automatic reincarnation without Steward intervention.
- **Spectral archeology**: Dead particles are never deleted; they remain in
  the Ground State as mathematical barriers and audit records.

## Rust Gate Constant

```rust
pub const SHAMASH_JUDGMENT_SCORE: f64 = 0.95;
```

## Sovereign Constraints

A Dead particle is never overwritten. Its archived state is permanent and
queryable by the storytelling system (StoryWay).

## See Also

- `04_gates/high_council.md`
- `04_gates/pauli_exclusion.md`
- `09_observatory/orbital_visualization.md`
