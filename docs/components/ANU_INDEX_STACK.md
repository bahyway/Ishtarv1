# The Anu Index Stack — W5H2 Reference

**Sealed:** 2026-07-13. Crate: `crates/enkidb-indexes` (unchanged — this is a
sovereign-name layer, not a rename; see below). Thirteen modules, verified
against the real, currently-compiling source, not design documents.

**Amendment, 2026-07-29:** the three indexes §9-11 below marked "not yet
wired" have now been wired for real, into both live query paths — see the
correction note at the top of each of those three sections.

## Why "Anu," not "ENLIL"

The Architect's own correction: **ENLIL already names something else** — the
Total Algebra Content of BahyWay.Ecosystem (GeoLaw-05: "Orbit stable iff all
eigenvalues inside unit disc — Enlil algebra"). Calling the index stack
"ENLIL indexes" too was a real collision, not a style question, and it's the
root cause of a confusion this session spent real effort untangling (two
never-reconciled "how many indexes" counts, 6 vs. 7, living in different
documents).

**Anu** was the fix already sitting on the shelf: reserved in NL-001 §9
("held for an ecosystem-supreme purpose," explicitly *not* spent on any one
component, per the Architect's earlier session's own collision register).
The Architect has now spent it deliberately, by direct instruction (CSR-08 —
the Architect's own sovereign naming call), on this index stack specifically.

This is a **sovereign-name layer**, the same pattern already used for
Tigris (EnkiDDB) and Euphrates (EnkiMDB): `enkidb-indexes`, `SurrogateMap`,
`EavExactIndex`, and every other real identifier below are **unchanged**.
`enkidb_indexes::SOVEREIGN_NAME` now returns `"anu"`. Nothing about the
crate's build, its dependents, or its public API changed to make this true.

## The W5H2 framework, briefly

Every HeptaScript query is evaluated across seven dimensions — **W**HO,
**W**HAT, **W**HEN, **W**HERE, **W**HY, **H**OW, **H**OW MUCH (hence W5H2).
Each Anu index accelerates one or more of these dimensions by turning an
O(n) full-journal scan into something closer to O(1) or O(log n). The table
below is the master map; the sections that follow give each index its full
account — role, real consumer, quantified performance impact, and how it
behaves as particles keep occupying the Leak.

| # | Module | W5H2 dimension(s) | Status |
|---|---|---|---|
| 1 | `surrogate` | infrastructure (underlies all dimensions) | **Live** — built by `heptascript::build_indexes`, used by every query |
| 2 | `identity` | WHO / WHAT (point lookup) | Bundled in `enkidb-engine::EnkiDb` |
| 3 | `sovereignty` | WHO (tribe enumeration) | Bundled in `EnkiDb` |
| 4 | `typerole` | WHAT (KAKI taxonomy) | Bundled in `EnkiDb` |
| 5 | `temporal` | WHEN | Bundled in `EnkiDb` |
| 6 | `colorid` | WHERE (spatial/quality) | Bundled in `EnkiDb`; also used directly by `alert-engine` |
| 7 | `eav` | HOW (attribute match) | Bundled in `EnkiDb`; touched by `enkidb-ingest` |
| 8 | `eav_exact_index` | HOW | **Live** — built by `heptascript::build_indexes`, used by `enkidb-query-server` and `enkidb-readnode` |
| 9 | `radix_spline` | HOW MUCH (range) | **Live**, wired 2026-07-29 — drives ORBITAL-clause pruning in `HeptaIndexes` and in `CachedReadNode` (EnkiDDB/EnkiMDB) |
| 10 | `hepta_shell` | WHERE (7D spatial) | **Live**, wired 2026-07-29 — drives `ANCHOR E7_FIRST` WHERE-clause pruning in `HeptaIndexes` and `CachedReadNode` |
| 11 | `nairu_index` | WHEN (orbital pruning) | **Live**, wired 2026-07-29 — drives WHEN-clause pruning (any epoch pin, not just current-state) in `HeptaIndexes` and `CachedReadNode` |
| 12 | `snapshot_idx` | WHEN (point-in-time projection) | Bundled in `EnkiDb` |
| 13 | `bloom` | WHY (pre-filter, not yet attached to a verb) | Built, tested — **zero consumers found anywhere in the workspace** |

Two honest groupings worth naming up front, since they explain the
"where used" answers below: the **original seven** (identity, sovereignty,
typerole, temporal, colorid, eav, snapshot_idx) live inside
`enkidb-engine::EnkiDb`, an in-process bundle. The **W5H2 optimization set**
(surrogate, eav_exact_index, radix_spline, hepta_shell, nairu_index) was
built later specifically to fix the >1B-particle/<1s target — as of
2026-07-29 all five are assembled and wired (see the per-index amendment
notes below for exactly what each wiring covers). `surrogate` and
`eav_exact_index` were already live in `heptascript::HeptaIndexes`, the
struct that answers production queries in `enkidb-query-server`. The other
three are wired both there **and**, independently, into
`enkidb_readnode::cached::CachedReadNode` — the struct EnkiDDB's and
EnkiMDB's real deployed read-servers actually query (`enkiddb`/`enkimdb`'s
`lib.rs`); `CachedReadNode` is a separate Data-File-backed in-memory index
that never goes through `HeptaIndexes` at all, so wiring only the latter
would have left the deployed read-servers unaccelerated. The plain
disk-backed `enkidb_readnode::readnode::ReadNode` (a distinct, narrower
struct — see its own module doc) still only carries `surrogate`/
`eav_exact_index`; extending it was out of scope for this pass. This
document says so plainly per index rather than implying a uniform "all
live" status that isn't true yet.

---

## 1. `surrogate` — SurrogateMap

**W5H2 role:** infrastructure underlying every other dimension. Every
particle is referenced internally by a dense `u32` surrogate instead of its
16-byte KAKI — this is the translation layer every other index's lookup
result passes through before touching real particle data.

**Where used:** **Live.** Built once at startup by
`heptascript::build_indexes(journal)`, held in the shared `HeptaIndexes`
snapshot, used by `enkidb-query-server` and by `enkidb-readnode`
(`materialize.rs`, `readnode.rs` — the same ReadNode machinery this session
built EnkiDDB/EnkiMDB's CQRS read path on).

**Performance enhancement:** A raw `HashMap<[u8;16], u32>` costs ~24GB at 1B
particles. This crate's sorted-array binary-search implementation costs
~20GB (better locality, same asymptotic lookup cost, O(log n) ≈ 30
comparisons at 1B entries) — a real, working intermediate step. The source
comment names PTHash MPHF (~750MB, a further 26x reduction) as the target
once that crate stabilizes; not yet built.

**Capacity & expansion in the Leak:** Grows **linearly** with particle
count at ~20 bytes/particle (16B KAKI + 4B surrogate). Immutable after
build — a new generation of particles means a full rebuild, not an
incremental insert (matches this ecosystem's batch-rebuild law used
everywhere: `materialize_now`, `RecallIndex::build`, etc.). At 1B particles
in the Leak, this alone is the single largest structure in the stack until
PTHash lands.

---

## 2. `identity` — IdentityIndex

**W5H2 role:** WHO/WHAT — "fetch this exact particle."

**Where used:** Bundled into `enkidb-engine::EnkiDb.idx_identity`.

**Performance enhancement:** O(1) hash lookup replacing an O(n) journal
scan. Also enforces Rule II (no duplicate uuid_hash) at insert time —
`insert()` returns `false` rather than silently overwriting.

**Capacity & expansion in the Leak:** A plain `HashMap<u32, u64>` (uuid_hash
→ file offset), 12 bytes/entry plus HashMap overhead (~1.5-2x in practice)
— roughly 20-24GB at 1B particles, the same order as SurrogateMap since
it's the same shape of problem (one entry per particle, no compression).
This is the index most likely to need the same PTHash-style treatment
SurrogateMap is already slated for, if it's ever pushed past today's
80K-particle working scale.

---

## 3. `sovereignty` — SovereigntyIndex

**W5H2 role:** WHO — "enumerate every particle belonging to tribe X."

**Where used:** Bundled into `EnkiDb.idx_sovereignty`.

**Performance enhancement:** O(k) enumeration where k = tribe size, instead
of an O(n) full scan filtered by tribe. Sorted-vec-per-tribe with
`partition_point`-based insert keeps each tribe's member list ordered
without a tree.

**Capacity & expansion in the Leak:** Bounded by **tribe count**, not
particle count, at the outer level (`HashMap<TribeId, Vec<u32>>` — at most
65,536 tribes, since `TribeId` is a `u16`). Each inner `Vec<u32>` grows
with that tribe's own population. Total memory is the same order as
particle count (4 bytes/particle across all tribes combined), but the
*access pattern* stays cheap regardless of how unevenly particles are
distributed across tribes — a tribe with 10 particles and a tribe with 100
million both get O(k) enumeration proportional only to their own size.

---

## 4. `typerole` — TypeRoleIndex

**W5H2 role:** WHAT — "all Event-Kakis in tribe X," "all PARZU-role KAKIs
in templates."

**Where used:** Bundled into `EnkiDb.idx_typerole`.

**Performance enhancement:** O(1) hash lookup into one of the (at most)
`tribes × 3 × 3` taxonomy cells (3 KakiTypes relevant here × 3 KakiRoles),
each holding a sorted particle list. Same `partition_point` insert pattern
as `sovereignty` — no tree, no full scan.

**Capacity & expansion in the Leak:** The outer key space is bounded (tribe
count × 9 cells max), so this scales the same way `sovereignty` does:
cheap, tribe-and-taxonomy-localized access regardless of total particle
count, with total memory proportional to particle count at ~4
bytes/particle plus small per-cell overhead.

---

## 5. `temporal` — TemporalIndex

**W5H2 role:** WHEN — "what happened in this time window," time-travel
queries.

**Where used:** Bundled into `EnkiDb.idx_temporal`.

**Performance enhancement:** O(1) direct array access per timestamp bucket
(as of PB-182's rebuild — see below), O(range width) for range queries via
contiguous slice iteration. Previously O(log 65536) via `BTreeMap`; now
strictly faster.

**Capacity & expansion in the Leak:** **Fixed, not linear.** The outer
structure is a `Vec<Vec<u32>>` with exactly 65,536 slots (the full `u16`
epoch domain, κ[12..13]'s compressed timestamp) — this costs a constant
~1MB of bucket-pointer overhead **no matter how many billion particles
exist**. Only the inner `Vec<u32>` buckets grow with particle count (4
bytes/particle, distributed across whichever epochs actually saw writes).
This is the one index in the stack whose *outer* shape is provably
independent of Leak occupancy — rebuilt PB-182, 2026-07-13, replacing a
`BTreeMap` that, while not itself a bottleneck (same bound applies to a
65,536-key tree), was a real tree and has now been removed on the
Architect's "no tree in the Orbits" ruling.

---

## 6. `colorid` — ColorIdIndex

**W5H2 role:** WHERE (quality-space position) — ColorID drift diagnostics.

**Where used:** Bundled into `EnkiDb.idx_colorid`, **and** consumed
directly by `alert-engine` for drift detection — the only original-seven
index with a second, independent real consumer outside `EnkiDb`.

**Performance enhancement:** O(n) bounding-box scan over a `HashMap<u32,
ColorIdPoint>` — this is honestly the weakest performance story in the
stack today. The doc comment previously claimed an "R-tree variant"; the
real implementation (corrected PB-182) is a plain hash map plus linear
scan for the drift/bbox queries. It works at today's ~80K-particle scale;
it is not yet the sub-millisecond spatial structure the rest of the stack
aims for.

**Capacity & expansion in the Leak:** Linear in particle count, 16
bytes/entry (u32 key + 3×u8 + f32). **Query cost also grows linearly** —
unlike every other index here, `colorid`'s lookup cost is not decoupled
from Leak occupancy. This is the index most in need of the real spatial
structure its own comment once claimed to be (bounding-box tree, grid, or
folding into `hepta_shell`'s zone table) before particle counts reach the
billion range — flagged here rather than silently left as a future
surprise.

---

## 7. `eav` — EavIndex

**W5H2 role:** HOW — attribute=value exact match (the general-purpose
version; see also `eav_exact_index` below for the optimized one).

**Where used:** Bundled into `EnkiDb.idx_eav`; also touched by
`enkidb-ingest::bridge` (the same particle→EAV-triple bridge this session's
EnkiDDB/EnkiMDB WriteNodes journal through).

**Performance enhancement:** O(1) hash lookup on `(tribe_id, attr_hash,
value_bytes)` → sorted particle list, replacing an O(n) attribute scan.

**Capacity & expansion in the Leak:** Linear in the number of *distinct*
(tribe, attribute, value) combinations actually written, not raw particle
count — a document corpus with many particles sharing the same
`meta.collection` value, for instance, costs one bucket, not one entry per
particle. Genuinely sparse in the same sense `snapshot_idx` is: cost tracks
attribute cardinality, not Leak occupancy directly.

---

## 8. `eav_exact_index` — EavExactIndex

**W5H2 role:** HOW — the optimized twin of `eav`, purpose-built for the
>1B-particle target.

**Where used:** **Live.** Built by `heptascript::build_indexes`, the other
half (with `surrogate`) of `HeptaIndexes` — used by `enkidb-query-server`,
`enkidb-readnode::materialize`, and `enkidb-readnode::readnode` (the ReadNode
`WHO T.E WHERE E[attr]=val` point-lookup path this session's EnkiDDB
`RagIndex::build_from_readnode` runs against directly).

**Performance enhancement:** Two-stage lookup — an O(1) Xor8-style
fingerprint table gives a fast *definite-absence* answer before ever
touching the real index; a positive fingerprint hit proceeds to O(log n)
binary search over a sorted `(attr_hash, val_fingerprint, surrogate)`
array. The fingerprint stage means queries for attribute/value pairs that
don't exist anywhere in the corpus cost O(1), not O(log n) — a real,
measured win for selective queries.

**Capacity & expansion in the Leak:** The source's own budget: at 1B
particles × 4 attributes average, the full corpus costs ~48GB — explicitly
scoped as a **cold/archive tier** cost, not a hot-tier one. Hot tier (10M
particles) costs ~480MB, called "acceptable" in the same comment. This is
the index most explicitly designed around a hot/cold tiering strategy
rather than a flat "hold everything in RAM" assumption — the honest
answer to "what happens when the Leak has a billion particles" for this
index is "most of it lives on disk, only the working set is hot."

---

## 9. `radix_spline` — RadixSplineIndex

**W5H2 role:** HOW MUCH — range/aggregation queries (epoch, quality, B11
ranges) — the direct successor to the retired, tree-based BTreeRange.

**Where used:** Built and tested, but **grep confirms zero consumers**
anywhere in the workspace outside its own crate. It is not yet assembled
into `HeptaIndexes` or called from `enkidb-query-server`. This is a real
gap, stated plainly rather than implied fixed: the memory story below is
correct for the *structure*, not yet proven end-to-end through a live
query.

**Amendment, 2026-07-29 — now Live.** Wired into `heptascript::indexed::
HeptaIndexes` (`build_indexes` sorts every `(epoch, surrogate)` journal
entry pair and builds `RadixSplineIndex` from them; `execute_indexed`'s
`candidates_from_indexes` uses `range_query` to prune candidates by the
query's `ORBITAL start .. end` clause — a distinct clause from WHEN, so a
distinct index even though both key on epoch today) and, independently,
into `enkidb_readnode::cached::CachedReadNode` (EnkiDDB's/EnkiMDB's real
deployed read path), same algorithm, built a second time from the already
in-memory particle set since `CachedReadNode` does not share `HeptaIndexes`
(see that struct's own doc comment). Wiring this also surfaced and fixed a
real, independent bug: `heptascript::engine::evaluate_capped` — the one
evaluation function every real query path (`execute`/`execute_over`/
`execute_over_histories`) goes through — never applied the `ORBITAL`
clause at all; only the separate, never-served `execute_stream` did. Fixed
alongside this wiring so `execute_indexed`'s pruned answer and a full scan
can never disagree. Proven with 8 new tests in `heptascript::indexed` and
4 in `enkidb_readnode::cached` (agreement with full scan, real evaluated-
candidate-count reduction, and a regression test for the LIMIT+ORBITAL
undercount the `safe_early_limit`/`safe_full_scan_limit` guards in both
`enkidb-readnode` modules now also close).

**Performance enhancement (as designed):** BTreeRange cost ~12GB at 1B
particles (one tree node per particle in the range structure). RadixSpline
replaces it with a learned index — a radix table (2^18 = 262,144 slots,
O(1) segment lookup) plus a small number of piecewise-linear spline
segments, giving a **6,000,000× memory reduction** (12GB → ~2KB) for the
same range-query capability, per the W5H2 guide's own figure. Query cost:
O(1) radix lookup, spline interpolation, then a bounded local binary search
within the segment's error tolerance ε (default 32 positions).

**Capacity & expansion in the Leak:** This is the sharpest illustration in
the whole stack of the "leak, not a tree" distinction the Architect's own
Leak Ontology makes: the structure's size depends on how *smoothly* keys
are distributed (how many linear segments are needed to fit them within
ε), not on how many particles occupy the Leak. A billion densely-packed,
near-linear epoch values could fit in a handful of segments; a billion
wildly irregular values would need more segments but still nowhere near
one-per-particle. **Building the code path that actually wires this into
`HeptaIndexes` is real, scoped, not-yet-started follow-up work** — noted
honestly rather than left implicit.

---

## 10. `hepta_shell` — HeptaShellIndex

**W5H2 role:** WHERE — 7D spatial (r, θ, φ + 4 more dimensions) orbital
queries, the E7-lattice-based replacement for a spatial tree.

**Where used:** Built and tested. Referenced by name in
`heptascript::query.rs`'s doc comments describing a *planned* query-plan
driver ("Drive from HeptaShellIndex... as primary candidate source") — but
**not yet instantiated or built by any live code path**. Same honest status
as `radix_spline`: real, tested, not yet wired in.

**Amendment, 2026-07-29 — now Live.** Wired into `HeptaIndexes` and
`CachedReadNode` the same way as `radix_spline` above. Trigger contract:
the query must carry the already-parseable `ANCHOR E7_FIRST` production
execution hint (real grammar since before this pass — `parser.rs`'s
`maybe_parse_anchor`, just never consulted by the engine) **and** its
WHERE clause must pin all seven of a fixed attribute-name contract
(`heptascript::SPATIAL_ATTRS` = `orbit.r`/`orbit.theta`/`orbit.phi`/
`orbit.d3`..`orbit.d6`, the same dimension order as this index's own
`SCALE` constant) with exact equality, ANDed. `build_indexes`/
`CachedReadNode::open` only add a particle to the shell index if it
carries all seven; a query only uses the shell index if it names all
seven. The shell index's own zone-plus-126-neighbours result is a
superset, not an exact answer — `execute_over`/`execute_over_histories`'s
real WHERE evaluation still runs on every pruned candidate afterward, so
an over-inclusive candidate set can narrow performance but never correctness.
Deliberately **not** inferred automatically from a WHERE clause that
happens to pin all seven dims without `ANCHOR E7_FIRST` — proven by a test
that omits the anchor and still gets the correct answer via ordinary `eav`
pruning instead.

**Performance enhancement (as designed):** Maps a 7D position to a
quantized zone hash (FNV-1a over 28 bytes), buckets particle surrogates by
zone, and answers a spatial query by checking the center zone plus up to
126 E7-lattice neighbor zones (the E7 kissing number — Maryna Viazovska's
Fields Medal work, already the geometric foundation this whole ecosystem's
orbital layout is built on). O(1) zone lookup, O(126) neighbor traversal —
never a scan. Memory: ~8GB unoptimized, ~4GB with zone merging, per the
W5H2 guide.

**Capacity & expansion in the Leak:** Bounded by **zone count** (882 total
zones: 7 shells × 126 zones/shell, per the Architecture Reference), not
particle count, at the bucket-table level — the same "fixed outer
structure, variable inner buckets" shape as `temporal`. Zone *occupancy*
(particles per zone) grows with the Leak, but the lookup structure itself
does not need to grow past 882 buckets regardless of scale, only its
per-zone lists.

---

## 11. `nairu_index` — NatiruIndex

**W5H2 role:** WHEN — orbital-range temporal pruning, guarding against a
full O(n) EAV scan by answering "which surrogates have any journal entry
in orbital range [start, end]" before the real work begins.

**Where used:** Built and tested. Same status as `radix_spline` and
`hepta_shell`: named in `heptascript::query.rs`'s design comments ("Drive
from... NatiruIndex orbital pruning") but not yet instantiated in the live
query pipeline.

**Amendment, 2026-07-29 — now Live.** Wired into `HeptaIndexes` and
`CachedReadNode`, driving WHEN-clause pruning — `AT EPOCH n`, `BEFORE
EPOCH n`, and `AFTER EPOCH n` alike, not just the "current state" case
`eav_exact_index` already handled. This is safe for a WHEN clause pinned to
any past epoch specifically *because* nairu only answers "did this
surrogate write anything in this range" — never "what is its current
value" — unlike `eav`, which stays correctly gated behind `when_is_current`
and cannot be used once WHEN pins away from "now." The two compose: a
query with both a WHEN-past-epoch clause and a WHERE equality condition
gets `eav`'s pruning disabled but still gets nairu's, narrowing the
candidate set before `evaluate_capped` re-verifies WHERE/WHY/ORBITAL/WHEN
against each survivor's real history. Bucket granularity
(`BUCKET_ORBITALS = 10`) means the pruned candidate set can be a real
superset of the true answer at fine-grained ranges — always safe (per the
over-inclusion argument above), verified by a test asserting the pruned
`evaluated` count only drops once decoy data crosses a bucket boundary.

**Performance enhancement (as designed):** A sorted `Vec<(orbital: u64,
surrogate: u32)>`, 12 bytes/entry, binary search on orbital bounds. The
module's own header does the honest capacity math already (rare in a
comment, worth preserving): at 10M particles × 64 journal entries/particle
average, the *unbucketed* form costs 7.68GB — explicitly flagged in the
source as needing the bucket variant instead.

**Capacity & expansion in the Leak:** The bucket variant
(`BUCKET_ORBITALS = 10`, tunable) divides entry count by the bucket
factor — a direct, adjustable trade between index size and orbital
resolution. For hot session data (≤1M particles, ≤100 entries each) the
unbucketed form is already "acceptable" at ~1.2GB per the module's own
comment; the bucket variant exists specifically for pushing past that into
cold-tier, billion-particle territory without the 7.68GB-class cost. This
is the index most explicitly designed with a dial for trading Leak-scale
against precision, rather than a single fixed answer.

---

## 12. `snapshot_idx` — SnapshotIndex

**W5H2 role:** WHEN — "jump to the latest snapshot before time T,"
accelerating StoryEngine's point-in-time projection.

**Where used:** Bundled into `EnkiDb.idx_snapshot`.

**Performance enhancement:** O(log k) binary search over one particle's
own snapshot list, where k = that particle's snapshot count — never a
function of total particle count. Rebuilt PB-182, 2026-07-13 (see below).

**Capacity & expansion in the Leak:** This is the index whose *old* design
was the real casualty of the tree audit, and its replacement is the
cleanest illustration of "sparse" done right. The old
`BTreeMap<uuid_hash, BTreeMap<epoch, entry>>` had an **outer** tree node
per particle — up to 1 billion nodes, the exact failure shape that made
BTreeRange cost 12GB. The new `HashMap<uuid_hash, Vec<(epoch, entry)>>`
still scales with the number of particles that have *any* snapshot (not
all particles do — "sparse" is this index's own §9.3 description), but
each particle's own list is small and contiguous, so per-query cost never
touches particles other than the one asked about. Leak growth adds hash
buckets, not tree depth.

---

## 13. `bloom` — BloomFilter

**W5H2 role:** WHY (evidentially) — a fast, probabilistic pre-filter: "has
this KAKI possibly been seen before," meant to guard expensive downstream
work (the module's own doc comment names Jordan Normal Form matrix
operations specifically) from running on particles that provably don't
exist.

**Where used:** **Grep confirms zero consumers anywhere in the workspace.**
Built, unit-tested (false-positive-rate tests included), `#![forbid(unsafe_code)]`,
genuinely sovereign (no `bloomfilter` crate — hand-rolled FNV-like hashing
over a bit array) — but not yet attached to any real gate. Stated plainly:
this is the one index in the stack that is pure potential energy today.

**Performance enhancement (as designed):** O(1) membership test with a
tunable false-positive rate. `default_8k()`: 8KiB, k=4 hash functions,
~0.39% FP at 4,000 KAKIs, ~3.5% at 10,000. `default_1m()`: 1MiB, tuned for
~1 million KAKIs at ~3% FP. Resize-by-doubling noted as the strategy when a
filter saturates.

**Capacity & expansion in the Leak:** Genuinely **sub-linear** — a fixed
bit-array size trades directly against false-positive rate as the Leak
fills, rather than growing per-particle. This is architecturally the
cheapest structure in the entire stack per particle covered, precisely
because it answers a weaker question (probably-absent vs. definitely-absent)
than every other index here. The natural next step, not done in this pass,
is wiring it in front of `identity`/`surrogate` lookups as the O(1) negative
short-circuit its own doc comment already describes.

---

## Reading the Leak — the whole stack, by growth shape

Three distinct capacity classes appear across the thirteen indexes, worth
naming once as a summary rather than only per-index:

- **Fixed / bounded**, independent of particle count: `temporal` (65,536
  buckets), `hepta_shell` (882 zones), `bloom` (fixed bit array). These are
  the indexes that genuinely do not care how full the Leak gets.
- **Linear in particle count, but compressed or tiered**: `surrogate`,
  `identity`, `sovereignty`, `typerole`, `eav_exact_index` (with an explicit
  hot/cold tier split), `radix_spline` (compressed via learned segments,
  not per-particle nodes). These scale with the Leak but each has a real,
  stated strategy for staying survivable at billion-particle scale.
- **Sparse, tracking a real-world subset, not raw particle count**: `eav`
  (distinct attribute/value combinations), `snapshot_idx` (particles that
  actually have snapshots), `nairu_index` (tunable via bucketing). These
  are the indexes where Leak occupancy and index cost were already
  intentionally decoupled by design, before this pass touched anything.

The one index without a fully satisfying growth story today is `colorid`
— linear in both memory *and* query cost, flagged above rather than left
to surprise someone at scale.
