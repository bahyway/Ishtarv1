# 𒁾 BahyWay.Ecosystem v4.0

> *DUB.SAR 𒁾 — Written in the Tablet House*
> Pure Rust · Zero external runtime dependencies · Sovereign by design

---

## What Is BahyWay.Ecosystem v4.0?

**BahyWay.Ecosystem v4.0** is a sovereign, self-contained data-physics operating system written entirely in Pure Rust.
It governs the full lifecycle of data — from ingestion through identity, quality scoring, orbital placement, storage, retrieval, and AI-driven decision-making — using a single unified formalism called the **Orbits-Oriented Ontology (Triple-O)**.

It is not a database. It is not a framework. It is an *ecosystem* — a living, self-regulating data organism where every piece of information is a **particle** governed by sovereign laws.

> "Data does not sit in tables. Data orbits."

---

## Philosophy: Triple-O — Orbits-Oriented Ontology

Triple-O is the philosophical and mathematical foundation of BahyWay v4.0. Every architectural decision flows from three axioms:

### Axiom 1 — Every datum is a Particle

A datum is not a row, a document, or a record. It is a **particle** — a physical object with position, velocity, density, and quality. It obeys the laws of the tribe it belongs to, and it orbits its sovereign ideal point.

### Axiom 2 — Identity and State are Separate

Every particle has two zones:

```
┌─────────────────────────────────────────────────────┐
│  NUCLEUS (KAKI)          │  ORBIT (EAV)              │
│  Immutable · Born once   │  Mutable · Managed by     │
│  16 bytes · Sovereign    │  Event-Kakis · Projected  │
│  uuid | tribe | type |   │  quality | attrs | state  │
│  role | reserved |       │  assessments | history    │
│  timestamp | checksum    │                           │
└─────────────────────────────────────────────────────┘
```

The **KAKI nucleus** encodes identity: who you are, what tribe you belong to, when you were born.
The **Orbit** encodes state: what you know, how good you are, where you live right now.
State changes never touch the nucleus. This is the **Structural-Facts-Only Rule (§2.4)**.

### Axiom 3 — Quality is Orbital Distance

A particle's quality is its distance from the sovereign ideal point. The closer it orbits, the higher its quality score. Quality is computed by the **H(P) sovereign equation**:

```
H(P) = 1 / (1 + √ Σ wᵢ(Pᵢ − Tᵢ)²)

where:
  Pᵢ = observed value on dimension i
  Tᵢ = sovereign target on dimension i
  wᵢ = Ibn Wahshiyya Arabic MDM weight for dimension i
       [D1=0.30, D2=0.20, D3=0.15, D4=0.15, D5=0.10, D6=0.05, D7=0.05]

B11 = round(H(P) × 240)   ← ADR-012: divisor is ALWAYS 240, NEVER 255
```

The result is a single byte `B11 ∈ [0..240]` that encodes the particle's **QualityLane**:

| B11 Range | Lane | Orbital Ring | Meaning |
|-----------|------|--------------|---------|
| ≥ 200 | **GEM** | Inner | Sovereign ideal — calibrates the centroid |
| 140–199 | **TRIBE** | Inner-Mid | Core tribe member |
| 100–139 | **ACTIVE** | Mid | Participating |
| 60–99 | **FUZZY** | Mid-Outer | Degrading — monitored by L5 |
| < 60 | **DEAD** | Outer | Candidate for Rule 7 sink |

---

## The KAKI Identity — 16 Bytes, Eternal

```
Offset  Bytes  Field
──────  ─────  ─────────────────────────────────────────
 0       4     uuid_hash       FNV-1a of domain UUID
 4       2     tribe_id        Sovereign tribe membership
 6       1     kaki_type       1=Identity 2=Event 3=CrossTribe
 7       1     kaki_role       Semantic role within tribe
 8       4     reserved        Reserved — always zero
12       2     timestamp       Birth time (compact epoch)
14       2     checksum        CRC-16/CCITT of bytes 0–13
```

Three KAKI types govern the system:

| Type | Name | Purpose |
|------|------|---------|
| **1** | IdentityKaki | Permanent sovereign particle — born once, never dies |
| **2** | EventKaki | Mutable state carrier — drives the J_mem kinetic term |
| **3** | CrossTribeKaki | Inter-tribe kinetic connector — enforces topology bridges |

---

## The AMMAS Master Kinetic Equation

The system evolves according to:

```
∂f/∂t = J_phys[f]  +  J_mem[f]  +  J_learn[f]

  J_phys  → orbital motion + SPH density + Rule 7 sink      (tribe-orbit-engine)
  J_mem   → BehaviorPolicy, 7 Tribal Laws, state transitions (ammas-engine)
  J_learn → H(P) quality drift, centroid recalibration       (score-engine + hepta-score)
```

---

## The 7 Tribal Laws

Every tribe governs itself through seven sovereign laws, applied in strict priority order:

```
Priority (ascending): P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)

L3  Scope Law        — defines the domain boundary; lowest priority
L7  Overflow Law     — Rule 7 density sink; fires when density ≥ 11
L1  Identity Law     — locks the nucleus; prevents identity drift
L2  Quality Law      — promotes particles that improve H(P)
L5  Transition Law   — stabilises FUZZY particles before they die
L4  Orbit Law        — governs ascent toward GEM ring; second highest
L6  Sovereignty Law  — supreme veto; protects GEM particles from demotion
```

When multiple laws fire simultaneously, the **highest-priority law wins**.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BahyWay.Ecosystem v4.0                              │
│                     58 Crates · 5 Binaries · Pure Rust                      │
└─────────────────────────────────────────────────────────────────────────────┘

Layer 11 ─ UI / IDE
  dubsar-ide          DubSar Sovereign IDE (VSCodium profile)
  dubsar-visualizer   egui/wgpu particle inspector · 4-panel sovereign UI

Layer 10 ─ Runtime / OS
  eridu-runtime       EriduOS cooperative task executor
  eridu-scheduler     Job scheduling and priority queues
  eridu-supervisor    Process management and crash recovery

Layer 9 ─ Languages
  aaol                Akkadian Actor Orchestration Language (.akk)
  heptascript         7-Dimensional Query Language (.hepta)

Layer 8 ─ Pipeline / Stations
  adad-gate           Sole ingestion entry point — KAKI minter (§10.1)
  musaru-security     Security validation station
  compare-tribe-schema  Schema comparison and validation
  vgca-validation     VGCA geometric cleansing validation beam
  data-structure-station    Structure conformance station
  data-cleansing-station    Cleansing transformation station
  data-steward-station      DataSteward governance station
  permanent-storage   Golden Records permanent vault

Layer 7 ─ Templates / Governance
  template-engine     Default + stakeholder template resolution (§6)
  template-library    Built-in sovereign template catalog (§6.3)
  diagnosis-templates ColorID quality/freshness/state diagnostics
  damadmbok-dictionary  DAMA-DMBOK data governance vocabulary

Layer 6 ─ Cross-Tribe / IDU
  idu-prober          Shamash IDU — cross-tribe identity resolution
  idu-batching        IDU bulk cross-tribe operations

Layer 4.5 ─ Physics & Intelligence Engines
  vgca-engine         7D FSV + 6D BFV + self-calibrating GEM centroid
  tribe-orbit-engine  SPH density, 7 Tribal Laws, orbital ring assignment
  ammas-engine        J_mem kinetic equation, BehaviorPolicy, state transitions

Layer 5 ─ Operational Engines
  story-engine        CQRS read-side projection — narrative output
  fuzzy-engine        Mamdani fuzzy logic quality scoring (ADR-005/006)
  score-engine        ColorID computation and quality metrics
  hepta-score         H(P) sovereign equation, B11, QualityLane
  alert-engine        ColorID drift monitoring and alerting
  snapshot-job        Point-in-time snapshot scheduler
  navi-engine         Sovereign routing — NaviCode, A*, 7-sector heptagram
  najaf-engine        Wadi al-Salam cemetery navigation · pilgrim routing
  dmw-engine          SQL query plan analyser · 7-Level Journey · The Oracle
  nusku-engine        Shared sovereign types — KakiPK, Hepta, pipeline
  azuga-engine        Medical thermal condition inference · 9-entry catalogue
  nusku-fuzzy         Mamdani fuzzy threat/medical classification
  iris-engine         IR camera HAL — FLIR Lepton, Optris PI, SEEK, simulated
  panam-engine        Face detection + KAKI face PK (IR + RGB dual-channel)
  nusku-score         Aggregate scoring — TRS + MedScore + FuzzyScore + Face
  shulman-engine      Arabic/English verdicts · KAKI audit trail
  wpd-engine          Water/pipeline defect detection · Baghdad ×7 · 12-band spectral
  homt-engine         Hepta-Orbit Manifold Theory — OrbitField, PDE interface

Layer 4 ─ EnkiDB Engine
  enkidb-engine       Main database orchestration (§3, §9)
  enkidb-query        Query execution against indexes and journal

Layer 3 ─ Native Indexes
  enkidb-indexes      All 7 sovereign index types
  enkidb-dictionary   TĒMĒNU — attr_name → attr_hash EAV manifold

Layer 2 ─ Storage Substrate
  enkidb-block        64 MB block-aligned cold storage
  enkidb-journal      Append-only sovereignty event log
  enkidb-storage      Native mmap + fsync storage primitives
  enkidb-snapshot     Point-in-time snapshot mechanism
  enkidb-recovery     Crash recovery and abrupt-termination handler
  enkidb-persist      Journal serialisation and replay (§11)
  enkidb-dw           ETL pipeline · LandingZone · WayCompiler · DW analytics

Layer 1 ─ KAKI Identity
  enkidb-kaki         16-byte KAKI sovereign identity (3 types, CRC-16 verified)
  enkidb-vector-id    Vector IDs for Default.Jobs and Default.Indexes

Layer 0 ─ Foundation
  bahyway-core        Core types, errors, traits — zero dependencies
  bahyway-crc         CRC-16/CCITT native implementation
  bahyway-algebra     H(P) equation, PA-13 shells, PA-14 orbital, JNF, Anshar
```

---

## Enterprise Data Workflow Diagrams

### 1. P01 → P04: Particle Birth & Quality Scoring Chain

```
External Data
     │
     ▼
┌────────────┐   KAKI minted      ┌──────────────┐
│ adad-gate  │──────────────────► │ enkidb-kaki  │
│ (Ingestion)│   16-byte identity │ (Nucleus)    │
└────────────┘                    └──────┬───────┘
     │  ArrivalRecord                    │
     │                                   │ kaki[6]=type, kaki[12]=azimuth
     ▼                                   ▼
┌──────────────┐  FSV 7D vector   ┌──────────────┐
│ vgca-engine  │─────────────────►│ hepta-score  │
│ (Geometric   │  BFV 6D vector   │ (H(P) eq.)   │
│  Cleansing)  │  GeometricFit    │ B11 ∈ 0..240 │
└──────────────┘                  └──────┬───────┘
                                         │ QualityLane
                                         ▼
                                  ┌──────────────┐
                                  │tribe-orbit-  │
                                  │engine        │
                                  │OrbitalRing   │
                                  │azimuth/alt   │
                                  └──────┬───────┘
                                         │
                                         ▼
                                  ┌──────────────┐
                                  │ enkidb-engine│
                                  │ (EAV Orbit)  │
                                  └──────────────┘
```

---

### 2. NajafEngine — Pilgrim Routing Workflow

```
Pilgrim Request (name + tribe)
        │
        ▼
┌───────────────┐  GraveId search  ┌───────────────┐
│ NajafEngine   │─────────────────►│ GraveRegistry │
│ (najaf-engine)│                  │ (7 sectors)   │
└───────┬───────┘                  └───────┬───────┘
        │                                  │ GraveParticle found
        │ PilgrimGuide                     │ NaviCoord
        ▼                                  ▼
┌───────────────┐  A* routing      ┌───────────────┐
│ navi-engine   │◄─────────────────│ NaviGraph     │
│ (NaviCode)    │                  │ (7-sector     │
│ SensorFeed    │                  │  heptagram)   │
└───────┬───────┘                  └───────────────┘
        │
        │ PilgrimRoute
        ▼
┌───────────────┐  verdict         ┌───────────────┐
│ shulman-engine│─────────────────►│ Arabic output │
│ (narrative)   │                  │ + KAKI audit  │
└───────────────┘                  └───────────────┘

Sectors: Sector-1 Imam Ali Gate · S2 Maryam Gate · S3 Sahn ·
         S4 Rawdha · S5 Sayed Bridge · S6 East · S7 Overflow
```

---

### 3. WPDEngine — Baghdad Water Pipeline Defect Detection

```
Field Sensor Data
       │
       ▼
┌─────────────┐  SpectralScan    ┌──────────────────┐
│ iris-engine │─────────────────►│   wpd-engine     │
│ (IR Camera  │  12-band VNIR +  │   (wpd-engine)   │
│  HAL)       │  SWIR + TIR      │                  │
└─────────────┘                  │  BaghdadSector×7 │
                                 │  ┌─────────────┐ │
GPS coordinates ────────────────►│  │SpectralBand │ │
                                 │  │DefectClass  │ │
                                 │  │  Water leak │ │
                                 │  │  Oil leak   │ │
                                 │  │  Sewage blk │ │
                                 │  └──────┬──────┘ │
                                 └─────────┼────────┘
                                           │ DefectEvent + KAKI
                                           ▼
                                 ┌──────────────────┐
                                 │ RepairScheduler  │
                                 │ RepairNavigator  │
                                 │ RepairPriority   │
                                 └────────┬─────────┘
                                          │
                                          ▼
                                 ┌──────────────────┐
                                 │  shulman-engine  │
                                 │  Arabic verdict  │
                                 │  + audit trail   │
                                 └──────────────────┘

Spectral bands: VNIR (400–1000 nm) + SWIR (1000–2500 nm) + TIR (8–14 μm)
Defect classes: WaterLeak · OilLeak · SewageBlockage · StructuralCrack
```

---

### 4. DMWEngine — SQL Query Intelligence (7-Level Journey)

```
QueryPlan (data structure — no DB connection)
       │
       ▼
┌──────────────────────────────────────────┐
│             DmwAnalyzer                  │
│                                          │
│  Level 1: Bottleneck Detection           │
│    → BottleneckDetector (16 patterns)    │
│  Level 2: Cardinality Analysis           │
│    → CardinalityEstimator                │
│  Level 3: Index Health                   │
│    → IndexHealthChecker                  │
│  Level 4: SARG-ability                   │
│    → SargabilityChecker                  │
│  Level 5: I/O Analysis                   │
│    → IoAnalyzer                          │
│  Level 6: CPU Analysis                   │
│    → CpuAnalyzer                         │
│  Level 7: Cache Analysis                 │
│    → CacheAnalyzer                       │
└──────┬───────────────────────────────────┘
       │ DmwReport
       │ alignment_pct
       │ bottlenecks[]
       ▼
┌──────────────────┐  APPLY-ROTATION   ┌─────────────┐
│   TheOracle      │──────────────────►│ Narrative   │
│ (recommendations)│  ranked fixes     │ Builder     │
└──────────────────┘                   └──────┬──────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │ ColorId report  │
                                    │ QueryParticle   │
                                    │ MotionState     │
                                    └─────────────────┘

ColorId encodes: query health as a sovereign quality byte
MotionState: Converging · Drifting · Stalled · Oscillating
```

---

### 5. Nusku Sovereign Bio-Security Pipeline

```
Person enters secure zone
          │
          ▼
┌─────────────────┐  IrisFrame       ┌──────────────────┐
│  iris-engine    │─────────────────►│  panam-engine    │
│  (FLIR/Optris/  │  16-zone temps   │  (Face detect    │
│   SEEK/Sim)     │                  │   KAKI face PK)  │
└─────────────────┘                  └────────┬─────────┘
                                              │ FaceDetection
          ┌───────────────────────────────────┘
          │
          ▼
┌─────────────────┐  ParticleSignal  ┌──────────────────┐
│  nusku-engine   │─────────────────►│  azuga-engine    │
│  (shared types  │  BodyScan        │  (medical thermal │
│   KakiPK,Hepta) │  PipelineContext │   9 conditions)  │
└─────────────────┘                  └────────┬─────────┘
                                              │ MedicalInferenceResult
          ┌───────────────────────────────────┘
          │
          ▼
┌─────────────────┐  FuzzyScore      ┌──────────────────┐
│  nusku-fuzzy    │─────────────────►│  nusku-score     │
│  (Mamdani fuzzy │                  │  (TRS + MedScore │
│   threat/med)   │                  │   + FuzzyScore   │
└─────────────────┘                  │   + FaceScore)   │
                                     └────────┬─────────┘
                                              │ ScanResult
                                              ▼
                                     ┌──────────────────┐
                                     │  shulman-engine  │
                                     │  Arabic verdict  │
                                     │  alert + report  │
                                     └──────────────────┘

Budget: ≤ 50 ms per pipeline frame (PIPELINE_BUDGET_MS sovereign constant)
Conditions detected: Fever · Hypothermia · Stress · Infection · Post-Surgery ·
                     Inflammation · Normal · SecurityThreat · TorsoOverlap
```

---

### 6. AMMAS Kinetic Timestep — Per-Tribe Evolution

```
Tribe (≤ 7 particles per timestep)
          │
          ├── positions[] ──────────────────────────────────────┐
          │                                                      ▼
          │                                           ┌─────────────────┐
          │                                           │tribe-orbit-     │
          │                                           │engine           │
          │                                           │ sph_kernel()    │
          │                                           │ density_count[] │
          │                                           │ DensityBand[]   │
          │                                           │ Rule7Clusters   │
          │                                           └────────┬────────┘
          │                                                    │ density_count[i]
          ├── b11[] ─────────────────────────────────────────► │
          │                                                    ▼
          │                                           ┌─────────────────┐
          │                                           │  ammas-engine   │
          │                                           │  policy_for()   │
          │                                           │  BehaviorPolicy │
          │                                           │  J_phys()       │
          │                                           │  J_mem()        │
          ├── delta_h[] (from score-engine) ─────────►│  J_learn()      │
          │                                           │  kinetic_step() │
          │                                           └────────┬────────┘
          │                                                    │ KineticState[]
          │                                                    ▼
          │                                           ┌─────────────────┐
          │                                           │  next_state[]   │
          │                                           │  outcome[]      │
          │                                           │  StepStats      │
          │                                           └─────────────────┘
          │
          └──► enkidb-engine (write EAV orbit updates)
```

---

### 7. Full Data Ingestion to Storage Chain

```
                    ┌─────────────────────────────────────────┐
                    │           External Data Source           │
                    └────────────────────┬────────────────────┘
                                         │
                                         ▼
                              ┌──────────────────┐
                              │    adad-gate      │ ← sole ingestion point
                              │  ArrivalRecord    │   mints KAKI identity
                              │  KAKI minter      │   §10.1
                              └────────┬──────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                   │
                    ▼                  ▼                   ▼
         ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
         │musaru-       │   │vgca-         │   │data-structure│
         │security      │   │validation    │   │-station      │
         │(security     │   │(geometric    │   │(schema       │
         │ check)       │   │ fit beam)    │   │ conformance) │
         └──────┬───────┘   └──────┬───────┘   └──────┬───────┘
                │                  │                   │
                └──────────────────┼───────────────────┘
                                   │
                                   ▼
                        ┌──────────────────┐
                        │data-cleansing-   │
                        │station           │
                        │(transforms)      │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │  hepta-score     │
                        │  H(P) equation   │
                        │  B11 → Lane      │
                        └────────┬─────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
                    ▼            ▼            ▼
           ┌─────────────┐  ┌────────┐  ┌──────────────┐
           │enkidb-engine│  │score-  │  │tribe-orbit-  │
           │EAV Orbit    │  │engine  │  │engine        │
           │write        │  │ColorID │  │OrbitalRing   │
           └──────┬──────┘  └────────┘  └──────────────┘
                  │
                  ▼
         ┌────────────────┐
         │ enkidb-journal │ ← append-only sovereignty log
         │ enkidb-storage │ ← mmap + fsync
         │ enkidb-block   │ ← 64 MB cold storage
         └────────┬───────┘
                  │
         ┌────────┴────────┐
         │                 │
         ▼                 ▼
  ┌─────────────┐  ┌───────────────┐
  │enkidb-      │  │permanent-     │
  │snapshot     │  │storage        │
  │(point-in-   │  │(Golden        │
  │ time)       │  │ Records)      │
  └─────────────┘  └───────────────┘
```

---

## Glossary of All Crates — Akkadian Names & Descriptions

### Layer 0 — Foundation 𒀭

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `bahyway-core` | *bāhu* (arm/power) | Sovereign foundation | Core types, errors, traits — zero dependencies. The primordial layer. |
| `bahyway-crc` | *bāhu* + CRC | Checksum arm | CRC-16/CCITT native implementation. Verifies every KAKI nucleus. |
| `bahyway-algebra` | *bāhu* + algebra | Unified algebra | H(P) equation, PA-13 shell decomposition, PA-14 orbital position, JNF manifold, Anshar particle trait. |

### Layer 1 — KAKI Identity 𒁾

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `enkidb-kaki` | *kakkum* (weapon/seal) | The sovereign seal | 16-byte KAKI identity — IdentityKaki, EventKaki, CrossTribeKaki. Born once, never altered. |
| `enkidb-vector-id` | *enkidu* + vector | Identity vector | NodeId and vector IDs for Default.Jobs and Default.Indexes. |

### Layer 2 — Storage Substrate 𒀭𒂗𒆤

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `enkidb-block` | *enkidu* + block | Cold tablet store | 64 MB block-aligned cold storage. Each block is a sovereign tablet. |
| `enkidb-journal` | *enkidu* + journal | Sovereignty scroll | Append-only event log. Every state change is an immutable entry. |
| `enkidb-storage` | *enkidu* + storage | Native clay tablet | mmap + fsync storage primitives. Pure OS calls, no middleware. |
| `enkidb-snapshot` | *enkidu* + snapshot | Point-in-time clay | Point-in-time recovery snapshots. |
| `enkidb-recovery` | *enkidu* + recovery | Crash resurrection | Abrupt-termination handler and crash recovery. |
| `enkidb-persist` | *enkidu* + persist | Journal durability | Journal serialisation and replay (§11). |
| `enkidb-dw` | *enkidu* + DW | Data warehouse | ETL pipeline, LandingZone, ZipEngine, WayCompiler, DW analytics (§12). |

### Layer 3 — Native Indexes 𒅖𒋼𒋻

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `enkidb-indexes` | *enkidu* + indexes | Seven sacred tablets | All 7 sovereign index types in one crate — the index heptagram. |
| `enkidb-dictionary` | *tēmēnu* (foundation stone) | Foundation vocabulary | TĒMĒNU — maps attr_name → attr_hash (u32) for the EAV Hepta manifold. |

### Layer 4 — EnkiDB Engine 𒀭𒂗𒆤

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `enkidb-engine` | *enkidu* (he who came from the steppe) | The living database | Main database orchestration — coordinates all layers (§3, §9). |
| `enkidb-query` | *enkidu* + query | Oracle of tablets | Query execution against indexes and the sovereign journal. |

### Layer 4.5 — Physics & Intelligence Engines 𒀭𒂗𒆤𒁾

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `vgca-engine` | *vgca* — Vector Geometric Cleansing | Geometric purifier | 7D FSV text features + 6D BFV binary features + self-calibrating GEM centroid. Classifies data as Clean/Suspect/Outlier/Alien. |
| `tribe-orbit-engine` | *tribe* + orbital | Orbital tribe mechanics | SPH cubic density kernel, DensityBand (Isolated/Cluster/Rule7Overflow), 7 Tribal Laws, OrbitalRing assignment. |
| `ammas-engine` | *ammatu* (cubit / sovereign measure) | Kinetic sovereign measure | AMMAS master kinetic equation — J_phys + J_mem + J_learn. AgentClass, BehaviorPolicy, kinetic_step(). |

### Layer 5 — Operational Engines 𒀭

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `story-engine` | *siprū* (writing/message) | Story narrator | CQRS read-side projection. Translates kinetic state into sovereign narrative. |
| `fuzzy-engine` | *fuzzy* logic | Fuzzy boundary keeper | Mamdani fuzzy logic quality scoring (ADR-005/006). Handles the grey zone between lanes. |
| `score-engine` | *šiknu* (arrangement) | ColorID scorer | ColorID quality computation and quality metrics across the tribe. |
| `hepta-score` | *hepta* (7) + *šiknu* | Sovereign 7D scorer | H(P) equation engine. Computes B11, QualityLane, GEM rate, DataSteward rate. |
| `alert-engine` | *napaltu* (inspection) | Drift watchman | ColorID drift monitoring and alerting. Fires when quality slides. |
| `snapshot-job` | *šamāru* (to preserve) | Preserving scribe | Point-in-time snapshot scheduler. |
| `navi-engine` | *nāviru* (navigator) | Sovereign pathfinder | NaviCode, A* routing, 7-sector heptagram topology. Shared routing substrate. |
| `najaf-engine` | *Najaf* (city of Imam Ali) | Wadi al-Salam pilgrim | Wadi al-Salam sovereign cemetery navigation. 7 sectors, pilgrim routing, grave search. |
| `dmw-engine` | *dāmiqtu* (goodness/path) + *mūtu* (motion) | Deep motion advisor | SQL query plan analyser. 7-Level Journey. TheOracle ranks APPLY-ROTATION fixes. |
| `nusku-engine` | *Nusku* (Akkadian god of fire and light) | Fire of shared light | Shared sovereign types — KakiPK, Hepta, ParticleSignal, BodyScan, PipelineContext. |
| `azuga-engine` | *azugallū* (chief physician) | Sovereign physician | Medical thermal condition inference. 9-condition catalogue. ≤50ms budget per frame. |
| `nusku-fuzzy` | *Nusku* + fuzzy | Thermal classifier | Mamdani fuzzy threat/medical classification for thermal body scans. |
| `iris-engine` | *Iris* (vision) | Sovereign eye | IR camera HAL — FLIR Lepton, Optris PI, SEEK Thermal, simulated backend. |
| `panam-engine` | *pānu* (face/front) | Face identifier | IR + RGB dual-channel face detection. Derives KAKI face PK from 128D facial embedding. |
| `nusku-score` | *Nusku* + *šiknu* | Aggregate fire score | TRS + MedScore + FuzzyScore + FaceScore → ScanResult. |
| `shulman-engine` | *Šulmu* (peace/wellbeing) | Verdict of peace | Arabic/English verdicts. KAKI audit trail. Sovereign report generator. |
| `wpd-engine` | *wūppu* (leakage) + *mūtu* (path) | Leakage motion tracker | Baghdad water pipeline defect detection. 12-band spectral (VNIR+SWIR+TIR). 7 sectors. |
| `homt-engine` | *HOMT* — Hepta-Orbit Manifold Theory | Manifold navigator | OrbitField, PDE interface, convergence conditions for the 7D manifold. |

### Layer 6 — Cross-Tribe / IDU 𒀭𒊓𒈨𒌍

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `idu-prober` | *Shamash* (sun god) + IDU | Sunlit identity probe | Cross-tribe identity resolution — the Shamash IDU probe. |
| `idu-batching` | *Shamash* + batch | Bulk identity light | Bulk cross-tribe IDU operations. |

### Layer 7 — Templates / Governance 𒀭𒌓

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `template-engine` | *tuppu* (tablet) | Sovereign template | Default + stakeholder template resolution (§6). |
| `template-library` | *tuppu* + library | Sacred template store | Built-in sovereign template catalog (§6.3) for all tribes. |
| `diagnosis-templates` | *tuppu* + diagnosis | Diagnostic tablet | ColorID quality/freshness/state diagnostics templates. |
| `damadmbok-dictionary` | *DAMA-DMBOK* | Governance lexicon | DAMA-DMBOK data governance vocabulary. Sovereign compliance vocabulary. |

### Layer 8 — Pipeline / Stations 𒀭𒌓𒆳

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `adad-gate` | *Adad* (storm god) | Storm gate | Sole ingestion entry point (§10.1). Mints KAKI. All data enters here — nothing bypasses Adad. |
| `musaru-security` | *mūsarû* (inscribed tablet) | Security inscription | Security validation station. Guards the gate. |
| `compare-tribe-schema` | *šāru* (comparison) | Schema comparator | Schema comparison and validation across tribes. |
| `vgca-validation` | *vgca* + validation | Geometric validation | VGCA geometric cleansing validation beam station. |
| `data-structure-station` | *bītu* (house/structure) | Structure station | Data structure conformance station. |
| `data-cleansing-station` | *ellētu* (pure) | Purification station | Data cleansing transformation station. |
| `data-steward-station` | *šāpiru* (steward) | Stewardship station | DataSteward governance station. Handles FUZZY/DEAD particles. |
| `permanent-storage` | *dāru* (eternal) | Eternal golden record | PERMANENT Golden Records vault. Immutable sovereign archive. |

### Layer 9 — Languages 𒅖𒋼𒋻

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `aaol` | *ayyālu* (runner/actor) | Actor runner | Akkadian Actor Orchestration Language (.akk). PMPVD 9-node sovereign IR. |
| `heptascript` | *hepta* (7) + script | Sevenfold query tongue | HeptaScript — 7-Dimensional Query Language (.hepta). |

### Layer 10 — Runtime / OS 𒀭𒂗𒆤𒌓

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `eridu-runtime` | *Eridu* (first sovereign city of Sumer) | City of first light | Synchronous cooperative task executor (§11). EriduOS lives here. |
| `eridu-scheduler` | *Eridu* + schedule | City clock | Job scheduling and priority queues for the sovereign runtime. |
| `eridu-supervisor` | *Eridu* + supervise | City guardian | Process management and crash recovery supervisor. |

### Layer 11 — UI / IDE 𒁾𒌓

| Crate | Akkadian Root | Meaning | Description |
|-------|--------------|---------|-------------|
| `dubsar-ide` | *dub-sar* 𒁾𒊬 (tablet writer) | Tablet writer IDE | DubSar Sovereign IDE — VSCodium profile, extensions, sovereign keybindings. |
| `dubsar-visualizer` | *dub-sar* + visualizer | Tablet vision | egui/wgpu sovereign particle inspector. 4 panels: Hepta 7D Radar · Particles · Baghdad WPD · Najaf Cemetery. |

---

## Sovereign Constants — ADRs

| Constant | Value | ADR | Meaning |
|----------|-------|-----|---------|
| `QUALITY_DIVISOR` | **240** | ADR-012 | B11 = round(H(P) × 240) — from Plimpton 322, eternal |
| `GEM_B11` | ≥ 200 | ADR-001 | GEM lane threshold |
| `TRIBE_B11` | ≥ 140 | ADR-001 | Tribe member threshold |
| `ACTIVE_B11` | ≥ 100 | ADR-001 | Active threshold |
| `FUZZY_DEAD_BOUNDARY` | 59 | ADR-001 | Dead ceiling |
| `PARTICLES_PER_TRIBE` | **7** | Heptagon | Seven is sovereign — the eternal constant |
| `SHC` | 1/255 | Heptagon | Sovereign Heptagon Constant — normalisation |
| `TAU_R7` | **11** | Rule 7 | Overflow density trigger — τ_R7 |
| `CLUSTER_THRESH` | 3 | SPH | Minimum cluster size for density effects |
| `RESONANCE_RADIUS` | 0.15 | SPH | SPH kernel bandwidth — neighbourhood radius |
| `DELTA_FRAG` | 0.35 | ADR-008 | Binary fragmentation threshold |
| `GEM_RATE_TARGET` | 35.4% | ADR-004 | Sovereign GEM rate SLA |
| `PIPELINE_BUDGET_MS` | 50 ms | Nusku | Maximum pipeline frame budget |
| `WADI_AL_SALAM_LAT` | 31.995° N | Najaf | Canonical cemetery latitude |
| `WADI_AL_SALAM_LON` | 44.320° E | Najaf | Canonical cemetery longitude |
| `LAW_PRIORITY_ORDER` | [2,6,0,1,4,3,5] | Tribal | 7 laws in 0-based sovereign priority |

---

## Build & Test

```bash
# Build the entire ecosystem
cargo build --workspace

# Run all 1,297 tests
cargo test --workspace

# Test a specific engine
cargo test -p ammas-engine
cargo test -p tribe-orbit-engine
cargo test -p vgca-engine
cargo test -p najaf-engine
cargo test -p wpd-engine
cargo test -p dmw-engine
```

**Result:** 0 errors · 0 failures · pure Rust · no external runtime

---

## Sovereign Rules — What We Never Do

1. **No third-party runtime dependencies.** No tokio, no postgres, no ndarray, no uuid (external), no rayon. Pure Rust stdlib + zero external IO.
2. **No unsafe code.** `#![forbid(unsafe_code)]` on all physics/intelligence crates.
3. **No assessments in the KAKI nucleus (§2.4).** Quality, state, and all mutable values live exclusively in the EAV Orbit via Event-Kakis.
4. **Never change sovereign constants.** ADR-012 (B11 divisor = 240), PARTICLES_PER_TRIBE = 7, TAU_R7 = 11 — these are eternal.
5. **No `Way` suffix on engine names.** The suffix `Way` is reserved for WAYv2.0 Language (Security Defending Layer Policies). Only `.way` file extension survives.

---

## Repository Structure

```
workspace/bahyway_v4/
├── Cargo.toml              ← workspace root (58 crates + 5 binaries)
├── README.md               ← this file
├── SOVEREIGN_MANUAL_PART2.md  ← W5H2 process architecture (51 KB)
├── crates/
│   ├── bahyway-core/       ← Layer 0: Foundation
│   ├── enkidb-kaki/        ← Layer 1: KAKI Identity
│   ├── enkidb-*/           ← Layer 2–4: Storage + Engine
│   ├── vgca-engine/        ← Layer 4.5: Physics
│   ├── tribe-orbit-engine/ ← Layer 4.5: Orbital Mechanics
│   ├── ammas-engine/       ← Layer 4.5: Kinetic System
│   ├── hepta-score/        ← Layer 5: H(P) Equation
│   ├── najaf-engine/       ← Layer 5: Cemetery Navigation
│   ├── wpd-engine/         ← Layer 5: Pipeline Defects
│   ├── dmw-engine/         ← Layer 5: SQL Intelligence
│   ├── nusku-*/            ← Layer 5: Bio-Security Suite
│   ├── aaol/               ← Layer 9: .akk Language
│   ├── eridu-*/            ← Layer 10: Runtime OS
│   └── dubsar-*/           ← Layer 11: IDE + Visualizer
├── bin/
│   ├── bahyway-server/
│   ├── bahyway-cli/
│   ├── najaf-ingest/
│   ├── dubsar/
│   └── enkidw/
└── tests/
    ├── e2e/
    └── chaos/
```

---

## License

Proprietary — BahyWay Sovereign Ecosystem.
© Bahaa Fadam — All rights reserved.

> *𒁾 DUB.SAR — Written in the Tablet House of Eridu*
> *BahyWay.Ecosystem v4.0 | Pure Rust | Sovereign by Design*
