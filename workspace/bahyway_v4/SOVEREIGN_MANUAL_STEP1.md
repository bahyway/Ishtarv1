# BahyWay v4.0 — Sovereign Manual · Step 1: Crate Reference

**W5H2 Transparency Framework**
**Author**: Bahaa Fadam — BahyWay Sovereign Ecosystem
**Version**: 4.0.0 | **Date**: 2026-05-29
**Status**: Step 1 of N — covers all crates at NaviEngine + NajafEngine + ModularNaviIndex milestone

---

## KAKI — Sovereign Declaration

**KAKI (Knowledge–Akkadian–Keyword–Identity)**

KAKI is BahyWay Ecosystem v4.0's sovereign approach to Semantic Data
Modeling (SDM) for deterministic Entity Resolution across heterogeneous
data sources.

The name carries two layers of meaning:

**Etymological** — from the Akkadian *kaku* (𒋼𒁀): armament, seal, sovereign
mark. In ancient Mesopotamia, a cylinder seal (*kaku*) was the proof of
identity and authority — pressed into clay, impossible to forge without
the original seal. BahyWay's KAKI is the digital equivalent: a 16-byte
sovereign seal minted once at birth and immutable for the particle's
lifetime.

**Semantic** — Knowledge–Akkadian–Keyword–Identity encodes the four pillars
of the framework:
- **Knowledge** — the particle carries structured semantic knowledge about a real-world entity via its EAV attribute space
- **Akkadian** — the system's philosophical and etymological root: the first writing system that encoded sovereign identity
- **Keyword** — Knowledge-Aware Keyword Indexing (KAKI) resolves entity identity through seven native sovereign indexes, not through probabilistic string matching
- **Identity** — each particle has one and only one KAKI — its permanent, immutable, sovereign coordinate in the data universe

**Implementation — Knowledge-Aware Keyword Indexing:**
KAKI resolves records representing the same real-world entity through
Semantic Data Modeling rather than probabilistic similarity scoring.
Identity is determined by sovereign structure: tribe membership (`κ[4..5]`),
content hash (`κ[0..3]`), creation ordinal (`κ[8..11]`), and sovereign epoch
(`κ[12..13]`). The 7D VGCA quality vector — not an ML embedding — provides
the geometric measure of how close a particle is to its sovereign ideal.
Entity resolution is deterministic, auditable, and requires no external
model, no training data, and no neural network.

---

## What is W5H2?

Every crate in this manual is documented through seven sovereign questions:

| Symbol | Question | Answers |
|--------|----------|---------|
| **Who** | Who builds it? Who uses it? | Owner crate, consumers |
| **What** | What does it do? | One declarative sentence |
| **When** | When is it invoked? | Lifecycle moment: startup / ingestion / query / shutdown |
| **Where** | Where does it live? | Architectural layer + file path |
| **Why** | Why does it exist? | The problem it solves |
| **How** | How does it work? | Key types, algorithms, data flow |
| **How Much** | How much does it deliver? | Test count, key sizes, dependency count |

**Architecture principle**: BahyWay v4.0 is **pure Rust, zero external dependencies**. Every crate in this manual compiles with only Rust's standard library and other BahyWay crates. No PostgreSQL. No serde. No tokio. Sovereign throughout.

---

## Dependency Layers (overview)

```
Layer 11 ─ UI / IDE          dubsar-ide · dubsar-visualizer
Layer 10 ─ Runtime / OS      eridu-runtime · eridu-scheduler · eridu-supervisor
Layer  9 ─ Languages         aaol · heptascript
Layer  8 ─ Pipeline          bahyway-dqm (Data Quality) ·
                              bahyway-fabric (Enterprise Fabric) ·
                              adad-gate → musaru-security → vgca-validation →
                              data-structure-station → data-cleansing-station →
                              data-steward-station → permanent-storage
Layer  7 ─ Governance        template-engine · template-library ·
                              diagnosis-templates · damadmbok-dictionary
Layer  6 ─ Cross-Tribe       idu-prober · idu-batching
Layer  5 ─ Op. Engines       story-engine · fuzzy-engine · score-engine ·
                              alert-engine · snapshot-job ·
                              navi-engine · najaf-engine
Layer 4.5 ─ Orbital Intel    tribe-orbit-engine · homt-engine · **orbital-trust-probe**
Layer  4 ─ Engine            enkidb-engine · enkidb-query
Layer  3 ─ Indexes           enkidb-indexes
Layer  2 ─ Storage           enkidb-block · enkidb-journal · enkidb-storage ·
                              enkidb-snapshot · enkidb-recovery ·
                              enkidb-persist · enkidb-dw
Layer  1 ─ KAKI Identity     enkidb-kaki · enkidb-vector-id
Layer  0 ─ Foundation        bahyway-core · bahyway-crc
```

---

## Layer 0: Foundation

### `bahyway-core`

| W5H2 | Answer |
|------|--------|
| **Who** | Built by the ecosystem root. Used by every other crate as the first transitive dependency. |
| **What** | Defines the sovereign primitive types and error hierarchy shared by the entire ecosystem. |
| **When** | Compiled before all other crates; its types are present at every lifecycle stage. |
| **Where** | `crates/bahyway-core/` — Layer 0, no upstream BahyWay dependency. |
| **Why** | Prevents circular imports by centralising `TribeId`, `ParticleState`, `BahywayError`, and `LinkState` in one place that every other crate can depend on without cycles. |
| **How** | Three modules: `tribe.rs` (TribeId — a u16 sovereign namespace, derives Hash+Eq for HashMap use), `particle_state.rs` (ParticleState enum: Golden/Fuzzy/Dead plus LinkState for cross-tribe probes), `error.rs` (BahywayError + Result type alias). |
| **How Much** | **0 tests** · 0 external deps · 3 source files |

**Key types:**
```rust
pub struct TribeId(u16);          // Sovereign namespace — 65 535 possible tribes
pub enum ParticleState { Golden, Fuzzy, Dead }
pub enum LinkState { Active, Degraded, Dead }
pub struct BahywayError(String);  // Ecosystem-wide error type
```

---

### `bahyway-crc`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-kaki (KAKI checksum), enkidb-block (block header), heptascript (ModularNaviIndex signature). |
| **What** | Computes and verifies CRC-16/CCITT checksums over arbitrary byte slices. |
| **When** | At KAKI mint time (κ[14..16] checksum bytes), at block write time (block header integrity), at ModularNaviIndex build time. |
| **Where** | `crates/bahyway-crc/` — Layer 0. |
| **Why** | Structural integrity without external crates. The CRC-16/CCITT polynomial (0x1021) catches single-bit corruption with 100% probability and burst errors of up to 16 bits. |
| **How** | `CRC_TABLE: [u16; 256]` is computed at compile time via a `const` block. `crc16(data: &[u8]) -> u16` folds data through the table in a single pass. `verify(data, expected)` recomputes and compares. |
| **How Much** | **4 tests** · 0 external deps · 1 source file |

---

## Layer 1: KAKI Identity

### `enkidb-kaki`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by every pipeline station, storage crate, navi-engine, najaf-engine, and heptascript. The sovereign identity authority. |
| **What** | Mints and validates immutable 16-byte KAKI sovereign identity tokens. |
| **When** | At particle birth (AdadGate calls KakiMinter); at deserialization from Journal; at KAKI validation in musaru-security. |
| **Where** | `crates/enkidb-kaki/` — Layer 1. |
| **Why** | Every entity in the BahyWay ecosystem requires an immutable, sovereign, tamper-evident identity. KAKI encodes tribe, type, role, and timestamp in 16 bytes with a CRC-16 integrity seal. No two KAKIs can be identical across the lifetime of a deployment. |
| **How** | `Kaki` is `#[repr(transparent)] struct Kaki { bytes: [u8; 16] }` — Copy, never mut. Byte layout: `[0..4]` uuid_hash (shard key), `[4..6]` tribe_id, `[6]` type, `[7]` role, `[8..12]` reserved (zeroed), `[12..14]` timestamp, `[14..16]` CRC-16. `KakiMinter` mixes SystemTime nanoseconds with a per-minter counter using FNV-like xor-shift to generate uuid_hash without any external RNG. Three type aliases: `IdentityKaki`, `EventKaki`, `CrossTribeKaki`. |
| **How Much** | **~20 tests** (in mint.rs + kaki.rs) · 2 deps (bahyway-core, bahyway-crc) · 5 source files |

**Immutability rules (KAKI_v4.0.pdf §2):**
- Rule I: byte values never modified after mint
- Rule II: never reassigned to a different particle
- Rule III: only held via Copy or shared &Kaki
- Rule IV: no assessment data in KAKI bytes

---

### `enkidb-vector-id`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by operational mechanisms (snapshot-job, idu-prober) to reference vectors by kind. |
| **What** | Provides typed vector identifiers for the seven operational mechanism kinds. |
| **When** | When indexing vectors in enkidb-indexes and routing operational events. |
| **Where** | `crates/enkidb-vector-id/` — Layer 1. |
| **Why** | Separates the identity namespace for "mechanism vectors" (snapshots, probes, batches) from particle KAKIs so the two never collide in the index. |
| **How** | `VectorId(u64)` wraps a u64 handle. `VectorIdKind` enum encodes the mechanism type as a tag byte in the high bits. |
| **How Much** | **0 tests** · 1 dep (bahyway-core) |

---

## Layer 2: Storage Substrate

### `enkidb-block`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-persist and enkidb-dw for physical cold-storage layout. |
| **What** | Defines the 64 MB block-aligned cold storage format for KAKI nucleus records and orbit chunks. |
| **When** | When flushing the Journal to durable block files and when reading during recovery. |
| **Where** | `crates/enkidb-block/` — Layer 2. |
| **Why** | A fixed block size (64 MB) enables O(1) block lookup by offset, memory-mapped reads without fragmentation, and efficient fsync at block boundaries. |
| **How** | `BlockHeader` (magic + version + CRC-16 + entry count) serialises to a fixed-size prefix. `KakiNucleusBlock` holds the primary KAKI record. `OrbitChunkHeader` + `OrbitChunk` hold EAV attribute data aligned to 512-byte pages. `BLOCK_MAGIC: [u8; 4]` distinguishes block files from raw Journal segments. |
| **How Much** | **0 tests** · 2 deps (bahyway-core, bahyway-crc) |

---

### `enkidb-journal`

| W5H2 | Answer |
|------|--------|
| **Who** | The write path's core — used by adad-gate, enkidb-persist, story-engine, and the recovery pipeline. |
| **What** | Maintains the append-only sovereign event log: the immutable source of truth for all particle state changes. |
| **When** | Every time a particle is created, updated, or linked — the Journal receives a `JournalEntry`. Replayed at startup by enkidb-persist to reconstruct current state. |
| **Where** | `crates/enkidb-journal/` — Layer 2. |
| **Why** | An append-only log (never overwrite, never delete) guarantees that the full causal history of every particle is recoverable. All reads (StoryEngine projections) are derived from this log. |
| **How** | `JournalEntry { kaki: Kaki, epoch: u64, eav: Vec<EavTriple> }`. `EavTriple { attr_hash: u32, value: Vec<u8> }` — attribute identity hashed to u32 for compact storage. `Journal` wraps an `AppendWriter` from enkidb-storage and a KAKI-keyed index for O(log n) particle lookup. `PartitionKey` segments the journal by tribe for cross-tribe isolation. |
| **How Much** | **0 tests** · 3 deps (bahyway-core, enkidb-kaki, enkidb-storage) |

---

### `enkidb-storage`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-journal, enkidb-persist, and enkidb-dw as the physical I/O layer. |
| **What** | Provides append-only file writing, memory-mapped reading, and fsync durability control. |
| **When** | On every Journal commit (AppendWriter.write + optional fsync); on startup (MmapReader for recovery). |
| **Where** | `crates/enkidb-storage/` — Layer 2. |
| **Why** | Centralises all unsafe memory-mapping and syscall surface so every other crate remains safe Rust. The FsyncPolicy enum lets operators trade durability for throughput. |
| **How** | `AppendWriter` wraps `std::fs::File` in append mode. `COMMIT_MARKER: [u8; 4]` = `b"ENKI"` — written after every committed entry so recovery can detect truncation. `MmapReader` uses `std::fs::File + unsafe` memory mapping for zero-copy reads. `FsyncPolicy { PerCommit, PerBlock, Manual }`. |
| **How Much** | **0 tests** · 1 dep (bahyway-core) |

---

### `enkidb-snapshot`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by story-engine (to accelerate projections) and snapshot-job (to create snapshots). |
| **What** | Stores and retrieves periodic particle state snapshots that enable O(1) state projection at recent epochs instead of full Journal replay. |
| **When** | Written by snapshot-job on a configurable schedule; read by story-engine's `project_at()` as an acceleration checkpoint. |
| **Where** | `crates/enkidb-snapshot/` — Layer 2. |
| **Why** | Without snapshots, projecting a particle's state at epoch T requires replaying its entire Journal history. Snapshots cap replay cost at at most one snapshot interval. |
| **How** | `SnapshotRecord { kaki, epoch, state: ParticleState, quality: f32, ... }` is persisted as a flat binary record. Constants `ATTR_SNAPSHOT_DATE`, `ATTR_SNAPSHOT_STATE`, `ATTR_SNAPSHOT_FREQ` are u32 attribute hashes for the EAV representation. `ProjectionAlgorithm` enum controls whether latest-snapshot or full-replay is used. |
| **How Much** | **0 tests** · 2 deps (bahyway-core, enkidb-kaki) |

---

### `enkidb-recovery`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-persist at startup and by eridu-supervisor on crash detection. |
| **What** | Handles crash recovery and abrupt-termination repair by replaying the Journal from the last known-good COMMIT_MARKER. |
| **When** | On every startup before the database is available for queries; on supervisor-detected crash. |
| **Where** | `crates/enkidb-recovery/` — Layer 2. |
| **Why** | The BahyWay sovereign guarantee: no data loss on crash. The Journal's COMMIT_MARKER protocol and CRC-16 checksums allow the recovery procedure to identify and discard partial writes. |
| **How** | `RecoveryProcedure` scans the Journal file from tail to head, searching for the last valid COMMIT_MARKER. Entries after the marker are truncated. `RecoveryOutcome { entries_replayed, bytes_truncated }`. `RecoveryObjective { MaxDurability, MaxSpeed }` controls the scan strategy. |
| **How Much** | **0 tests** · 3 deps (bahyway-core, enkidb-journal, enkidb-storage) |

---

### `enkidb-persist`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by adad-gate (write path), najaf-ingest (binary), and any station that commits particles to disk. |
| **What** | Provides the unified disk persistence API: open a sovereign database, register particles, commit EAV events, and replay on startup. |
| **When** | Open at process start → replay journal → accept writes during ingestion → fsync on policy. |
| **Where** | `crates/enkidb-persist/` — Layer 2. |
| **Why** | Hides the Journal, storage, recovery, and snapshot layers behind a single `PersistedDb` handle so pipeline stations do not need to orchestrate raw I/O. |
| **How** | `PersistedDb::open(path, tribe_id, fsync_policy)` calls `RecoveryProcedure`, then wraps `Journal` + `AppendWriter`. `register_particle(kaki)` adds to the identity index. `commit(event_kaki, particle, epoch, eav)` appends a `JournalEntry` and conditionally fsyncs. `stats() -> PersistStats { entries_written, entries_replayed }`. |
| **How Much** | **0 tests** · 5 deps |

---

### `enkidb-dw`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by the enkidw binary and dashboard tooling for analytics, ETL, and ZIP export. |
| **What** | Provides the BahyWay Data Warehouse layer: landing zone management, ETL pipeline, ZIP-format analytics bundles, and sovereign KPI reports. |
| **When** | In batch/analytics mode — not in the real-time ingestion path. Run by the enkidw binary on schedule or on-demand. |
| **Where** | `crates/enkidb-dw/` — Layer 2. |
| **Why** | Separates the operational (OLTP) path from the analytical (OLAP) path. The DW layer exports compressed Golden-Record snapshots for reporting without blocking the live Journal. |
| **How** | `LandingZone` watches a directory for incoming files (`LandingFileKind: CSV, JSON, WayFile`). `EtlPipeline` ingests landing files → structures → cleanses → stores to `PermanentStore`. `build_store_zip()` packages a tribe's records into a sovereign ZIP bundle. `DwAnalytics` computes `ParticleStat { total, golden, fuzzy, dead }`. `DwReport` formats KPI summaries. |
| **How Much** | **0 tests** · 4 deps |

---

## Layer 3: Native Indexes

### `enkidb-indexes`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-engine and enkidb-query to answer queries without full-Journal scans. |
| **What** | Provides the seven sovereign in-memory indexes that enable O(log n) or O(1) lookup across all index dimensions. |
| **When** | Populated from the Journal on startup; updated on every commit; queried on every HeptaScript SELECT. |
| **Where** | `crates/enkidb-indexes/` — Layer 3. |
| **Why** | Seven indexes mirror the seven KAKI dimensions, so any query predicate can find matching KAKIs without scanning the full Journal. This is the read-side counterpart to the Journal's write-side append. |
| **How** | Seven index types, each wrapping a BTreeMap or HashMap keyed by the relevant dimension: `IdentityIndex` (kaki → particle), `SovereigntyIndex` (tribe_id → Vec<kaki>), `TypeRoleIndex` ((type, role) → Vec<kaki>), `TemporalIndex` (epoch range → Vec<kaki>), `ColorIdIndex` (color_rgb → Vec<kaki>), `EavIndex` (attr_hash + value → Vec<kaki>), `SnapshotIndex` (kaki → latest SnapshotRecord). `IndexVectorId` coordinates updates across all seven. |
| **How Much** | **0 tests** · 4 deps |

---

## Layer 4.5: Orbital Intelligence

### `orbital-trust-probe`

| W5H2 | Answer |
|------|--------|
| **Who** | Built by the Orbital Intelligence layer. Consumed by score-engine (feedback loop), eridu-supervisor (observability), and dubsar-visualizer (deviation dashboard panel). |
| **What** | Runs a 4-step causal attribution pipeline on consecutive orbital snapshots to distinguish legitimate 7D particle motion from unexplained trust violations, then feeds a `trust_penalty` back into `FuzzyDimensions.d9` to close the scoring feedback loop. |
| **When** | After every `score-engine` pass and before the next `tribe-orbit-engine` ring assignment. Runs per-particle or in batch. |
| **Where** | `crates/orbital-trust-probe/` — Layer 4.5, between score-engine (Layer 5) and tribe-orbit-engine (Layer 4.5). |
| **Why** | At billion-particle scale, particles are in continuous legitimate motion due to data evolution, SPH density shifts, freshness decay, and rule updates. Without causal attribution every orbital change generates a false trust alarm, causing exponential trust-penalty cascades that collapse cluster health. |
| **How** | Four sequential checks: (1) FuzzyRules fingerprint — abort if rules changed. (2) StoryEngine EAV event count delta — legitimate evolution if new events exist. (3) ScoreEngine field analysis — neighbour density shift / freshness decay / threshold boundary noise (hysteresis ±5 B11 units). (4) Unexplained residual — sealed into `OrbitalDeviationJournal` (CRC-16) and fed back to `FuzzyDimensions.d9_orbital_trust_penalty`. |
| **How Much** | **22 tests** · 5 source files · 840 lines · `forbid(unsafe_code)` · 0 external deps |

**Key types:**
```rust
pub struct OrbitalSnapshot { epoch, b11, tier, ring, assignment,
                              neighbour_count, freshness_byte, rules_fingerprint }
pub enum   DeviationCause  { FuzzyRulesChanged, LegitimateStateEvolution,
                              NeighborDensityShift, FreshnessDecay,
                              ThresholdBoundaryNoise, Unexplained }
pub struct OrbitalDeviationJournal { /* append-only, CRC-16, dedup */ }
```

**Closed feedback loop:**
```
OrbitalDeviationJournal::accumulated_penalty(id)
  → ScoreInput::orbital_trust_penalty
  → FuzzyDimensions::d9_orbital_trust_penalty
  → fuzzify() → effective D6 = source_trust.with_orbital_penalty(d9)
  → FuzzyEngine::score() → lower B11 → particle drifts to correct ring organically
```

---

## Layer 4: EnkiDB Engine

### `enkidb-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | The unified API surface — used by enkidb-query, pipeline stations, and binaries. |
| **What** | Provides `EnkiDb` — the single entry point that wires Journal, Indexes, StoryEngine, and FuzzyEngine into one coherent database handle. |
| **When** | Instantiated once per process at startup. All reads and writes go through EnkiDb. |
| **Where** | `crates/enkidb-engine/` — Layer 4. |
| **Why** | Without a unified handle, every caller would need to coordinate seven indexes + journal + story engine independently. EnkiDb enforces the sovereign invariant: all writes go through Journal, all reads go through StoryEngine. |
| **How** | `EnkiDb { journal, indexes, story, fuzzy, snapshot }`. `EnkiDb::open()` replays the journal into all seven indexes. Write path: `EnkiDb::commit(entry)` → Journal append → index update → optional snapshot trigger. Read path: `EnkiDb::query(plan)` → index lookup → StoryEngine projection → FuzzyEngine scoring. |
| **How Much** | **0 tests** · 6 deps |

---

### `enkidb-query`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by the bahyway-cli REPL and by binaries that expose HeptaScript query endpoints. |
| **What** | Bridges HeptaScript query plans to EnkiDb index lookups and returns QueryResult. |
| **When** | On every `.hepta` query submitted by a user or application. |
| **Where** | `crates/enkidb-query/` — Layer 4. |
| **Why** | Decouples the language layer (heptascript) from the storage layer (enkidb-engine) — neither needs to know about the other directly. |
| **How** | `query(plan: &QueryPlan, db: &EnkiDb) -> QueryResult`. Translates WHERE conditions into index key lookups, fetches candidate KAKIs from enkidb-indexes, then passes them to heptascript's `execute()` for condition evaluation via StoryEngine projections. |
| **How Much** | **0 tests** · 3 deps (heptascript, enkidb-engine, enkidb-indexes) |

---

## Layer 5: Operational Engines

### `story-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-engine (read path), heptascript (condition evaluation), and idu-prober (cross-tribe state). |
| **What** | Projects the current or historical state of any particle by replaying its Journal entries up to a given epoch. |
| **When** | On every query that needs particle state — the CQRS read side. Never called on the write path. |
| **Where** | `crates/story-engine/` — Layer 5. |
| **Why** | CQRS separation: the Journal stores events (what happened), the StoryEngine answers questions (what is the state now?). Without this separation, every query would mutate state. |
| **How** | `StoryEngine { journal, snapshots }`. `project_at(kaki, epoch) -> ProjectedState`. Algorithm: find latest snapshot ≤ epoch → replay Journal entries from snapshot epoch to target epoch → fold EAV triples into `ProjectedState { state, quality, color_rgb, freshness, ... }`. `projected_state::ProjectedState` exposes the 7 EAV dimensions as typed fields. |
| **How Much** | **0 tests** · 3 deps |

---

### `fuzzy-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-engine (quality scoring on write path) and score-engine. |
| **What** | Applies Mamdani fuzzy logic rules to 9 quality dimensions to compute a crisp B11 quality byte (0–255). |
| **When** | On particle ingestion — after EAV structuring, before ColorID computation. Also during orbital-trust-probe feedback cycles. |
| **Where** | `crates/fuzzy-engine/` — Layer 5. |
| **Why** | Real-world data quality is not binary. Fuzzy logic captures partial membership — a record can be 70% complete and 40% timely, yielding a nuanced score rather than a hard reject. D9 closes the orbital trust feedback loop without requiring a separate scoring pass. |
| **How** | `FuzzyDimensions` holds 9 dimensions (D1–D8 data quality, D9 orbital trust penalty). `fuzzify()` applies D9 to degrade D6 effective score before rule evaluation: `effective_d6 = source_trust.with_orbital_penalty(d9_penalty)`. `evaluate_rules(dims) → AggregatedOutput` applies 5 Mamdani SR-rules. `centroid_defuzz(output) → B11`. |
| **How Much** | **47 tests** · 2 deps |

**D9 orbital trust penalty:**
```rust
// In FuzzyDimensions::fuzzify():
let trust = self.d6_source_trust.with_orbital_penalty(self.d9_orbital_trust_penalty);
// degraded = base - penalty × (base - 0.10)   [UNKNOWN_FLOOR = 0.10]
```

---

### `score-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-engine after fuzzy-engine; produces the ColorId that drives alert-engine. Receives orbital trust penalty from orbital-trust-probe. |
| **What** | Computes B11 → ColorRgb → HpsScore → ParticleState from fuzzy dimensions plus orbital trust feedback. |
| **When** | After fuzzy scoring, before storing EAV attributes to Journal. |
| **Where** | `crates/score-engine/` — Layer 5. |
| **Why** | The ColorID is BahyWay's sovereign quality signature. `ScoreInput.orbital_trust_penalty` connects the orbital physics layer to the fuzzy scoring layer, completing the closed feedback loop. |
| **How** | `score(input) → ScoreResult`. Pipeline: dims.d9 ← penalty → fuzzify → B11 → tier_to_state → HpsScore → FreshnessDecay → ColorRgb → 4 EAV triples. `ScoreInput::civil_with_penalty(dims, domain, elapsed, penalty)` is the canonical constructor for post-probe cycles. |
| **How Much** | **23 tests** · 2 deps |

---

### `alert-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidb-engine on the write path and by data-steward-station for action queuing. |
| **What** | Monitors ColorID drift and emits typed alerts when quality drops below thresholds. |
| **When** | After score-engine produces a new ColorRgb — if it differs from the previous score beyond a threshold, an Alert is emitted. |
| **Where** | `crates/alert-engine/` — Layer 5. |
| **Why** | Silent quality degradation is the most dangerous data problem. The alert-engine ensures that any drift in particle quality is detected in the same transaction as the update. |
| **How** | `AlertEngine` holds the previous `ColorRgb` per KAKI. On each new score: `drift = euclidean_distance(prev_color, new_color)`. If drift > threshold: emit `Alert { kaki, severity: AlertSeverity, cause: DriftCause }`. `DriftCause { FreshnessDecay, CompletenessLoss, ValidityBreak, StateTransition }`. |
| **How Much** | **0 tests** · 2 deps |

---

### `snapshot-job`

| W5H2 | Answer |
|------|--------|
| **Who** | Driven by eridu-scheduler; writes to enkidb-snapshot. |
| **What** | Periodically snapshots the current projected state of all active particles to accelerate future StoryEngine projections. |
| **When** | On a configurable schedule (e.g., every 1000 epochs or every hour of wall-clock time). |
| **Where** | `crates/snapshot-job/` — Layer 5. |
| **Why** | Without periodic snapshots, StoryEngine must replay the full Journal from genesis for every query. Snapshots cap replay depth to one interval. |
| **How** | `SnapshotJob { db, schedule }`. `run() -> SnapshotJobResult`. Iterates all KAKIs in `IdentityIndex`, calls `story_engine.project_at(kaki, current_epoch)`, writes the result as a `SnapshotRecord` to the snapshot store. `SnapshotSchedule { EpochInterval(u64), WallClockSeconds(u64) }`. |
| **How Much** | **0 tests** · 4 deps |

---

### `navi-engine` ✦

| W5H2 | Answer |
|------|--------|
| **Who** | Built this session. Used by najaf-engine, WPDEngine (planned), DMWEngine (planned). |
| **What** | Sovereign routing engine — plans optimal paths across a 7-sector heptagram map using the NaviCode NC1–NC6 pipeline and quality-weighted A*. |
| **When** | On demand — each call to `RouteEngine::plan()` runs the full pipeline and returns a `RoutePlan`. No persistent state. |
| **Where** | `crates/navi-engine/` — Layer 5. |
| **Why** | Geographic routing in a sovereign ecosystem must account for tribal sovereignty, sensor disruption, surface quality, congestion, and sector topology — all in a single coherent pipeline without any external routing library. |
| **How** | **NaviMap** parses `.navimap` text format (NODE/BEAM lines) into `MapNode` + `MapBeam` structs. **NaviGraph** builds a directed adjacency graph with pre-computed effective edge costs. **NaviParticle** holds 7 spatial dimensions (lat, lon, alt, speed, congestion, surface, tribe). **NaviCode NC1–NC6**: (1) SpawnOrigin — lock origin, seed EdgeCostMatrix; (2) ResonanceSeek — tribal affinity bonus; (3) RepelDead — tombstone blocked edges (∞ cost); (4) FrictionAdjust — surface×speed scaling; (5) TribeCluster — group nodes by tribe; (6) GoldenPath — Dijkstra using BinaryHeap with integer cost encoding. **SensorFeed** applies 6 real-time event types (RoadClosure, RoadOpen, Congestion, WeatherChange, SovereignAlert, SpeedChange) to a live NaviGraph. |
| **How Much** | **118 tests** (103 unit + 15 integration) · 2 deps (bahyway-core, enkidb-kaki) |

**Key constants:**

| Constant | Value | Meaning |
|----------|-------|---------|
| `NAVI_SECTORS` | 7 | Heptagram sectors |
| `NAVICODE_STAGES` | 6 | NC1–NC6 pipeline stages |
| `MAX_ROUTE_WAYPOINTS` | 512 | Max path length |
| `GOLDEN_PATH_SYMBOL` | `✦` | Blessed route marker |

**HeptaChordType multipliers:**

| Chord | Multiplier | Meaning |
|-------|-----------|---------|
| Spoke | 0.80× | Centre ↔ outer — fast arterial |
| Rim | 1.00× | Adjacent outer — neutral ring |
| Local | 1.10× | Same sector — intra-zone |
| Diagonal | 1.40× | Non-adjacent outer — expensive jump |

---

### `najaf-engine` ✦

| W5H2 | Answer |
|------|--------|
| **Who** | Built this session. Used by najaf-ingest binary. Depends on navi-engine. |
| **What** | Domain-specific routing engine for Wadi al-Salam — the world's largest cemetery — mapping 7 sacred zones to heptagram sectors and guiding pilgrims from the entrance to any grave. |
| **When** | On pilgrim guidance requests and during cemetery management operations (grave registration, sector search, epoch filtering). |
| **Where** | `crates/najaf-engine/` — Layer 5. |
| **Why** | NaviEngine provides general routing; NajafEngine adds the sovereign cemetery domain: GraveParticle with KAKI identity, sacred-weight sector costs, Islamic Hijri epoch tracking, and pilgrim route planning with spiritual cost semantics. |
| **How** | **NajafSector** maps 7 zones (Entrance/Shuhadaa/Awliya/Huffaz/Momineen/Ulamaa/Anbiya) 1-to-1 onto NaviEngine HeptaSectors with sacred weights [0.85–1.00]. **GraveParticle** holds KAKI (minted via KakiMinter), NaviCoord, NajafSector, TribeId, Hijri epoch, and GraveState (Occupied/Reserved/Available/Sealed). **GraveRegistry** provides by_sector/by_tribe/by_epoch_range/nearest/nearest_accessible lookup. **PilgrimGuide** routes pilgrims using RouteEngine from navi-engine; applies sacred_weight to final cost; marks routes below BLESSED_COST_THRESHOLD (2000.0) as `is_blessed`. Trivial case (entrance == destination sector) handled with zero-cost single-waypoint route. |
| **How Much** | **82 tests** (61 unit + 21 integration) · 3 deps (bahyway-core, enkidb-kaki, navi-engine) |

**NajafSector sacred weights (hand-chosen; E₂ Fourier derivation available via heptascript):**

| Sector | k | Zone | Sacred Weight | Meaning |
|--------|---|------|--------------|---------|
| Entrance | 0 | Reception | 1.00 | Neutral — highest traffic |
| Shuhadaa | 1 | Martyrs | 0.85 | Elevated priority |
| Awliya | 2 | Saints | 0.90 | Elevated priority |
| Huffaz | 3 | Memorisers | 0.95 | Slightly elevated |
| Momineen | 4 | Believers | 1.00 | Neutral |
| Ulamaa | 5 | Scholars | 0.92 | Elevated priority |
| Anbiya | 6 | Prophets | 0.88 | Elevated priority |

---

## Layer 6: Cross-Tribe / IDU

### `idu-prober`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by idu-batching and cross-tribe dashboard queries. |
| **What** | Probes the effective state of a cross-tribe particle link using a `CrossTribeKaki`. |
| **When** | When a dashboard or report needs the live state of a particle that belongs to a different tribe. |
| **Where** | `crates/idu-prober/` — Layer 6. |
| **Why** | Tribes are sovereign namespaces — a Tribe A particle cannot directly read Tribe B's Journal. The IDU Probe is the authorised cross-tribe state query, returning `CrossTribeLinkState` without exposing raw data. |
| **How** | `IduProbe { cross_kaki, requester_tribe }`. `probe(journal) -> IduProbeResult { link_state: CrossTribeLinkState, effective_state: ParticleState }`. Uses `compose_link_state()` from bahyway-core to derive effective state from both the local link state and the remote particle's last known state. |
| **How Much** | **0 tests** · 3 deps |

---

### `idu-batching`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by enkidw binary and dashboard generation. |
| **What** | Batches multiple `IduProbe` calls into a single pass for production-scale cross-tribe dashboards. |
| **When** | During dashboard refresh cycles where thousands of cross-tribe links must be evaluated at once. |
| **Where** | `crates/idu-batching/` — Layer 6. |
| **Why** | Individually probing 10 000 cross-tribe links with separate Journal reads would be O(n × Journal_size). Batching reads the Journal once and answers all probes in a single pass. |
| **How** | `BatchProbe { probes: Vec<IduProbe> }`. `run(journal) -> BatchResult { results: Vec<LinkResult> }`. Groups probes by source tribe partition to minimise Journal seeks. Returns `LinkResult { kaki, state, link_state }` per probe. |
| **How Much** | **0 tests** · 2 deps |

---

## Layer 7: Templates / Governance

### `template-engine`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by vgca-validation, template-library, and any station that validates EAV triples against a schema. |
| **What** | Defines the sovereign EAV schema registry — named templates that specify which attribute fields are required, optional, and typed. |
| **When** | At validation time in the pipeline (vgca-validation calls template-engine to check incoming EAV). |
| **Where** | `crates/template-engine/` — Layer 7. |
| **Why** | Without a schema, any EAV triple could be committed to the Journal. Templates enforce data contracts at ingestion time, preventing schema drift before it reaches permanent storage. |
| **How** | `Template { name, fields: Vec<FieldSpec> }`. `FieldSpec { attr_hash: u32, field_type: FieldType, required: bool }`. `FieldType { Text, Integer, Float, Blob, Kaki }`. `validate_required(template, eav) -> bool` checks that all required fields are present. `TemplateRegistry` stores templates by name hash. |
| **How Much** | **0 tests** · 2 deps |

---

### `template-library`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by najaf-ingest (civil_registry_template), adad-gate, and any station needing default schemas. |
| **What** | Ships the built-in template catalog — civil registry, operational, and sensor stream templates — ready to use without external configuration. |
| **When** | Loaded at process startup; templates are immutable once loaded. |
| **Where** | `crates/template-library/` — Layer 7. |
| **Why** | Providing default templates prevents every deployment from needing to define common schemas from scratch. The civil registry template is the canonical schema for the najaf-ingest binary. |
| **How** | `load_defaults() -> TemplateRegistry` pre-populates the registry. `civil_registry_template()` returns the template for `{ ATTR_STATE, ATTR_QUALITY, ATTR_COLOR_RGB, ATTR_FRESHNESS, ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_STATE, ATTR_SNAPSHOT_FREQUENCY }` — the 7 sovereign EAV dimensions. Attribute hashes are stable u32 constants computed at compile time from attribute names. |
| **How Much** | **0 tests** · 2 deps |

---

### `diagnosis-templates`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by data-steward-station and reporting tools. |
| **What** | Classifies particles by ColorID quality and freshness to produce human-readable diagnosis reports. |
| **When** | After scoring — the diagnosis step explains WHY a particle has a given quality tier. |
| **Where** | `crates/diagnosis-templates/` — Layer 7. |
| **Why** | A ColorRgb score tells you the quality; a diagnosis tells you the cause. The data steward needs causal explanation to take corrective action, not just a colour code. |
| **How** | `classify(color: ColorRgb, state: ParticleState) -> DiagnosisKind`. `DiagnosisKind { FreshGolden, StaleFuzzy, IncompleteRecord, ValidityBreak, Dead }`. `diagnose(kaki, story_engine) -> Diagnosis { kind, message: &'static str }`. |
| **How Much** | **0 tests** · 2 deps |

---

### `damadmbok-dictionary` ✦

| W5H2 | Answer |
|------|--------|
| **Who** | Built this session (expanded from 12 → 117 terms). Used by dubsar IDE `.dama` command and DAMA-DMBOK alignment tooling. |
| **What** | Provides the sovereign data-governance vocabulary — 117 DAMA-DMBOK terms mapped to BahyWay v4.0 concepts, with alignment to specific crates and D1–D8 quality dimensions. |
| **When** | At DubSar IDE startup (term lookup); in reports and documentation generation. |
| **Where** | `crates/damadmbok-dictionary/` — Layer 7. |
| **Why** | DAMA-DMBOK is the international standard for data management knowledge. Embedding it as a sovereign crate (no external database) means BahyWay can always explain itself in internationally recognized governance terms. |
| **How** | `DmBokTerm { code, name, definition, area: KnowledgeArea }`. `KnowledgeArea` covers all 11 DAMA knowledge areas. `DICTIONARY: &[DmBokTerm]` — static slice of 117 terms. `lookup(code)`, `by_area(area)`, `search(keyword)`. `BahywayAlignment { dama_code, bahyway_crate, bahyway_concept, mapping_note }` — 42 explicit alignments. `DimensionMapping { dimension: u8, dama_code, bahyway_field }` — D1–D8 Fourier quality dimension mappings. |
| **How Much** | **22 tests** · 1 dep (bahyway-core) |

---

## Layer 8: Pipeline / Stations

The BahyWay data fabric flows: **bahyway-fabric (Enterprise Fabric) → AdadGate → MusaruSecurity → VgcaValidation → DataStructure → DataCleansing → DataSteward → PermanentStorage**.

`bahyway-fabric` is the outermost layer — it receives data from all external enterprise systems and routes it through the sovereign pipeline via `AdadGate`. `bahyway-dqm` is the quality assessment engine used within the `Stage::Validate` step of `bahyway-fabric` and independently by `data-cleansing-station`.

### `bahyway-fabric` *(added 2026-06-04)*

| W5H2 | Answer |
|------|--------|
| **Who** | Built by Bahaa Fadam. Used by `bahyway-server`, `enkidw`, and all domain orchestrators that receive data from external enterprise systems. |
| **What** | Sovereign Enterprise Data Fabric — the transparent, production-level answer to enterprise spaghetti processing. Provides declarative pipeline execution with schema enforcement, per-record lineage, and structured exceptions. |
| **When** | At ingestion time, before `adad-gate`. Any data arriving from ERP, CRM, HR, Legacy, Partner, Excel, Email, or Third-Party API systems enters the ecosystem through `bahyway-fabric`. |
| **Where** | `crates/bahyway-fabric/` — Layer 8 (top of the pipeline stack). |
| **Why** | Without a unified fabric layer, each domain engine (NajafEngine, NuskuEngine, WPDEngine) implemented its own extraction and routing — exactly the spaghetti pattern that BahyWay.Ecosystem exists to eliminate. `bahyway-fabric` enforces the same quality, lineage, and exception model across all data entry points. |
| **How** | `FabricOrchestrator` holds registries of `SourceConnector` + `TargetConnector` trait objects and `PipelineDeclaration` structs. `run_pipeline(&PipelineId, &ExtractionCursor)` extracts, enforces `SchemaContract`, runs stages (Cleanse → Validate → Enrich → Deduplicate → Aggregate), builds a `LineageChain` per record, delivers to all targets, returns `OrchestratorResult`. |
| **How Much** | **39 tests** (38 unit + 1 doc-test) · 0 failures · 7 modules · 8 source adapters · 7 target adapters · ~1,641 lines |

**Key types:**
```rust
pub struct FabricOrchestrator          // Central coordinator
pub struct PipelineDeclaration         // Versioned declarative pipeline
pub struct SchemaContract              // Typed boundary enforcement
pub trait SourceConnector              // External source attachment point
pub trait TargetConnector              // External target attachment point
pub struct LineageChain                // Per-record immutable audit trail
pub struct FabricException             // Typed structured failure (7 kinds)
pub struct OrchestratorResult          // Run result: receipts, exceptions, lineage
```

**Five sovereign guarantees:**
1. Every record that enters has its source declared in a `SchemaContract`
2. Every record that exits has been through every declared `Stage` in order
3. Every stage transition is recorded in an append-only `LineageChain`
4. Every failure is a typed `FabricException` — no silent errors
5. Adding a new source or target never modifies an existing connector or pipeline

---

### `bahyway-dqm` *(added 2026-06-04)*

| W5H2 | Answer |
|------|--------|
| **Who** | Built by Bahaa Fadam. Used by `bahyway-fabric` Stage::Validate, `data-cleansing-station`, `bahyway-server`, and any crate that must score record quality against a DAMA-DMBOK SLA before accepting data into the sovereign pipeline. |
| **What** | Sovereign Data Quality Management engine — implements all 6 DAMA-DMBOK quality dimensions (Completeness, Validity, Accuracy, Consistency, Uniqueness, Timeliness) with pure Rust algorithms, producing a composite score, B11 = score × 240 (ADR-001), and a per-SLA-preset compliance verdict. |
| **When** | At quality-gate time: after a record has been extracted and structurally validated (by `bahyway-fabric` or `data-structure-station`), before the record is accepted into the pipeline or committed to permanent storage. Also invoked in batch mode over large record sets via `DqmBatchReport` to compute aggregate compliance rates. |
| **Where** | `crates/bahyway-dqm/` — Layer 8 (Data Quality), alongside `bahyway-fabric`, `adad-gate`, and `vgca-validation`. |
| **Why** | Enterprise data quality is validated piecemeal across the ecosystem — completeness in one station, validity in another, uniqueness nowhere. `bahyway-dqm` provides the sovereign answer: one engine, all six DAMA-DMBOK dimensions, one composite score, one SLA verdict — with zero external dependencies and deterministic algorithms. |
| **How** | `DqmEngine::assess_record(record, epoch)` evaluates six dimensions in sequence: (1) field-presence counting for Completeness; (2) deterministic `RuleEngine` (5 rule types) + Welford `RunningStats` Z-score for Validity; (3) FNV-1a binary `MerkleTree` root for Accuracy; (4) 0xFF cross-field conflict marker scan for Consistency; (5) mean of Levenshtein Wagner-Fischer + Jaro-Winkler + Soundex American NARA for Uniqueness; (6) epoch linear decay for Timeliness. Composite = arithmetic mean of all 6. B11 = (composite × 240.0).round() as u8. |
| **How Much** | **77 tests** (76 unit + 1 doc-test) · 0 failures · 6 modules · 3 string-similarity algorithms · 5 deterministic rule types · 1 Welford stats engine · ~850 lines of sovereign Rust |

**Six quality dimensions:**

| Dimension | Algorithm | Sovereign Constraint | DAMA-DMBOK |
|---|---|---|---|
| Completeness | Non-empty field counting | O(n) scan, no regex | §13.3.1 |
| Validity | Rule Engine + Welford Z-score | Deterministic rules; numerically stable Welford | §13.3.2 |
| Accuracy | FNV-1a Merkle Tree root | FNV-1a not SHA-256 — no crypto dep | §13.3.3 |
| Consistency | 0xFF conflict marker scan | Byte-level, no schema knowledge required | §13.3.4 |
| Uniqueness | Levenshtein + Jaro-Winkler + Soundex | Wagner-Fischer O(min(m,n)) space; American NARA Soundex | §13.3.5 |
| Timeliness | Epoch linear freshness decay | Deterministic decay formula; no wall-clock calls | §13.3.6 |

**Key types:**
```rust
pub struct DqmEngine          // Orchestrates all 6 dimensions; holds DqmSla
pub struct DqmReport          // Per-record: 6 scores + composite + b11 + sla_compliant
pub struct DqmBatchReport     // Aggregate: add() + compliance_rate()
pub struct DqmSla             // Per-dimension thresholds; 3 presets + custom
pub struct MerkleTree         // FNV-1a binary tree; inclusion_proof + verify_proof
pub struct RuleEngine         // 5 deterministic rule types; evaluate() → (passed, total)
pub struct RunningStats       // Welford online mean + variance + z_score()
// Free functions:
pub fn levenshtein_similarity(s1: &str, s2: &str) -> f64
pub fn jaro_winkler(s1: &str, s2: &str) -> f64
pub fn sounds_like(s1: &str, s2: &str) -> bool
```

**B11 quality lane thresholds (ADR-001, divisor = 240):**

| Lane | B11 | Composite Score |
|---|---|---|
| GEM | ≥ 200 | ≥ 0.833 |
| TRIBE | ≥ 140 | ≥ 0.583 |
| ACTIVE | ≥ 100 | ≥ 0.417 |
| FUZZY | ≥ 60 | ≥ 0.250 |
| DEAD | < 60 | < 0.250 |

**Sovereign constraints:**
1. Zero external dependencies — only `bahyway-core`; no crates.io imports
2. ADR-001 enforced — `B11 = (score × 240.0).round()` — never 255
3. FNV-1a not SHA-256 — deterministic, sovereign, no cryptographic dependency
4. Welford algorithm — numerically stable, single-pass, O(1) space
5. Wagner-Fischer Levenshtein — O(min(m,n)) space, exact edit distance
6. American NARA Soundex — official standard, locale-independent, reproducible

---

### `adad-gate`

| W5H2 | Answer |
|------|--------|
| **Who** | The first station in the pipeline. Called by najaf-ingest, bahyway-server, and any producer. |
| **What** | The sole authorised ingestion entry point — accepts `ArrivalRecord`, mints a KAKI pair (identity + event), and produces a `GateResult` ready for downstream stations. |
| **When** | On every new particle arrival — the "gate" that every sovereign record must pass through. |
| **Where** | `crates/adad-gate/` — Layer 8. |
| **Why** | Centralising KAKI minting at the gate ensures that: (a) every particle gets exactly one IdentityKaki, (b) the tribe affiliation is established at entry, (c) no record bypasses the sovereignty check. |
| **How** | `AdadGate { tribe_id, minter: KakiMinter }`. `ingest(record: ArrivalRecord) -> GateResult { particle: IdentityKaki, event_kaki: EventKaki, epoch, eav }`. `ArrivalRecord { attrs: Vec<(attr_hash, Vec<u8>)>, epoch: u32, role: KakiRole }`. |
| **How Much** | **0 tests** · 3 deps |

---

### `musaru-security`

| W5H2 | Answer |
|------|--------|
| **Who** | Called by adad-gate after minting, and by pipeline stations that need sovereignty verification. |
| **What** | Validates that a particle's KAKI tribe matches the gate's tribe and that the role is authorised for the operation. |
| **When** | Immediately after AdadGate produces a GateResult, before the record proceeds to validation. |
| **Where** | `crates/musaru-security/` — Layer 8. |
| **Why** | The sovereignty invariant: no particle from Tribe A can be committed under Tribe B's authority. MusaruSecurity is the enforcement point. |
| **How** | `check_sovereignty(tribe_id: TribeId, particle: &IdentityKaki) -> SecurityResult`. Checks `particle.tribe_id() == tribe_id`. `SecurityResult { is_approved: bool, reason: &'static str }`. |
| **How Much** | **0 tests** · 2 deps |

---

### `compare-tribe-schema`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by governance tooling and the bahyway-cli for schema audit commands. |
| **What** | Compares two versions of a tribe's EAV schema to detect field additions, removals, type changes, and completeness gaps. |
| **When** | During schema migration reviews and compliance audits. |
| **Where** | `crates/compare-tribe-schema/` — Layer 8. |
| **Why** | Schema drift between tribes causes data quality failures downstream. The comparison engine catches breaking changes before they reach production. |
| **How** | `compare_versions(v1: &SchemaVersion, v2: &SchemaVersion) -> CompareReport { verdict: CompareVerdict, diffs: Vec<FieldDiff> }`. `FieldDiff { kind: FieldChange, field: FieldMeta }`. `rank_tribes(registries) -> Vec<TribeRank>` computes `TribeCompleteness` scores. `most_organized()` returns the tribe with the highest schema completeness. |
| **How Much** | **0 tests** · 2 deps |

---

### `vgca-validation`

| W5H2 | Answer |
|------|--------|
| **Who** | Called after musaru-security, before data-structure-station. |
| **What** | Validates incoming EAV triples against the particle's template, checking required fields, type constraints, and value bounds. |
| **When** | On every ingestion event, after sovereignty is confirmed. |
| **Where** | `crates/vgca-validation/` — Layer 8. |
| **Why** | VGCA (Validation Gate for Content Accuracy) is the schema enforcement point — without it, any EAV triple could reach the Journal regardless of completeness or type correctness. |
| **How** | `validate(template: &Template, eav: &[EavTriple]) -> ValidationResult { is_valid: bool, missing_required: Vec<u32>, type_errors: Vec<u32> }`. Checks each required `FieldSpec.attr_hash` against the EAV triple list. |
| **How Much** | **0 tests** · 2 deps |

---

### `data-structure-station`

| W5H2 | Answer |
|------|--------|
| **Who** | Called after vgca-validation. Produces typed EAV triples for downstream cleansing. |
| **What** | Maps raw key-value pairs from incoming records to typed `EavTriple` structures using the template's FieldSpec. |
| **When** | After validation — the "typing" step that converts raw bytes to sovereign EAV. |
| **Where** | `crates/data-structure-station/` — Layer 8. |
| **Why** | Raw ingestion data (CSV fields, JSON values) must be coerced to BahyWay's typed EAV format before quality scoring. This station is where "raw bytes" become "sovereign data". |
| **How** | `structure(raw: &[(attr_hash, Vec<u8>)], template: &Template) -> StructureReport { triples: Vec<EavTriple>, coercion_errors: Vec<u32> }`. Each field is parsed according to its `FieldType` (Text → UTF-8 check, Integer → big-endian i64, Float → IEEE 754 f64, Kaki → 16-byte validation). |
| **How Much** | **0 tests** · 3 deps |

---

### `data-cleansing-station`

| W5H2 | Answer |
|------|--------|
| **Who** | Called after data-structure-station. Produces a `CleansingReport` for the steward. |
| **What** | Applies DAMA-DMBOK Data Quality dimension checks (D1–D8) to structured EAV triples and flags issues. |
| **When** | After structuring — the "quality gate" before committing to permanent storage. |
| **Where** | `crates/data-cleansing-station/` — Layer 8. |
| **Why** | DAMA-DMBOK defines 8 data quality dimensions (Completeness, Accuracy, Consistency, Timeliness, Uniqueness, Validity, Accessibility, Integrity). The cleansing station operationalises all 8 without external tooling. |
| **How** | `cleanse(triples: &[EavTriple], template: &Template) -> CleansingReport { findings: Vec<DqFinding> }`. Each `DqFinding { dimension: u8, attr_hash: u32, severity: &'static str }` identifies a quality issue by DAMA dimension number. The report feeds into FuzzyEngine for quality scoring. |
| **How Much** | **0 tests** · 3 deps |

---

### `data-steward-station`

| W5H2 | Answer |
|------|--------|
| **Who** | Called by alert-engine when drift alerts are emitted. Manages the steward action queue. |
| **What** | Queues remediation actions for the data steward when quality alerts exceed thresholds. |
| **When** | When an Alert from alert-engine has severity High or Critical — triggers steward review. |
| **Where** | `crates/data-steward-station/` — Layer 8. |
| **Why** | Automated quality scoring detects problems; the data steward station ensures human review is requested for issues that automation cannot resolve. It's the sovereign "last mile" of data governance. |
| **How** | `StewardStation { queue: Vec<Alert> }`. `enqueue(alert: Alert)`. `pending() -> &[Alert]`. `resolve(kaki)` removes a resolved alert. Integrates with the `DubSar IDE` to surface alerts to the operator. |
| **How Much** | **0 tests** · 2 deps |

---

### `permanent-storage`

| W5H2 | Answer |
|------|--------|
| **Who** | The final station in the ingestion pipeline. Called after all quality checks pass. |
| **What** | Commits Golden Record particles to the permanent append-only store, finalising the sovereign write path. |
| **When** | After all pipeline stations approve an ingestion — the particle becomes permanent. |
| **Where** | `crates/permanent-storage/` — Layer 8. |
| **Why** | Separating "committed to Journal" from "in permanent storage" allows for two-phase durability: fast Journal writes for hot path, then bulk fsync to permanent storage for cold-path durability. |
| **How** | `PermanentStore::open(path, tribe_id) -> Self`. `commit(entry: JournalEntry) -> CommitStats`. Appends the entry to a tribe-partitioned file with `FsyncPolicy::PerCommit`. `CommitStats { total_committed, bytes_written }`. |
| **How Much** | **0 tests** · 3 deps |

---

## Layer 9: Languages

### `aaol`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by eridu-runtime to interpret actor orchestration scripts. |
| **What** | The Akkadian Actor Orchestration Language — a simple scripting language (`.akk` files) for defining sovereign actor workflows. |
| **When** | At runtime when an operator deploys a new actor workflow via the DubSar IDE. |
| **Where** | `crates/aaol/` — Layer 9. |
| **Why** | Complex ingestion and monitoring workflows should be expressible as declarative scripts rather than hardcoded Rust logic. AAOL provides a minimal, auditable language for sovereign orchestration. |
| **How** | `tokenize(source: &str) -> Vec<Token>` lexes `.akk` source. `Parser::parse() -> Program { stmts: Vec<Statement> }`. `Statement { actor, action, args }`. The language is intentionally minimal — no loops, no branching — to remain auditable and deterministic. |
| **How Much** | **0 tests** · 1 dep (bahyway-core) |

---

### `heptascript` ✦

| W5H2 | Answer |
|------|--------|
| **Who** | Built this session (ModularNaviIndex + ROUTE extension added). Used by enkidb-query, bahyway-cli, and the DubSar IDE `.hepta` REPL. |
| **What** | The 7-dimensional sovereign query language for the particle space — plus the `ModularNaviIndex` routing topology index inspired by Viazovska's modular forms. |
| **When** | On every user query in the DubSar REPL; on every SELECT/ROUTE statement from the CLI; during ModularNaviIndex construction from a NaviGraph. |
| **Where** | `crates/heptascript/` — Layer 9. |
| **Why** | A sovereign database needs a sovereign query language. HeptaScript is purpose-built for BahyWay's 7-dimensional particle space and NaviMap topology — no SQL adapter needed. |
| **How** | **Stage 1**: `ModularNaviIndex` encodes a NaviMap's cost landscape as a theta-series histogram `a(n) = edges in bucket n`. Provides `resonance_score(cost)`, `signature()`, `is_equivalent()`, `fourier_repr()`. `ResonanceScorerNc2` replaces flat tribe-bonus with spectral-peak-weighted resonance. **Stage 2**: `E2FourierWeights` computes Eisenstein-E₂-inspired sacred weights for all 7 sectors using `C(k,N) = (1/N) Σ σ₁(n)/n × cos(2πnk/7)`. Result: Entrance→1.00 (neutral), Awliya/Ulamaa→0.80 (most elevated by cosine minimum). **Lexer**: 47 token types including new `Route`, `Index`, `Modular`, `AttrWeight`, `AttrLevel`, `SignedInteger`. |
| **How Much** | **84 tests** (stage 1: 59, stage 2: 25) · 6 deps (bahyway-core, bahyway-crc, navi-engine, story-engine, enkidb-kaki, enkidb-journal, enkidb-snapshot) |

**E₂ Fourier result (N=100 terms):**

| Sector k | Zone | C(k) raw | Derived Weight |
|----------|------|----------|----------------|
| 0 | Entrance | +1.622 | **1.000** (DC singularity, neutral) |
| 1 = 6 | Shuhadaa = Anbiya | +0.020 | **0.804** |
| 2 = 5 | Awliya = Ulamaa | −0.010 | **0.800** (most elevated) |
| 3 = 4 | Huffaz = Momineen | +0.016 | **0.803** |

*Mathematical discovery*: The Fourier analysis disagrees with hand-chosen NajafEngine weights — Awliya/Ulamaa are mathematically most elevated, not Shuhadaa. Both representations are available: hand-chosen (`NajafSector::sacred_weight()`) and Fourier-derived (`E2FourierWeights::compute()`).

---

## Layer 10: Runtime / OS

### `eridu-runtime`

| W5H2 | Answer |
|------|--------|
| **Who** | The cooperative task executor — used by all long-running processes (bahyway-server, enkidw). |
| **What** | Provides a synchronous cooperative task executor that runs BahyWay tasks in a deterministic single-threaded loop. |
| **When** | At process start — creates the runtime, registers tasks, and drives the main loop. |
| **Where** | `crates/eridu-runtime/` — Layer 10. |
| **Why** | BahyWay makes the deliberate architectural choice of cooperative (not preemptive) concurrency — no Tokio, no async/await. This keeps the runtime deterministic, auditable, and free of data races without `Mutex`. |
| **How** | `EriduRuntime { tasks: Vec<Task>, state: RuntimeState }`. `Task { name, run: Box<dyn FnMut() -> TaskResult> }`. `run_loop()` calls each task's `run()` in registration order, checking `TaskResult { Continue, Done, Error }` after each call. `RuntimeState { tick: u64, active: usize }`. |
| **How Much** | **0 tests** · 1 dep (bahyway-core) |

---

### `eridu-scheduler`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by eridu-runtime to drive time-based jobs (snapshot-job, DW ETL). |
| **What** | Tick-based job scheduler — fires registered jobs when their tick interval has elapsed. |
| **When** | On every runtime tick — the scheduler checks each job's last-fired tick and calls jobs that are due. |
| **Where** | `crates/eridu-scheduler/` — Layer 10. |
| **Why** | Without a scheduler, periodic jobs (snapshots, ETL batches) would need manual timer management in each crate. The scheduler provides one source of scheduled truth. |
| **How** | `EriduScheduler { jobs: Vec<ScheduledJob>, current_tick: u64 }`. `ScheduledJob { interval_ticks: u64, last_fired: u64, job: Box<dyn FnMut()> }`. `tick()` increments `current_tick` and fires any job where `current_tick - last_fired >= interval_ticks`. |
| **How Much** | **0 tests** · 1 dep (bahyway-core) |

---

### `eridu-supervisor`

| W5H2 | Answer |
|------|--------|
| **Who** | Wraps the runtime and monitors task health. Used by bahyway-server. |
| **What** | Lifecycle and health management — detects task failures, triggers recovery, and maintains system health status. |
| **When** | Throughout process lifetime — checks health after each runtime tick and initiates recovery on crash detection. |
| **Where** | `crates/eridu-supervisor/` — Layer 10. |
| **Why** | The cooperative runtime does not crash by design, but external failures (disk full, memory exhaustion) can cause task errors. The supervisor is the "immune system" that detects and responds to degradation. |
| **How** | `EriduSupervisor { runtime, recovery }`. `tick()` calls `runtime.run_loop()`, checks `HealthStatus { Healthy, Degraded, Critical }` for each task. On `TaskResult::Error`: calls `RecoveryProcedure`. `HealthStatus` is derived from the ratio of error tasks to total tasks. |
| **How Much** | **0 tests** · 3 deps |

---

## Layer 11: UI / IDE

### `dubsar-ide`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by the dubsar binary. The operator's primary interaction point. |
| **What** | Language services for the DubSar IDE — provides diagnostics, source classification, and HeptaScript/AAOL validation for the terminal environment. |
| **When** | On every command entered in the DubSar REPL — validates syntax before execution. |
| **Where** | `crates/dubsar-ide/` — Layer 11. |
| **Why** | The IDE layer separates "language understanding" (is this valid HeptaScript?) from "execution" (run the query). This enables future LSP integration without touching the execution pipeline. |
| **How** | `DubSarIde { hepta_parser, aaol_parser }`. `check(source: &str, kind: SourceKind) -> Vec<Diagnostic>`. `SourceKind { HeptaScript, Akkadian }`. `Diagnostic { line, col, message, severity }`. |
| **How Much** | **0 tests** · 3 deps |

---

### `dubsar-visualizer`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by the dubsar binary for ColorID display in the terminal. |
| **What** | Renders `ColorRgb` values as ANSI-coloured terminal text blocks for the DubSar IDE. |
| **When** | On every query result that includes particle ColorID data. |
| **Where** | `crates/dubsar-visualizer/` — Layer 11. |
| **Why** | The ColorID is a visual signal — looking at `ColorRgb { r:200, g:180, b:50 }` conveys nothing. Rendering it as a coloured block in the terminal gives the operator immediate quality intuition. |
| **How** | `ColorIdDisplay { color: ColorRgb, label: &'static str }`. `render(display) -> String` produces ANSI escape sequences (`\x1b[38;2;{r};{g};{b}m█\x1b[0m`) for 24-bit terminal colour. |
| **How Much** | **0 tests** · 1 dep (score-engine) |

---

## Binaries

### `bahyway-server`

| W5H2 | Answer |
|------|--------|
| **Who** | The production daemon — all client connections go through it. |
| **What** | The main BahyWay server daemon — accepts ingestion requests, runs the pipeline, and serves HeptaScript queries. |
| **When** | Runs as a long-lived process managed by the OS or eridu-supervisor. |
| **Where** | `bin/bahyway-server/` |
| **Why** | Centralises all pipeline orchestration behind a single sovereign process boundary. |

---

### `bahyway-cli`

| W5H2 | Answer |
|------|--------|
| **Who** | Used by operators and developers for ad-hoc queries and administration. |
| **What** | CLI: HeptaScript REPL + admin operations (tribe management, schema comparison, health checks). |
| **When** | On-demand — invoked by the operator. |
| **Where** | `bin/bahyway-cli/` |
| **Why** | A command-line interface is the fastest path from operator question to sovereign answer. |

---

### `najaf-ingest`

| W5H2 | Answer |
|------|--------|
| **Who** | Run by cemetery data management operators. |
| **What** | Ingests civil registry records for Wadi al-Salam from tab-separated input (stdin or file) through the full BahyWay pipeline. |
| **When** | When new civil registry records are received or when importing historical records. |
| **Where** | `bin/najaf-ingest/` |
| **Why** | Provides a ready-to-run ingestion tool for the Najaf cemetery domain without requiring operators to write custom pipeline code. Input format: `<name>\t<epoch>\t<state>` (Golden/Fuzzy/Dead). |

---

### `dubsar`

| W5H2 | Answer |
|------|--------|
| **Who** | The operator's sovereign terminal IDE. |
| **What** | DubSar IDE — a sovereign terminal environment with HeptaScript REPL, DAMA-DMBOK dictionary lookup (`.dama`), ShoWay dashboard generation (`.dashboard`), and NABA hologram agent (`.naba`). |
| **When** | Used interactively by operators and data stewards throughout the system lifecycle. |
| **Where** | `bin/dubsar/` |
| **Why** | A sovereign ecosystem needs a sovereign IDE — one that embeds the governance dictionary, the quality dashboard, and the query language in a single terminal tool. No browser required. |
| **Commands**: `.dama [CODE]`, `.dama search <keyword>`, `.dashboard`, `.naba`, `.hepta <query>` |

---

### `enkidw`

| W5H2 | Answer |
|------|--------|
| **Who** | Run by data warehouse operators and scheduled ETL jobs. |
| **What** | EnkiDW CLI — drives the ETL pipeline, watches the landing zone for incoming files, and generates DW analytics reports and sovereign ZIP bundles. |
| **When** | Scheduled (via eridu-scheduler) or on-demand for batch analytics runs. |
| **Where** | `bin/enkidw/` |
| **Why** | The DW layer needs its own binary separate from the OLTP server — batch analytics should not compete with real-time ingestion for process resources. |

---

## Appendix A: Test Summary

| Crate | Tests | Layer | Status |
|-------|-------|-------|--------|
| bahyway-core | 0 | 0 | Stable |
| bahyway-crc | 4 | 0 | Stable |
| enkidb-kaki | ~20 | 1 | Stable |
| enkidb-vector-id | 0 | 1 | Stable |
| enkidb-block | 0 | 2 | Stable |
| enkidb-journal | 0 | 2 | Stable |
| enkidb-storage | 0 | 2 | Stable |
| enkidb-snapshot | 0 | 2 | Stable |
| enkidb-recovery | 0 | 2 | Stable |
| enkidb-persist | 0 | 2 | Stable |
| enkidb-dw | 0 | 2 | Stable |
| enkidb-indexes | 0 | 3 | Stable |
| enkidb-engine | 0 | 4 | Stable |
| enkidb-query | 0 | 4 | Stable |
| story-engine | 0 | 5 | Stable |
| fuzzy-engine | 5 | 5 | Stable |
| score-engine | 0 | 5 | Stable |
| alert-engine | 0 | 5 | Stable |
| snapshot-job | 0 | 5 | Stable |
| **navi-engine** ✦ | **118** | 5 | **Built this session** |
| **najaf-engine** ✦ | **82** | 5 | **Built this session** |
| idu-prober | 0 | 6 | Stable |
| idu-batching | 0 | 6 | Stable |
| template-engine | 0 | 7 | Stable |
| template-library | 0 | 7 | Stable |
| diagnosis-templates | 0 | 7 | Stable |
| **damadmbok-dictionary** ✦ | **22** | 7 | **Built this session** |
| **bahyway-dqm** *(added 2026-06-04)* | **77** | 8 | **Added this update** |
| adad-gate | 0 | 8 | Stable |
| musaru-security | 0 | 8 | Stable |
| compare-tribe-schema | 0 | 8 | Stable |
| vgca-validation | 0 | 8 | Stable |
| data-structure-station | 0 | 8 | Stable |
| data-cleansing-station | 0 | 8 | Stable |
| data-steward-station | 0 | 8 | Stable |
| permanent-storage | 0 | 8 | Stable |
| aaol | 0 | 9 | Stable |
| **heptascript** ✦ | **84** | 9 | **Built this session** |
| eridu-runtime | 0 | 10 | Stable |
| eridu-scheduler | 0 | 10 | Stable |
| eridu-supervisor | 0 | 10 | Stable |
| dubsar-ide | 0 | 11 | Stable |
| dubsar-visualizer | 0 | 11 | Stable |
| **TOTAL** ✦ | **≥412** | — | — |

---

## Appendix B: The Seven Sovereign Laws

Every crate in BahyWay v4.0 honours these invariants:

1. **Zero external dependencies** — only Rust std + BahyWay crates.
2. **Immutable identity** — once a KAKI is minted, its bytes never change.
3. **Append-only truth** — the Journal never overwrites; recovery reads only.
4. **Tribal sovereignty** — no particle crosses a tribe boundary without explicit CrossTribeKaki authorisation.
5. **Deterministic quality** — the same inputs always produce the same ColorRgb, quality tier, and particle state.
6. **Pure Rust safety** — no `unsafe` outside enkidb-storage (memory mapping only).
7. **W5H2 transparency** — every crate can be explained to any stakeholder through Who/What/When/Where/Why/How/How Much.

---

## Appendix C: What Comes Next

This is **Step 1** of the Total BahyWay v4.0 Sovereign Manual. Subsequent steps will cover:

| Step | Content |
|------|---------|
| Step 2 | **WPDEngine** — work-permit domain engine (depends on navi-engine) |
| Step 3 | **DMWEngine** — domain-management workflow engine |
| Step 4 | **Integration tests** — `tests/e2e/` and `tests/chaos/` W5H2 coverage |
| Step 5 | **Deployment Manual** — eridu-runtime configuration, tribe provisioning |
| Step 6 | **Governance Manual** — DAMA-DMBOK alignment table, sovereignty audit checklist |
| Step 7 | **Total Manual** — complete sovereign reference after all enterprise apps are built |

---

## Appendix D: Sovereign Glossary

Terms specific to BahyWay v4.0 that cannot be found in any standard technical reference.
Each definition is self-contained and requires no external lookup.

---

### KAKI

**Pronunciation**: /kɑːkiː/ — from Akkadian *kaku* (𒋼𒁀), meaning "armament" or "sovereign seal."

**Definition**: A **Sovereign Identity Particle** — the atomic unit of trust and entity recognition in BahyWay v4.0. A KAKI is not a row ID, not a UUID, not a JWT, and not a foreign key. It is a **deterministic, tribe-bound, immutable identity claim** that any sovereign entity carries from the moment it is created.

**Structure** (4 fields):

| Field | Type | Meaning |
|-------|------|---------|
| `tribe_id` | `TribeId` (u32) | The sovereign tribe that owns and controls this identity |
| `uuid_hash` | `u64` | Deterministic 64-bit hash of the entity's primary key — same input → same bytes, always |
| `role` | `KakiRole` | The entity's sovereign role (Zikru = memory/burial, Nabu = scribe/document, Nergal = boundary/security, etc.) |
| `epoch` | `u32` | Hijri calendar epoch when the KAKI was minted |

**How it is created**: A `KakiMinter` — bound to exactly one tribe — produces KAKIs using its tribe's `TribeId` as an irremovable prefix. No entity may claim a KAKI from a tribe other than its own without a `CrossTribeKaki` authorisation.

**Immutability law**: Once minted, a KAKI's bytes never change. Reassigning roles or epochs creates a *new* KAKI; it does not mutate the existing one. This is Sovereign Law #2.

**Why it exists**: Standard databases identify entities through auto-increment integers or UUIDs that carry no semantic weight — they say nothing about *who owns the entity*, *what role it plays*, or *which epoch it belongs to*. A KAKI encodes sovereignty, role, and time in one compact value, allowing any crate in BahyWay v4.0 to verify an entity's provenance without consulting a central authority.

**Analogy**: Think of the ancient Akkadian cylinder seal — a carved stone cylinder that an official rolled into wet clay to leave an unforgeable imprint. Only that official's seal produced that imprint. A KAKI is the digital equivalent: only the owning tribe's `KakiMinter` can produce a KAKI with that tribe's prefix.

**What it is NOT**:
- NOT a JWT (no expiry, no permission claims, no cryptographic signature)
- NOT a UUID v4 (KAKIs are deterministic, not random)
- NOT a database foreign key (KAKIs are portable identity claims, not relational pointers)
- NOT a hash of the full entity (only the primary identifier is hashed, not the mutable fields)

**Example usage in code**:
```rust
// NajafEngine: mint a KAKI for grave #101 in tribe 0x0001
let minter = KakiMinter::new(0x0001);
let kaki   = minter.mint_identity(101, KakiRole::Zikru);
// kaki.tribe_id  == 0x0001
// kaki.uuid_hash == deterministic_hash(101)
// kaki.role      == KakiRole::Zikru   (memory / burial)
```

**Crate location**: `crates/enkidb-kaki/src/` — Layer 1 (KAKI Identity).

---

### Particle

A **Particle** is the fundamental data unit in BahyWay v4.0 — the equivalent of a "row" in a relational database, but richer. Every particle:
- Carries a KAKI (sovereign identity)
- Has a `ParticleState` (Golden / Fuzzy / Dead / BlackBox)
- Is governed by a tribe
- Belongs to exactly one epoch
- Travels through a pipeline (VaultWay → CompareMWay → cleansing stations → storage)

---

### HeptaScript

BahyWay's native 7-dimensional query language (file extension `.hepta`). Queries the particle space across 7 EAV dimensions: `state`, `quality`, `color_rgb`, `freshness`, `snapshot_date`, `snapshot_state`, `snapshot_frequency`. Extended with `ROUTE` queries over NaviMap topology using the `ModularNaviIndex`. Has no external parser dependencies — pure Rust lexer and query engine.

---

### Sovereign

In BahyWay, **sovereign** means: self-contained, tribe-governed, zero external dependency, deterministic, and auditable. A sovereign component produces the same output for the same input on any machine, requires no network call, no database connection, and no third-party license.

---

*BahyWay v4.0 — Sovereign Ecosystem — Pure Rust — Zero External Dependencies*
*"Seven sectors. Seven engines. One sovereign truth."*
