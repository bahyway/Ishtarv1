# 𒁾 MAN-BR-001 — THE BĀRÛTU MANUAL & GLOSSARY
### Operating the Problems Generator with HeptaScript · Building Analyzing Tools · Sovereign Data Mining · Particle Data Modeling
### Status: DRAFT — companion to GL-GEN-001 (proposed NIPPUR 4.6) · unsealed until the Architect's ceremony (CSR-08)
### Implementation playbook designation proposed: PB-302 (after UttuEngine PB-300, ShalaEngine PB-301)

---

## Part 0 — What This Manual Is

BĀRÛTU 𒌉𒈬 is the ecosystem's Problems Generator: the diviner's combinatorial lattice,
redeemed with falsifiability. It mints **hypothesis particles** — problems that do not exist
yet, born with their instruments strapped on — and holds them InFlight (the TIAMAT twinship)
until real streaming density DETECTs and an independent witness PROVEs an instance.

This manual is written for three readers, per the Three Gates:

- **Ṭupšarru ṣeḫru** (the young scribe) — reads Parts I and VII, operates the Šala tab,
  and learns why the padlock on PRESCRIBE is the most important pixel on the screen.
- **Engineer** — reads Parts II–IV, writes HS-EXT-004 clauses, builds instruments and
  mining pipelines.
- **Architect** — reads everything, and alone performs promotion, demotion, sealing, and
  axis amendment.

One rule stands over the whole book: **the generator proposes; the staircase disposes.**

---

## Part I — Concepts

### I.1 The Omen Table

The primary face of BĀRÛTU is a 7×7 lattice:

- **Rows — the Seven Rhythms**, each guarded by its dial:
  RING (S₁) · TWO POLES (S₂) · THREE-FOLD (S₃) · FOUR SQUARE (S₄) ·
  FIVE SUN-TIMED (S₅) · SIX WITH A VOID (S₆) · SEVEN OF THE WEEK (S₇)
- **Columns — the Seven Lawful Deviations**:
  COLLAPSE · SECOND TRIBE · KOMMA DETUNE · MIGRATING VOID · RATE-LAW BREAK ·
  LOPSIDED SURGE · RADIAL ESCAPE

The full generative product extends the table with three more axes chosen at the Mint:
**URUK register** (which counting system owns the domain — GL-MET-001),
**opacity sin** (which of the seven ways the record may lie — Zibānītu daily cases v2),
and **bridge status** (which archaic ratio is implicated, honored or dishonored).

Cell states: **occupied** (history — calibration, marked 𒁾), **sacred void** (no coherent
omen forms; drawn dark, honored empty), **empty** (a prediction waiting), **hypothesis**
(⟡ — minted, InFlight). The table renders in two faces — tablet grid and orbit — sharing
one state.

### I.2 The Hypothesis Particle Lifecycle

```
        MINT ──► INFLIGHT ──► WATCH ──► DETECT ──► PROVE ──► PROMOTE
                    │                                            │
                    │ (ages without detection)                   └─► occupied cell +
                    ▼                                                sealed divination
                 ARKÛ ARCHIVE (NUZI)                                 trail beside the
                 recoverable by Gap-list rite                        detection KAKI
```

- **MINT** — a BR-numbered particle is created from a chosen cell + Mint axes. It receives
  at birth: lattice coordinates, guardian dial, draft Θ, τ decomposition, cited laws, rank,
  and a divination trail (who cast it, when, from which axes).
- **INFLIGHT** — the TIAMAT discipline: the particle is real, KAKI-bearing,
  Hepta-positioned, and uncommitted. It may be studied, ranked, and instrumented.
  It may not prescribe.
- **WATCH** — an instrument (a Zibānītu gauge over a stream) is assigned. Watching is
  passive: gauges, watermarks, windows. Never intervention.
- **DETECT** — real streaming density crosses the sealed τ threshold with sustain.
- **PROVE** — a second, independent witness corroborates (Shakkanakku two-instrument rule).
- **PROMOTE** — the Architect seals it: the cell becomes occupied, the trail is sealed
  beside the detection KAKI, and the record shows the table predicted before the world
  performed (the Mendeleev event).
- **ARKÛ demotion** — unpromoted hypotheses age into the NUZI archive with full lineage.
  Nothing is deleted; the Gap-list rite can recall them if the world later moves.

### I.3 The Two Disciplines (never waived)

1. **No prescription before promotion.** PRESCRIBE is padlocked for every InFlight
   particle, in every lens, in every export.
2. **The Void is Sacred in problem-space.** Declared voids are never minted from. A
   petition to un-void a cell is a law amendment (lineage particle on GL-GEN-001),
   Architect-only.

---

## Part II — HeptaScript Integration (HS-EXT-004, DRAFT)

HS-EXT-004 proposes the BĀRÛTU clause family for HeptaScript v1.2+. It follows the
established extension style of HS-EXT-002 (ORBIT PAIR / PROVE PAIR) and HS-EXT-003
(CUBE COUNT). **HeptaScript is Anti-SQL**: no foreign query idioms appear below or ever;
these are sovereign constructs over tribes, orbits, and particles.

> All syntax in this Part is DRAFT grammar awaiting the Architect's seal at Gate G4.

### II.1 Casting an omen (declaring the cell)

```heptascript
OMEN CAST
  RHYTHM    FOUR SQUARE            // guardian dial binds automatically: S4
  DEVIATION KOMMA DETUNE
  REGISTER  U                      // URUK: the Seasons — time, oversight windows
  SIN       CLOCK FOR SUN
  BRIDGE    KOMMA DISHONORED
YIELD OMEN oversight_drift
```

`OMEN CAST` names a coordinate in the product. It creates nothing durable — it is the
stylus hovering over the clay.

### II.2 Minting the hypothesis particle

```heptascript
MINT HYPOTHESIS FROM OMEN oversight_drift
  RANK BY COHERENCE NOVELTY STAKES
  TRAIL SEALED                       // divination trail: caster, date, axes
YIELD PARTICLE BR-0007 INFLIGHT
```

Minting emits the KAKI **only through enkidb-ingest::bridge** — never by hand, per
standing law. The particle takes a unique Hepta Space position (Uniqueness Law) and
enters the InFlight shard beside TIAMAT's predicted particles.

### II.3 Strapping the instrument

```heptascript
STRAP INSTRUMENT ON BR-0007
  DIAL      S4
  TEMPLATE  THETA FROM STATION RITE    // Θ is sealed with a stakeholder, or stays DRAFT
  TAU       HARM 0.45  KEEP 0.35  MISS 0.20  EPSILON DECLARED
  TOLERANCE SEALED 0.22 SUSTAIN 28
```

An instrument without a Station-rite Θ carries `TEMPLATE DRAFT` and cannot DETECT —
it can only rehearse. This is the undeclared-template sin, blocked at the grammar.

### II.4 Watching a stream

```heptascript
WATCH STREAM oversight_events
  THROUGH BR-0007
  WINDOW AGING 240
  WATERMARK ADANNU LAG 6
  ARKU LAWFUL TOLERANCE SEALED
  MILU LADDER BUFFER SHED THROTTLE     // never a silent drop
```

`WATCH` binds the instrument to a PALGU-conformant stream. All four river-laws apply by
name; omitting the MĪLU ladder is a parse error, not a warning.

### II.5 Detection, proof, promotion

```heptascript
ON BR-0007 WHEN TAU CROSSES SEALED WITH SUSTAIN
  RAISE DETECT                          // rung 1 lights; nothing else happens

PROVE PAIR BR-0007 WITH WITNESS second_instrument
  RESONANCE REQUIRED TOLERANCE SEALED   // HS-EXT-002 grammar, reused as the 2nd witness

PROMOTE BR-0007
  BY ARCHITECT SEAL CSR-08
  OCCUPY CELL  SEAL TRAIL BESIDE DETECTION KAKI
```

`PROMOTE` fails at parse time without `BY ARCHITECT SEAL`. There is no programmatic
promotion; the ceremony is the security model.

### II.6 Demotion and recall

```heptascript
DEMOTE BR-0004 TO NUZI AS ARKU HYPOTHESIS   // aging, not deletion
RECALL FROM NUZI BY GAP LIST WHERE-- (forbidden idiom removed)
RECALL FROM NUZI BY GAP LIST MATCHING OMEN oversight_drift   // sovereign form
```

### II.7 Counting over the table

```heptascript
CUBE COUNT HYPOTHESES ACROSS RHYTHM DEVIATION      // HS-EXT-003, unchanged
CUBE COUNT PROMOTIONS ACROSS REGISTER SINCE SEAL zagesi_one
```

---

## Part III — Building New Analyzing Tools on BĀRÛTU

A new tool is an **instrument** until it earns Engine-hood. The path:

**Step 1 — Choose the coordinate, not the code.** Every tool begins as an omen: which
rhythm, which deviation, which register does it watch? A tool that cannot state its cell
is a dashboard, and Way of Work 5 has opinions about dashboards.

**Step 2 — Define the instrument.** A BahyWay instrument is exactly four things:
a guardian dial (or a new order parameter — see Growth, Part VI), a Θ source (Station
rite), a τ decomposition with declared ε, and a verdict vocabulary (SEALED CLEAR /
ADVISORY / PARZU 0x03) with plain-words lines for the Three Gates audiences.

**Step 3 — Prove at Gate G4.** Design-time only: Z3/Lean4/BahyWay-Algebra obligations —
the Θ contract, the watermark rule, replay determinism, and (recommended) a τ-stability
bound: bounded input perturbation ⇒ bounded verdict drift. Z3 never rides at runtime.

**Step 4 — Crate discipline.** `crates/barutu-<toolname>` in the Forge workspace;
`#![forbid(unsafe_code)]`; URUK register types as newtypes (no bare integers for counted
things); EAV Mandatory Attributes on every emission; KAKI minting only via
enkidb-ingest::bridge; integration tests before anything ships (the Fable-gate, PB-153).
All installation and every correction ships as a **numbered Ansible playbook** — this
manual intentionally contains no shell commands.

**Step 5 — Earn a name.** A tool that survives its first promotion may petition for
Engine-hood and an NL-001 name; patterns it contributes join the Apkallu arsenal by
amendment A2. Until then it is `barutu-<toolname>`, and that is honorable.

---

## Part IV — Sovereign Data Mining

Mining, in BahyWay, is not extraction — it is **census**. Four lawful mining motions:

**IV.1 Calibration mining.** Harvest history into occupied cells: for each documented
case, resolve its coordinate (rhythm × deviation × register × sin × bridge) and register
it as calibration. ShamashEngine (Polars/ndarray/nalgebra) does the density work; the
result is the table's gold.

**IV.2 Density mining.** Run archived streams (NUZI) back through instruments in rehearsal
mode (`TEMPLATE DRAFT` watching) to discover which empty cells the past already visited
unnoticed. A rehearsal DETECT on historical data is evidence for **rank**, never for
promotion — the world must perform live.

**IV.3 Hypothesis de-duplication (the Šala threshing).** Minted hypotheses are text+axes;
near-duplicates waste the watch-list. ShalaEngine's Masking Law (ML-001) applies directly:
shingle the hypothesis statements, measure Jaccard S-1…S-5 against sealed tolerances,
elect the medoid root, and thresh the rest into lineage beneath it. The Omen Table stays
a lattice, not a landfill.

**IV.4 Structure mining.** LamassuEngine's persistent homology runs over the promoted-cell
point cloud in the product space: persistent components reveal *families* of problems
(cells that promote together), and β₁ loops reveal cyclic problem chains — a deviation in
one register that reliably casts an omen in another. NabuEngine graphs the divination
trails so that the question "which axes have predicted well?" is a walk, not a dig.

Prohibited mining: any motion that targets an individual person's life. BĀRÛTU mines
pattern-space, never people. The Uruinimgina clause is not decorative.

---

## Part V — Particle Data Modeling with BĀRÛTU

BĀRÛTU extends PDM at three points:

**V.1 The Station rite grows a step.** After Gate G4 and the SLA dual seal, the Architect
may cast **birth omens**: for a newly ingested tribe, mint the hypotheses its rhythm-shape
makes plausible (a FOUR SQUARE tribe is born with its KOMMA DETUNE and COLLAPSE omens
already InFlight, instruments strapped, watching from first light). The tribe arrives in
GOLDEN already knowing how it might one day break.

**V.2 Orbit View gains the omen layer.** In GL-TPL-002's living tribe, InFlight hypotheses
render as faint ⟡ ghosts at their predicted coordinates — visible to the Engineer gate and
above, invisible to Ṭupšarru ṣeḫru by default. Aging/camping/reject/arkû drift classes are
joined, for hypotheses only, by **unborn** — the drift class of what has not happened.

**V.3 DubSar Theater gains the eighth lens.** Beside the seven lenses, the BĀRÛTU lens
shows the Omen Table (both faces), the watch-list ordered by rank, and the promotion
ledger. The lens carries the padlock rendering as a first-class UI law: any surface that
shows an InFlight hypothesis must show its locked PRESCRIBE rung in the same view.

---

## Part VI — Governance and Room to Grow

**Rank orders attention, never action.** Budgets of watching (instrument-hours, stream
bandwidth) are allocated by rank; consequences in the world are allocated only by
promotion.

**Growth happens by lineage.** Reserved extension points, each requiring an amendment
particle on GL-GEN-001 with Architect seal:

- **New deviation classes** beyond the seven (a candidate must name its guardian
  measurement before admission).
- **New order parameters** beyond S₁…S₇ (e.g., radial or topological dials) — admitted
  as instrument dials first, table axes only after two promotions.
- **New registers** — drawn only from the URUK thirteen (GL-MET-001 M-1); a fourteenth
  system is a metrology amendment, not a BĀRÛTU one.
- **Un-voiding a cell** — the rarest amendment; requires a demonstrated coherent omen.

**The audit.** Every mint, watch, detect, prove, promote, and demote is a KAKI. The
question "what did the diviner cast, and was the diviner right?" must always be
answerable by ledger walk alone. A generator that cannot be audited is an oracle,
and this house does not keep oracles.

---

## Part VII — Glossary

**A2S** — Analysis-to-Solution Law (NIPPUR 4.2): DETECT→PROVE→PREDICT→PRESCRIBE; every
prescription an immutable KAKI with its PROVE certificate.

**adannu** — event time: when the world did it (ADANNU law, NIPPUR 5.2). Paired with
**kašādu**, arrival time. Neither may impersonate the other.

**arkû** — "the late one." (1) The fourth drift class of the living tribe: lawful late
particles. (2) An aged, unpromoted hypothesis archived to NUZI.

**bārûtu** — the Mesopotamian diviner's discipline; here, the Problems Generator that
keeps the old lattice's courage and adds error bars.

**birth omens** — hypotheses minted for a tribe at its Station rite, watching from
first ingest.

**BR-number** — the designation of a hypothesis particle (BR-0001, …), minted in sequence,
never reused.

**bridge** — one of the four sealed archaic conversion ratios (Komma 80⁄81, Leimma 24⁄25,
Diesis 15⁄16, Euboic 5⁄6); the only lawful passages between URUK systems, each with a
declared loss flowing into ε.

**calibration** — history resolved onto the Omen Table as occupied cells; the gold against
which predictions are judged plausible.

**cell** — one coordinate of the Omen Table (rhythm × deviation), extended at the Mint by
register, sin, and bridge.

**DETECT** — the first rung: sealed τ threshold crossed with sustain in real streaming
density. Lights a rung; changes nothing in the world.

**deviation classes** — the seven lawful breakings: COLLAPSE, SECOND TRIBE, KOMMA DETUNE,
MIGRATING VOID, RATE-LAW BREAK, LOPSIDED SURGE, RADIAL ESCAPE.

**dial** — an order parameter Sₙ = |1/N Σ e^{inθ}| (n = 1…7); each rhythm has a guardian
dial. Rotation-invariant (Puhu-flavored): blind to rotation, awake to deformation.

**divination trail** — the sealed record of a mint: caster, date, axes, rank. Sealed
beside the detection KAKI at promotion.

**Gate G4** — design-time proof gate (Z3/Lean4/BahyWay-Algebra). Z3 never runs at runtime.

**hypothesis particle** — a problem that does not exist yet, minted as a first-class
InFlight particle with instrument attached and PRESCRIBE padlocked.

**InFlight** — the shared TIAMAT/BĀRÛTU discipline: sovereign custody of the uncommitted.
TIAMAT holds predicted particles until physical confirmation; BĀRÛTU holds predicted
problems until DETECT+PROVE.

**instrument** — dial + Θ + τ decomposition + verdict vocabulary, bound to a stream by
WATCH.

**Mendeleev event** — a promotion: the dated, certified record that the table predicted
before the world performed.

**Mint, the** — the act and the interface of creating a hypothesis particle from a cell
plus the remaining axes.

**Omen Table** — the 7×7 rhythm × deviation lattice; two faces (tablet, orbit), one state.

**opacity sins** — the seven ways a record lies: silence mistaken for absence; two tribes
averaged; an undeclared template judging; the proxy mistaken for the deed; the clock
mistaken for the sun; an exception without a tablet; detection crowned as diagnosis.

**PARZU 0x03** — the advisory severity band; also the class of KAKI filed against a
failing instrument rather than the world.

**PROMOTE / DEMOTE** — Architect-only ceremonies moving a hypothesis to occupied cell or
to the NUZI arkû archive. No programmatic path exists.

**PROVE** — the second rung: independent corroboration by a second witness (Shakkanakku
two-instrument rule; HS-EXT-002 PROVE PAIR grammar).

**rank** — sealed scoring of coherence × novelty × stakes; orders attention, never action.

**register** — the URUK counting system that owns a domain (GL-MET-001): S the Living,
S′ the Dead, B, B*, G, Š-family, E, U, D-family. *A dead thing is not a negative living
thing.*

**rhythm** — a Θ-shape family (RING … SEVEN OF THE WEEK), each with its guardian dial.

**sacred void** — a cell where no coherent omen forms, drawn dark and honored empty.
The Void is Sacred applies to problem-space.

**Station rite** — the eight-step DataStructure ceremony where Θ is sealed with a
stakeholder; extended by birth omens (Part V.1).

**τ (tau)** — the Transparency Deficit: the honest distance between declaration and
observation, decomposed (Δ_harm, Δ_keep, Δ_miss, ε) and always printed with its parts.

**Θ (theta)** — the declared template: the stakeholder's sealed rhythm, never the
analyst's assumption. `TEMPLATE DRAFT` instruments may rehearse but not DETECT.

**unborn** — the hypothesis-only drift class in Orbit View: the ghost of what has not
happened, rendered ⟡.

**WATCH** — passive binding of an instrument to a PALGU stream with ADANNU watermark,
lawful arkû, and the MĪLU no-silent-drop ladder.

**ZIBĀNĪTU** — the scales: the seven-dial symmetry gauge, third instrument of the
Shakkanakku survey, and the measurement heart every BĀRÛTU instrument inherits.

---

## Appendix A — HS-EXT-004 Quick Reference (DRAFT)

```
OMEN CAST … YIELD OMEN <name>
MINT HYPOTHESIS FROM OMEN <name> … YIELD PARTICLE BR-#### INFLIGHT
STRAP INSTRUMENT ON BR-#### … 
WATCH STREAM <stream> THROUGH BR-#### …
ON BR-#### WHEN TAU CROSSES SEALED WITH SUSTAIN RAISE DETECT
PROVE PAIR BR-#### WITH WITNESS <instrument> …
PROMOTE BR-#### BY ARCHITECT SEAL CSR-08 …
DEMOTE BR-#### TO NUZI AS ARKU HYPOTHESIS
RECALL FROM NUZI BY GAP LIST MATCHING OMEN <name>
CUBE COUNT HYPOTHESES ACROSS <axis> <axis>
```

## Appendix B — The Operator's Oath (all three gates)

*I cast without fear, mint without belief, watch without touching, and prescribe
nothing the world has not signed. The Void stays sacred; the trail stays sealed;
the padlock is mine to honor, and the seal is not mine to give.*

---

*Manual drafted in service of DUB.SAR. Nothing herein is sealed; grammar, playbook
numbers, and lattice seats await the Architect alone. 𒁾*
