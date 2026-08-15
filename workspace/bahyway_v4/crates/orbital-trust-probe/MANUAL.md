# 𒁾 orbital-trust-probe — Manual
**Version:** 4.0.2 | **Layer:** 4.5 — Orbital Intelligence
**W5H2 Transparency Framework**
**Author:** Bahaa Fadam — BahyWay Sovereign Ecosystem

---

## What It Solves

At billion-particle scale, particles are in **constant legitimate motion**. They
move through 7D orbital space when their data evolves, when neighbouring
particles shift the SPH density field, when freshness decays over time, or when
the fuzzy scoring rules are updated. Without causal attribution, any monitoring
system would generate a continuous stream of false-positive trust alarms.

The naive approach — "flag any particle that changed orbital position" — produces
a **trust-penalty cascade**: penalising P₁ lowers its SourceTrust score (D6),
which lowers its B11 byte, which changes its ring, which makes its neighbours
appear deviant, which flags them too. One false positive becomes an exponential
cluster collapse.

**`orbital-trust-probe` solves this by requiring causal attribution before any
trust penalty is issued.**

---

## How It Works — The 4-Step Pipeline

```
After each score-engine pass:
  OrbitalSnapshot(prev) + OrbitalSnapshot(curr) + StoryEngine event count delta
      ↓
  Step 1 — FuzzyRules fingerprint changed?
              YES → DeviationCause::FuzzyRulesChanged → ABORT (no penalty)
              NO  → continue
      ↓
  Step 2 — New EAV journal events between epochs?
              YES → DeviationCause::LegitimateStateEvolution → log only
              NO  → continue
      ↓
  Step 3 — ScoreEngine field explains the move?
              3a. Neighbour count delta ≥ 2?  → NeighborDensityShift → log only
              3b. Freshness BLUE byte drop ≥ 15?  → FreshnessDecay → log only
              3c. B11 within ±5 of ring boundary? → ThresholdBoundaryNoise → log only
              All NO → continue
      ↓
  Step 4 — No explanation found
              → DeviationCause::Unexplained
              → OrbitalDeviationJournal.append(entry, CRC-16 sealed)
              → trust_penalty = f(cartesian_delta, ring_changed)
              → fed back to FuzzyDimensions.d9_orbital_trust_penalty
              → fuzzy-engine degrades D6 effective score
              → B11 drops on next scoring cycle
              → orbital position self-corrects without external intervention
```

Only `Unexplained` deviations become sovereign audit events. The other five
causes are recorded as **observability context** for the distributed density
dashboard — not security events.

---

## Closed Feedback Loop

```
  OrbitalDeviationJournal
    │  accumulated_penalty(particle_id)  →  [0.0, 1.0]
    ▼
  ScoreInput.orbital_trust_penalty
    │  written into FuzzyDimensions.d9_orbital_trust_penalty
    ▼
  FuzzyDimensions.fuzzify()
    │  effective D6 = d6_source_trust.with_orbital_penalty(d9)
    │  degraded = base - penalty × (base - 0.10)
    ▼
  FuzzyEngine.score() → lower B11 byte
    ▼
  ring_from_b11() → particle drifts to correct ring organically
    ▼
  Next cycle: OrbitalSnapshot shows legitimate evolution → no new penalty
```

The loop is **self-terminating**: once a particle's B11 drops to match its
observed ring, the next probe cycle attributes the position to
`LegitimateStateEvolution` and no further penalty is issued. Recovery is
automatic when the underlying data quality improves.

---

## W5H2 Reference

| W5H2 | Answer |
|------|--------|
| **Who** | Built by `orbital-trust-probe`. Consumed by the scoring pipeline (score-engine), eridu-supervisor, and the DubSar observability dashboard. |
| **What** | Determines whether an orbital position change is a legitimate physics event or an unexplained trust violation, and feeds the result back into the fuzzy scoring loop. |
| **When** | After every `score-engine` pass, before the next `tribe-orbit-engine` assignment. Runs per-particle or in batch via `batch_probe()`. |
| **Where** | `crates/orbital-trust-probe/` — Layer 4.5 (Orbital Intelligence), sits between score-engine and tribe-orbit-engine. |
| **Why** | Prevents false-positive trust cascades at billion-particle scale. Distinguishes the five legitimate causes of orbital motion from genuine unexplained deviation. |
| **How** | 4-step causal attribution pipeline: rules fingerprint → StoryEngine EAV delta → ScoreEngine field analysis → unexplained residual. CRC-16 sealed OrbitalDeviationJournal. Penalty fed back into FuzzyDimensions.d9. |
| **How Much** | **22 tests** · 5 source files · 840 lines · `forbid(unsafe_code)` · 0 external deps |

---

## Key Types

### `OrbitalSnapshot`
Point-in-time capture of a particle's orbital state after one scoring cycle.

```rust
pub struct OrbitalSnapshot {
    pub epoch:             u64,            // scoring cycle epoch
    pub b11:               u8,             // quality byte that drove ring assignment
    pub tier:              QualityTier,    // Gem / Tribe / Active / NonActive
    pub ring:              OrbitalRing,    // Inner / Mid / Outer
    pub assignment:        OrbitalAssignment, // full azimuth/altitude/radius
    pub neighbour_count:   usize,          // SPH neighbour count at snapshot time
    pub freshness_byte:    u8,             // BLUE byte from score-engine
    pub rules_fingerprint: u64,            // FNV-1a hash of rule constants
}
```

### `DeviationCause`

```rust
pub enum DeviationCause {
    FuzzyRulesChanged,          // Step 1 — abort, no log
    LegitimateStateEvolution,   // Step 2 — EAV events found
    NeighborDensityShift,       // Step 3a — SPH field moved
    FreshnessDecay,             // Step 3b — BLUE byte dropped
    ThresholdBoundaryNoise,     // Step 3c — B11 near ring boundary
    Unexplained,                // Step 4 — genuine trust signal
}
```

### `OrbitalDeviationJournal`
Append-only, CRC-16 sealed, deduplicated evidence record. Parallel to
`AuditJournal` in `enkidullm-audit`.

```rust
journal.accumulated_penalty(particle_id)  // → f32 in [0.0, 1.0]
```

---

## Ring Boundaries and Hysteresis

| Boundary | B11 value | HYSTERESIS band |
|---|---|---|
| Inner → Mid | 200 | 195–205 |
| Mid → Outer | 100 | 95–105 |

Particles with B11 in a hysteresis band always receive `ThresholdBoundaryNoise`
attribution, never a trust penalty. This prevents oscillation artefacts when a
particle's score fluctuates near a discrete boundary.

---

## Trust Penalty Formula

```
cartesian_delta = Euclidean distance between prev and curr orbital Cartesian positions
base_penalty    = clamp(delta / 4.0, 0.0, 0.5)
trust_penalty   = ring_changed ? clamp(base_penalty × 2.0, 0.0, 1.0) : base_penalty
```

A ring change doubles the penalty weight because crossing a ring boundary signals
a qualitative state change, not just positional drift.

---

## Integration Example

```rust
use orbital_trust_probe::{OrbitalSnapshot, ProbeInput, probe_and_journal,
                           OrbitalDeviationJournal};
use score_engine::{ScoreInput, score};
use fuzzy_engine::FuzzyDimensions;

// After two consecutive scoring cycles:
let mut journal = OrbitalDeviationJournal::new();
let input = ProbeInput { previous: &snap_prev, current: &snap_curr, new_eav_events: 0 };
let result = probe_and_journal(&input, particle_id, &mut journal);

// Feed penalty into next scoring cycle:
let penalty  = journal.accumulated_penalty(particle_id);
let mut dims = FuzzyDimensions::from_eav(&particle_eav);
// penalty is applied automatically via ScoreInput:
let score_result = score(&ScoreInput::civil_with_penalty(dims, domain_byte, elapsed, penalty));
```

---

## Sovereign Constants

| Constant | Value | Meaning |
|---|---|---|
| `MIN_DEVIATION_DISTANCE` | 0.05 | Orbit-space units below which no attribution is needed |
| `FRESHNESS_DECAY_THRESHOLD` | 15 | BLUE byte drop that explains a ring crossing on its own |
| `NEIGHBOUR_DELTA_THRESHOLD` | 2 | Neighbour count change considered a meaningful density shift |
| `HYSTERESIS` | 5 | B11 units either side of a ring boundary treated as noise |
