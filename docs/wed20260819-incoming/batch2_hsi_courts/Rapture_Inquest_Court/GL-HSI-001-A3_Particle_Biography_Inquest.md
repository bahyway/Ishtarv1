# GL-HSI-001-A3 — The Particle Biography & The Rupture Inquest
## A rupture without an inquest is spectacle; the court must name the first mover

**Ecosystem:** BahyWay.Ecosystem v4.0
**Amends:** GL-HSI-001-A2 (Rupture Ceiling) · consumes GL-NSR-001-A2 (Rigmu Inquest: W5H2 frame,
wrong-cause refusal, no-silent-close), GL-STY-001 (StoryEngine), GL-AGE-001 (Šību witnesses)
**Status:** DRAFT — awaiting Architect seal (CSR-08)

---

## 1. The Biography Law

Every born particle carries a **per-epoch trajectory record** — r(t) in PU,
shell index, and flags — from which **Event KAKIs** are minted at law-defined
moments (kaki_type = Event, 0x02; the biography is particles about particles):

| Event | minted when | the story it tells |
|---|---|---|
| **BIRTH** | mask-cut | KAKI minted; shell and radius at birth recorded |
| **INFLECTION** | first epoch where the second difference of r(t) is positive and stays positive for 2 consecutive epochs | *the curving moment* — the radius leaves its law |
| **SHELL-CROSS** | r(t) crosses a sealed shell boundary | geometric layer shift, direction recorded (outward = pressure, inward = decay) |
| **DECAY-ONSET** | first epoch of sustained negative dr (2 consecutive) | shrinking — particle decay, not membrane pressure |
| **RUPTURE-WITNESS** | S reaches 1 while the particle lies within the tear radius | present at the opening; its ledger becomes evidence |

The StoryEngine journal is the *readable rendering* of this ledger — clicking
a particle (NASARU bounce) opens its biography; the biography is derived from
Event KAKIs, never from animation state (GL-DST-001: the theater is stage,
never truth).

## 2. The Shell Law

Shells are sealed radius bands in PU (default: L1 < 1.5 · L2 1.5–2.5 ·
L3 ≥ 2.5; per-corridor override by decree only). A SHELL-CROSS is an event,
not a redraw: crossing times are computed from the trajectory record, so the
same sealed history yields the same crossings forever (deterministic replay).

## 3. The Inquest Law

On rupture, the court opens an **inquest** and may not close without it
(no-silent-close, Rigmu kinship). The inquest is derived purely from the
ledger:

- **Causal order** = particles sorted by INFLECTION epoch (ties broken by
  particle index — deterministic).
- **The first mover** = earliest INFLECTION among particles whose shell path
  reached the tear region. Decay-only particles are *exculpated*: shrinking
  cannot open a wall.
- **The W5H2 finding**: WHO (first-mover KAKI) · WHAT (inflection) · WHEN
  (the curving moment, epoch-exact) · WHERE (joint + shell path) · WHY
  (pressure vs decay, from event types) · HOW (curvature magnitude) ·
  HOW-MUCH (Δr in PU and OU).
- **Wrong-cause refusal**: a proposed closing that names any particle other
  than the ledger's first mover is REFUSED with the citation, and the inquest
  stays open. A decay particle proposed as cause is refused with the
  exculpation clause.

## 4. The Replay Law

The court owns a **timeline**: any past instant may be re-viewed, and the
view at time t is recomputed from sealed trajectories — never cached frames
(a judgment cites the epoch, never the cache). Scrubbing is lawful; bending
is not: the ledger is append-only and the past does not change under review.

## 5. Law Tests (sealed with PB-349)

- **L-HSI-16** — inflection minted at the true onset epoch on a synthetic
  staggered-quadratic ramp; NEVER minted on linear growth or pure noise
- **L-HSI-17** — shell-cross events fire exactly at boundary crossings, with
  correct direction; count matches analytic crossings
- **L-HSI-18** — decay-onset detected on sustained shrink; a decayer never
  receives INFLECTION from its own shrink curvature
- **L-HSI-19** — inquest ordering deterministic; first mover = earliest
  qualifying inflection; ties broken by index, bit-stable across runs
- **L-HSI-20** — no-silent-close: closing without an inquest report, or with
  a wrong-cause candidate, returns REFUSED with citation; only the ledger's
  first mover closes the inquest

## 6. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
