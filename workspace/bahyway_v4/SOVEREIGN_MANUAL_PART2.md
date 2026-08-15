# BahyWay v4.0 — Sovereign Ecosystem Manual · Part 2
## Process Architecture & Call Chain Reference

**W5H2 Transparency Framework · Process Edition**
**Author**: Bahaa Fadam — DUB.SAR 𒁾
**Version**: 4.0.0 | **Date**: 2026-05-30
**Status**: Part 2 of N — Process Architecture (complements Part 1: Crate Reference)
**Purpose**: Stakeholder transparency — every process answers WHO, WHAT, WHERE, WHEN, WHY, HOW, HOW MUCH

---

## How to Read This Document

**Part 1** (SOVEREIGN_MANUAL_STEP1.md) answered: *What does each crate do?*
**Part 2** (this document) answers: *What does each Process do, and which Process calls which?*

A **Process** in BahyWay v4.0 is a named sovereign unit of work with:
- A single responsibility (one purpose, one owner crate)
- A defined trigger (what starts it)
- A defined output (what it produces)
- A defined downstream (what it calls next)

Every process is traceable to a KAKI-stamped StoryWay event — no process runs silently.

---

## Part A — The Sovereign Process Map

### A.1 Process Layers (Dependency Order)

```
╔══════════════════════════════════════════════════════════════════════════════╗
║  LAYER 6 — STAKEHOLDER INTERFACE                                             ║
║  P14: DataSteward Review  ←  P13: Dashboard Render  ←  P11: OBSERVE Query   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 5 — AUTONOMY                                                          ║
║  P12: Self-Healing Loop  →  P10: .akk Compilation  →  P09: AMMAS Kinetic    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 4 — ORBITAL INTELLIGENCE                                              ║
║  P15: Orbital Trust Probe  →  P07: Rule 7 Execution                         ║
║  P07: Rule 7 Execution    ←  P06: Orbital Mechanics  ←  P05: Lane Assignment║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 3 — QUALITY INTELLIGENCE                                              ║
║  P04: HeptaScore          →  P05: Lane Assignment    →  P08: Story Journal   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 2 — DATA PROCESSING                                                   ║
║  P03: VGCA Cleansing      →  P04: HeptaScore         →  P08: Story Journal   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 1 — INTAKE & IDENTITY                                                 ║
║  P02: VaultGate Intake    →  P01: KAKI Birth          →  P03: VGCA Cleansing ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  LAYER 0 — MATHEMATICAL FOUNDATION                                           ║
║  bahyway-algebra · bahyway-crc · enkidb-kaki · hepta-score                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### A.2 Master Process Call Chain

The canonical path from file drop to Golden Record visualisation:

```
FILE DROP (Data Steward drops ZIP onto DubSar window)
    │
    ▼
P02: VaultGate Intake
    │  emits → FileRegisteredEvent (KAKI-stamped, StoryWay)
    ▼
P01: KAKI Birth  ←────────────── enkidb-kaki::KakiMinter
    │  assigns 16-byte sovereign PK to every record
    │  emits → KakiBornEvent (StoryWay)
    ▼
P03: VGCA Cleansing  ←─────────── vgca-engine (VGCA-Σ / VGCA-Δ / VGCA-Λ)
    │  scores 7D FSV: [char_count, digit_density, arabic_density ...]
    │  emits → CleansingCompleteEvent (StoryWay)
    ▼
P04: HeptaScore  ←────────────── hepta-score::HeptaScorer
    │  computes H(P) = 1/(1+√Σwᵢ(Pᵢ−Tᵢ)²) → B11 byte (0–240)
    │  emits → ScoredEvent (B11 written to EAV Orbit, not KAKI nucleus)
    ▼
P05: Lane Assignment  ←─────────── hepta-score::QualityLane
    │  GEM(B11≥200) / TRIBE(≥140) / ACTIVE(≥100) / FUZZY(≥59) / DEAD(<59)
    │  emits → LaneAssignedEvent (StoryWay)
    ▼
P06: Orbital Mechanics  ←──────── tribe-orbit-engine + bahyway-algebra::orbital
    │  azimuth = 2π×κ[12]/256  altitude = (κ[14]/256−0.5)×H_max
    │  radius = r_max × δ  (inner orbit = high quality)
    │  emits → OrbitAssignedEvent (StoryWay)
    ▼
P15: Orbital Trust Probe  ←─────── orbital-trust-probe
    │  snapshot(prev) + snapshot(curr) + StoryEngine.event_count delta
    │  4-step causal attribution → DeviationCause
    │  if Unexplained → OrbitalDeviationJournal entry (CRC-16)
    │  trust_penalty → FuzzyDimensions.d9 → next ScoreEngine pass (feedback)
    │  emits → OrbitalDeviationEvent (if Unexplained)
    ▼
P07: Rule 7 Check  ←────────────── tribe-orbit-engine::DensityRule
    │  if cluster density ≥ τ_R7=11 → overflow sink → DeadArchive
    │  emits → Rule7FiredEvent (StoryWay)  [conditional]
    ▼
P08: StoryWay Journal  ←─────────── story-engine::StoryEngine
    │  all events → immutable append-only KAKI-stamped audit trail
    │  enables full replay from any snapshot
    ▼
P09: AMMAS Kinetic  ←─────────── ammas-engine (J_phys + J_mem + J_learn)
    │  ∂f/∂t = J_phys[f] + J_mem[f] + J_learn[f]
    │  BehaviorPolicy: tribal law transitions P(L3)<P(L7)<P(L1)...
    ▼
P10: .akk Compilation  ←──────── aaol::PmpvdParser → AkkIR
    │  .akk source → PmpvdProgram (9 AkkNode types) → target emission
    ▼
P11: OBSERVE Query  ←────────────── heptascript::QueryEngine
    │  Heptagon rotation → particle states (not SQL rows)
    ▼
P12: Self-Healing Loop  ←───────── ammas-engine::EquationEngine
    │  EquationDecl observes dH/dt drift → EMIT repair.akk
    ▼
P13: Dashboard Render  ←──────── dubsar-visualizer (4 sovereign panels)
    │  orbital rings, quality heatmap, story timeline, particle inspector
    ▼
P14: DataSteward Review  ←─────── data-steward-station
    Steward approves/rejects FUZZY particles → rises back to TRIBE orbit
```

### A.3 Process Dependency Matrix

| Process | Calls | Called By | Reads From | Writes To |
|---------|-------|-----------|------------|-----------|
| P01 KAKI Birth | P08 StoryWay | P02 VaultGate | bahyway-crc | enkidb-kaki |
| P02 VaultGate | P01 KAKI Birth, P08 | External (file drop) | vault-engine | enkidb-storage |
| P03 VGCA Cleanse | P04 HeptaScore, P08 | P01 KAKI Birth | EAV Orbit attrs | EAV Orbit (VGCA attrs) |
| P04 HeptaScore | P05 Lane, P08 | P03 VGCA | EAV VGCA attrs | EAV Orbit (B11 attr) |
| P05 Lane Assign | P06 Orbital, P08 | P04 HeptaScore | EAV B11 attr | EAV Orbit (lane attr) |
| P06 Orbital | P15 OrbitalTrust, P07 Rule7, P08 | P05 Lane | KAKI κ[12], κ[14] | EAV Orbit (position attrs) |
| P15 Orbital Trust Probe | P07 Rule7, P08 | P06 Orbital | OrbitalSnapshot pairs, StoryEngine event count | OrbitalDeviationJournal, FuzzyDimensions.d9 feedback |
| P07 Rule 7 | P08 StoryWay | P15 OrbitalTrust | SPH density field | DeadArchive (overflow) |
| P08 Story Journal | — | All processes | — | enkidb-journal (append-only) |
| P09 AMMAS Kinetic | P10 .akk Compile, P08 | P06 Orbital, P07 | BehaviorPolicy | EAV Orbit (state attrs) |
| P10 .akk Compile | P12 Self-Heal, P08 | P09 AMMAS | .akk source files | AkkIR (PmpvdProgram) |
| P11 OBSERVE Query | P13 Dashboard | Data Steward request | EAV Orbit (read-only) | — |
| P12 Self-Heal | P10 .akk Compile, P08 | P09 AMMAS | dH/dt EAV history | repair.akk → vault |
| P13 Dashboard | P14 Steward | P11 OBSERVE | ProjectedState | — |
| P14 Steward | P08 StoryWay | P13 Dashboard | FUZZY queue | EAV Orbit (approval) |

---

## Part B — W5H2 Process Specifications

---

### P01 — KAKI Birth (Identity Issuance)

| | |
|---|---|
| **Owner Crate** | `enkidb-kaki` |
| **Akkadi Root** | 𒆳𒀭 — *kišib* (seal, sovereign stamp) |

**WHO**
- *Who runs it*: `KakiMinter` — called by VaultGate at record intake and by AkkadianCompiler at .akk compile time
- *Who it serves*: Every downstream process — no process touches a record that does not have a KAKI PK
- *Who cannot bypass it*: No crate in v4.0 may issue its own PK. The `enkidb-kaki` crate is the sole minting authority

**WHAT**

KAKI Birth assigns a 16-byte cryptographic sovereign primary key to every particle. The KAKI never changes for the life of the particle, even when its values are corrected. It is the only true identity in the system.

```
KAKIv4.0 layout (16 bytes, IMMUTABLE):
  B00–B03  uuid_hash       (4)  content-derived hash of raw bytes
  B04–B05  tribe_id        (2)  assigned tribe — 2 bytes only
  B06      kaki_type       (1)  0x01=Identity 0x02=Event 0x03=CrossTribe
  B07      kaki_role       (1)  0x01=KISHIB 0x02=ZIKRU 0x03=PARZU
  B08–B11  reserved        (4)  for future sovereign extension
  B12–B13  timestamp       (2)  intake epoch (2-byte sovereign clock)
  B14–B15  checksum        (2)  CRC-16/CCITT of B00–B13
```

**WHERE**

```
Layer 0 — Foundation
  enkidb-kaki/src/mint.rs   ← KakiMinter::mint_identity()
  enkidb-kaki/src/kaki.rs   ← Kaki struct (16 bytes, Copy)
  enkidb-kaki/src/types.rs  ← KakiType, KakiRole enums
```

**WHEN**

Three birth moments:
1. **File KAKI** — born when a file lands at VaultGate (S0). One per physical file.
2. **Record KAKI** — born after CompareResultEvent confirms the record is not a duplicate (S2).
3. **.akk KAKI** — born at AkkadianCompiler compile time. One per AkkNode in the IR tree.

**WHY**

Every other MDM platform identifies records by integer surrogate keys or external UUIDs. When a record is corrected, a new copy is created with a new ID — and the relationship between the corrected version and the original is lost. KAKI solves this permanently: the key is derived from the content hash of the original raw bytes. Every corrected version of the record carries the same KAKI. The lineage is indestructible.

**HOW**

```rust
// Pseudocode — actual impl in enkidb-kaki/src/mint.rs
fn mint_identity(raw_bytes: &[u8], tribe_id: u16, role: KakiRole) -> Kaki {
    let uuid_hash = crc32_of(raw_bytes)[0..4];     // content-derived, stable
    let tribe     = tribe_id.to_be_bytes();          // B04–B05 (2 bytes only)
    let ktype     = KakiType::Identity as u8;        // B06
    let krole     = role as u8;                      // B07
    let reserved  = [0u8; 4];                        // B08–B11
    let timestamp = sovereign_clock_2bytes();         // B12–B13
    let payload   = [uuid_hash, tribe, [ktype, krole], reserved, timestamp].concat();
    let checksum  = crc16_ccitt(&payload);           // B14–B15
    Kaki::from_bytes([payload, checksum].concat())
}
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| KAKI size | 16 bytes (fixed, forever) |
| Mint time | O(1) — pure hash + concat |
| Three types | Identity / Event / CrossTribe |
| Sovereign constant | tribe_id = 2 bytes (B04–B05) — never 4 bytes |

---

### P02 — VaultGate Intake

| | |
|---|---|
| **Owner Crate** | `enkidb-persist` + `enkidb-storage` |
| **Akkadi Root** | 𒌍𒁁 — *abullum* (city gate — the sovereign entry point) |

**WHO**
- *Who triggers it*: Data Steward drops a file/ZIP onto the DubSar window
- *Who runs it*: `vault-engine` (immutable append-only store)
- *Who it calls*: P01 KAKI Birth → P08 StoryWay

**WHAT**

VaultGate is the single entry point for all external data. No record enters the system without passing through VaultGate. It assigns a File KAKI, writes the raw bytes to the immutable WAL, and emits `FileRegisteredEvent` before any processing begins.

**WHERE**

```
Layer 2 — Storage
  enkidb-persist/src/store.rs   ← PersistentStore::write_raw()
  enkidb-storage/src/         ← block allocation + WAL management
  permanent-storage/src/store.rs ← immutable append log
```

**WHEN**

- Triggered by: external file drop event (DubSar drag-and-drop)
- Produces: `FileRegisteredEvent` in StoryWay
- Next: P01 KAKI Birth

**WHY**

Without an immutable gate, raw data can be modified before it is indexed. If a record is cleansed before its original form is preserved, the audit trail is incomplete. VaultGate writes the raw bytes first — every subsequent transformation is a derived view of what the Vault holds.

**HOW**

```
1. Receive file/stream
2. Write raw bytes → enkidb-storage (append-only, WAL)
3. Compute uuid_hash (CRC-32 of raw bytes)
4. Call KakiMinter::mint_file_kaki(uuid_hash)
5. Emit FileRegisteredEvent(file_kaki, byte_count, timestamp) → StoryWay
6. Return file_kaki → P03 VGCA Cleansing trigger
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| WAL durability | fsync after every write |
| Max file size | Constrained by block size (enkidb-block header) |
| Dependency | enkidb-block, enkidb-kaki, story-engine |

---

### P03 — VGCA Cleansing

| | |
|---|---|
| **Owner Crate** | `vgca-engine` (Vector Geometric Cleansing Analysis) |
| **Akkadi Root** | 𒆳𒊒 — *barû* variant — to examine and judge |

**WHO**
- *Who runs it*: VGCA algorithms (VGCA-Σ, VGCA-Δ, VGCA-Λ)
- *Who calls it*: P02 VaultGate (after KAKI Birth)
- *Who it calls*: P04 HeptaScore

**WHAT**

VGCA Cleansing computes the 7-dimensional Feature Score Vector (FSV) from raw text, and uses geometric fit analysis to classify the particle as Clean, Suspect, Outlier, or Alien — before any quality scoring.

**WHERE**

```
Layer 2 — Data Processing
  vgca-engine/src/fsv.rs     ← 7D FSV computation (VGCA-Σ)
  vgca-engine/src/bfv.rs     ← 6D Binary Feature Vector (VGCA-Δ)
  vgca-engine/src/cgd.rs     ← Column Geometry Descriptor (VGCA-Λ)
  vgca-engine/src/centroid.rs ← self-calibrating centroid (Gem-only)
```

**WHEN**

- Triggered by: FileRegisteredEvent from P02
- Produces: VGCA attrs written to EAV Orbit (not KAKI nucleus — §2.4)
- Next: P04 HeptaScore

**WHY**

Standard ETL pipelines apply rules to data without first understanding its geometric shape. VGCA sees each record as a point in a 7D quality manifold. An Arabic name with no digits, correct diacritic density, and moderate entropy sits near the centroid of Arabic MDM data — this is Clean. A numeric ID in a name field is an Outlier — far from the centroid. This geometric pre-classification makes quality scoring self-calibrating.

**HOW**

```
VGCA-Σ (7D FSV — text geometry):
  FSV = [
    char_count_norm,      // D1: normalised character count
    digit_density,        // D2: fraction of digits
    arabic_density,       // D3: fraction of Arabic Unicode chars
    latin_density,        // D4: fraction of Latin chars
    punct_density,        // D5: punctuation fraction
    word_count_norm,      // D6: normalised word count
    shannon_entropy_norm, // D7: normalised Shannon entropy H(x)
  ]

GeometricFit thresholds (sovereign):
  Clean   ≥ 0.80  (close to centroid)
  Suspect  0.50–0.79
  Outlier < 0.50
  Alien   ≈ 0.00  (pure noise — no resemblance to domain data)

VGCA-Δ (6D BFV — binary block analysis):
  δ_frag = 0.35  ← sovereign constant (ADR-008)
  If fragmentation score > δ_frag → block is suspect

Self-calibrating centroid:
  Only GEM-lane particles (B11 ≥ 200) update the centroid.
  Dead and Suspect particles NEVER shift the centroid.
  → centroid always converges toward sovereign data ideal
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| FSV dimensions | 7 (D1–D7) |
| BFV dimensions | 6 |
| δ_frag constant | 0.35 (ADR-008, sovereign) |
| GeometricFit | Clean ≥ 0.80, Suspect 0.50–0.79, Outlier < 0.50 |
| EAV attrs written | 7 VGCA FSV floats + 1 geometric fit category |

---

### P04 — HeptaScore (Quality Equation)

| | |
|---|---|
| **Owner Crate** | `hepta-score` |
| **Akkadi Root** | 𒀭𒂗𒆤 — *Enlil* — lord who measures all things |

**WHO**
- *Who runs it*: `HeptaScorer` — called by pipeline bridge after VGCA
- *Who calls it*: P03 VGCA Cleansing
- *Who it calls*: P05 Lane Assignment

**WHAT**

HeptaScore computes the single sovereign health metric H(P) from the 7 quality dimensions. H(P) determines the particle's orbital radius, its B11 quality byte, and its QualityLane. This is the only valid way to compute B11 — never manually assigned.

**WHERE**

```
Layer 3 — Quality Intelligence
  hepta-score/src/equation.rs       ← hepta_health_score(p, t, w)
  hepta-score/src/domain.rs         ← HeptaVector, QualityLane, TribeIdealPoint
  hepta-score/src/pipeline_bridge.rs ← StationScores → HeptaScore
  hepta-score/src/scorer.rs         ← HeptaScorer, BatchScoringResult
```

**WHEN**

- Triggered by: P03 VGCA emitting CleansingCompleteEvent
- Produces: B11 byte written to EAV Orbit + ScoredEvent in StoryWay
- Next: P05 Lane Assignment

**WHY**

Before PMPVD, quality was a number in a column — meaningless without context. H(P) is a geometric distance from the tribe ideal point in 7D space. The closer the particle is to the ideal, the higher H(P), the smaller the orbital radius, the more inner the orbit. Quality IS physics — not a chart.

**HOW**

```
Sovereign Equation (ADR-001):

         1
H(P) = ─────────────────────────
        1 + √ Σᵢ₌₁⁷ wᵢ(Pᵢ − Tᵢ)²

where:
  Pᵢ ∈ [0.0, 1.0]  — particle score in dimension i (from BeeMDM station)
  Tᵢ = 1.0          — tribe ideal (always perfect)
  wᵢ                — sovereign weights (sum to 1.0)

Arabic MDM weight profile (sovereign default):
  [Accuracy=0.30, Completeness=0.20, Consistency=0.15,
   Validity=0.15, Uniqueness=0.10, Timeliness=0.05, Integrity=0.05]

B11 derivation (ADR-012 — SOVEREIGN CONSTANT):
  B11 = round(H(P) × 240.0)

  WHY 240 NOT 255:
  Bytes 241–255 are reserved for sovereign system signals.
  Derived from Plimpton 322 Babylonian sexagesimal mathematics.
  This constant is NEVER changed. NEVER approximated.

Dimension → Pipeline Station mapping:
  D1 Accuracy     → S3 data-cleansing-station (Arabic normalization)
  D2 Completeness → S1 KAKI Issuance (null field count)
  D3 Consistency  → S2 compare-tribe-schema (cross-system contradictions)
  D4 Validity     → S1 KAKI Issuance (NID format, phone E.164)
  D5 Uniqueness   → S2 compare-tribe-schema (dedup confidence)
  D6 Timeliness   → S0 VaultGate (file/record freshness)
  D7 Integrity    → S7 enkidb-engine (EAV referential completeness)
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| Equation | H(P) = 1/(1+√Σwᵢ(Pᵢ-Tᵢ)²) |
| QUALITY_DIVISOR | **240.0** — sovereign, eternal |
| GEM_B11 | ≥ 200 (H(P) ≥ 0.833) |
| Test coverage | 27 unit tests in hepta-score |
| Weight profiles | Arabic MDM · Equal · Environmental · Government |

---

### P05 — Lane Assignment

| | |
|---|---|
| **Owner Crate** | `hepta-score::QualityLane` + `bahyway-algebra::shells` |

**WHO**
- *Who runs it*: `QualityLane::from_b11(b11)` — one line, no state
- *Who calls it*: P04 HeptaScore
- *Who it calls*: P06 Orbital Mechanics

**WHAT**

Lane Assignment maps the B11 quality byte to one of five orbital lanes and a shell index. This is a pure function — same B11 always yields the same lane.

**WHERE**

```
Layer 3 — Quality Intelligence
  hepta-score/src/domain.rs          ← QualityLane::from_b11()
  bahyway-algebra/src/shells.rs      ← sovereign_5_shells(), shell_index()
```

**WHEN**

- Triggered by: H(P) computation completing
- Produces: LaneAssignedEvent → StoryWay
- Next: P06 Orbital Mechanics

**HOW**

```
Sovereign Lane Table:

  B11 ≥ 200  →  GEM         inner orbit   shell 0   DubSar: gold burst
  B11 ≥ 140  →  TRIBE       mid orbit     shell 1   DubSar: silver ring
  B11 ≥ 100  →  ACTIVE      outer orbit   shell 2   DubSar: green orbit
  B11 ≥  59  →  FUZZY       steward queue shell 3   DubSar: pink (#FF44AA)
  B11 <  59  →  DEAD        dead grid     shell 4   DubSar: gray dot

Shell boundaries (sovereign_5_shells()):
  δ < 0.167  → GEM     (δ = 1 − H(P))
  δ < 0.417  → TRIBE
  δ < 0.583  → ACTIVE
  δ < 0.754  → FUZZY
  δ ≥ 0.754  → DEAD
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| Lanes | 5 (GEM / TRIBE / ACTIVE / FUZZY / DEAD) |
| Shell lookup | O(log 4) = O(1) binary search |
| GEM rate SLA | ≥ 35.4% of batch (ADR-004) |
| UNRESOLVED_COLOR | `#FF44AA` (Catppuccin Mocha Pink — sovereign) |

---

### P06 — Orbital Mechanics

| | |
|---|---|
| **Owner Crate** | `tribe-orbit-engine` + `bahyway-algebra::orbital` |
| **Akkadi Root** | 𒀭𒂗𒍪 — *šamû* (heavens — the sovereign orbital field) |

**WHO**
- *Who runs it*: `orbital_position(kaki, delta, r_max, h_max)` — deterministic pure function
- *Who calls it*: P05 Lane Assignment
- *Who it calls*: P07 Rule 7 Check, P13 Dashboard Render

**WHAT**

Orbital Mechanics computes the 3D spatial position of every particle derived entirely from its KAKI sovereign PK and its quality distance δ. Same KAKI + same quality = same position in space, always. No randomness. No layout algorithm. Physics.

**WHERE**

```
Layer 4 — Orbital Intelligence
  bahyway-algebra/src/orbital.rs     ← orbital_position(), OrbitalPosition
  tribe-orbit-engine/src/orbit.rs    ← TribeOrbit, ring assignment
  tribe-orbit-engine/src/density.rs  ← SPH density field computation
```

**WHEN**

- Triggered by: LaneAssignedEvent
- Produces: OrbitalPosition written to EAV Orbit + OrbitAssignedEvent
- Runs continuously: particles move as their quality changes (J_phys term)

**HOW**

```
PA-14 Orbital Position Equations (sovereign):

  azimuth  = 2π × κ[12] / 256
             κ[12] = first byte of timestamp field (B12–B13)
             → 256 evenly-spaced azimuths around tribe ring

  altitude = (κ[14] / 256 − 0.5) × H_max
             κ[14] = first byte of checksum field (B14–B15)
             → ±H_max/2 vertical scatter above/below equator

  radius   = r_max × δ   where δ = 1 − H(P)
             → GEM particles orbit innermost (radius ≈ 0)
             → DEAD particles orbit outermost (radius = r_max)

Cartesian conversion for renderer:
  x = radius × cos(azimuth)
  y = radius × sin(azimuth)
  z = altitude

SPH Density Field (tribe-orbit-engine):
  Smoothed-Particle Hydrodynamics: each particle contributes
  a Gaussian kernel to its neighbourhood density.
  High density → cluster forming → Rule 7 watch begins.

  3 density bands:
    Isolated:      0–2  particles within resonance_radius=0.15
    Cluster:       3–10
    Rule7Overflow: 11+  → triggers P07
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| PARTICLES_PER_TRIBE | 7 (Heptagon constant) |
| SHC | 1.0/255.0 (Sovereign Heptagon Constant) |
| resonance_radius | 0.15 |
| τ_R7 | 11 (Rule 7 density trigger) |
| cluster_thresh | 3 |

---

### P07 — Rule 7 Execution

| | |
|---|---|
| **Owner Crate** | `tribe-orbit-engine::DensityRule` |
| **Akkadi Root** | 𒀭𒂗𒆜 — *eṭemmu* (the dispersed spirit — overflow release) |

**WHO**
- *Who triggers it*: SPH density field exceeds τ_R7 = 11
- *Who runs it*: `tribe-orbit-engine::DensityRule::check_overflow()`
- *Who it calls*: P08 StoryWay (Rule7FiredEvent) → DeadArchive sink

**WHAT**

Rule 7 is the sovereign overflow protection: when 11 or more dead particles cluster within resonance radius 0.15, the cluster is released to the DeadArchive. This prevents gravitational collapse — a situation where low-quality particles pull each other further from the tribe ideal through density effects.

**WHERE**

```
Layer 4 — Orbital Intelligence
  tribe-orbit-engine/src/law.rs     ← TribeLaw::L7 (lowest priority law)
  tribe-orbit-engine/src/density.rs ← DensityRule::check_overflow()
```

**WHEN**

- Triggered by: density scan after every orbital update
- Produces: Rule7FiredEvent → StoryWay; cluster → DeadArchive

**WHY**

Without overflow protection, a dead cluster acts as a gravitational sink — it distorts the SPH density field and drags nearby active particles toward the dead zone. Rule 7 quarantines the cluster before it can contaminate the tribe.

**HOW**

```
7 Tribal Laws (priority order — highest priority acts last):
  P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)

Law 7 (lowest priority — fires only when density saturates):
  IF count(dead_particles, neighbourhood(radius=0.15)) ≥ 11
  THEN route cluster → DeadArchive
       EMIT Rule7FiredEvent(cluster_kaki_list) → StoryWay
       update SPH density field

Visual effect (DubSar):
  When Rule 7 fires, the cluster implodes with a gravitational-lens
  warp — the HBZo zoom effect warps the view around the mass
  (deferred to v5 GPU compute layer).
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| τ_R7 | **11** — sovereign constant |
| resonance_radius | 0.15 |
| 7 Laws | priority P(L3)<P(L7)<P(L1)<P(L2)<P(L5)<P(L4)<P(L6) |

---

### P08 — StoryWay Journal (Audit Trail)

| | |
|---|---|
| **Owner Crate** | `story-engine` + `enkidb-journal` |
| **Akkadi Root** | 𒁾 — *dubsar* (scribe — records all that happens) |

**WHO**
- *Who calls it*: Every other process — P01 through P14 all write events here
- *Who reads it*: P11 OBSERVE Query (read-only replay), P12 Self-Healing (dH/dt from journal spline)
- *Who may NOT modify it*: No process. Journal is immutable append-only.

**WHAT**

StoryWay is the sovereign audit trail. Every decision the system makes — every lane transition, every Rule 7 firing, every .akk generation — is written as a KAKI-stamped event in the immutable journal. No event can be erased. Every state is reconstructible by replaying the journal from any snapshot.

**WHERE**

```
Layer 1 — Storage Foundation
  enkidb-journal/src/         ← immutable append-only event log
  story-engine/src/story_engine.rs ← StoryEngine (CQRS write side)
  story-engine/src/projection.rs   ← ProjectedState (CQRS read side)
  enkidb-snapshot/src/             ← periodic snapshot for replay efficiency
```

**WHEN**

- Triggered by: every process event (synchronous write before returning)
- Always on: the journal never sleeps
- Read by: P11 OBSERVE, P12 Self-Healing, P14 DataSteward audit view

**HOW**

```
CQRS (Command-Query Responsibility Segregation):

  Write path:
    process_event(kaki, event_type, payload)
      → append to enkidb-journal (WAL + fsync)
      → return EventKaki (confirms event was written)

  Read path (P11 OBSERVE):
    StoryEngine::project(particle_kaki, since: Option<SnapshotId>)
      → load latest snapshot (if exists)
      → replay delta events from journal
      → return ProjectedState { quality_history, current_lane, ... }

  Snapshot schedule (enkidb-snapshot):
    Every N events → write snapshot → enables fast O(delta) replay
    N is configurable per tribe (not global)

Every event carries:
  - EventKaki (16-byte PK) — derived from the originating IdentityKaki
  - event_type: u8
  - tribe_id: u16
  - payload: EAV attrs (key-value map)
  - timestamp: sovereign 2-byte clock
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| Durability | fsync on every event write |
| Reversibility | Every .akk event can be un-applied by journal replay |
| Event types | FileRegistered, KakiBorn, Cleansed, Scored, LaneAssigned, OrbitAssigned, Rule7Fired, StewardApproved, .akk emitted |
| Snapshot | Periodic — O(delta) replay from snapshot |

---

### P09 — AMMAS Kinetic (Multi-Agent System)

| | |
|---|---|
| **Owner Crate** | `ammas-engine` |
| **Akkadi Root** | 𒀭𒈠𒌓 — *ammu* (foundation — the master kinetic law) |

**WHO**
- *Who runs it*: `ammas-engine::BehaviorPolicy` — continuous background process
- *Who calls it*: P06 Orbital (J_phys), P08 StoryWay (J_mem event feed), P10 .akk Compile (J_learn output)
- *Who it calls*: P10 .akk Compilation, P08 StoryWay

**WHAT**

AMMAS is the master kinetic equation that governs how particles evolve over time. It is not a rule engine in the traditional sense. It is a differential equation with three force terms operating simultaneously — orbital physics, memory of tribal law, and learned policy drift.

**WHERE**

```
Layer 5 — Orbital Intelligence
  ammas-engine/src/behavior.rs     ← BehaviorPolicy (J_mem term)
  ammas-engine/src/kinetic.rs      ← master kinetic equation
  ammas-engine/src/agent.rs        ← AgentClass (mirrors 3 KAKI types)
  ammas-engine/src/density.rs      ← DensityRule (J_phys bridge)
```

**WHEN**

- Runs continuously in background after intake completes
- J_phys fires on every orbital tick
- J_mem fires on every tribal law transition event
- J_learn fires when fuzzy-engine + akkadian-compiler detect quality drift

**HOW**

```
AMMAS Master Kinetic Equation (sovereign):

  ∂f/∂t = J_phys[f] + J_mem[f] + J_learn[f]

  where f(x,t) = particle distribution in 7D Hepta space

J_phys[f]  — Physical orbital motion + Rule 7 sink
  → tribe-orbit-engine (SPH density, resonance radius 0.15)
  → Rule 7: τ_R7 = 11 → overflow → DeadArchive

J_mem[f]   — Memory: BehaviorPolicy + tribal law transitions
  → ammas-engine (reads own event log from enkidb-journal)
  → 7 Tribal Laws govern priority of state transitions
  → AgentClass mirrors 3 KAKI types:
       Agent::Identity  ← IdentityKaki
       Agent::Event     ← EventKaki
       Agent::CrossTribe ← CrossTribeKaki

J_learn[f] — Sovereign intelligence: quality drift detection + .akk repair
  → fuzzy-engine: Mamdani inference on quality degradation signals
  → akkadian-compiler: synthesises repair .akk from nearest template
  → NOT neural-network learning — sovereign, auditable, KAKI-stamped

The three terms always act simultaneously.
No term can disable another.
Every term's output is an EAV event in StoryWay (fully auditable).
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| 3 kinetic terms | J_phys · J_mem · J_learn |
| AgentClass | 3 (mirrors IdentityKaki / EventKaki / CrossTribeKaki) |
| 7 Tribal Laws | Priority: P(L3)<P(L7)<P(L1)<P(L2)<P(L5)<P(L4)<P(L6) |
| BehaviorPolicy | Read from .akk governance files |

---

### P10 — .akk Compilation (AkkadianCompiler)

| | |
|---|---|
| **Owner Crate** | `aaol` |
| **Akkadi Root** | 𒀭𒂗 — *Enlil* — the law-giver, the language of the gods |

**WHO**
- *Who runs it*: `aaol::PmpvdParser` + `aaol::AkkIR`
- *Who calls it*: P09 AMMAS (J_learn) and P12 Self-Healing (EMIT nodes)
- *Who it calls*: P08 StoryWay (compilation event), target emitters (Rust/WASM/JS)

**WHAT**

The AkkadianCompiler takes `.akk` source files, tokenizes them, parses them into the 9-node PMPVD AkkIR, and emits target-agnostic sovereign code. One `.akk` program can emit simultaneously to Rust structs, JavaScript particles, and WASM modules — from one sovereign source.

**WHERE**

```
Layer 5 — Autonomy
  aaol/src/token.rs    ← lexer (PMPVD + orchestration keywords)
  aaol/src/pmpvd.rs    ← PmpvdParser, 9 AkkNode types, NodeId
  aaol/src/ast.rs      ← orchestration layer parser
```

**WHEN**

- At boot: loads governance .akk files defining tribes, rules, equations
- During intake: .akk KAKI born for each AkkNode
- During self-healing: P12 generates repair .akk → immediately compiled
- On demand: Data Steward can submit new .akk rules via ShoWay

**HOW**

```
9 AkkNode Types (sovereign PMPVD IR):

  Identity Group:
    PARTICLE — declares a sovereign data particle type
                Fields: TEXT(accuracy=0.98), KAKI, TRIBE: name
    TRIBE    — declares ideal point + weight profile
                WEIGHTS: [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05]
                IDEAL:   [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]

  Intelligence Group:
    RULE     — WHEN conditions THEN actions (lane/orbit/emit)
    EQUATION — self-referential: observes own output, emits fixes
    GUARD    — REQUIRE conditions ALLOW [lanes]

  Flow Group:
    FLOW     — SOURCE → PIPE chain → SINK
    OBSERVE  — FROM TRIBE WHERE condition SORT field LIMIT n

  Autonomy Group:
    EMIT     — target("payload") → KAKI-stamped .akk event
    PIPELINE — ordered stage list (boot sequence)

NodeId = FNV-1a(kind|name) — deterministic, maps to KAKI PK in production

Parse pipeline:
  .akk source → tokenize() → Vec<Token> → PmpvdParser → PmpvdProgram
  PmpvdProgram → [Rust emitter | JS emitter | WASM emitter]
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| AkkNode types | 9 (sovereign, immutable set) |
| Tokens | 40+ (orchestration + PMPVD + operators) |
| Tests | 28 (aaol crate) |
| NodeId | FNV-1a 64-bit — deterministic |
| Emit targets | Rust · JavaScript · WASM (one source) |

---

### P11 — OBSERVE Query (HeptaScript)

| | |
|---|---|
| **Owner Crate** | `heptascript` |
| **Akkadi Root** | 𒆷𒍪 — *lipit* (reading the tablets — query is rotation) |

**WHO**
- *Who triggers it*: Data Steward dashboard request, DubSar OBSERVE panel
- *Who runs it*: `heptascript::QueryEngine`
- *Who it calls*: P13 Dashboard Render

**WHAT**

OBSERVE is a geometric rotation query — it rotates the 7D Heptagon manifold to a new analytical frame and returns particle states as they appear in that frame. It does not return rows. It returns orbital positions. No JOIN. No mutation. No side effects.

**WHERE**

```
Layer 9 — Languages
  heptascript/src/query.rs         ← QueryPlan, QueryParser
  heptascript/src/engine.rs        ← execute(plan) → QueryResult
  heptascript/src/modular_index.rs ← ModularNaviIndex (Viazovska-inspired)
```

**WHEN**

- Triggered by: DubSar OBSERVE panel, Data Steward drill-down
- Always read-only — never mutates EAV Orbit
- Returns: particle states as OrbitalPosition + ProjectedState

**HOW**

```
OBSERVE golden_citizens {
  FROM: TRIBE IraqiCitizenMDM
  WHERE: health >= 0.833
  SORT: orbital_radius ASC
  LIMIT: 10000
}

Execution plan:
  1. Resolve tribe → load TribeIdealPoint
  2. Apply WHERE filter against EAV Orbit (ATTR_HASH_QUALITY scan)
  3. Compute OrbitalPosition for each result
  4. Sort by orbital_radius ASC (inner → outer)
  5. Apply LIMIT
  6. Return: Vec<(Kaki, OrbitalPosition, ProjectedState)>

No SQL. No JOIN. No LINK. The Heptagon IS the projection engine.
7 independent analytical read models from one write stream.
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| Query type | Geometric rotation (not SQL SELECT) |
| Side effects | Zero — read-only always |
| Index | ModularNaviIndex (Viazovska Fourier topology) |
| Max result | Configurable LIMIT (no unbounded scans) |

---

### P12 — Self-Healing Loop (EquationEngine)

| | |
|---|---|
| **Owner Crate** | `ammas-engine::EquationEngine` |
| **Akkadi Root** | 𒀭𒌓𒆤 — *šamaštu* (self-renewing sun — the healing cycle) |

**WHO**
- *Who runs it*: `EquationEngine` — continuous background loop
- *Who triggers it*: J_learn term detects dH/dt < 0 (quality declining)
- *Who it calls*: P10 .akk Compilation (generates repair .akk), P08 StoryWay

**WHAT**

The Self-Healing Loop monitors quality trajectory (dH/dt) for every active particle. When a particle's quality is declining faster than the sovereign threshold, the EquationEngine generates a `.akk` repair script autonomously — without human intervention. The generated script is KAKI-stamped, human-readable, and fully reversible.

**WHERE**

```
Layer 5 — Autonomy
  ammas-engine/src/equation.rs  ← EquationEngine::observe_and_emit()
  ammas-engine/src/emit.rs      ← .akk file generator
  bahyway-algebra/src/axioms.rs ← PA13_THETA_MAX threshold
```

**WHEN**

- Runs continuously: every quality event triggers a trajectory check
- Fires: when θ(p) = |Δhepta/Δt| > PA13_THETA_MAX (currently 1.0)
- Produces: repair.akk → vault → P10 compile → rule applied

**HOW**

```
Three Levels of Self-Reference (PMPVD autonomic capability):

Level 1 — Equation observes its own prediction error:
  EQUATION cardinality_rule {
    WHEN deviation(predicted, actual) > 10.0
    THEN EMIT update_self.akk {
      rule: self
      new_weight: recalibrate(deviation)
    }
  }
  → equation detects wrong prediction → generates .akk to fix own weights

Level 2 — Equation generates new equations:
  EQUATION pattern_detector {
    WHEN observed_pattern.is_novel()
    THEN EMIT synthesize_equation.akk {
      template: nearest_known_equation(pattern)
      parameters: fit_parameters(pattern)
      kaki: new_kaki(domain=AkkadianDomain)
    }
  }
  → novel quality pattern → new equation synthesised from template library

Level 3 — Meta-equation manages the equation library:
  EQUATION meta_observer {
    OBSERVE equation_health {
      FROM: TRIBE AkkadianDomain
      WHERE: invocation_rate < 0.01
    }
    WHEN equation_health.has_dead_rules()
    THEN EMIT quarantine_equation.akk { grace_period: 30.days }
  }
  → unused rules quarantined → equation ecosystem is alive

PA-13 θ threshold:
  θ(p) = |Δhepta(t) / Δt| > PA13_THETA_MAX=1.0 → trigger
  Trajectory momentum from StoryWay journal spline (not stored as field)
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| PA13_THETA_MAX | 1.0 (sovereign threshold) |
| Self-reference levels | 3 (observe/generate/manage) |
| Every repair | KAKI-stamped, human-readable, VaultWay-persisted, reversible |
| THIS is not AI | It is sovereign transparent autonomy — auditable by design |

---

### P13 — Dashboard Render (DubSar Visualizer)

| | |
|---|---|
| **Owner Crate** | `dubsar-visualizer` |
| **Akkadi Root** | 𒁾 — *dubsar* (scribe who makes the invisible visible) |

**WHO**
- *Who runs it*: `dubsar-visualizer` — the primary user-facing application
- *Who calls it*: P11 OBSERVE (feeds particle states)
- *Who it calls*: P14 DataSteward Review (when Steward clicks a FUZZY particle)

**WHAT**

DubSar renders the BahyWay v4.0 sovereign ecosystem in real-time across 4 panels. Every particle is visible as a physically-simulated orbital body. No log file needs to be read. Quality is physics — observable in three dimensions.

**WHERE**

```
Layer 11 — UI / IDE
  dubsar-visualizer/src/panels/
    hepta.rs    ← 7D quality heatmap panel
    particles.rs ← orbital ring + particle inspector (3-tab parameter editor)
    najaf.rs    ← NajafEngine topology panel
    wpd.rs      ← WPD (Water Particle Data) analysis panel
  dubsar-visualizer/src/visualizer.rs ← main render loop
  dubsar-visualizer/src/theme.rs      ← Catppuccin Mocha sovereign theme
```

**WHEN**

- Runs continuously while application is open
- Updated on every ProjectedState change
- OBSERVE queries run on a configurable refresh interval

**HOW**

```
4 Sovereign Panels:

  Panel 1 — Particle Inspector (3-tab Houdini-style):
    Tab 1: KAKI identity (16 bytes decoded, tribe_id, kaki_type, role)
    Tab 2: Quality (B11, H(P), 7D bar chart, lane, orbital radius)
    Tab 3: History (StoryWay event timeline, quality trajectory)

  Panel 2 — 7D Hepta Heatmap:
    Each dimension D1–D7 rendered as a colour band
    Width = weight wᵢ (Accuracy=30% of bar, etc.)
    Intensity = Pᵢ score
    Weakness highlighted red (fix_priority dimension)

  Panel 3 — Story Timeline:
    KAKI-stamped events in chronological order
    Click → jump to particle in orbital view
    Filter by event type or KAKI

  Panel 4 — NajafEngine Topology:
    Simplicial complex of active tribe connections
    Rule 7 clusters shown as dense sub-graphs
    Edge weight = cross-tribe KAKI relationship strength

Rendering modes (RenderingMode — sovereign enum):
  PointSprite   → count < 10K   (simple 2D dots, no GPU shader)
  Instanced     → 10K–200K      (GPU instanced mesh)
  Volumetric    → 200K+         (WGSL compute + ray-march) [v5]
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| Panels | 4 sovereign panels |
| PointSprite cap | 10K particles |
| Instanced cap | 200K particles (GTX 1650) |
| UNRESOLVED_COLOR | `#FF44AA` — FUZZY particle visual identity |
| Theme | Catppuccin Mocha (sovereign, never changed) |

---

### P15 — Orbital Trust Probe (Causal Deviation Attribution)

| | |
|---|---|
| **Owner Crate** | `orbital-trust-probe` |
| **Akkadi Root** | 𒁾𒀭𒋾 — *orbital watchman, sovereign eye on the field* |

**WHO**
- *Who runs it*: `orbital_trust_probe::probe_and_journal()` — called after every `score-engine` pass, before the next `tribe-orbit-engine` ring assignment
- *Who it serves*: The entire orbital field — any particle at any scale. At one billion particles, it runs in O(particles) with zero cross-particle state
- *Who benefits from it*: Data stewards (clean violation reports), the AMMAS kinetic engine (accurate density field), and the DubSar visualiser (attributed orbital motion panel)
- *Who cannot bypass it*: Any system wishing to issue a trust penalty for orbital deviation must route through this probe. Direct B11 manipulation bypasses the causal check and violates the sovereign scoring contract

**WHAT**

P15 determines *why* a particle moved in 7D orbital space and acts accordingly. It never issues a trust penalty without exhausting all legitimate causal explanations first. It is the sovereign gatekeeper between orbital physics and trust accounting.

```
Input:  OrbitalSnapshot(prev)  +  OrbitalSnapshot(curr)  +  new_eav_events: usize
Output: ProbeResult { cause: DeviationCause, trust_penalty: f32, cartesian_delta: f32 }
        + optional OrbitalDeviationJournal entry (when cause == Unexplained)
        + trust_penalty fed into FuzzyDimensions.d9 for next scoring cycle
```

**WHERE**
```
Layer 4.5 — Orbital Intelligence
  orbital-trust-probe/src/lib.rs      ← probe_and_journal() entry point
  orbital-trust-probe/src/probe.rs    ← 4-step causal pipeline
  orbital-trust-probe/src/cause.rs    ← DeviationCause enum (6 classes)
  orbital-trust-probe/src/snapshot.rs ← OrbitalSnapshot + hysteresis constants
  orbital-trust-probe/src/journal.rs  ← OrbitalDeviationJournal (CRC-16, append-only)
```

Sits between `score-engine` (upstream, provides B11 and freshness byte) and
`tribe-orbit-engine` (downstream, performs ring assignment on next cycle).

**WHEN**

```
score-engine pass N completes → OrbitalSnapshot(N) captured
P15 runs → compares Snapshot(N-1) vs Snapshot(N)
    → if Unexplained: journal entry + penalty
    → penalty written to ScoreInput.orbital_trust_penalty for pass N+1
score-engine pass N+1 → FuzzyDimensions.d9 = penalty → lower B11 → organic ring correction
```

**WHY**

At billion-particle scale, the orbital field is a **dynamic physics system**, not a
static map. Particles move legitimately for five distinct reasons (rules changed,
data evolved, neighbours shifted, freshness decayed, boundary noise). Without P15,
any monitoring system generates false trust alarms for all five, triggering a
cascade: false alarm → trust penalty → lower B11 → ring change → neighbours appear
deviant → more false alarms. The cascade is exponential and self-sustaining.

P15 breaks the cascade at its root by requiring causal attribution before any
penalty is issued.

**HOW**

```
Step 1 — Rules Fingerprint (FNV-1a hash of fuzzy-engine rule constants)
          prev.rules_fingerprint ≠ curr.rules_fingerprint?
          YES → DeviationCause::FuzzyRulesChanged → ABORT, zero penalty
          (Rule set updates are structural changes, not suspicious)

Step 2 — StoryEngine EAV Event Count
          new_eav_events > 0?
          YES → DeviationCause::LegitimateStateEvolution → log, zero penalty
          (Data legitimately changed; score followed; orbit followed)

Step 3a — SPH Neighbourhood Delta
          |prev.neighbour_count − curr.neighbour_count| ≥ NEIGHBOUR_DELTA_THRESHOLD (2)?
          YES → DeviationCause::NeighborDensityShift → log, zero penalty
          (A neighbour joined or left the resonance_radius=0.15 shell)

Step 3b — Freshness Decay
          prev.freshness_byte − curr.freshness_byte ≥ FRESHNESS_DECAY_THRESHOLD (15)?
          YES → DeviationCause::FreshnessDecay → log, zero penalty
          (BLUE byte decayed past a ring-boundary threshold naturally)

Step 3c — Threshold Boundary Noise
          near_boundary(prev.b11) OR near_boundary(curr.b11)?
          (near_boundary = within HYSTERESIS=5 of 200 or 100)
          YES → DeviationCause::ThresholdBoundaryNoise → log, zero penalty
          (Discrete ring assignment oscillates near boundary — scoring noise)

Step 4 — Unexplained Residual
          No causal explanation found for movement > MIN_DEVIATION_DISTANCE (0.05)
          → DeviationCause::Unexplained
          → trust_penalty = clamp(delta/4.0, 0, 0.5) × (2 if ring_changed else 1)
          → OrbitalDeviationJournal.append(entry)   [CRC-16, deduplicated]
          → penalty fed to ScoreInput.orbital_trust_penalty → FuzzyDimensions.d9
          → next FuzzyEngine.score() degrades D6: effective = base − penalty×(base−0.10)
          → lower B11 → particle drifts to correct ring organically
          → subsequent probe cycle: StoryEngine shows no new events, but position
            now matches lower B11 → Within Tolerance → cascade terminates
```

**HOW MUCH**

| Metric | Value |
|---|---|
| Tests | **22** (cause × 3, snapshot × 3, probe × 8, journal × 5, lib × 3) |
| Source lines | 840 |
| Source files | 5 |
| External deps | 0 |
| Unsafe code | `forbid(unsafe_code)` |
| Sovereign constants | 4 (`MIN_DEVIATION_DISTANCE`, `FRESHNESS_DECAY_THRESHOLD`, `NEIGHBOUR_DELTA_THRESHOLD`, `HYSTERESIS`) |
| Journal integrity | CRC-16/CCITT per entry, deduplicated by (particle_id, epoch, checksum) |
| Penalty range | [0.0, 1.0] accumulated, clamped — particle never silenced entirely |
| Cascade termination | Guaranteed — feedback loop is self-terminating once ring corrects |

---

### P14 — Data Steward Review

| | |
|---|---|
| **Owner Crate** | `data-steward-station` |
| **Akkadi Root** | 𒀭𒌓 — *šamû* — the judge who weighs (data quality court) |

**WHO**
- *Who runs it*: Human Data Steward — the sovereign expert
- *Who triggers it*: FUZZY particles entering the steward queue
- *Who it calls*: P08 StoryWay (approval event), re-triggers P04 HeptaScore

**WHAT**

When a particle's B11 falls to the FUZZY lane (59 ≤ B11 < 100), it enters the Data Steward queue. The Steward inspects the particle, applies remediation (corrects the raw data via .akk script), and approves or rejects it. Approval triggers a new H(P) computation from the corrected values — the particle rises from FUZZY to TRIBE or GEM orbit.

**WHERE**

```
Layer 8 — Pipeline
  data-steward-station/src/steward.rs ← DataStewardStation
  dubsar-visualizer/src/panels/particles.rs ← steward review UI
```

**WHEN**

- Triggered by: particle entering FUZZY lane
- Blocked until: Steward approves or rejects
- On approval: re-triggers P03 VGCA → P04 HeptaScore → new B11

**HOW**

```
Steward Workflow:

  1. FUZZY particle appears in DubSar particle inspector (pink dot)
  2. Steward opens 3-tab inspector → sees:
       Tab 2: B11=73, H(P)=0.304, Weakest: Accuracy (Arabic name garbled)
  3. Steward writes remediation .akk:
       EMIT fix_arabic_name.akk {
         target: particle.arabic_name
         value:  "محمد أحمد علي"   ← corrected
         reason: "tashkeel normalization"
       }
  4. .akk compiled → P10 AkkadianCompiler → rule applied
  5. P03 VGCA re-runs on corrected value → new FSV
  6. P04 HeptaScore re-computes H(P) → new B11
  7. If B11 ≥ 140 → particle rises to TRIBE orbit (DubSar: visual arc)
  8. StewardApprovedEvent → P08 StoryWay (immutable record)

SLA: DataSteward queue must not exceed 9.46% of batch (STEWARD_RATE)
```

**HOW MUCH**

| Metric | Value |
|--------|-------|
| STEWARD_RATE | 9.46% (sovereign SLA — ADR-004) |
| FUZZY boundary | 59 ≤ B11 < 100 |
| Approval → effect | Full re-score pipeline re-triggered |
| Audit | Every approval is a KAKI-stamped StoryWay event |

---

## Part C — Stakeholder Transparency Matrix

Who sees what — at which process level:

| Stakeholder | Visibility | Processes | Tools |
|-------------|------------|-----------|-------|
| **Data Steward** | Individual particle quality, FUZZY queue, remediation history | P03 P04 P05 P14 | DubSar particle inspector, P14 review panel |
| **Domain Architect** | Tribe weight profiles, SLA rates, batch quality | P04 P05 P11 | DubSar hepta heatmap, OBSERVE queries |
| **System Architect** | Crate dependency graph, KAKI lineage, SPH density | P01–P13 | DubSar topology panel, StoryWay journal |
| **BahyWay Architect** | Full sovereign algebra, orbital constants, law priorities | All processes | Full codebase + bahyway-algebra |
| **Enterprise Partner** | Golden Records (GEM lane only), transparency reports | P11 P13 | OBSERVE queries → EAV Orbit export |
| **Regulator / Auditor** | KAKI lineage, every decision's .akk, full StoryWay | P08 P12 | StoryWay journal replay, .akk file archive |
| **Junior Developer** | Current particle state, B11, lane, 7D scores | P04 P05 P13 | DubSar particle inspector (read-only) |

### Transparency Guarantee (PMPVD Physical Homoiconicity)

Every autonomous decision PMPVD makes is expressed as a `.akk` file:
- **KAKI-stamped** — unique sovereign identity, traceable to the originating particle
- **Human-readable** — written in AkkadianAOL syntax, readable by any trained developer
- **VaultWay-persisted** — stored in the immutable append-only log, cannot be erased
- **Fully reversible** — every `.akk` can be un-applied by replaying the StoryWay journal

This separates PMPVD from neural networks and genetic algorithms: decisions are **transparent by architecture**, not by monitoring.

---

## Part D — Sovereign Boot Sequence (Process Startup Order)

When BahyWay v4.0 starts, processes start in this exact order — each must complete before the next begins:

```
PIPELINE sovereign_boot v4_0 {

  Stage 1: bahyway-core + bahyway-crc
           ← mathematical foundation loaded
           ← CRC-16/CCITT available

  Stage 2: enkidb-kaki
           ← KAKI minting available
           ← 3 types registered (Identity/Event/CrossTribe)

  Stage 3: enkidb-storage + enkidb-block + enkidb-journal
           ← storage engine open
           ← WAL active
           ← journal accepting writes

  Stage 4: story-engine
           ← StoryEngine ready
           ← StoryStartedEvent emitted (boot sequence is itself in the journal)

  Stage 5: hepta-score
           ← H(P) equation loaded
           ← sovereign weight profiles loaded
           ← SLA constants confirmed (GEM_RATE=35.4%, STEWARD_RATE=9.46%)

  Stage 6: aaol (AkkadianCompiler)
           ← .akk tokenizer ready
           ← PmpvdParser ready
           ← governance .akk files compiled (TRIBE declarations loaded)

  Stage 7: template-engine + template-library
           ← repair templates available for P12 self-healing

  Stage 8: tribe-orbit-engine + ammas-engine
           ← SPH density field initialised
           ← BehaviorPolicy loaded from governance .akk
           ← 7 Tribal Laws activated (priority order enforced)

  Stage 9: dubsar-visualizer
           ← LIVE
           ← DubSar window open, all 4 panels rendering
           ← Data Steward may now drop files

}
```

---

## Part E — Process Anti-Patterns (What NEVER Happens)

These are architectural violations. If you see them in a PR, reject immediately.

| Anti-Pattern | Why It Violates Sovereignty | Correct Alternative |
|---|---|---|
| Writing quality score to KAKI nucleus | §2.4: nucleus is immutable structural facts only | Write quality to EAV Orbit via EventKaki |
| Setting B11 manually (not from H(P)×240) | ADR-012: QUALITY_DIVISOR=240 is sovereign | Always derive B11 from `hepta_health_score()` |
| Using tribe_id with 4 bytes | v4.0 KAKIv4.0: tribe_id = 2 bytes (B04–B05) | Mask to top 2 bytes only |
| Querying EAV Orbit with SQL JOIN | §2.4: no JOINs, no LINKs in EnkiWay | Use OBSERVE rotation query via heptascript |
| Storing projected state as mutable field | CQRS: state is computed from journal replay | Use StoryEngine::project() on demand |
| Using `unwrap()` in production code | No panics allowed in sovereign path | Propagate errors through thiserror types |
| Adding tokio/async to any crate | Pure sync Rust — no async runtime | Use std::thread if parallelism needed |
| Using ndarray/rayon/glam for math | No third-party math crates | Use bahyway-algebra pure Rust functions |
| Emitting an event without StoryWay write | Every decision must be auditable | Always emit KAKI-stamped event to story-engine |
| Healing a particle without .akk file | Self-healing must be transparent | Always use EquationEngine → EMIT → .akk → compile |

---

## Part F — Sovereign Constants Reference

These values are eternal. They are never changed by any process. Any PR that modifies them must be rejected.

| Constant | Value | Source | WHY ETERNAL |
|---|---|---|---|
| QUALITY_DIVISOR | 240.0 | ADR-012 | Derived from Plimpton 322. Bytes 241–255 reserved for signals. |
| GEM_B11 | 200 | ADR-001 | H(P) ≥ 0.833 — the Golden Record threshold in all sovereign domains |
| TRIBE_B11 | 140 | ADR-001 | H(P) ≥ 0.583 — full tribal membership |
| ACTIVE_B11 | 100 | ADR-001 | H(P) ≥ 0.417 — active but sub-optimal |
| FUZZY_DEAD_BOUNDARY | 59 | ADR-001 | Below this: steward review required |
| PARTICLES_PER_TRIBE | 7 | Heptagon | 1 Particle Unit = 7 KAKI particles. Heptagon constant. |
| SHC | 1.0/255.0 | Heptagon | Sovereign Heptagon Constant — normalisation factor |
| τ_R7 | 11 | Orbital | Rule 7 density trigger — never lowered (prevents premature collapse) |
| resonance_radius | 0.15 | Orbital | SPH kernel bandwidth — calibrated to 7-particle tribe clusters |
| cluster_thresh | 3 | Orbital | Minimum cluster size before density effects apply |
| δ_frag | 0.35 | ADR-008 | VGCA-Δ fragmentation threshold (binary block analysis) |
| PA13_THETA_MAX | 1.0 | PA-13 | Maximum trajectory momentum before FUZZY transition |
| ENKIWAY_PORT | 9080 | DR-001 | Sovereign service port — changing breaks all consumers |
| UNRESOLVED_COLOR | #FF44AA | Visual | Catppuccin Mocha Pink — FUZZY particle visual identity |
| GEM_RATE_TARGET | 0.354 | ADR-004 | 35.4% of batch must be GEM — SLA floor |
| STEWARD_RATE | 0.0946 | ADR-004 | 9.46% of batch may enter steward queue — SLA ceiling |

---

## Part G — Implementation Status

As of v4.0.0-dev (2026-05-30):

| Process | Owner Crate | Status | Tests |
|---------|-------------|--------|-------|
| P01 KAKI Birth | `enkidb-kaki` | ✅ Built | ✓ |
| P02 VaultGate | `enkidb-persist` + `enkidb-storage` | ✅ Built | ✓ |
| P03 VGCA Cleansing | `vgca-engine` | ⚙ Pending | — |
| P04 HeptaScore | `hepta-score` | ✅ Built | 27 tests |
| P05 Lane Assignment | `hepta-score::QualityLane` | ✅ Built | ✓ |
| P06 Orbital Mechanics | `bahyway-algebra::orbital` | ✅ Built | 11 tests |
| P07 Rule 7 | `tribe-orbit-engine` | ⚙ Pending | — |
| P08 StoryWay | `story-engine` + `enkidb-journal` | ✅ Built | ✓ |
| P09 AMMAS Kinetic | `ammas-engine` | ⚙ Pending | — |
| P10 .akk Compilation | `aaol` | ✅ Built | 28 tests |
| P11 OBSERVE Query | `heptascript` | ✅ Built | ✓ |
| P12 Self-Healing | `ammas-engine::EquationEngine` | ⚙ Pending | — |
| P13 Dashboard | `dubsar-visualizer` | ✅ Built (4 panels) | ✓ |
| P14 DataSteward | `data-steward-station` | ✅ Built | ✓ |
| P15 Orbital Trust Probe | `orbital-trust-probe` | ✅ Built | 22 tests |
| PA-13 Shells | `bahyway-algebra::shells` | ✅ Built | 12 tests |
| PA-14 Orbital | `bahyway-algebra::orbital` | ✅ Built | 11 tests |
| PMPVD AkkIR | `aaol::pmpvd` | ✅ Built | 14 tests |
| H(P) Equation | `hepta-score` | ✅ Built | 27+ tests |
| Hepta 7D Types | `hepta-score::domain` | ✅ Built | ✓ |
| Topology (20 ops) | `bahyway-algebra::topology` | ✅ Built | ✓ |

**55 crates total in workspace.**

Next implementation priorities:
1. `vgca-engine` — VGCA-Σ FSV computation + GeometricFit scoring
2. `tribe-orbit-engine` — SPH density + Rule 7 + orbital ring assignment
3. `ammas-engine` — BehaviorPolicy + kinetic equation + EquationEngine
4. 14 linguistic types in `enkidb-dictionary` (ADR-010)

---

**𒁾 The tablets are preserved. Build with Sovereignty.**

DUB.SAR 𒁾 — Bahaa Fadam · BahyWay.Ecosystem v4.0 · Amsterdam
