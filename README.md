# 𒁾 Ishtarv1 — BahyWay.Ecosystem v4.0, Final Build

**This is the official, final-build snapshot of BahyWay.Ecosystem v4.0
prior to production.** If you have found this repository among many
others carrying the `BahyWay`/`bahyway_v4` name, this is the one to
trust: it exists specifically to answer "which of these is real?" —
everything else is working history, this is the accepted result.

## What this is

- A complete, working copy of the ecosystem — the full `workspace/bahyway_v4`
  Cargo workspace (all crates, all path-dependencies intact and buildable),
  every sealed and draft law document, every playbook, every prototype,
  the three public websites. Nothing partial: this repo builds and runs
  on its own, the same way the source it was cut from does.
- **A single snapshot branch (`main`), not a multi-branch OTAP pipeline.**
  Day-to-day development and the `dev → test → accept → master` promotion
  discipline continue to live in `bahyway/EnkiDB` (see
  `docs/08_pipeline_alaktu/OTAP_PIPELINE.md` and
  `playbooks/playbook_557_production_golive_from_accept.yml` in this
  snapshot for how that works). This repository receives the *finished
  result* each time a milestone is accepted — it is not itself where
  promotion happens.
- Re-cut (a fresh commit, deliberately not a merge or a history import)
  each time a new milestone is accepted, so this repo's own history stays
  small and legible instead of accumulating the sprawl that motivated
  creating it in the first place.

## What this is not

- Not a fork meant for independent development. Changes belong in
  `bahyway/EnkiDB`; this repo is a mirror of accepted results, not a
  place to branch new work from.
- Not one of the ~800 working/experimental `bahyway_v4`-related
  repositories that have accumulated over time. Those remain what they
  were — session history, drafts, one-off experiments. None of them are
  "the" official build. This one is, by construction and by declaration.

## Provenance

| | |
|---|---|
| Source repository | [`bahyway/EnkiDB`](https://github.com/bahyway/EnkiDB) |
| Source commit | `4d1a07f5fa23833c633b0f25aa0033647cef7bbe` |
| Source branch | `master` |
| Source tag | [`gudea-v4.0-sealed`](https://github.com/bahyway/EnkiDB/releases/tag/gudea-v4.0-sealed) |
| Cut date | 2026-08-27 |
| Cut by | DUB.SAR 𒁾 (Bahaa Fadam), via Claude Code |
| Previous cut | `69e5deae478418085922344be0efb8921f2a0e7e` (2026-08-15) |

**What changed since the previous cut:** the build phase of BahyWay.Ecosystem
v4.0 closed for real. The Palû Crossing (law `GL-PAL-001`, coordinator
`IsimudEngine`) sealed the **Gudea** reign (v4.0) at `2026-08-27T12:37:44Z`
across all seven sovereign databases and opened the **Zagesi** reign (v4.1)
for new work — proof in `palu/COMPLETION-STELE-4.0.tsv` and
`palu/PALU-STELE-4.0-to-4.1.md`, both carried in this cut. Since the previous
cut: `enkidb-con-engine` gained **CSR-08 Architect Sovereignty**, the eighth
Connection Security Rule, append-only-honest by design (`Create`/`Supersede`/
`Retire`, never a literal modify/delete — see `docs/18_security/CONENGINE_CSR.md`);
`enkidb-ingest::kispu` landed the four-way atomic commit binding the Event
KAKI, the NATIRU orbital-range index, the zakāru audit journal, and the
Orbital position into one all-or-nothing write; `pdm-shape-admission` shipped
as GL-EAV-001 Layer 2's first real implementation; 17 DRAFT law tablets
(`GL-NSR-001`, `GL-LBR-001`, `GL-NJF-001`, `GL-DST-004`, `GL-VSL-001`,
`GL-SHP-001`, `GL-ISM-001`, `HS-EXT-003`, and nine more) were sealed with
their matching crates built and tested; the E-004/E-005 performance gates
were closed for real against the production Read Node path; and two real
bugs found live on bare-metal hardware (`playbook_687`'s stale-store reset,
`playbook_688`'s SELinux/traversal-permission chain) were diagnosed and
fixed. The workspace grew from 79 to **261 crates**. Full workspace
`cargo check --workspace` verified clean before this cut (1m31s, exit 0);
a full `cargo test --workspace` run across all 261 crates was not re-run
for this specific cut — the individual crates landed this era each carry
their own passing `cargo test -p <crate>` counts documented in their own
tablets and `docs/18_security/CONENGINE_CSR.md`.

**Honest note on the source branch**: this cut is taken from `master`
directly — unlike the previous cut (taken from a Phase 2 integration branch
because the accepted work hadn't reached `master` yet at that time), the
Gudea-era build phase is sealed on `master` itself, so this and future
re-cuts should keep tracking `master` going forward.

To verify this snapshot against its source at any time:
```bash
git clone https://github.com/bahyway/EnkiDB
cd EnkiDB
git diff 4d1a07f5fa23833c633b0f25aa0033647cef7bbe -- . ':(exclude).git'
# compare against a checkout of this repo's main branch -- should be empty
# except for this README.md's Ishtarv1 framing above and below the divider,
# which the source repository does not carry.
```

---

# 𒁾 EnkiDB — BahyWay.Ecosystem v4.0

<div align="center">

**A Sovereign Data-Physics Operating System**

*Pure Rust · Zero External Runtime Dependencies · Orbits-Oriented Ontology*

[![Language](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-1%2C804%20passing-brightgreen)](#build--test)
[![Crates](https://img.shields.io/badge/crates-261-blueviolet)](#architecture)
[![Era](https://img.shields.io/badge/era-Gudea%20v4.0%20sealed-8a2be2)](#v40--build-phase-sealed-gudea-era)
[![License](https://img.shields.io/badge/license-Proprietary-red)](#license)

> *𒁾 DUB.SAR — Written in the Tablet House of Eridu*
> © Bahaa Fadam — BahyWay Sovereign Ecosystem

</div>

## KAKI — Sovereign Declaration

**KAKI (Knowledge–Akkadian–Keyword–Identity)**

KAKI is BahyWay Ecosystem v4.0's sovereign approach to Semantic Data Modeling (SDM) for deterministic Entity Resolution across heterogeneous data sources.

The name carries two layers of meaning:

**Etymological** — from the Akkadian *kaku* (𒋼𒁀): armament, seal, sovereign mark. In ancient Mesopotamia, a cylinder seal (*kaku*) was the proof of identity and authority — pressed into clay, impossible to forge without the original seal. BahyWay's KAKI is the digital equivalent: a 16-byte sovereign seal minted once at birth and immutable for the particle's lifetime.

**Semantic** — Knowledge–Akkadian–Keyword–Identity encodes the four pillars of the framework:
- **Knowledge** — the particle carries structured semantic knowledge about a real-world entity via its EAV attribute space
- **Akkadian** — the system's philosophical and etymological root: the first writing system that encoded sovereign identity
- **Keyword** — Knowledge-Aware Keyword Indexing (KAKI) resolves entity identity through seven native sovereign indexes, not through probabilistic string matching
- **Identity** — each particle has one and only one KAKI — its permanent, immutable, sovereign coordinate in the data universe

**Implementation — Knowledge-Aware Keyword Indexing:**
KAKI resolves records representing the same real-world entity through Semantic Data Modeling rather than probabilistic similarity scoring. Identity is determined by sovereign structure: tribe membership (`κ[4..5]`), content hash (`κ[0..3]`), creation ordinal (`κ[8..11]`), and sovereign epoch (`κ[12..13]`). The 7D VGCA quality vector — not an ML embedding — provides the geometric measure of how close a particle is to its sovereign ideal. Entity resolution is deterministic, auditable, and requires no external model, no training data, and no neural network.

---

## What Is EnkiDB?

**EnkiDB** is the sovereign development repository for **BahyWay.Ecosystem v4.0** — a self-contained, pure-Rust data-physics operating system that governs the entire lifecycle of data through a formalism called **Triple-O (Orbits-Oriented Ontology)**.

It is not a traditional database, framework, or library. It is an *ecosystem* — a living, self-regulating data organism where every piece of information is a **particle** that orbits its sovereign ideal point, governed by seven tribal laws, and scored by the H(P) quality equation.

> *"Data does not sit in tables. Data orbits."*

### Named After

**Enki** (𒀭𒂗𒆤) — the Sumerian god of wisdom, water, and craftsmanship. Keeper of the *me* (divine laws that govern the universe). In the BahyWay philosophy, the database is not a passive store but an active sovereign — it knows, judges, and governs the data entrusted to it.

---

## Repository Structure

```
EnkiDB/
├── workspace/
│   └── bahyway_v4/          ← Main v4.0 sovereign ecosystem (261 crates)
│       ├── Cargo.toml        ← Workspace root
│       ├── ROADMAP.md        ← Full v4.0 roadmap with W5H2 crate entries
│       ├── SOVEREIGN_MANUAL_STEP1.md  ← Layer-by-layer W5H2 reference
│       └── crates/           ← 261 sovereign crates across 12+ layers
│           ├── bahyway-fabric/        ← NEW L8: Enterprise Data Fabric
│           ├── bahyway-dqm/           ← NEW L8: Data Quality Metrics Engine
│           └── data-cleansing-station/← UPGRADED: VGCA-powered cleansing
├── docs/
│   └── FABRIC_DASHBOARD_PROMPT.md  ← 3-part dashboard implementation spec
├── lab_creation_docs/        ← Architecture, design references, ADRs
├── bahyway.sh                ← Sovereign environment orchestrator: env · scaffold · build
├── docker-compose.yml        ← Sovereign container stack
└── Dockerfile                ← Rust compiler image (Debian slim + LLD)
```

---

## Philosophy: Triple-O — Orbits-Oriented Ontology

Every design decision in BahyWay v4.0 flows from three axioms:

### Axiom 1 — Every Datum Is a Particle

A datum is not a row, document, or record. It is a **particle** with position, velocity, density, and quality. It obeys the laws of its tribe and orbits its sovereign ideal.

### Axiom 2 — Identity and State Are Permanently Separated

```
┌──────────────────────────────────┬──────────────────────────────────┐
│  NUCLEUS  (KAKI)                 │  ORBIT  (EAV)                    │
│  Immutable · Born once           │  Mutable · Event-driven          │
│                                  │                                  │
│  uuid_hash    [4 bytes]          │  quality score    (B11)          │
│  tribe_id     [2 bytes]          │  attributes       (key/value)    │
│  kaki_type    [1 byte]           │  state history    (Event-Kakis)  │
│  kaki_role    [1 byte]           │  assessments                     │
│  reserved     [4 bytes]          │                                  │
│  timestamp    [2 bytes]          │  → Lives here, never in nucleus  │
│  checksum     [2 bytes]  ──────► │    (Structural-Facts Rule §2.4)  │
└──────────────────────────────────┴──────────────────────────────────┘
              16 bytes total — eternal
```

### Axiom 3 — Quality Is Orbital Distance

A particle's quality is its distance from the sovereign ideal point, computed by the **H(P) equation**:

```
H(P) = 1 / (1 + √ Σ wᵢ(Pᵢ − Tᵢ)²)

B11 = round(H(P) × 240)    ← ADR-001: divisor is ALWAYS 240, NEVER 255
```

| B11 | Lane | Ring | Status |
|-----|------|------|--------|
| ≥ 200 | **GEM** | Inner | Sovereign ideal — calibrates the centroid |
| 140–199 | **TRIBE** | Inner-Mid | Core member |
| 100–139 | **ACTIVE** | Mid | Participating |
| 60–99 | **FUZZY** | Mid-Outer | Degrading — monitored |
| < 60 | **DEAD** | Outer | Rule 7 sink candidate |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    BahyWay.Ecosystem v4.0                           │
│              261 Crates · 5 Binaries · Pure Rust                    │
├─────────────────────────────────────────────────────────────────────┤
│ L12  bahyway-web                                                    │  ← Website / WASM UI
│ L11  dubsar-ide · dubsar-visualizer                                 │  ← UI / IDE
│ L10  eridu-runtime · eridu-scheduler · eridu-supervisor             │  ← OS Runtime
│ L9.5 enkidullm-core · enkidullm-ingest · zikru-embed · audit       │  ← Sovereign LLM
│ L9.1 kupru · akkvalue · istar                                       │  ← Crypto / Access
│ L9   aaol (.akk) · heptascript (.hepta) · akkadi · akkadi-ir       │  ← Languages
│ L8   bahyway-fabric ◄NEW · bahyway-dqm ◄NEW                        │  ← Data Fabric + DQM
│      adad-gate · musaru-security · data-cleansing-station ◄UPGRADED │
│      data-structure-station · data-steward-station · permanent-storage
│ L7   template-engine · damadmbok-dictionary · diagnosis-engine      │  ← Governance
│ L6   idu-prober · idu-batching                                      │  ← Cross-Tribe
│ L5   najaf · wpd · dmw · nusku · azuga · panam · shulman · …       │  ← Engines (18)
│      quant-engine ◄NEW (returns · risk · factor model · particles)  │
│ L4.5 vgca-engine · tribe-orbit-engine · ammas-engine               │  ← Physics
│ L4   enkidb-engine · enkidb-query                                   │  ← DB Engine
│ L3   enkidb-indexes · enkidb-dictionary                             │  ← Indexes
│ L2   enkidb-block · journal · storage · snapshot · persist · dw     │  ← Storage
│      enkidb-quantdb ◄NEW (tick/OHLC time-series, fixed-point)       │
│ L1   enkidb-kaki · enkidb-vector-id                                 │  ← Identity
│ L0   bahyway-core · bahyway-crc · bahyway-algebra                  │  ← Foundation
└─────────────────────────────────────────────────────────────────────┘
```

---

## The Sovereign Answer to Enterprise Spaghetti Processing

Enterprise data pipelines typically look like this: 8 sources routed through 10 tangled processing nodes to 7 targets — no schema enforcement, no traceability, silent failures.

**BahyWay.Ecosystem v4.0 solves this with Layer 8: the Enterprise Data Fabric.**

```
BEFORE — Spaghetti Processing
  ERP ─┐
  CRM ─┼──→ Node A ──→ Node C ──→ Node E ──→ DW ?
  HR  ─┤      ↕          ↕          ↕
  ...  └──→ Node B ──→ Node D ──→ ... ──→ ???
  (No schema · No lineage · Silent failures · No audit)

AFTER — BahyWay Fabric (bahyway-fabric)
  ERP ──SchemaContract──┐
  CRM ──SchemaContract──┤            ┌──SchemaContract──→ DW
  HR  ──SchemaContract──┤            ├──SchemaContract──→ Reports
  ...                   └──Pipeline──┤
                          │          └──SchemaContract──→ Dashboard
                          │
                  Stage::Cleanse  ← VGCA 7D+6D geometry
                  Stage::Validate ← DQM 6-dimension + SLA
                  Stage::Dedup    ← idu-prober cross-tribe
                  Stage::Enrich   ← shulman-engine
                          │
                   LineageChain   ← every hop auditable
                   FabricException← every failure typed
```

### The 5 Sovereign Fabric Guarantees

| Guarantee | Mechanism |
|-----------|-----------|
| **Schema at every boundary** | `SchemaContract` validated at source extraction and target delivery |
| **Every record has a lineage chain** | `LineageChain` — append-only, FNV-1a hashed, audit-ready |
| **No silent failures** | `FabricException` — 8 typed variants; dead-letter queue or halt |
| **VGCA geometric data cleansing** | FSV 7D (text) + BFV 6D (binary); Alien fields stripped at `Stage::Cleanse` |
| **Data quality SLA enforcement** | `bahyway-dqm` 6 DAMA-DMBOK dimensions; DQM composite B11 gate at `Stage::Validate` |

---

## Data Quality — DAMA-DMBOK 6 Dimensions (`bahyway-dqm`)

Every particle passing through the pipeline is scored across **six industry-standard quality dimensions**:

| Dimension | Algorithm | B11 Impact | DAMA-DMBOK |
|-----------|-----------|------------|------------|
| **Completeness** | Field-presence counting vs SchemaContract | Fails if required fields missing | §11.4 |
| **Validity** | Deterministic Rule Engine + Welford Z-score | Fails on rule violation or outlier | §11.5 |
| **Accuracy** | Merkle Tree FNV-1a lineage integrity | Fails if any hop hash mismatch | §11.6 |
| **Consistency** | Cross-field 0xFF conflict sentinel detection | Penalises conflicting fields | §11.7 |
| **Uniqueness** | Levenshtein + Jaro-Winkler + Soundex (NARA) | Score from idu-prober dedup | §11.8 |
| **Timeliness** | Epoch freshness window — linear decay | 0.0 if lag ≥ freshness_window | §11.9 |

```
DqmEngine::assess_record(record, rules, required_count, epoch_now, epoch_source, window)
  → DqmReport {
      composite:    f32,   // mean of 6 dimension scores
      composite_b11: u8,  // B11 = round(composite × 240)  ← ADR-001
      sla_compliant: bool, // all 6 dimensions ≥ their SLA threshold
    }
```

### SLA Presets

| Preset | Completeness | Validity | Accuracy | Consistency | Uniqueness | Timeliness |
|--------|-------------|----------|----------|-------------|------------|------------|
| `enterprise()` | 0.98 | 0.95 | 0.90 | 0.95 | 0.99 | 0.90 |
| `master_data()` | 1.00 | 0.99 | 0.99 | 0.99 | 1.00 | 0.95 |
| `exploratory()` | 0.80 | 0.75 | 0.70 | 0.75 | 0.90 | 0.70 |

---

## VGCA — Vector Geometric Cleansing Analysis (`vgca-engine`)

VGCA classifies data geometrically in two algorithm families:

### VGCA-Σ — 7D Feature Score Vector (text fields)
```
D1 char_count_norm      — normalised character count
D2 digit_density        — fraction of digit chars
D3 arabic_density       — fraction of Arabic Unicode
D4 latin_density        — fraction of Latin ASCII
D5 punct_density        — fraction of punctuation
D6 word_count_norm      — normalised whitespace tokens
D7 shannon_entropy_norm — normalised Shannon entropy

→ GeometricFit score = 1 / (1 + ||fsv − centroid||₂)
→ Clean ≥ 0.80  |  Suspect 0.50–0.79  |  Outlier < 0.50  |  Alien < 0.05
```

### VGCA-Δ — 6D Binary Feature Vector (binary fields)
```
B1 byte_entropy     — Shannon byte entropy
B2 null_density     — fraction of null bytes
B3 printable_ratio  — fraction of printable ASCII
B4 high_byte_ratio  — fraction of high bytes (0x80–0xFF)
B5 run_length_norm  — longest run / block length
B6 pattern_score    — repeating 4-byte pattern strength

→ fragmentation_score > δ_frag (0.35) → SUSPECT
```

### Self-Calibrating Domain Centroid
The centroid is the domain's living definition of "ideal data".
**Only GEM-lane particles (B11 ≥ 200) update it.** Dead and Suspect particles never do —
so the centroid always converges toward the sovereign data ideal, never toward degraded data.

```
ADR-004 GEM Rate Target: 35.4%
— At least 35.4% of particles must reach the GEM lane per pipeline run.
— The GEM rate is displayed on the Dashboard with a green/red ADR-004 indicator.
```

---

## Particle Orbit Visualization

The Fabric Dashboard shows three live canvases depicting particle motion:

```
┌────────────────────────────────────────────┐
│        𒀭 Tribe Orbit Lanes                 │
│   ·  · · DEAD · · ·  (r=220, dark blue)   │
│      · FUZZY · (r=180, red)               │
│       ACTIVE (r=140, orange)              │
│       TRIBE (r=100, cyan)                 │
│         ⊕ GEM (r=55, green)              │
│   Centroid pulsing at origin              │
│   New particles spiral inward (60 ticks)  │
└────────────────────────────────────────────┘
```

```
┌────────────────────────────────────────┐
│  𒁾 VGCA 7D → 2D Manifold Projection    │
│  X: latin_density  Y: arabic_density   │
│  Dots coloured by lane, sized by B11   │
│  Centroid crosshair at ⊕              │
└────────────────────────────────────────┘
```

```
┌────────────────────────────────────────┐
│  ⬛ Orbit Lane Density Distribution     │
│  GEM▓▓▓│TRIBE▓▓│ACTIVE▓│FUZZY│DEAD   │
│         ▲ ADR-004 target 35.4%        │
└────────────────────────────────────────┘
```

---

## Enterprise Applications

### 🏛️ NajafEngine — Wadi al-Salam Sovereign Cemetery Navigation

Navigates the world's largest cemetery (Wadi al-Salam, Najaf, Iraq) across 7 sovereign sectors. Routes pilgrims from the gate to a specific burial site using A* pathfinding over a sovereign NaviGraph.

```
Pilgrim Request → GraveRegistry (7 sectors) → A* routing
→ PilgrimRoute → Arabic/English verdict (shulman-engine)
```

### 🔧 WPDEngine — Baghdad Water Pipeline Defect Detection

Detects water leaks, oil spills, and sewage blockages across Baghdad's 7-sector pipeline network using 12-band spectral imaging (VNIR + SWIR + TIR) from IR cameras.

```
IR Camera (iris-engine) → 12-band SpectralScan
→ DefectClassifier (Water/Oil/Sewage/Structural)
→ RepairScheduler + RepairNavigator → KAKI audit trail
```

### 📊 DMWEngine — SQL Query Intelligence (7-Level Journey)

Analyses SQL query execution plans in pure Rust — no database connection required. TheOracle ranks APPLY-ROTATION fixes and produces a ColorId health byte.

```
QueryPlan → BottleneckDetector (16 patterns) → 7-Level Journey
→ TheOracle (APPLY-ROTATION) → ColorId + Narrative report
```

### 🏥 Nusku Bio-Security Pipeline — Sovereign Thermal Screening

Real-time thermal biometric screening pipeline. Detects 9 medical conditions and security threats in ≤ 50ms per frame.

```
FLIR/Optris/SEEK (iris-engine) → Face detect (panam-engine)
→ Medical inference (azuga-engine) → Mamdani fuzzy (nusku-fuzzy)
→ Aggregate score (nusku-score) → Arabic verdict (shulman-engine)
```

### ⚛️ AMMAS — Kinetic Particle System

Evolves the entire tribe in discrete timesteps under the master kinetic equation:

```
∂f/∂t = J_phys[f]  ← orbital SPH density + Rule 7 sink
       + J_mem[f]   ← 7 Tribal Law transitions
       + J_learn[f] ← H(P) quality drift
```

---

## Key Sovereign Constants

| Constant | Value | Rule |
|----------|-------|------|
| `QUALITY_DIVISOR` | **240** | ADR-001 — from Plimpton 322. Eternal. |
| `PARTICLES_PER_TRIBE` | **7** | Heptagon constant. Sovereign. |
| `SHC` | **1/255** | Sovereign Heptagon Constant. |
| `TAU_R7` | **11** | Rule 7 overflow density trigger. |
| `RESONANCE_RADIUS` | **0.15** | SPH kernel bandwidth. |
| `GEM_B11` | **≥ 200** | GEM lane threshold. |
| `GEM_RATE_TARGET` | **35.4%** | ADR-004 — sovereign SLA. |
| `DELTA_FRAG` | **0.35** | ADR-008 — VGCA-Δ fragmentation threshold. |
| `OOO_LAYERS` | **8** | ADR-008 + ADR-009 — five original + three added 2026-06-05. |
| `PIPELINE_BUDGET_MS` | **50 ms** | Max thermal frame budget. |

> These constants are **eternal**. Any PR modifying them is rejected.

---

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup` recommended)
- `lld` linker (`dnf install lld` on Fedora / `apt install lld` on Debian)

### Using the Sovereign Orchestrator

```bash
# Full unattended environment setup + build
bash bahyway.sh --all --yes

# Individual phases
bash bahyway.sh --env        # Fedora packages + Rust toolchain
bash bahyway.sh --scaffold   # Create workspace structure
bash bahyway.sh --build      # Build all 261 crates
bash bahyway.sh --verify     # Verify everything compiles
bash bahyway.sh --build --release  # Release build
```

### Using Docker

```bash
# Start the sovereign container stack
docker-compose up -d

# Enter the Rust compiler container
docker exec -it bahyway_rust_compiler bash
```

### Manual Build & Test

```bash
cd workspace/bahyway_v4

# Build entire ecosystem
cargo build --workspace

# Run all tests
cargo test --workspace

# Test the new Layer 8 crates
cargo test -p bahyway-fabric
cargo test -p bahyway-dqm
cargo test -p data-cleansing-station
cargo test -p vgca-engine

# Test physics and intelligence engines
cargo test -p ammas-engine
cargo test -p tribe-orbit-engine
cargo test -p hepta-score
cargo test -p najaf-engine
cargo test -p wpd-engine
cargo test -p dmw-engine
```

**Last full-suite snapshot: 1,804 tests · 0 failures · 0 unsafe code · 0 external runtime deps**
*(pre-dates the Gudea-era additions below — CSR-08, KISPU, pdm-shape-admission, and the
17 C-4 tablet crates each carry their own passing `cargo test -p <crate>` counts documented
in their own tablets; re-run `cargo test --workspace` for the current combined total rather
than trusting this badge alone.)*

---

## Crate Glossary — Akkadian Names

| Crate | Akkadian Root | Role |
|-------|--------------|------|
| `bahyway-core` | *bāhu* (arm/power) | Zero-dep foundation types |
| `bahyway-algebra` | *bāhu* + algebra | H(P), shells, orbital, JNF |
| `enkidb-kaki` | *kakkum* (sovereign seal) | 16-byte eternal identity |
| `enkidb-engine` | *Enkidu* (from the steppe) | Main DB orchestrator |
| `hepta-score` | *hepta* (7) | H(P) equation, B11, QualityLane |
| `adad-gate` | *Adad* (storm god) | Sole ingestion entry point |
| `bahyway-fabric` ◄ NEW | Enterprise Data Fabric | SchemaContract, LineageChain, FabricOrchestrator — sovereign answer to spaghetti processing |
| `bahyway-dqm` ◄ NEW | Data Quality Metrics | 6 DAMA-DMBOK dimensions: Completeness, Validity, Accuracy, Consistency, Uniqueness, Timeliness |
| `data-cleansing-station` ◄ UPGRADED | Sovereign Cleansing | VgcaCleansingStation — VGCA geometry + DQM scoring + ParticleLane + OrbitDensitySnapshot |
| `vgca-engine` | Vector Geometric Cleansing | VGCA-Σ FSV 7D (text) + VGCA-Δ BFV 6D (binary) + self-calibrating DomainCentroid |
| `ammas-engine` | *ammatu* (sovereign measure) | Kinetic equation J_phys+J_mem+J_learn |
| `tribe-orbit-engine` | tribal orbit | SPH density, 7 Laws, OrbitalRing |
| `najaf-engine` | *Najaf* (city of Imam Ali) | Cemetery pilgrim navigation |
| `wpd-engine` | *wūppu* (leakage) | Water pipeline defect detection |
| `dmw-engine` | *dāmiqtu* (path of good) | SQL query plan advisor |
| `nusku-engine` | *Nusku* (god of fire/light) | Shared bio-security types |
| `azuga-engine` | *azugallū* (chief physician) | Medical thermal inference |
| `iris-engine` | *Iris* (vision) | IR camera hardware abstraction |
| `panam-engine` | *pānu* (face/front) | Face detection + KAKI face PK |
| `shulman-engine` | *Šulmu* (peace) | Arabic/English verdicts + audit |
| `homt-engine` | Hepta-Orbit Manifold | OrbitField + PDE convergence |
| `aaol` | *ayyālu* (actor/runner) | .akk language + PMPVD 9-node IR |
| `heptascript` | *hepta* (7) | .hepta 7D query language |
| `akkadi` | *Akkadu* (city of Akkad) | Sovereign query language |
| `kupru` | *kuprum* (copper/seal) | Ed25519 sovereign sealing + entropy |
| `akkvalue` | *akk* + value | Sovereign value types (decimal, ratio) |
| `istar` | *Ištar* (goddess of war) | Access control + capability tokens |
| `eridu-runtime` | *Eridu* (first city of Sumer) | Cooperative task executor |
| `dubsar-ide` | *dub-sar* 𒁾𒊬 (tablet writer) | VSCodium sovereign IDE |
| `dubsar-visualizer` | *dub-sar* + vision | Particle inspector (Canvas 2D) |
| `permanent-storage` | *dāru* (eternal) | Golden Records vault |
| `enkidb-dictionary` | *tēmēnu* (foundation stone) | EAV attr_name → attr_hash |
| `story-engine` | *siprū* (writing) | CQRS read-side projection |
| `musaru-security` | *mūsarû* (inscribed tablet) | Security validation station |
| `enkidb-quantdb` | *enkidb* + quant | Sovereign tick/OHLC time-series store (Layer 2) |
| `quant-engine` | *šipru* (work) + quant | Sovereign quant analytics: returns, risk, factor model (Layer 5) |

*Full 261-crate glossary with workflow diagrams: [`workspace/bahyway_v4/ROADMAP.md`](workspace/bahyway_v4/ROADMAP.md)*

---

## The 7 Tribal Laws

```
Priority (lowest → highest):

  L3  Scope Law        — defines domain boundary
  L7  Overflow Law     — Rule 7 density sink (density ≥ 11)
  L1  Identity Law     — locks the nucleus; prevents drift
  L2  Quality Law      — promotes particles that improve H(P)
  L5  Transition Law   — stabilises FUZZY particles
  L4  Orbit Law        — governs ascent toward GEM ring
  L6  Sovereignty Law  — supreme veto; protects GEM particles
```

When multiple laws fire simultaneously, the highest-priority law wins.

---

## Sovereign Rules — What We Never Do

1. **No third-party runtime dependencies.** No tokio, postgres, ndarray, uuid (external), rayon. Pure Rust stdlib only.
2. **No unsafe code.** `#![forbid(unsafe_code)]` on all physics and intelligence crates.
3. **No state in the nucleus (§2.4).** Quality, assessments, and mutable values live exclusively in the EAV Orbit via Event-Kakis.
4. **Sovereign constants are eternal.** B11 divisor = 240, PARTICLES_PER_TRIBE = 7, TAU_R7 = 11, DELTA_FRAG = 0.35. PRs modifying these are rejected.
5. **No SHA-256 or external crypto.** FNV-1a (sovereign) for hashing. Ed25519 via `kupru` for sealing.
6. **No SQL, no PostgreSQL, no Redis.** Pure sovereign storage via `enkidb-*` crates only.
7. **No `Way` suffix on engine names.** Reserved exclusively for WAYv2.0 Language (Security Defending Layer).

---

## v4.0 — Build Phase Sealed (Gudea Era)

**The build phase of BahyWay.Ecosystem v4.0 is sealed.** At `2026-08-27T12:37:44Z`,
the Palû Crossing (law `GL-PAL-001`, coordinator `IsimudEngine`, seal `CSR-08`) closed
the **Gudea** reign (v4.0) across all seven sovereign databases and opened the
**Zagesi** reign (v4.1) for new enhancement work. From that timestamp forward, `era="Gudea"`
is an immutable, physics-provable fact stamped on every particle of the closed reign —
the registry does not reset, it stratifies. Proof lives in `palu/palu_certificates.jsonl`,
`palu/COMPLETION-STELE-4.0.tsv`, and `palu/PALU-STELE-4.0-to-4.1.md`; a durable Git marker
sits at the tag [`gudea-v4.0-sealed`](https://github.com/bahyway/EnkiDB/releases/tag/gudea-v4.0-sealed).

### Completion Stele — C-1 through C-6 (all PASS)

| Line | Requirement | Stamp | Verdict |
|------|-------------|-------|---------|
| C-1 | Deployment gates green, incl. 1B particles / 1s | MEASURED | ✅ PASS |
| C-2 | TESTING_PHASE1 Blocks A–F passed | MEASURED | ✅ PASS |
| C-3 | Arsenal census closed | DERIVED | ✅ PASS |
| C-4 | All DRAFT law tablets sealed or DEFERRED (17 tablets, `docs/09_observatory/`) | DERIVED | ✅ PASS |
| C-5 | The Five Refusals affirmed (`laws/GX-COMMENCEMENT-001.md`) | MEASURED | ✅ PASS |
| C-6 | Seven journals healthy (7 probed, 7 present) | MEASURED | ✅ PASS |

### ConEngine — Connection Sovereignty Rules (CSR-01…CSR-08)

`enkidb-con-engine` enforces eight rules on every connection request — passport
validation, role checks, audit journaling, credential expiry, cross-tribe gating,
KIBRATU event emission, tribe isolation, and (new this era) Architect Sovereignty.
12/12 tests passing (`cargo test -p enkidb-con-engine`).

| Rule | Name | Status |
|------|------|--------|
| CSR-01 | Sargon Gate | ✅ real, coded |
| CSR-02 | Role Gate | ✅ real, coded |
| CSR-03 | NĀRU Frame Journal | ✅ real, coded |
| CSR-04 | Credential Check | ⚠️ real trait, but only `StubCredentialStore` implements it |
| CSR-05 | Gilgamesh Gate | ✅ real, coded |
| CSR-06 | KIBRATU Emission | ⚠️ emission is real; the full 7-variant cause taxonomy is not |
| CSR-07 | Tribe Isolation | ✅ real, coded |
| CSR-08 | **Architect Sovereignty** | ✅ real, coded — gates any `Create`/`Supersede`/`Retire` particle affecting a crate, engine, agent, template, KAKI, tribe, session, playbook, or configuration behind `architect_confirmed`; append-only honest (no literal in-place modify/delete, per §0.3) |

Full account, including open PAZUZU threat-simulation gaps: [`docs/18_security/CONENGINE_CSR.md`](docs/18_security/CONENGINE_CSR.md).

### KISPU — the four-way atomic commit

`enkidb-ingest::kispu` binds every real write into one all-or-nothing commit across
four operations: the Event KAKI, the NATIRU orbital-range index, the zakāru audit
journal, and the Orbital position — audit-leg-first ordering, no partial writes reach
the Golden Store.

### BeeMDM — the 7-Station ETL pipeline

`bin/bee-watchdog` runs every dropped ZIP through the real per-record station chain:

```
adad-gate (sole KAKI issuer) → DataStructureStation → data-cleansing (DAMA-DMBOK)
→ VGCA beam → client-dq-profile → score-engine (→ B11)
→ routed by B11: Gem/Tribe → Golden · Active → Fuzzy+Steward · Dead → PersistedDb only
```

preceded by the Musarû security gate, VGCA-Δ zip-bomb detection, and a
compare-tribe-schema gate. Full account: [`docs/08_pipeline_alaktu/BEEMDM_ETL_PIPELINE.md`](docs/08_pipeline_alaktu/BEEMDM_ETL_PIPELINE.md).

### What's next — Testing Plan Phase 1

With the build phase sealed, the ecosystem enters its Testing Plan (tablet `GL-TST-001`,
still DRAFT): layers L0 (Kernel) through L10 (Watcher), a TP-00…TP-13 playbook register,
and corpora C-01…C-07 including a deterministic 1-billion-particle Wadi al-Salam corpus
generator with an adversarial taint layer for defect-detection grading.

---

## Development Branches

| Branch | Purpose |
|--------|---------|
| `master` | **Sealed v4.0 trunk** — tag `gudea-v4.0-sealed` marks the Gudea-era build-complete state |
| `claude/ecosystem-delivery-rd1ksb` | Active Zagesi/v4.1 delivery branch — PRs against `master` open from here |
| `claude/eriduscaffold-existence-91xe19` | Bare-metal tracking branch — kept fast-forwarded to mirror what actually runs on the Fedora host (`uruk`/`girsu`) |

---

<!-- BEGIN PB-686 WED0826 LANDINGS -->
## Recent Landings — Wed0826 (2026-08-26)

Four law tablets (GL-AGT-001 the Watcher, GL-BND-001 the Band, GL-IMM-001 the shared
membrane, GL-ALG-001 the Algebra Register) landed in `playbooks/files/` and
`docs/09_observatory/`, each with a seal playbook (`playbook_681`..`playbook_684`) — as
of this run, all four sealed real Ed25519 receipts confirmed present at ~/bahyway/laws/receipts/.
GL-ALG-001's four visualization tabs (`sala_algebra_register_v1..v4.html`) are promoted by
`playbook_685`. All five are walked by `playbook_672`'s extended backlog
(`playbooks/data/run_manifest_672_backlog.yml`, group `wed0826_corpus`).

See `docs/00_codex/BahyWay_Ecosystem_Manual.md` §6 and
`docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` (Wed0826 addenda) for the full account.

---
<!-- END PB-686 WED0826 LANDINGS -->
## Documentation

| Document | Location | Contents |
|----------|----------|---------|
| Algebra Glossary | [`workspace/bahyway_v4/ALGEBRA_GLOSSARY.md`](workspace/bahyway_v4/ALGEBRA_GLOSSARY.md) | Every algebraic concept: definition · BahyWay action · escalation map · build order for Rust implementation |
| Ecosystem ROADMAP | [`workspace/bahyway_v4/ROADMAP.md`](workspace/bahyway_v4/ROADMAP.md) | W5H2 entries for every crate added in 2026, ADR references, full architecture map, HOW MUCH metrics |
| Sovereign Manual | [`workspace/bahyway_v4/SOVEREIGN_MANUAL_STEP1.md`](workspace/bahyway_v4/SOVEREIGN_MANUAL_STEP1.md) | Layer-by-layer W5H2 reference, ≥ 412 tests documented |
| bahyway-fabric Manual | [`workspace/bahyway_v4/crates/bahyway-fabric/MANUAL.md`](workspace/bahyway_v4/crates/bahyway-fabric/MANUAL.md) | Full W5H2: all 39 tests, 3 integration patterns, exception diagnosis |
| bahyway-dqm Manual | [`workspace/bahyway_v4/crates/bahyway-dqm/MANUAL.md`](workspace/bahyway_v4/crates/bahyway-dqm/MANUAL.md) | W5H2: 6 algorithm deep-dives, 5 integration patterns, 76 tests |
| Fabric Dashboard Prompt | [`workspace/bahyway_v4/docs/FABRIC_DASHBOARD_PROMPT.md`](workspace/bahyway_v4/docs/FABRIC_DASHBOARD_PROMPT.md) | 3-part Canvas 2D dashboard spec: Pipeline Map, DQM/SLA radar, Particle Orbit visualization |
| Docker Setup | [`run_docker.md`](run_docker.md) | Container start guide |

### Architecture Decision Records (ADRs)

All ADRs live in [`workspace/bahyway_v4/docs/14_decisions_adr/`](workspace/bahyway_v4/docs/14_decisions_adr/).

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | No External DB — Pure Rust sovereign storage | Accepted |
| ADR-002 | Naming Discipline — suffix and vocabulary rules | Accepted |
| ADR-003 | KAKI Sovereignty — 16-byte immutable particle identity | Accepted |
| ADR-004 | BeeMDM 4-Lane Pipeline — GEM/TRIBE/ACTIVE/FUZZY/DEAD | Accepted |
| ADR-005 | Enterprise Data Fabric as Sovereign Layer 8 | Accepted |
| ADR-006 | No DELETE + Mandatory Partitioning (corrected 2026-06-05 from KAKIv4.0 §3.1 and §9.1) | Accepted |
| ADR-007 | Mandatory Snapshot Scheduler + Index 7 sparse B-tree | Accepted |
| ADR-008 | Orbits-Oriented Ontology — 8-layer mathematical foundation, KAKI roles, 17 Forbidden Operations | Accepted |
| ADR-009 | Missing Algebra Parts: Graph Algebra (L6) + Information Theory (L7) + Markov Chains (L8) + complete hardening evaluation | Accepted |
| ADR-010 | HeptaScript Language Design — sovereign vocabulary, no SQL keywords, complete grammar specification | Accepted |

---

## License

Proprietary — All rights reserved.
© Bahaa Fadam — BahyWay Sovereign Ecosystem.

> *𒁾 DUB.SAR — Written in the Tablet House of Eridu*
> *EnkiDB · BahyWay.Ecosystem v4.0 · Pure Rust · Sovereign by Design*

---

*This page merges Ishtarv1's own snapshot framing (above the divider) with the
source repository's full technical README (below it), so everything is visible
on one page without a click-through. A pristine, byte-for-byte copy of the
source's own root `README.md` is also kept at
[`ECOSYSTEM_OVERVIEW.md`](./ECOSYSTEM_OVERVIEW.md) for the verification-diff
method above.*
