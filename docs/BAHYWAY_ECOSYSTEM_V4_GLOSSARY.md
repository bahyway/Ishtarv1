# BahyWay.Ecosystem v4.0 — Complete Glossary

> Mesopotamian naming conventions honour the civilization that invented writing,
> cities, and data management. Every term below has a precise technical meaning.

**Transparency notice:** every entry below carries a tag from
`docs/TRANSPARENCY_STANDARD.md` — ✅ VERIFIED, 🧩 PARTIAL, 📄 DOCUMENTED,
⚠ COLLISION, ❌ NOT FOUND, 🔒 LAW, or ⏳ UNREACHABLE — plus a citation.
This pass (2026-07-11) re-checked every term against real code and
found two entries that had been silently wrong for weeks despite
appearing in multiple "SEALED" documents: **HEPT Protocol**'s magic
bytes (§H) and **KibratuCause**'s specific variant list (§K). Both are
corrected below, in place, with the wrong claim kept visible rather
than deleted, per this ecosystem's own "old state is never erased"
law.

---

## A

### AnchorStrategy
**✅ VERIFIED** — `heptascript/src/query.rs:375`, `enum AnchorStrategy`.
The plan the HeptaScript engine uses to find the first candidate surrogate
before executing filters. Five variants:
- `Auto` — engine selects the cheapest strategy from the query shape
- `SurrogateTime` — binary search on NatiruIndex (fastest for time-range queries)
- `StateStation` — filter by orbital state (GOLDEN_GEM, etc.) first
- `E7First` — use the E7 region index as primary filter
- `FullScan` — sequential pass (fallback for unindexed queries)

### ArchiveError (NRM codes)
**✅ VERIFIED** — `naramsin-archive/src/error.rs:5`, `enum ArchiveError`.
Errors produced by the NARAMSIN Stage 0 archive engine:
- `NRM_TRUNCATED` — archive truncated mid-transfer (reject, no partial extraction)
- `NRM_UNKNOWN_FORMAT` — no recognised magic bytes (surfaced for architect review)
- `EmptyInput` — zero-byte input
- `MaxRecursionDepth(n)` — nested archive exceeds depth limit (zip-bomb guard)
- `UnsupportedFormat(msg)` — format detected but sovereign module not yet built

### ArchiveFormat
**✅ VERIFIED** — `naramsin-archive/src/format.rs:5`. Real `magic()` method,
byte values confirmed by direct read:
- `Zip` — `[0x50, 0x4B, 0x03, 0x04]` (`PK\x03\x04`)
- `TarGz` — `[0x1F, 0x8B]`
- `TarBz2` — `[0x42, 0x5A, 0x68]` (`BZh`)
- `TarXz` — `[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]`
- `SevenZip` — `[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]`
- `None` — plain file, no archive wrapper

---

## B

### BeeMDM
**🧩 PARTIAL.** BahyWay.Ecosystem Master Data Management pipeline. Two
linked sub-pipelines:

**Pre-KAKI structural pipeline (📄 DOCUMENTED, conceptual — no single
crate implements this exact three-station sequence as one pipeline;
its component stations are individually real):**
1. DataStructure — schema detection, column profiling (`data-structure-station`)
2. DataCompare — diff vs reference tribe schema (`compare-tribe-schema`)
3. DataCleansing — PII vault, null imputation, type coercion (`data-cleansing-station`, `pii-vault`)

**Post-KAKI tier-transition pipeline (✅ VERIFIED, real, tested in
`eridu-runtime::SchedulerLoop`):**
```
EnkiSDB (staged, Pending)
    v ValidationSweep (every 900 ticks)
    +-- pass ------------------------> EnkiODB (Active)
    +-- fail (Quarantined) -> BlackBox Station scan
                                 +-- malware_flag=true  -> Storage Sector (terminal jail)
                                 +-- malware_flag=false  -> EnkiQDB (fuzzy, pending Steward review)
                                                                v Data Steward resolves
                                                     +-- clean -> requeued into EnkiSDB (Pending)
                                                     +-- confirmed harmful -> Storage Sector
```
GOLD / final commit is EnkiDB itself (the append-only Golden Store), reached from
EnkiODB once a particle's operational lifecycle in the pipeline is complete.

### BIGRING
**✅ VERIFIED** — referenced throughout `heptascript` (`engine.rs`,
`lib.rs`, `parser.rs`, `query.rs`, `token.rs`) as a real HeptaScript
`ACROSS BIGRING` clause target, bridged to `bahyway-algebra::orbital`'s
position math (confirmed working end-to-end on `eriduous-vdi` per
`docs/RM-001_ADDENDUM_PB111-145_VERIFIED_2026-07-01.md`). **🧩 PARTIAL**
on the specific claim "particles entering BIGRING require no human
gate, governed automatically by β₀/β₁/β₂" — the Betti-number
computation itself is real (`bahyway-algebra`), but the automatic-entry
gating logic specifically was not re-traced this pass.

### BlackBox Station (`blackbox-station`)
**✅ VERIFIED** — real crate, built and tested 2026-07-01. The Error
Handling Station between EnkiSDB quarantine and a particle's final
jail. `ValidationSweep` marks a particle `Quarantined` but does not decide
*where* it is jailed — BlackBox Station performs that scan on every
Quarantined particle still sitting in EnkiSDB: `malware_flag == true` routes
to the Storage Sector (confirmed harmful, terminal); `malware_flag == false`
routes to EnkiQDB (fuzzy/unknown, pending Data Steward review). Journals
`EventCause::BlackBoxRoutedHarmful` / `BlackBoxRoutedFuzzy` — both confirmed
present in `enkidb-journal/src/event_cause.rs` (0x60/0x61) this pass. **⚠
COLLISION, resolved:** not to be confused with `orbital-trust-probe`, a
separate, unrelated crate that scores causal attribution for orbital drift.

### BUCKET_ORBITALS
**✅ VERIFIED** — `enkidb-indexes/src/nairu_index.rs:24`,
`pub const BUCKET_ORBITALS: u64 = 10`. Controls the granularity of
NatiruIndex temporal buckets. Orbital values 0-9 map to bucket 0, 10-19
to bucket 1, etc. Reduces index size by 10x compared to storing raw
orbital values.

---

## C

### ConEngine (`enkidb-con-engine`)
**✅ VERIFIED** — `enkidb-con-engine/src/lib.rs:53`, `impl ConEngine`.
Sovereign Connection Engine. Every operation passes through CSR-01
through CSR-07 (coded) before any database access is permitted — see
CSR Rules (§C) for CSR-08's separate, law-not-code status. Uses sync
`std::net::TcpStream` and `std::sync::Mutex` — no async runtime.

### ConContext
**✅ VERIFIED** — `enkidb-con-engine/src/csr.rs:18`,
`pub struct ConContext<'a>`. The security context evaluated by the CSR
rules for one operation: `caller_role`, `caller_tribe`, `target_tribe`,
`operation`, `passport_valid`, `credential_valid`, `journal`.

### CRC-16/CCITT
**✅ VERIFIED** — implemented in `bahyway-crc`. Polynomial 0x1021, init
0xFFFF. Used for KAKI structural integrity checks (bytes κ[14..15]) and
NĀRU WAL entry checksums.

### CRC-32 (ISO 3309)
**✅ VERIFIED** — implemented in `bahyway-crc`. Polynomial 0xEDB88320
(reflected). Used for ZIP local-file-header and gzip checksum
validation.

### CSR Rules (8 rules)
**✅ VERIFIED / 🔒 LAW for CSR-08.** Connection Security Rules enforced
by ConEngine. CSR-01 through CSR-07 confirmed coded in
`enkidb-con-engine/src/csr.rs` this pass (`cargo test -p
enkidb-con-engine`: 6 tests passing).

| # | Name | Rule | Status |
|---|------|------|---|
| CSR-01 | Sargon Gate | Caller passport must be valid | ✅ coded |
| CSR-02 | Role Check | Write requires TabletWriter or DubSar | ✅ coded |
| CSR-03 | NĀRU Audit | Every operation journalled to WAL before proceeding | ✅ coded |
| CSR-04 | Credential | Credential must not be expired | ✅ coded (stub validator — accepts any zeroed blob, per PAZUZU-01 finding) |
| CSR-05 | Gilgamesh Gate | Cross-tribe Write blocked unless DubSar | ✅ coded |
| CSR-06 | KIBRATU Emit | Audit event emitted to KibratuSink | ✅ coded, stub/no-op — see KIBRATU (§K) for the separate, unresolved KibratuCause question |
| CSR-07 | Tribe Isolation | caller_tribe must equal target_tribe unless DubSar exempt | ✅ coded |
| CSR-08 | Architect Sovereignty | No sovereign component (including AI agents) may create, modify, or delete any organ — crate, engine, agent, template, KAKI, tribe, session, playbook, or configuration — without explicit DUB.SAR confirmation. Diagnosis is autonomous; prescription is proposed; execution is the Architect's alone. Unlike CSR-01–07, this is not a single ConEngine code path — it is a cross-cutting rule enforced at every agent boundary (NINSUN, TamuzAI, EaAgent, NuskuAgent). | 🔒 LAW, sealed identically in two independent 2026-06-26/27 documents. **NOT coded** as of 2026-07-11 — `csr.rs` implements CSR-01–07 only. This is the real, open PB-170 gate. |

### Naming Law — "-Way" suffix
**🔒 LAW, sealed 2026-07-03.** *"All suffix 'WAY' in crate names are
deprecated, to preserve 'WAY' as the Security Protocols Language that
has the `.way` file type."* In force: `AkkadiSafeWay`→`AkkadiSafeEngine`,
`AkkadiRulesWay`→`AkkadiRulesEngine`, `AkkadiCipherWay`→`AkkadiCipherEngine`.
**Explicitly exempted:** WAY v2.0 itself keeps its name unchanged.
v3.5-era source comments that self-label as v3.5 are historical record,
not current-law violations.

---

## D

### DataSteward
**✅ VERIFIED.** Human role responsible for approving flagged particles
in the BeeMDM pipeline. Also `SovereignRole::DataSteward` (READ only)
in ConEngine — confirmed `enkidb-con-engine/src/roles.rs:5`. Two real,
tested queues (`data-steward-station`, `ninsun-steward-bridge`):
- `QuarantineReviewQueue` — pulls the fuzzy (`BlackBoxRoutedFuzzy`) backlog
  from EnkiQDB; resolves clean (`StewardResolvedRequeue`) or confirms
  harmful (seals into Storage Sector). EnkiQDB is append-only.
- `NinsunAdvisoryQueue` — confidence-ordered inbox of NINSUN's
  `RefineProposal`s. Confirm/reject
  (`NinsunAdvisoryConfirmed`/`NinsunAdvisoryRejected`, both confirmed
  present in `event_cause.rs` at 0x64/0x65 this pass). NINSUN never
  decides and never modifies a particle.

### DEFLATE (RFC 1951)
**📄 DOCUMENTED** — the `naramsin-archive` crate and its 9 tests are
✅ VERIFIED real (confirmed via `TESTING_PLAYBOOK_PHASE1.md` and this
session's own test runs); the specific claim "pure safe Rust inflate,
no `flate2`/`miniz_oxide`, supports stored/fixed-Huffman/dynamic-Huffman
blocks" was not re-traced line-by-line against `naramsin-archive::zip`'s
inflate implementation this pass.

### DubSar (𒁾𒊬)
**✅ VERIFIED** — `SovereignRole::DubSar`, confirmed in
`enkidb-con-engine/src/roles.rs`. "Tablet writer" in Akkadian. Highest
`SovereignRole` — WRITE access, exempt from tribe-isolation (CSR-07).

---

## E

### E7 (Engine 7)
**✅ VERIFIED**, consistent with EnkiDB Types (§E) below. The seventh
EnkiDB engine type (EnkiDDB, port 7007). Also used as a region-filter
key in HeptaScript (`FILTER_ORDER E7Region` — confirmed token exists).

### EAV (Entity-Attribute-Value)
**🔒 LAW / ✅ VERIFIED pervasively.** The storage model for all particle
attributes. Every attribute write is one EAV journal entry:
`(surrogate, attribute_hash, value_bytes, orbital)`.

### EavExactIndex
**✅ VERIFIED** — `enkidb-indexes/src/eav_exact_index.rs:76`,
`pub struct EavExactIndex`. Xor8-style fingerprint bloom filter plus
sorted `Vec<(attr_hash, val_fp, surrogate)>`. O(1) negative answers,
O(log n) positive lookups.

### EnkiDB Types (7 total)
**✅ VERIFIED, updated 2026-07-11** — see
`docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md` §3 for the full,
current table with per-type crate citations. Summary: EnkiMDB (7006)
and EnkiDDB (7007) are now real (`enkimdb`, `enkiddb` crates) — every
document dated through 2026-07-07 recorded these as not yet built.

### epoch_orbital
**✅ VERIFIED** — `enkidb-indexes/src/nairu_index.rs:32`. The orbital
value that maps to bucket 0 in a NatiruIndex.

### EventCause (`enkidb-journal::event_cause`)
**✅ VERIFIED** — `enkidb-journal/src/event_cause.rs:11`, full enum read
this pass. One `u8` discriminant, stable (never re-ordered). Groups:
particle lifecycle (`0x01`), four Pauli Exclusion Gates (`0x10-0x17`
— note: gate names in code are `Adad/Anu/Marduk/Shamash`, not the
`ADAD/SHEDU/MUMMU/...` HeptaGate names used elsewhere — **⚠ COLLISION,
unresolved**, these appear to be two different 4/7-gate systems both
using Mesopotamian names, not yet disambiguated anywhere), orbital
assignment (`0x20`), Musarû security (`0x30-0x32`), Diagnosis Engine
color events (`0x40-0x42`), five-tier EnkiDB transitions (`0x50-0x54`),
BlackBox/Storage Sector/Steward loop-back (`0x60-0x63`), NINSUN
advisory review (`0x64-0x65`). **This enum does NOT contain any
KIBRATU-named cause codes** — see KIBRATU (§K) for the correction.

### execute_stream
**✅ VERIFIED** — `heptascript/src/engine.rs:152`. Streams matching
particles to a callback, avoiding allocation of a 1B-element result
Vec. Returns `StreamStats { matched, evaluated, aborted, plan }`
(`StreamStats` confirmed at `engine.rs:127`).

---

## F

### FilterOrder / FilterStage
**✅ VERIFIED** — `heptascript/src/query.rs:395`, `enum FilterStage`.
Eight stages: `SurrogateRange`, `OrbitalRange`, `State`,
`DeriveStation`, `Lane`, `QualityByte`, `EavAttr`, `E7Region` (names
not individually re-diffed against the enum this pass, but the type
and its 8-stage design are confirmed real).

### FanOutPolicy
**✅ VERIFIED** — `heptascript/src/sumuukin.rs:90`, `enum FanOutPolicy`.
ŠUMU-UKIN routing policy: `AllParallel` (real `std::thread::spawn`
dispatch, built and tested this session), `AllSerial`, `FirstOnly`.

### FingerprintTable
**✅ VERIFIED** — `enkidb-indexes/src/eav_exact_index.rs:49`. Internal
Xor8-style bloom substitute, one byte per slot in a power-of-2 table.

### FnvHash (FNV-1a 32-bit)
**✅ VERIFIED** — `enkidb-indexes/src/eav_exact_index.rs:26`,
`fn fnv1a_32`. Used for attribute names and value fingerprints
(`val_fingerprint`, confirmed same file).

---

## G

### GATE (HeptaScript clause)
**✅ VERIFIED** — `heptascript/src/token.rs`, `Token::Gate`. Security
predicate evaluated before query execution.

### Gilgamesh Gate (CSR-05)
**✅ VERIFIED** — `enkidb-con-engine/src/csr.rs`, confirmed this
session (`CSR-05: Gilgamesh gate — cross-tribe Write blocked unless
DubSar`). Named for the king of Uruk who crossed forbidden boundaries
to reach Utnapishtim.

### GOLD Station
**❌ NOT FOUND** as a distinct implemented station. "GOLD" appears only
as one label in a lane-classification comment
(`heptascript/src/query.rs:404`: "Lane classification filter (GOLD /
SILVER / WHITE / GRAY / RED / BLACK)"). No `GoldStation` type or module
exists. The concept "final BeeMDM station, permanent KAKI written to
EnkiDB" is real in spirit (that's just EnkiDB's Golden Store role, §E),
but "GOLD Station" as a distinct named component is not.

---

## H

### HEPT Protocol
**❌ CORRECTED 2026-07-11 — this entry was wrong.** Previously stated:
"Magic bytes: `0x48455054` ('HEPT'). Each frame: `[magic:4][length:4][payload:n]`."
**Searched directly this pass — no `0x48455054`, no `b"HEPT"`, and no
`HEPT_MAGIC` exists anywhere in the codebase.** The real wire protocol
is `enkidu-protocol::frame::EnkiFrame`
(`enkidu-protocol/src/frame.rs:1`), format
`[kind:1 byte][flags:1 byte][len:4 bytes][payload:len bytes]` — no
magic bytes at all. Frame kinds: `Query=0x01`, `QueryFragment=0x02`,
`ResultStream=0x03`, `ResultEnd=0x04`, `Error=0x05`, `Ping=0x06`,
`Pong=0x07`. This wrong claim had been copied forward unverified
across the 2026-06-27 Architecture Reference, this glossary, and (for
one day) this session's own 2026-07-11 Architecture Reference before
being caught by a direct grep. Kept here, corrected in place, per the
"old state is never erased" law — do not delete this entry, extend it
if the protocol changes again.

### HeptaScript v2.0 W5H2
**✅ VERIFIED** — BahyWay.Ecosystem native query language. W5H2 = five
Ws (Who/What/When/Where/Why) plus two Hs (How/How-much). Clause tokens
confirmed present in `heptascript/src/token.rs` this pass: `NODE`,
`ACROSS`, `TIER`, `STATE`, `NASH`, `PATTERN`, `LINEAGE`, `GATE`,
`SATAMU`, `ANCHOR`, `STREAM`, `DERIVE_STATION`, `ABORT_SCAN`,
`FILTER_ORDER`. `cargo test -p heptascript`: 183 tests passing as of
2026-07-11 (up from the 164/170 cited in earlier documents — see
`docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md` §4 for the
KISPU-fix history behind that number).

---

## I

### inflate
**📄 DOCUMENTED**, not re-traced line-by-line this pass. Claimed: pure
safe Rust DEFLATE decompressor inside `naramsin-archive::zip`,
supporting block types 00/01/02. The crate and its passing tests are
✅ VERIFIED real; this specific internal implementation detail was not
independently re-checked.

---

## J

### Journal (EAV Journal)
**✅ VERIFIED**, real and pervasive (`enkidb-journal`). Append-only log
of all EAV writes.

---

## K

### KAKI (𒂵𒆪)
**✅ VERIFIED, corrected 2026-07-11 — full re-derivation from
`enkidb-kaki/src/kaki.rs`'s own header comment (cites `KAKI_v4.0.pdf
§1.2`), not copied from a prior document.** This entry previously
stated a byte layout (`timestamp_ns: u64 [0..8]`, `tribe_id: u32
[8..12]`, `surrogate: u16 [12..14]`, `crc16: u16 [14..16]`) that **does
not match the real code** and has been replaced below. See
`docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md` §2 for full
detail, including two still-open items (κ[8..12]'s exact use, and the
undocumented `Pattern=0x04` `kaki_type`).

16-byte universal particle identity key. Immutable — generated once at
particle creation, never changed, never refused by any future release.
Real layout:
```
κ[0..4]   minted_id / uuid_hash
κ[4..6]   tribe_id     (u16)
κ[6]      kaki_type    (0x01 Identity / 0x02 Event / 0x03 CrossTribe)
κ[7]      kaki_role    (0x01 KISHIB / 0x02 ZIKRU / 0x03 PARZU)
κ[8..12]  reserved     (zeroed — disputed, see open items)
κ[12..14] timestamp
κ[14..16] crc16
```

### KIBRATU (CSR-06)
**⚠ COLLISION / ❌ CORRECTED 2026-07-11 — this entry was wrong on the
specific detail below.** Two separate things share the name KIBRATU:

1. **✅ VERIFIED, real:** CSR-06's audit-emission step. `csr.rs`
   confirms "CSR-06: KIBRATU emit — audit event stub" — real, coded,
   but explicitly a stub/no-op (`TESTING_PLAYBOOK_PHASE1.md`'s own
   D-002 block: "CSR-06 KIBRATU emitted (stub, no-op)").
2. **❌ NOT FOUND:** this entry previously listed a `KibratuCause` enum
   with 7 variants (`Stale`, `ParallelFailure`, `UnknownState`,
   `Misclassified`, `HiddenPattern`, `IntruderCorrupt`, `Puzru`).
   **Searched `enkidb-journal/src/event_cause.rs` directly this pass —
   none of these variant names exist.** The real `EventCause` enum
   (see §E) has an entirely different variant set. If this 7-cause
   taxonomy is still wanted, it needs to be designed and built — it is
   not sitting in code anywhere under this or any other name found
   this session.

### KibratuCause
**❌ NOT FOUND — see KIBRATU above.** Kept as a separate heading (per
the original glossary's structure) so a reader searching specifically
for this term finds the correction directly rather than assuming it
was silently dropped.

---

## L

### LINEAGE (HeptaScript clause)
**✅ VERIFIED** — `heptascript/src/token.rs:38`, `Token::Lineage`
("LINEAGE clause — causality depth"), with `LineageDepth`/`LineageFull`
sub-tokens confirmed at lines 90-91. Traces ancestry or descendant
particles across orbital time.

---

## M

### MAX_RECURSION
**✅ VERIFIED** — `naramsin-archive/src/lib.rs:37`,
`const MAX_RECURSION: u8 = 4`. Maximum nested archive depth before
`ArchiveError::MaxRecursionDepth` is returned (zip-bomb protection).

---

## N

### NARAMSIN (𒈎𒋀𒁲)
**✅ VERIFIED.** Named for Naram-Sin, grandson of Sargon. The
BahyWay.Ecosystem sovereign archive and format reader engine.
- Stage 0 (`naramsin-archive`): archive detection and decompression
- Stage 1 (`naramsin-format`): format parsing (CSV / JSON / XML)

### NaruEntry
**✅ VERIFIED** — `enkidb-con-engine/src/audit.rs:16`,
`pub struct NaruEntry`. One 64-byte fixed-width binary NĀRU WAL journal
record.

### NaruJournal
**✅ VERIFIED** — `enkidb-con-engine/src/audit.rs:61`,
`pub struct NaruJournal`. Append-only in-memory NĀRU WAL buffer.

### NatiruIndex
**✅ VERIFIED** — `enkidb-indexes/src/nairu_index.rs:28`,
`pub struct NatiruIndex`. Temporal surrogate index for orbital-range
pruning. O(log n + result_size) range queries.

### NĀRU (𒀭𒈠𒊒)
**✅ VERIFIED**, real and pervasive. "River" in Akkadian. The WAL
(Write-Ahead Log) audit journal in ConEngine.

### NINSUN (𒀭𒊩𒇻) / `ninsun-agent` / `ninsun-steward-bridge`
**✅ VERIFIED.** Fourth member of the sovereign Agent Quartet — Healer
/ Progressive Refiner. Real, tested (`ninsun-agent::analyze()`,
`ninsun-steward-bridge::NinsunAdvisoryQueue`). This session additionally
wired ESARHADDON's real SMI urgency thresholds into its advisory-queue
priority ordering, and wired `adapa-recall` (real TF-IDF+cosine
retrieval) into its memory search path. Never modifies a committed
particle, never blocks the pipeline — advisory only.

---

## O

### Orbital
**🔒 LAW / ✅ VERIFIED pervasively.** Logical timestamp unit in
BahyWay. Similar to a Lamport clock. Monotonically increasing per
particle.

### Orbital Degradation
**📄 DOCUMENTED**, real strings confirmed present in Godot scripts
(`utnapishtim/src/godot/mod.rs` and others) but the 7-state Rust-side
enum itself was not re-located this pass. Seven visual states:
`GOLDEN_GEM` → `GREEN_BRONZE` → `YELLOW_CLAY` → `ORANGE_RUST` →
`RED_CRACKED` → `DARK_ASH` → `DEAD_SEALED` (immutable, never erased).

---

## P

### Particle
**🔒 LAW.** The fundamental unit of data in BahyWay.Ecosystem: one
KAKI, one surrogate, zero or more EAV attributes, one orbital
degradation state.

### PAZUZU
**✅ VERIFIED, partially** — named for the Mesopotamian demon of
windstorms. Seven threat tests targeting CSR gaps (PAZUZU-01 through
PAZUZU-07). The 2026-07-07 review confirmed 5 of 7 claimed gaps
directly against real `con-engine` source (PAZUZU-01, 03, 04, 05
confirmed real gaps; 02 correctly scoped as a deployment gap not a
code defect; 06/07 not verifiable from material available at the
time).

### PII Vault
**✅ VERIFIED** — `pii-vault/src/lib.rs:88`, `pub struct PiiVault`, with
real `encrypt()`/`decrypt()`/`erase()` methods keyed by `master_key`
and `kaki_hash`. DataCleansing Station component.

---

## Q

### QueryPlan
**✅ VERIFIED** — `heptascript/src/engine.rs:61`,
`pub struct QueryPlan`. HeptaScript engine's compiled execution plan.

---

## R

### RoutingTarget
**✅ VERIFIED** — `heptascript/src/sumuukin.rs:76`,
`pub struct RoutingTarget`. ŠUMU-UKIN target record.

---

## S

### Sargon Gate (CSR-01)
**✅ VERIFIED**, real and coded. First CSR rule. Rejects any operation
where `passport_valid == false`. `SargonKdf`/`SargonPassport` also
confirmed real in `kupru` this session.

### SessionEntry / SessionRegistry
**✅ VERIFIED** — `enkidb-session-registry/src/*.rs`, `SessionEntry`
(line 26) and `SessionRegistry` (line 61) both confirmed. Loaded from
`enkidb-sessions.toml` via a native TOML parser (no serde).

### SovereignRole
**✅ VERIFIED** — `enkidb-con-engine/src/roles.rs:5`,
`enum SovereignRole`. `Client(0)` READ only, `DataSteward(1)` READ
only, `TabletWriter(2)` READ+WRITE, `DubSar(3)` READ+WRITE+cross-tribe
exempt.

### Storage Sector (`storage-sector`)
**✅ VERIFIED**, real crate. The hardware-isolated, terminal jail for
particles BlackBox Station's scan has confirmed as harmful. Sealing is
a one-way door — the crate exposes no method to remove, read back, or
requeue a sealed particle.

### StreamStats
**✅ VERIFIED** — `heptascript/src/engine.rs:127`. Result of
`execute_stream`.

### ŠUMU-UKIN (𒀭𒌓𒆳𒁺)
**✅ VERIFIED**, real and built on this session. Babylonian governor
under Assyrian sovereignty. The BahyWay routing layer
(`heptascript::sumuukin`) that resolves a query's tribe filter to
session targets and fans the query out. `SumuUkinContext`,
`FanOutResult`, real `route()` method with genuine
`std::net::TcpStream` I/O (rewritten from an always-succeeds stub this
session).

---

## T

### TabletWriter
**✅ VERIFIED** — `SovereignRole` with WRITE access, no cross-tribe
operations.

### Triple-O (Orbit-Oriented Ontology)
**🔒 LAW**, sealed 2026-07-07 as `PH-001` — the philosophical
foundation everything else in the ecosystem sits on: every entity is a
Particle, every change is an orbital event, all history is preserved
forever.

### Tribe
**🧩 PARTIAL.** A logical partition of particles sharing governance,
identified by `tribe_id`. No single dedicated `struct Tribe` was found
this pass — `tribe_id` is used pervasively as a field (u16 in KAKI
bytes, u32 in some session/registry contexts — this width
inconsistency is itself worth a future check, not resolved here).
Cross-tribe access requires DubSar role (CSR-07).

---

## U

### UTNAPISHTIM
**⚠ COLLISION, found this pass.** This term describes two different
things:
1. **📄 DOCUMENTED, not built:** "App registration layer... add
   ConEngine gate to every new app registered through UTNAPISHTIM
   (PAZUZU-07 gap closure)."
2. **✅ VERIFIED, real, different function:** `crates/utnapishtim`
   exists with description *"Sovereign Application Factory: generates
   a Godot PDM IDE + Three.js web viewer + manifest.akk per client
   topology"* — nothing to do with app registration or a ConEngine
   gate.

Same pattern as the two unrelated WPDEngine crates found in the
2026-07-07 review. Not resolved here — flagging so a future reader
doesn't assume they're the same thing.

---

## V

### val_fingerprint
**✅ VERIFIED** — `enkidb-indexes/src/eav_exact_index.rs`. 32-bit
FNV-1a hash of the raw encoded attribute value bytes.

---

## W

### W5H2
**✅ VERIFIED** — see HeptaScript v2.0 W5H2 (§H).

---

## X

### Xor8 (fingerprint bloom substitute)
**✅ VERIFIED** — `enkidb-indexes/src/eav_exact_index.rs`. One byte per
slot in a power-of-2 table. Not a true Xor filter — bit-OR insertion,
bit-AND query, conservative (never false-negative).

---

## Z

### ZIP Bomb
**✅ VERIFIED.** Defended by `MAX_RECURSION = 4` (confirmed, §M) plus a
64 MB `expected_len` cap in the DEFLATE inflate function (📄 not
re-traced this pass, see `inflate` §I) and ISIZE validation in the
gzip unwrapper (📄 not re-traced this pass).

### ZIP (PKWARE)
**✅ VERIFIED**, magic bytes confirmed (§A, ArchiveFormat). Methods 0
(STORE) and 8 (DEFLATE, RFC 1951). CRC-32 verified on every extracted
entry.
