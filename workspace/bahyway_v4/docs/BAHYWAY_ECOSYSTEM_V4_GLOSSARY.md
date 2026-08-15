# BahyWay.Ecosystem v4.0 — Complete Glossary

> Mesopotamian naming conventions honour the civilization that invented writing,
> cities, and data management.  Every term below has a precise technical meaning.

---

## A

### AnchorStrategy
The plan the HeptaScript engine uses to find the first candidate surrogate before
executing filters.  Five variants:
- `Auto` — engine selects the cheapest strategy from the query shape
- `SurrogateTime` — binary search on NatiruIndex (fastest for time-range queries)
- `StateStation` — filter by orbital state (GOLDEN_GEM, etc.) first
- `E7First` — use the E7 region index as primary filter
- `FullScan` — sequential pass (fallback for unindexed queries)

### ArchiveError (NRM codes)
Errors produced by the NARAMSIN Stage 0 archive engine:
- `NRM_TRUNCATED` — archive truncated mid-transfer (reject, no partial extraction)
- `NRM_UNKNOWN_FORMAT` — no recognised magic bytes (surfaced for architect review)
- `EmptyInput` — zero-byte input
- `MaxRecursionDepth(n)` — nested archive exceeds depth limit (zip-bomb guard)
- `UnsupportedFormat(msg)` — format detected but sovereign module not yet built

### ArchiveFormat
Enum of formats NARAMSIN Stage 0 recognises by magic bytes:
- `Zip` — `PK\x03\x04`
- `TarGz` — `\x1F\x8B`
- `TarBz2` — `BZh`
- `TarXz` — `\xFD7zXZ\x00`
- `SevenZip` — `7z\xBC\xAF\x27\x1C`
- `None` — plain file, no archive wrapper

---

## B

### BeeMDM
BahyWay.Ecosystem Master Data Management pipeline.  Six stations process raw
ingest data into immutable GOLD particles:
1. DataStructure — schema detection, column profiling
2. DataCompare — diff vs reference tribe schema
3. DataCleansing — PII vault, null imputation, type coercion
4. DataSteward — human review queue for flagged records
5. BlackBox — orbital trust probe, anomaly detection
6. GOLD — finalise particles, assign KAKI, write to EnkiDB

### BIGRING
Mathematical topology classification applied automatically to particles that do not
require Tribe_KAKI or DataSteward approval.  Governed by Betti numbers (β₀, β₁, β₂)
from `bahyway-algebra`.  Particles entering BIGRING require no human gate.

### BUCKET_ORBITALS
Constant (= 10) controlling the granularity of NatiruIndex temporal buckets.
Orbital values 0-9 map to bucket 0, 10-19 to bucket 1, etc.  Reduces index size
by 10× compared to storing raw orbital values.

---

## C

### KAKI (𒂵𒆪)
16-byte universal particle identity key.  Immutable — generated once at particle
creation, never changed, never refused by any future release.  Layout:
```
[0..8]   timestamp_ns: u64 LE (nanoseconds since epoch)
[8..12]  tribe_id: u32 LE
[12..14] surrogate: u16 LE
[14..16] crc16: u16 LE (CRC-16/CCITT over bytes [0..14])
```

### ConEngine (enkidb-con-engine)
Sovereign Connection Engine.  Every operation passes through all 7 CSR security
rules before any database access is permitted.  Uses sync `std::net::TcpStream`
and `std::sync::Mutex` — no async runtime.

### ConContext
The security context evaluated by all 7 CSR rules for one operation:
- `caller_role: SovereignRole`
- `caller_tribe: u32`
- `target_tribe: u32`
- `operation: Operation` (Read / Write / CrossTribe / Admin)
- `passport_valid: bool`
- `credential_valid: bool`
- `journal: &mut NaruJournal`

### compare_state (meta.compare_state, planned)
A planned single-scalar particle per playbook variant recording how that
variant's own code compares against what is currently checked into this
repo — never a binary "in repo = keep, else = ignore." Values are the 8
**Shala_&lt;State&gt;** tags (see the S section). Not yet implemented; see
§22 of `BAHYWAY_ECOSYSTEM_MANUAL_V4.md` and Part 2 of `ALL_PBS_ROADMAP.md`.

### CRC-16/CCITT
Polynomial 0x1021, init 0xFFFF.  Used for KAKI structural integrity checks (bytes
κ[14..15]) and NĀRU WAL entry checksums.  Implemented in `bahyway-crc`.

### CRC-32 (ISO 3309)
Polynomial 0xEDB88320 (reflected).  Used for ZIP local-file-header and gzip
checksum validation.  Implemented in `bahyway-crc`.

### CSR Rules (7 rules)
Connection Security Rules enforced by ConEngine on every operation:
| # | Name | Rule |
|---|------|------|
| CSR-01 | Sargon Gate | Caller passport must be valid |
| CSR-02 | Role Check | Write requires TabletWriter or DubSar |
| CSR-03 | NĀRU Audit | Every operation journalled to WAL before proceeding |
| CSR-04 | Credential | Credential must not be expired |
| CSR-05 | Gilgamesh Gate | Cross-tribe Write blocked unless DubSar |
| CSR-06 | KIBRATU Emit | Audit event emitted to KibratuSink |
| CSR-07 | Tribe Isolation | caller_tribe must equal target_tribe unless DubSar exempt |

---

## D

### DataSteward
Human role responsible for approving flagged particles in the BeeMDM pipeline.
Also the `SovereignRole::DataSteward` (READ only) credential in ConEngine.

### depends-on edge
A real, dedicated EnkiDDB edge entity minted by `WriteNode::mint_link_edge`
recording that one catalogued playbook's own text mentions another (a
candidate real-world dependency, approved by an Architect, never
auto-decided). Carries `link.source`/`link.target` (KakiPk) plus
`link.source_title`/`link.target_title`/`link.description = "depends-on"`
(Text, queryable). See **Gate Orbits** and **Pbs_compare_schema**.

### DEFLATE (RFC 1951)
Compression algorithm used inside ZIP and gzip.  NARAMSIN implements a pure safe
Rust inflate (decompressor) inline — no `flate2`, no `miniz_oxide`.  Supports
stored blocks (type 00), fixed Huffman (type 01), dynamic Huffman (type 10).

### Domain (Gate Orbits classification)
A second-level classification inside a **HeptaGate** sector — 7 domains
per gate, 49 total (e.g. Adad/ETL's domains are Extraction, Transformation,
Loading, Validation, Scheduling, Error Handling, Source Connectors). A
single `meta.domain` Text particle per playbook, written via
`WriteNode::tag_domain` only after Architect approval of a scanner's
suggestion — same suggest-then-approve discipline as gate tagging.

### DubSar (𒁾𒊬)
"Tablet writer" in Akkadian.  Highest `SovereignRole` — has WRITE access AND is
exempt from tribe-isolation checks (CSR-07), allowing cross-tribe operations.

---

## E

### E7 (Engine 7)
The seventh EnkiDB engine type (EnkiDDB, port 7007), dedicated to disaster
recovery.  Also used as a region-filter key in HeptaScript (`FILTER_ORDER E7Region`).

### EAV (Entity-Attribute-Value)
The storage model for all particle attributes.  Every attribute write is one EAV
journal entry: `(surrogate, attribute_hash, value_bytes, orbital)`.

### EavExactIndex
Xor8-style fingerprint bloom filter plus sorted `Vec<(attr_hash, val_fp, surrogate)>`.
Provides O(1) negative answers (bloom says absent → definitely absent) and
O(log n) positive lookups.  Lives in `enkidb-indexes`.

### EnkiDB Types (7 total)
| Port | Name | Role |
|------|------|------|
| 7001 | EnkiDB | Primary OLTP particle store |
| 7002 | EnkiDW | Analytical data warehouse |
| 7003 | EnkiSDB | Schema / structure database |
| 7004 | EnkiODB | Orbital / temporal database |
| 7005 | EnkiQDB | Quantum / probabilistic queries |
| 7006 | EnkiMDB | Measurement database |
| 7007 | EnkiDDB | Disaster-recovery database |

### epoch_orbital
The orbital value that maps to bucket 0 in a NatiruIndex.  All bucket calculations
are relative to this offset: `bucket = (orbital - epoch_orbital) / BUCKET_ORBITALS`.

### execute_stream
HeptaScript engine function that streams matching particles to a callback `F`,
avoiding allocation of a 1B-element result Vec:
```rust
pub fn execute_stream<F>(query: &HeptaQuery, journal: &EavJournal, mut callback: F) -> StreamStats
where F: FnMut(u32) -> ControlFlow<()>
```
Returns `StreamStats { matched, evaluated, aborted, plan }`.

---

## F

### FilterOrder / FilterStage
The ordered pipeline of filters applied per particle after anchoring.  Eight stages:
1. `SurrogateRange` — numeric range on surrogate ID
2. `OrbitalRange` — time-range using NatiruIndex buckets
3. `State` — orbital degradation state filter
4. `DeriveStation` — BeeMDM station of origin
5. `Lane` — data lane (GOLD / CLEANSING / RAW etc.)
6. `QualityByte` — DQM quality score threshold
7. `EavAttr` — attribute-value exact match (EavExactIndex)
8. `E7Region` — region-specific filter

### FanOutPolicy
ŠUMU-UKIN routing policy for multi-target queries:
- `AllParallel` — dispatch to all targets via `std::thread::spawn`
- `AllSerial` — dispatch to all targets sequentially
- `FirstOnly` — use the first resolved target

### FingerprintTable
Internal Xor8-style bloom substitute in EavExactIndex.  One byte per slot in a
power-of-2 table.  Insert sets bits; query checks bits.  False positives possible;
false negatives impossible.

### FnvHash (FNV-1a 32-bit)
Non-cryptographic hash used for attribute names and value fingerprints:
- Prime: 0x01000193
- Offset: 0x811C9DC5

---

## G

### GATE (HeptaScript clause)
Security predicate evaluated before query execution: `GATE role = "DataSteward"`.

### Gate Orbits (Shala4)
The Shala dashboard tab that browses catalogued playbooks first as the 7
**HeptaGate** sectors, then by **Domain**, then individually, plus real
`depends-on`/`depended-on-by` dependency discovery — each level rendered
as a real Three.js orbit/cube-stack scene backed by live EnkiDDB queries,
not simulated data. See §22 of `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`.

### Gilgamesh Gate (CSR-05)
Blocks cross-tribe Write operations unless the caller is DubSar.  Named for the
king of Uruk who crossed forbidden boundaries to reach Utnapishtim.

### GOLD Station
Final BeeMDM ETL station.  Particles passing all prior stations receive their
permanent KAKI, are written to EnkiDB (port 7001), and become immutable.

---

## H

### HEPT Protocol
BahyWay binary wire protocol carried over TCP.  Magic bytes: `0x48455054` ("HEPT").
Each frame: `[magic:4][length:4][payload:n]`.  Defined in `enkidu-protocol`.

### HeptaGate
The real, sealed 7-sector enum in `bahyway_core::hepta_gate::HeptaGate`:
Apsu (Storage), Adad (ETL), Shedu (Security), Mummu (Algebra), Enkidu (AI
Agents), Dubsar (Languages), Enlil (Governance) — domains of the codebase
itself, used by **Gate Orbits** to classify playbooks. Not to be confused
with the BeeMDM ETL pipeline-stage mapping in Manual §15 (Ingestion/
Validation/Enrichment/Transformation/Federation/Indexing/Mastering), a
different, earlier 7-gate concept that happens to share the Sumerian gate
names and the number 7.

### HeptaScript v2.0 W5H2
BahyWay.Ecosystem native query language.  W5H2 = five Ws (Who/What/When/Where/
Why) plus two Hs (How/How-much).  Clauses:
- `NODE` — particle selection
- `ACROSS` — cross-tribe traversal
- `TIER` — database tier selection
- `STATE` — orbital state filter
- `NASH` — Nash-equilibrium scoring
- `PATTERN` — structural pattern matching
- `LINEAGE` — ancestor/descendant tracing
- `GATE` — security predicate
- `SATAMU` — result aggregation
- `ORBITAL` — time-range clause
- `ANCHOR` — execution strategy hint
- `STREAM` — streaming output mode
- `DERIVE_STATION` — BeeMDM station filter
- `ABORT_SCAN` — emergency particle cut-off
- `FILTER_ORDER` — explicit filter pipeline ordering

---

## I

### inflate
Pure safe Rust DEFLATE decompressor.  Supports block types 00 (stored), 01 (fixed
Huffman), 02 (dynamic Huffman).  Returns `None` on any decode error.  Lives inside
`naramsin-archive::zip`.

---

## J

### Journal (EAV Journal)
Append-only log of all EAV writes.  Each 64-byte entry: surrogate + attribute_hash
+ value_bytes_length + orbital + crc16.  Never truncated or deleted.

---

## K

### KIBRATU (CSR-06)
Audit event emission step.  Every operation that passes CSR-01 through CSR-05 emits
a `KibratuCause` event to the KibratuSink before the database call executes.

### KibratuCause
Reason codes for KIBRATU events:
- `Stale` — particle not updated within expected orbital window
- `ParallelFailure` — concurrent write conflict
- `UnknownState` — particle in unrecognised orbital state
- `Misclassified` — BeeMDM station assigned wrong classification
- `HiddenPattern` — orbital pattern detected that wasn't expected
- `IntruderCorrupt` — external corruption signature detected
- `Puzru` — unexplained state transition (requires DataSteward review)

---

## L

### LINEAGE (HeptaScript clause)
Traces ancestry or descendant particles across orbital time:
`LINEAGE depth = 3 direction = ANCESTOR`.

---

## M

### MAX_RECURSION
Constant = 4 in NARAMSIN Stage 0.  Maximum nested archive depth before
`ArchiveError::MaxRecursionDepth` is returned (zip-bomb protection).

### mint_link_edge (CrossTribe-Kaki link primitive)
`WriteNode::mint_link_edge` — mints a dedicated edge entity per relationship
(never a repeated particle on a shared entity, which would collapse under
last-write-wins). The general CrossTribe-Kaki link mechanism; reused as-is
for **depends-on edge**s and for `pb_catalog`'s own found-in/contains edges.

---

## N

### NARAMSIN (𒈎𒋀𒁲)
Named for Naram-Sin, grandson of Sargon.  The BahyWay.Ecosystem sovereign archive
and format reader engine (EN-NARAMSIN-001).
- Stage 0 (`naramsin-archive`): archive detection and decompression
- Stage 1 (`naramsin-format`): format parsing (CSV / JSON / XML)

### NaruEntry
One 64-byte fixed-width binary NĀRU WAL journal record.  Layout:
```
[0..4]   entry_seq: u32 LE
[4..8]   timestamp_epoch: u32 LE
[8..12]  tribe_id: u32 LE
[12..16] operation_code: u32 LE
[16..48] surrogate_kaki: [u8; 32]
[48..62] reserved: [u8; 14]
[62..64] crc16: u16 LE
```

### NaruJournal
Append-only in-memory NĀRU WAL buffer.  `max_entries` cap prevents unbounded
growth.  `verify_all()` checks CRC-16 on every entry.

### NatiruIndex
Temporal surrogate index for orbital-range pruning.  Stores sorted
`Vec<(bucket_id: u32, surrogate: u32)>`.  O(log n + result_size) range queries.
Lives in `enkidb-indexes`.

### NĀRU (𒀭𒈠𒊒)
"River" in Akkadian.  The WAL (Write-Ahead Log) audit journal in ConEngine.  Every
operation is journalled to NĀRU before any database mutation proceeds (CSR-03).

---

## O

### Orbital
Logical timestamp unit in BahyWay.  Similar to a Lamport clock.  Monotonically
increasing per particle.  Used by NatiruIndex for temporal range pruning.

### Orbital Degradation
Seven visual states a particle passes through as its data quality evolves:
1. `GOLDEN_GEM` (β₀=1, β₁=0, β₂=0)
2. `GREEN_BRONZE`
3. `YELLOW_CLAY`
4. `ORANGE_RUST`
5. `RED_CRACKED`
6. `DARK_ASH`
7. `DEAD_SEALED` (immutable, never erased)

Visualised in Godot 4 GDScript via `OrbitalDegradation.gd`.

---

## P

### Particle
The fundamental unit of data in BahyWay.Ecosystem.  A particle has:
- One KAKI (immutable identity)
- One surrogate (u32, internal join key)
- Zero or more EAV attributes (mutable, journalled as orbital entries)
- One orbital degradation state

### PAZUZU
Named for the Mesopotamian demon of windstorms.  Seven threat tests targeting each
CSR gap (PAZUZU-01 through PAZUZU-07), run as part of Phase 2 security validation.

### Pbs_compare_schema
The `meta.collection` tag under which this feature's own Roadmap/Manual/
Glossary documentation (this document included) is minted into EnkiDDB as
real, versioned, KAKI-sealed particles — one distinct Identity-Kaki per
document, via `WriteNode::ingest_document_categorized`, mirroring the
`Preparing_bare_metal_PBs_Run` collection pattern. Named for what the
underlying feature actually does: compare each catalogued playbook's text
against every other's to discover real `depends-on` relationships.

### PII Vault
DataCleansing Station component that detects and vaults personally identifiable
information before particles reach the GOLD station.  Crate: `pii-vault`.

---

## Q

### QueryPlan
HeptaScript engine's compiled execution plan.  Fields include `anchor`,
`derive_station`, `abort_scan`, `filter_order`, `orbital_range`.

---

## R

### RoutingTarget
ŠUMU-UKIN target record: `{ session_id, host, port, tribe_id }`.

---

## S

### Sargon Gate (CSR-01)
First CSR rule.  Rejects any operation where `passport_valid == false`.  Named for
Sargon of Akkad, who established the first empire with a centralised passport system.

### SessionEntry
One entry in the session registry (`enkidb-session-registry`):
```
id, host, port, role (Write/Read), node_type (EnkiDB..EnkiDDB),
tribe_ids (empty = all tribes), label, enabled
```

### SessionRegistry
Loaded from `enkidb-sessions.toml` via a native TOML parser (no serde).  Methods:
`by_role()`, `by_tribe()`, `by_id()`, `enabled()`.

### Shala_&lt;State&gt; (PB Compare State values, planned)
The 8 values a **compare_state** particle can hold, each prefixed with the
producing service's namespace (`Shala_`) rather than the shared
`meta.compare_state` attribute name — so the same generic attribute stays
self-describing forever even if a future non-Shala producer (Godot-side,
BeeMDM-side) ever writes into it with its own comparison semantics:
- `Shala_Active` — freshly discovered variant, not yet compared
- `Shala_Fuzzy` — partial/uncertain match against current repo code
- `Shala_Golden` — matches current repo code, canonical
- `Shala_PartiallyAccepted` — some content merged, rest not
- `Shala_Deprecated` — was Golden, now superseded (via `supersede_document`, never deleted)
- `Shala_Aged` — stale, no recent activity, lower confidence
- `Shala_Rejected` — Architect explicitly reviewed and set aside
- `Shala_Dead` — sealed, immutable, historical only

Not yet implemented; see §22 of `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`.

### SovereignRole
Access level for ConEngine operations:
- `Client` (0) — READ only
- `DataSteward` (1) — READ only
- `TabletWriter` (2) — READ + WRITE
- `DubSar` (3) — READ + WRITE + cross-tribe exempt

### StreamStats
Result of `execute_stream`: `{ matched: u64, evaluated: u64, aborted: bool, plan: QueryPlan }`.

### ŠUMU-UKIN (𒀭𒌓𒆳𒁺)
Babylonian governor under Assyrian sovereignty.  The BahyWay routing layer that
resolves a query's tribe filter to session targets and fans the query out.
Crate: `heptascript::sumuukin`.

---

## T

### TabletWriter
`SovereignRole` with WRITE access.  Cannot perform cross-tribe operations.

### Triple-O (Orbit-Oriented Ontology)
BahyWay.Ecosystem core architectural principle: every entity is a Particle, every
change is an orbital event, and all history is preserved forever.

### Tribe
A logical partition of particles sharing governance.  Identified by `tribe_id: u32`.
Cross-tribe access requires DubSar role (CSR-07).

---

## U

### UTNAPISHTIM
App registration layer.  Phase 2 deliverable: add ConEngine gate to every new app
registered through UTNAPISHTIM (PAZUZU-07 gap closure).

---

## V

### val_fingerprint
32-bit FNV-1a hash of the raw encoded attribute value bytes.  Used as the second
key component in EavExactIndex entries.  Collision rate ≈ 1 in 4 billion.

---

## W

### W5H2
Query language paradigm: Who, What, When, Where, Why + How, How-much.
HeptaScript v2.0 maps each dimension to clauses that compose into a single query.

---

## X

### Xor8 (fingerprint bloom substitute)
One byte per slot in a power-of-2 table.  Used inside EavExactIndex to provide
O(1) negative answers.  Not a true Xor filter — uses bit-OR insertion and bit-AND
query, giving conservative (never false-negative) behaviour.

---

## Z

### ZIP Bomb
A maliciously crafted archive that expands to enormously large output.  Defended by:
1. `MAX_RECURSION = 4` in NARAMSIN Stage 0 (nested archive depth cap)
2. `expected_len` cap of 64 MB in the DEFLATE inflate function
3. ISIZE validation in gzip unwrapper

### ZIP (PKWARE)
Archive format with local-file-header signature `PK\x03\x04`.  Supported methods:
- Method 0 (STORE) — no compression
- Method 8 (DEFLATE) — RFC 1951

CRC-32 verified on every extracted entry.
