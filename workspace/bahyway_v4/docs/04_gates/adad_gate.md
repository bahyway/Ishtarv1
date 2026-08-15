# ADAD Gate — Temporal Exclusion

> **DubSar Help** | `ADAD` | Gates

## Akkadian Identity

ADAD (also Adad or Hadad) — the Lord of Wind, Storm, and Decree. Controls the
"air" through which signals enter the system. He decides when a signal is
allowed into the lungs of the database.

## Purpose

ADAD is Gate 1. It de-duplicates rapid-fire signals for the same Identity-KAKI
to prevent "Data Flapping" where the Golden Record switches between two
identical-looking values faster than the system can stabilise.

## Mechanism

- **Temporal window**: `ADAD_BREATH_MS` (configurable; default 50 ms).
- If two signals for the same Identity-KAKI arrive within the window, ADAD
  promotes one to the Active Orbit and queues the other in the Temporal Buffer
  (next shell).
- Combined with the Bloom Filter pre-check, existence tests are O(1) before
  any Jordan Block lookup is attempted.

## Rust Gate Constant

```rust
pub const ADAD_BREATH_MS: u64 = 50;
```

## Sovereign Constraints

ADAD operates at the signal input layer, before any particle state is written.
It does not modify existing particles — it only controls admission.

## See Also

- `04_gates/high_council.md`
- `02_identity/events_kaki.md`
- `01_mathematics/tri_kaki_index.md`
