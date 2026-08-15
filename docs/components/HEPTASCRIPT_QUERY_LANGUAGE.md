# HeptaScript Query Language — Unified W5H2 Manual (v1.0 + v2.0 + v2.1)

**Crate:** `crates/heptascript` (6,443 lines across 10 modules). Every clause
below was verified against that real, currently-compiling source — its
AST (`query.rs`), its parser (`parser.rs`, whose own test suite is the
canonical syntax proof for every example here), and its two executors
(`engine.rs`'s `execute_over_histories` — what actually answers a query
today — and `engine.rs`'s `execute_stream` — a second, parallel executor
that almost nothing calls). This is the single place that collects every
clause fragment scattered across `enkiddb-node-write`/`enkiddb-node-read`
work, `enkidb-query-server`, and the Anu Index Stack documents into one
production-level reference.

**Status tags** follow this ecosystem's own `docs/TRANSPARENCY_STANDARD.md`
convention, used identically to `docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`:

| Tag | Meaning |
|---|---|
| ✅ VERIFIED | Parses **and** is evaluated by the executor that answers real queries today (`execute_over_histories`, used by `execute()`, `execute_indexed()`, and `enkidb-readnode::ReadNode::query()` — i.e. the exact path EnkiDDB's and EnkiMDB's real Read Node servers run). |
| 🧩 PARTIAL | Parses cleanly into the AST and is carried in `QueryPlan`, but nothing in this workspace consumes it yet — no dispatcher, no filter, no error. It is accepted syntax with no observable effect. |
| 📄 DOCUMENTED | Named in design comments / doc headers, no parser support at all. |

Do not read a 🧩 tag as "broken." It means: type this clause into a real
`QUERY:` string sent to `enkiddb-read-server`/`enkimdb-read-server` today,
and it will be silently accepted and silently ignored — no error, no
effect on the result set. That is a deliberate design choice (see
`indexed.rs`'s own doc comment: "correctness is never gambled for
speed" — the same discipline applies here: an unimplemented clause never
pretends to filter).

---

## 1. What HeptaScript is

HeptaScript is BahyWay.Ecosystem's query language for the EAV particle
space — Triple-O (Orbit-Oriented Ontology, 🔒 LAW, sealed `PH-001`): every
entity is a Particle, every change is an orbital event, history is never
overwritten. There is no SELECT/INSERT/UPDATE/DELETE. `crates/heptascript/
src/operations.rs` names the conceptual replacement, the **Five Sovereign
Operations** — ORBIT (read/scan), EMIT (birth), PROVE (point lookup),
SYNC (replication), WITNESS (consensus). **All five are now real,
parseable, executable HeptaScript verbs** — an optional keyword leading
the whole query (`ORBIT`/`EMIT`/`PROVE`/`SYNC`/`WITNESS`, defaulting to
`ORBIT` when omitted, so every query written before this existed still
parses and executes identically). §4 below is their full reference.
As realized in HeptaScript specifically, all five stay strictly
read-only — none bypasses the Write Node / Read Node split. Where a
verb's name plausibly suggests an external side effect (EMIT birthing a
particle, SYNC replicating, WITNESS attesting via a second party), it
means the *retrieval act* of that concept, or the stored particle
representing it, never a mutating or networked action taken by the query
itself — `operations::Operation`'s own `is_read_only()` predicate still
describes the fuller conceptual role these names could take on
elsewhere in the ecosystem (EMIT's real minting only ever happens in
`enkidb-ingest`; SYNC's real replication is the Ansible cross-host
pipeline from PB-213; WITNESS's fuller consensus role isn't built
anywhere yet), which is why that predicate and this section's ✅ status
for the query-verb realization aren't the same claim.

The name **W5H2** is the seven v1 core clauses: **W**HO, **W**HAT,
**W**HEN, **W**HERE, **W**HY, **H**OW, **H**OW MUCH. v2.0 adds ten more
routing/intelligence clauses (§5) plus five production-execution hints
(§6). v2.1 (2026-07-22) adds two Anti-SQL aggregate clauses, MEASURE and
GRAVITY (§3, after HOW MUCH) — twenty-four clauses in total, all in one
`HeptaQuery` AST (`crates/heptascript/src/query.rs`).

**The Anti-SQL Law**, as the Architect stated it directly: *"it will [be]
no use of SQL Methods in my HeptaScript Query Language or any other
Dependencies of third parties technologies."* MEASURE/GRAVITY are the
concrete answer to "how do I count/sum/average/group-by, then" —
COUNT/SUM/AVG/GROUP BY equivalents built from this ecosystem's own
`bahyway-algebra`/hash-index primitives, never SQL, never a third-party
aggregation engine. See the new subsection at the end of §3 for the full
design story, including a real bug this design deliberately avoids (a
naive wedge-product "count" that silently breaks past 7 particles).

## 2. Where a query actually runs

Three real, different places execute a `HeptaQuery`, and they don't all
support the same subset — this is the single most important thing to get
right before writing a query against real infrastructure:

| Runner | Function | WHERE support | v2.0 clauses |
|---|---|---|---|
| **EnkiDDB/EnkiMDB Read Node** (`enkiddb-read-server:7102`, `enkimdb-read-server:7202` — the real deployed servers) | `enkidb_readnode::ReadNode::query()` → `execute_over_histories` | **One leading exact-equality (`=`) condition only**, ANDed extras allowed but not used to prune; any `OR` or non-`=` leading condition returns `Err(RequiresWriteNode(...))` | Parsed into `QueryResult.plan`, then **discarded** — the read-server's `query_rows_to_json` only reads `.matched` |
| **Write Node / in-memory Journal** (`enkiddb-cli`, `WriteNode` before materialization) | `heptascript::execute()` / `execute_indexed()` → same `execute_over_histories` core | **Full grammar** — multiple conditions, `AND`/`OR`, all six operators, `EXISTS`/`NOT EXISTS` | Same: parsed, carried in `.plan`, not applied |
| **`enkidb-query-server`** (port 7001 — real code, but **not part of the currently deployed EnkiDDB/EnkiMDB fleet**; see `docs/EnkiDDB_MANUAL.md` §1) | `execute_indexed` | Same as Write Node | Reads exactly one field of `.plan` — `across`, only to append a `"[BIGRING]"` tag to its own stderr log line. No federation actually happens. |

Every runner shares the identical `execute_over_histories` core (`engine.rs`
line 288's doc comment: "there is exactly one evaluation implementation
regardless of where the history came from") — so **WHO/WHAT/WHERE/WHEN/
WHY/HOW/HOW_MUCH always mean the same thing everywhere**. What differs is
only (a) how much of WHERE's grammar a given runner's index can serve
without an error, and (b) whether anything downstream reads `.plan`.

## 3. The v1 core — WHO, WHAT, WHERE, WHEN, WHY, HOW, HOW MUCH

All seven: ✅ VERIFIED. This is the part of HeptaScript that is genuinely
production-level today — proven this session against real EnkiDDB data
(201 ingested documents, 8,456+ entities) and real EnkiMDB data (152
crates, 108 playbooks cataloged).

Grammar order is fixed by the parser (`parser.rs::parse()`): **WHO → WHAT
→ WHERE → WHEN → WHY**, then HOW/HOW_MUCH at the end. This matters more
than it looks — the parser has no "unexpected clause" check at the end of
input, so a clause in the wrong position is silently dropped, not
rejected. (Real bug found this session, PB-218: a WHERE-before-WHAT query
against the live Read Node returned rows with empty `attrs` — not because
projection was broken, but because `WHAT` had already returned `None` by
the time the parser reached it out of order, and the trailing `WHAT ...`
tokens were never re-parsed. There is no substitute for the canonical
order.)

### WHO — entity variable scope

**Purpose:** declare which entities the query ranges over, bound to a
name you'll reference in WHAT/WHERE/HOW.

**Syntax:** `WHO <Tribe>.<Var>` — e.g. `WHO T.E`, `WHO Citizens.E`.

**What "Tribe" and "Var" actually do today:** nothing, semantically.
`WhoClause.primary.{tribe,var}` are parsed and stored, but neither
`execute_over_histories` nor `project_what`/`eval_where` ever reads them
back — every `WHAT`/`WHERE`/`HOW` clause's own `var` field (e.g. the `E`
in `E[meta.title]`) is likewise never cross-checked against WHO's
declared var. In practice `T.E` is a documentation convention, not an
enforced binding — a query already only ever runs against one Journal
(or one Read Node's Data Files), which is implicitly "the tribe" already.
Any tribe/var name parses identically. Write `WHO T.E` by convention
(matches every example in this codebase's own test suite) unless you have
a specific documentation reason to name it otherwise.

**One Tribe** (the only form that changes real behavior — there is only
one, since Tribe/Var aren't enforced):
```
QUERY:WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
```

**"Multiple Tribes," BOUND_TO:**
```
WHO Tribes.E BOUND_TO Citizens.C
```
🧩 PARTIAL. `parse_who()` accepts any number of `BOUND_TO <Tribe>.<Var>`
bindings (`WhoClause.bound_to: Vec<EntityBinding>`) — but `grep` across
`engine.rs` and `indexed.rs` confirms **zero references to `bound_to`
anywhere in the executor**. This parses as a cross-tribe join; it
executes as if you'd written `WHO Tribes.E` alone. Do not rely on
`BOUND_TO` to actually join two tribes' particles yet — see §7 for the
three genuinely different "more than one tribe" mechanisms this ecosystem
has today, only one of which (ŠUMU-UKIN fan-out) is real.

**BIGRING:** WHO itself has no BIGRING-specific form — BIGRING federation
is the separate `ACROSS` clause (§5).

### WHAT — attribute projection

**Purpose:** choose which EAV attributes come back per matched entity.

**Syntax:** `WHAT <Var>[attr1, attr2, ...]` or `WHAT <Var>[*]` (all
attributes the entity actually has — `project_what` returns them keyed
by `#<hex attr_hash>` since the engine only ever stores hashes, never
attribute name strings, so `*` output is not human-labeled without a
separate reverse lookup).

Attribute names are dotted (`meta.title`, `link.target`, `artifact.kind`)
via `parse_dotted_name()` — any depth of dots works, the engine treats the
whole dotted string as one opaque attribute key.

**Missing attributes never error.** `project_what` silently omits any
requested attribute the matched entity doesn't have (`get_val` returns
`None` → skipped) — so one `WHAT` clause safely covers heterogeneous
entities. This is real and load-bearing: the Graph Explorer (PB-219)
requests `WHAT E[meta.title, meta.collection, link.target,
link.description]` in a single query and gets back whichever subset each
row actually carries — a plain document returns only `meta.title`/
`meta.collection`; a section or concept-mention also returns `link.*`.

```
QUERY:WHO T.E
WHAT E[meta.title, meta.collection]
WHERE E[meta.collection] = "component"
```

### WHERE — EAV conditions

**Purpose:** filter entities by attribute value.

**Syntax:** `WHERE <Var>[attr] <op> <value>`, chained with `AND`/`OR`
(also bare — a following `Ident` with no explicit `AND` implicitly ANDs,
per `maybe_parse_where`'s loop), or `<Var>[attr] EXISTS` / `<Var>[attr]
NOT EXISTS`.

Operators (`Op`): `=`, `!=`, `>`, `<`, `>=`, `<=`. Values (`HeptaValue`):
string (`"..."`), int, float, or the bare identifiers `true`/`false`/
`null`.

**The Read Node constraint — the single most important operational fact
in this manual:** `enkidb-readnode::indexable_condition()` requires the
**first** WHERE condition to be an exact-equality (`=`) test, and rejects
the whole query with `RequiresWriteNode(...)` if any later condition is
`OR`-combined. Extra `AND`-combined conditions after the first are parsed
and evaluated (the engine re-checks the *full* WHERE list against each
candidate's real EAV map, never trusting the index alone — the
first-condition index only narrows *candidates*), so this is safe:
```
QUERY:WHO T.E
WHAT E[meta.title]
WHERE E[link.description] = "section-of" AND E[meta.title] EXISTS
```
This is **not** safe against a Read Node — it will error, not silently
under-filter:
```
QUERY:WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component" OR E[meta.collection] = "roadmap"
```
Run OR queries (or a non-`=` leading condition, e.g. `WHERE
E[quality] > 0.5`) against the Write Node / `enkiddb-cli` path instead —
`execute()`/`execute_indexed()` there support the full grammar (see
`indexed.rs`'s own `or_condition_falls_back_to_full_scan` test).

Real worked examples against this session's actual EnkiDDB data:
```
QUERY:WHO T.E
WHAT E[meta.title, link.target, link.description]
WHERE E[link.description] = "section-of"
```
```
QUERY:WHO T.E
WHAT E[artifact.name, artifact.kind, artifact.version]
WHERE E[artifact.kind] = "crate"
```
(EnkiMDB's real schema is a flat `artifact.{name,kind,path,version}`
namespace — no `DocOrbit` sections, confirmed via PB-216's live
152-crate/108-playbook catalog run.)

### WHEN — temporal constraint

**Purpose:** pin the query to journal history rather than current state.

**Syntax:** `WHEN AT EPOCH <n | NOW>`, `WHEN BEFORE EPOCH <n>`, `WHEN
AFTER EPOCH <n>`.

`AT EPOCH NOW` is a no-op filter (`epoch_filter_from_when` maps it to
`None` — identical to omitting WHEN entirely). A **Read Node** query
pinned to a specific past epoch (`AT EPOCH <n>` where n isn't "now",
`BEFORE`, or `AFTER`) cannot use the exact-match index shortcut — the
Data Files snapshot only ever holds the *latest* value per attribute —
and falls back to a full scan over every candidate's reconstructed
history (still correct, just not index-accelerated; proven by
`indexed.rs::when_before_epoch_falls_back_and_stays_correct`).
```
QUERY:WHO T.E
WHAT E[status]
WHERE E[status] = "active"
WHEN AT EPOCH 7
```

### WHY — lane / quality / existence evidence

**Purpose:** a second filtering dimension, semantically distinct from
WHERE — sovereign quality/provenance evidence rather than raw EAV
content.

**Syntax:** `WHY LANE <op> <lane-value>` (lane values: `Gold`, `Silver`,
`White`, `Gray`, `Red`, `Black` — case-insensitive), `WHY QUALITY_BYTE
<op> <0-255>`, or `WHY <Var>[attr] EXISTS` / `NOT EXISTS`. Chainable with
`AND`/`OR` exactly like WHERE.
```
QUERY:WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
WHY LANE != Black
```
```
QUERY:WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
WHY QUALITY_BYTE > 150
```
Note WHY's own `<Var>[attr] EXISTS` form duplicates WHERE's — both are
real, both evaluated the same way over the same EAV map; use whichever
reads more truthfully for the condition (existence-of-provenance belongs
conceptually to WHY; existence-of-content belongs to WHERE).

### HOW — result ordering

**Syntax:** `HOW BY <Var>[attr] [ASC | DESC]` (default `ASC` if omitted).
Sorts the already-filtered result set; `sort_by_attr` treats a missing
attribute as sorting last regardless of direction (`cmp_akk`'s `None`
handling).
```
HOW BY E[meta.title] ASC
```

### HOW MUCH — cardinality

**Syntax:** `HOW_MUCH LIMIT <n>` or `HOW_MUCH TOP <n> BY <Var>[attr]
[ASC|DESC]`. `LIMIT` truncates after HOW's sort (if any); `TOP` sorts by
its own attribute regardless of a separate HOW clause, then truncates —
the two are independent knobs, not aliases.

`execute_over_histories` applies an **early-exit** optimization keyed off
`HOW_MUCH`: it stops evaluating candidates once `LIMIT`/`TOP`'s `n` is
reached, rather than filtering the entire candidate set first (the
comment at `engine.rs:295` is explicit: "without this, LIMIT 100 still
scans all 80,272 entries"). This means `LIMIT` genuinely bounds work, not
just output size — put a `LIMIT` on any exploratory query against a large
corpus.
```
HOW_MUCH LIMIT 25
```
```
HOW_MUCH TOP 5 BY E[artifact.version] DESC
```

### MEASURE / GRAVITY — v2.1 Anti-SQL aggregate clauses

Both: ✅ VERIFIED — added 2026-07-22, after evaluating (and correcting)
an uploaded proposal document for COUNT/SUM/AVG/GROUP BY equivalents.
Parsed after HOW_MUCH (`parser.rs::parse()`'s clause order: `...how_much?
gravity? measure?`), executed inside the same `execute_over_histories`
core §2's table already covers — so every real runner inherits them with
no per-runner code, same as §4's verbs. `Operation` (`operations.rs`)
stays a closed set of exactly five — MEASURE/GRAVITY are ordinary
clauses that combine with any of the five sovereign verbs (almost always
the default ORBIT), never new verbs.

**Why not the obvious shortcut:** an earlier proposal computed COUNT as
a Clifford-algebra wedge-product magnitude. `bahyway-algebra`'s own test
`wedge_of_dependent_is_zero` proves wedging more vectors than the
algebra's dimension (`Cl(7)`, 7 basis vectors) collapses to zero — so
that approach silently returns 0 for any real Tribe beyond 7 particles.
MEASURE DENSE is instead a real, single-pass O(N) tally; FLUX/ROTOR_MEAN
are likewise real one-pass numeric folds, not disguised algebra.

**Correctness rule, load-bearing:** an aggregate must reflect *every*
particle that matched WHERE/WHY, never just the first few. Two separate
early-exit optimizations elsewhere in this codebase (`execute_over_
histories`'s own `HOW_MUCH LIMIT` short-circuit, and `enkidb-readnode`'s
indexed/full-scan fast paths) are explicitly disabled whenever a query
carries MEASURE or GRAVITY — confirmed by dedicated regression tests
(`engine::tests::measure_dense_reflects_full_matched_set_even_under_how_much_limit`,
`cached::tests::cached_measure_dense_ignores_how_much_limit_on_the_indexed_fast_path`,
`readnode::tests::measure_dense_ignores_how_much_limit`). `HOW_MUCH`
still bounds the *rows returned*; it never bounds the aggregate.

#### MEASURE — one aggregate value per group (or the whole matched set)

**Syntax:** `MEASURE DENSE`, `MEASURE FLUX <Var>[attr]`, or `MEASURE
ROTOR_MEAN <Var>[attr]`.

| Form | Meaning | Real computation |
|---|---|---|
| `DENSE` | COUNT equivalent | `acc.count += 1` per matched particle — real O(N) tally, correct at any N |
| `FLUX <Var>[attr]` | SUM equivalent | Component-wise `Multivector::add` fold degenerated to ordinary addition — the EAV store only ever holds scalar `AkkValue`s, so this is exactly SUM for the values this schema can hold |
| `ROTOR_MEAN <Var>[attr]` | AVG equivalent, for angle-valued attributes | Closed-form circular mean `atan2(Σsin θ, Σcos θ)`, one pass — correct specifically because this codebase's `Rotor` (`bahyway-algebra::rotor`) is a single bivector-plane rotor, not a general multi-plane one that would need an iterative Fréchet-mean instead |

Circuit structure (whole-set aggregate, no GRAVITY partition):
```
QUERY:WHO T.E
WHERE E[event.type] = "entry"
MEASURE DENSE
```
```
QUERY:WHO T.E
WHERE E[station.id] = "S-14"
MEASURE FLUX E[pressure]
```
Live-verified: DENSE stays exact past the algebra's own N=7 danger zone
(`engine::tests::measure_dense_counts_correctly_past_the_algebra_dimension`,
10 particles), FLUX sums correctly
(`engine::tests::measure_flux_sums_the_matched_attribute`), and
ROTOR_MEAN is proven to be a *real* circular mean — not a disguised
arithmetic mean — via two orientations just past ±π (geometrically
almost identical, but whose arithmetic mean would wrongly land near 0)
(`engine::tests::measure_rotor_mean_is_a_true_circular_mean_not_an_arithmetic_one`).

#### GRAVITY — partition the matched set into groups

**Syntax:** `GRAVITY <Var>[attr] BAND <width>` or `GRAVITY <Var>[attr]
MAX_GROUPS <n>`. **One of BAND or MAX_GROUPS is mandatory** — the parser
(`maybe_parse_gravity`) refuses to parse GRAVITY without one, and BAND
additionally refuses a non-positive width. This is the concrete,
enforced answer to GROUP BY's real unbounded-memory risk: grouping by a
near-unique attribute can otherwise grow the grouping structure toward
O(N) memory, the same property any database's GROUP BY has.

| Mode | Group key space | Safety property |
|---|---|---|
| `BAND <width>` | Numeric bucket `floor(value / width)` (`GravityKey::Band`) | Bounded by `(attribute range / width)`, independent of value cardinality — safe at any N with no cap needed |
| `MAX_GROUPS <n>` | Exact attribute value (`GravityKey::Exact`) | Hard-capped at runtime: once `n` distinct groups exist, every further new value folds into a single `GravityKey::Overflow` sentinel bucket instead of growing memory further — `GravityResult.capped` reports whether this happened |

Every matched particle lands in exactly one group — including
`GravityKey::Missing` for particles that lack the GRAVITY attribute
entirely — so group counts always sum to the true total matched, never
silently drop particles. Gate G4 (`bahyway-z3`, see below) additionally
flags a `MAX_GROUPS` value so large it stops functioning as a meaningful
cap in practice (`MAX_REASONABLE_GROUPS = 100,000`), independent of
whether the query's WHERE/WHY is itself satisfiable.

GRAVITY combines naturally with MEASURE — this is the real "COUNT/SUM
per GROUP BY key" shape:
```
QUERY:WHO T.E
WHERE E[event.type] = "entry"
GRAVITY E[station.id] MAX_GROUPS 1000
MEASURE DENSE
```
```
QUERY:WHO T.E
WHERE E[sensor.kind] = "pressure"
GRAVITY E[reading] BAND 5.0
MEASURE FLUX E[reading]
```
GRAVITY alone (no MEASURE) still partitions and reports a plain count per
group — MEASURE simply defaults to DENSE when absent. Live-verified:
BAND partitions into the correct numeric buckets
(`engine::tests::gravity_band_partitions_into_numeric_buckets`),
MAX_GROUPS genuinely caps and reports `capped = true` once the cap is
hit (`engine::tests::gravity_max_groups_caps_and_reports_capped_true`),
and particles missing the GRAVITY attribute are counted, not dropped
(`engine::tests::gravity_group_missing_attribute_particles_are_not_silently_dropped`).

**Design law behind both clauses**, stated by the Architect and confirmed
against every real risk found while designing this feature (a naive
wedge-as-COUNT, an unwired-but-linear-scan `pauli_check`, an uncapped
GRAVITY, a hypothetical naive Tribe-vs-Tribe dedup station): *the index
determines which particles a query ever touches; the algebra determines
what gets computed once that set is already bounded.* GRAVITY's
MAX_GROUPS cap and BAND's inherent range-bound are what keep this feature
an index-shaped O(N) operation rather than an algebra-shaped O(N²) one.

**Gate G4 extension** (`bahyway-z3::compile::check_query`, dev-time-only
— see `bahyway-z3`'s own crate doc for why it never ships in a sovereign
server binary): `ValidationReport.gravity_warnings` flags a `MAX_GROUPS`
value larger than `MAX_REASONABLE_GROUPS`, independent of the query's
SAT/UNSAT outcome (`bahyway_z3::compile::tests::gravity_max_groups_past_the_practical_ceiling_is_flagged`).

**Wire protocol:** the seven `*-read-server` binaries append a new
aggregate trailer to the existing binary response frame, after the
verb trailer (sync_fingerprint/witness_digest) since MEASURE/GRAVITY can
co-occur with any of the five sovereign verbs: `[u8 aggregate_tag]`
0=none 1=measured 2=grouped, each carrying the `MeasureValue`/
`GravityResult` payload — see any read-server's own module doc comment
(e.g. `bin/enkidb-read-server/src/main.rs`) for the exact byte layout.

**CompareEngine, a related but separate station:** the same O(N×M)
naive-comparison risk GRAVITY's cap guards against also applies to
Tribe-vs-Tribe Pauli-duplicate detection in the BeeMDM ETL pipeline's
CompareEngine — `compare-tribe-schema::pauli_dedup::dedup_tribes` builds
a `HashMap` index over the previous Tribe snapshot once (O(N)), then
probes it once per current-Tribe particle (O(M)): O(N+M), never O(N×M).
Not a HeptaScript clause — a Rust-level ETL station — documented here
because it shares the exact same design law as GRAVITY's cap. See
`docs/components/BEEMDM_ETL_PIPELINE.md`.

## 4. The Sovereign Operation verb — ORBIT, EMIT, PROVE, SYNC, WITNESS

All five: ✅ VERIFIED — added and live-verified this pass (PB-220,
2026-07-20), after the Architect's explicit request to make
`operations.rs`'s Five Sovereign Operations real, parseable clauses
rather than leave four of them as conceptual vocabulary. Reuses
`operations::Operation` as `HeptaQuery.verb`'s type — one definition,
not a second parallel enum — tokenized (`token.rs`'s `Token::VerbOrbit`
.. `VerbWitness`), parsed as an optional keyword leading the entire
query (before even `NODE`/`ACROSS`), and executed inside the same
`execute_over_histories` core every other clause in this manual shares
— so every real runner (§2's table) inherits verb support automatically,
with no per-runner code.

**Syntax:** one of `ORBIT`, `EMIT`, `PROVE`, `SYNC`, `WITNESS`, as the
very first token of the query, or omitted entirely — omitted means
`ORBIT`, byte-for-byte identical to every query written before this verb
syntax existed. `WHO T.E WHAT ...` and `ORBIT WHO T.E WHAT ...` produce
identical results (proven by
`engine::tests::orbit_is_the_default_verb_and_an_explicit_keyword_is_identical`).

**Design constraint that shaped all four non-ORBIT verbs:** the Architect
was explicit that where a verb's name suggests an executable action, it
should mean "the retrieval act of that executable, or the executable
itself, in databases" — never a literal side effect. All five are
therefore strictly read-only: none mutates the journal, none bypasses
the Write Node / Read Node split, and none requires anything beyond what
a Read Node already has on disk.

### ORBIT — read/scan (default)

The plain W5H2 read documented in §3, unchanged. Writing `ORBIT`
explicitly is purely stylistic.
```
QUERY:ORBIT
WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
```

### EMIT — birth retrieval

**Meaning:** "show me what was emitted (born) matching this filter."
Real minting still only ever happens in `enkidb-ingest::bridge` — EMIT
never births a particle itself. Instead it ANDs an implicit
`hist.event = "BIRTH"` condition onto WHERE before evaluating. Every
document/section/concept-mention particle this ecosystem's
`DocumentEmitter` already mints carries `hist.event = "BIRTH"`
(`crates/enkiddb/src/emitter.rs`), so EMIT reuses real, already-written
data — no new particle kind, no schema change.

The implicit condition is applied inside `execute_over_histories`
itself, after a Read Node's own index-based candidate pruning (which
still runs against the query's own first WHERE condition, untouched) —
so it composes safely with the Read Node constraint from §3/WHERE: it
only ever narrows the candidate set further, never requires a second
indexable condition.
```
QUERY:EMIT
WHO T.E
WHAT E[meta.title, hist.event]
WHERE E[meta.collection] = "component"
```
Live-verified against a real local write/read server pair: an entity
carrying `hist.event = "BIRTH"` matches; an otherwise-identical entity
without it does not (`engine::tests::emit_only_returns_birth_event_particles`,
`emit_combines_with_an_explicit_where_condition`).

### PROVE — full per-epoch history

**Meaning:** "verify this particle's history" — the same WHO/WHERE/WHY
candidate resolution as ORBIT, but each matched entity's WHAT projection
is reported **once per journal entry in its real history**, oldest to
newest, not only its current last-write-wins state. The current-state
`projected` field is completely unaffected — PROVE is additive, reported
in a new `history: [{"epoch": n, "attrs": [...]}, ...]` array alongside
it, present in the wire JSON only when non-empty (so ORBIT/EMIT rows'
shape literally does not change).

Shares the exact same last-write-wins accumulation rule ORBIT's
`projected` uses (`apply_entry_to_map`, factored out of `build_eav_map`
specifically so the two can never disagree) — applied incrementally
after each entry instead of once at the end.
```
QUERY:PROVE
WHO T.E
WHAT E[status]
WHERE E[status] EXISTS
```
Wire response shape:
```json
[{"kaki":"...","attrs":[["status","archived"]],
  "history":[{"epoch":1,"attrs":[["status","active"]]},
             {"epoch":5,"attrs":[["status","archived"]]}]}]
```
Live-verified end to end (a real single-epoch entity's `history` comes
back as a one-element array matching `projected` exactly); the
multi-epoch case is unit-tested
(`engine::tests::prove_reports_full_per_epoch_history_alongside_current_state`)
since the plain document-ingest wire protocol has no way to add a second
epoch to an existing entity to demonstrate live over TCP.

### SYNC — order-independent state fingerprint

**Meaning:** "compare our orbit states." Same result set as ORBIT, plus
a CRC32 fingerprint (`bahyway_crc::crc32`, already an ecosystem-standard
checksum — no new dependency) XOR-folded over every matched entity's
KAKI bytes. XOR makes the fold order-independent: two SYNC calls whose
matched *sets* agree produce the identical fingerprint regardless of
scan order — e.g. one call against a Write Node's Journal and one
against a Read Node's materialized generation, to detect replication
drift without diffing full row content.

Because the response shape needs a place to carry the fingerprint
alongside the rows, SYNC (and WITNESS below) are the one place this
manual's wire-protocol promise changes: instead of ORBIT/EMIT/PROVE's
bare `[...]` array, the response is an object:
```
QUERY:SYNC
WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
```
```json
{"rows":[{"kaki":"...","attrs":[["meta.title","..."]]}],
 "state_fingerprint":"0a2cd49f"}
```
This is a real, deliberate shape change — but a safe one: no existing
client (the HeptaScript Editor, Grid & Orbit, Graph Explorer) can send
`SYNC` today, since the verb didn't parse before this pass, so nothing
that worked yesterday is affected. Live-verified: two identical SYNC
calls against the same real Read Node agree exactly; adding a matching
particle between calls changes the fingerprint
(`engine::tests::sync_fingerprint_is_present_and_order_independent`,
`sync_fingerprint_changes_when_the_matched_set_changes`).

### WITNESS — content digest

**Meaning:** "I attest to this state." Same result set as ORBIT, plus a
real SHA3-256 digest (`sha3` crate, the same one `enkidb-replication::
event` already uses for its own frame digests, domain-separated the same
way — `b"HEPT_WITNESS"` here) over every matched entity's KAKI *and* its
full projected content. Any later re-run reproducing the identical
digest has proven byte-identical results; a content change (even to the
same entity, matched set unchanged) moves the digest, which is what
distinguishes WITNESS from SYNC — SYNC only tracks *which* particles
matched, WITNESS tracks *what they actually said*.

**Honest scope:** this is a single-node content attestation — a real,
cryptographic-strength checksum — not multi-party consensus. No signing
key, no second witness, no Byzantine agreement protocol. That fuller
role is what `operations::Operation::is_read_only()` still describes
WITNESS as *not* being read-only for, conceptually — this HeptaScript
realization is deliberately narrower.
```
QUERY:WITNESS
WHO T.E
WHAT E[meta.title]
WHERE E[meta.collection] = "component"
```
```json
{"rows":[{"kaki":"...","attrs":[["meta.title","..."]]}],
 "witness_digest":"272b01648e209e63ec6d642cb2bd02f574e8b0785e7fe2ea5d813f7d13e9b720"}
```
Live-verified against a real Read Node: a 64-hex-character digest,
identical across repeated identical queries, and confirmed to change
when the underlying content changes even though the same single entity
still matches
(`engine::tests::witness_digest_is_present_deterministic_and_content_sensitive`).

## 5. The v2.0 clauses — routing, tiering, intelligence, governance

All ten below share the same status: 🧩 PARTIAL. Each parses correctly,
each is copied verbatim into `QueryResult.plan` (a `QueryPlan` struct),
and **none of them filters, sorts, or routes anything** in the code paths
that answer a real query today (§2's table). `QueryPlan`'s own doc
comment (`engine.rs:56`) is honest about the intent: "the plan tells the
upstream dispatcher... where else to send the query and what
post-processing to apply" — that dispatcher (conceived as
`enkidu-protocol`) exists as a crate, but it is a **TCP wire-frame codec**
(ring buffer + buffer pool + frame codec — `SPSC ring → buffer pool →
QUERY/RESULT_STREAM/RESULT_END frames`), not a NODE/ACROSS/TIER/etc.
dispatcher. No crate in this workspace reads more than one field
(`.plan.across`, for a log tag only) of any `QueryPlan`.

Write these clauses today as **reserved, forward-compatible syntax** —
they cost nothing to include (no parse error, no runtime cost beyond
copying a few enum values into `.plan`), and a query that includes them
is not "wrong," just not yet acted on beyond WHO/WHAT/WHERE/WHEN/WHY/HOW/
HOW_MUCH.

### NODE — target database type(s)

**Syntax:** `NODE <target> [| <target> ...]` or `NODE ALL`. Targets:
`EnkiDB` (7001), `EnkiDW` (7002), `EnkiSDB` (7003), `EnkiODB` (7004),
`EnkiQDB` (7005), `EnkiMDB` (7006 — **note:** this port number is the
`DbTarget::port()` value from the original 7-type design; the *actually
deployed* `enkimdb-read-server` binds `7202`, not `7006` — a real,
unreconciled numbering split between this clause's aspirational port
table and the CQRS redeploy's real ports. Use the real deployed ports
from `docs/EnkiMDB_MANUAL.md`, not this clause's `port()` values, for
anything you actually connect to.), `EnkiDDB` (7007, same caveat —
real deployed port is `7102`), `NARUDU`, `EnkiPattern`.

Parsed before `WHO`, per `parser.rs::parse()`'s clause order (`node`,
`across` are the only two clauses parsed *before* WHO — everything else
follows the W5H2 order).
```
NODE EnkiMDB | EnkiDDB
WHO T.E
...
```
Godot's real multi-target fan-out (`SumuUkinClient.route()`, used by the
HeptaScript Editor's NODE checkboxes) is a **separate, working**
mechanism — see §7. It does not read this HeptaScript `NODE` clause at
all; it's driven by which checkboxes the operator ticks in the editor UI.

### ACROSS — cross-BIGRING federation

**Syntax:** `ACROSS BIGRING <ClientName>` or `ACROSS ALL`.

The only v2.0 clause with *any* runtime consumer: `enkidb-query-server`
reads `result.plan.across` to append `" [BIGRING]"` to its own log line
when the value is `Some(Bigring(_))` or `Some(All)` — cosmetic, not
federation. No query is ever actually sent to another BIGRING. See §7 for
what "multiple BIGRINGs/Tribes" really means today across this ecosystem.
```
ACROSS BIGRING ClientAlpha
WHO T.E
...
```

### TIER — storage tier filter

**Syntax:** `TIER <tier> [| <tier> ...]`. Tiers: `HOT` (0–2 orbitals),
`WARM` (3–29), `COLD` (30–364), `CRYSTALLIZED` (365+).
```
TIER HOT | WARM
```

### STATE — lifecycle filter

**Syntax:** `STATE <state> [| <state> ...]`. States: `EMERGING`
(confidence ≥ 0.60), `STABLE` (≥ 0.85), `CANONICAL` (AI-Council-promoted),
`DEPRECATED`.
```
STATE CANONICAL | STABLE
```

### NASH — Nash equilibrium constraint

**Syntax:** `NASH SCORE <op> <float>` or `NASH BREAKING` (shorthand for
score > 0.85).
```
NASH SCORE < 0.20
```
```
NASH BREAKING
```

### PATTERN — ENKI-PATTERN conditions

**Syntax:** repeatable — `PATTERN TYPE <kind>`, `PATTERN CONFIDENCE <op>
<float>`, `PATTERN MAX_CONSTITUENTS <op> <int>`, each a separate `PATTERN
...` line, ANDed together (`parse_pattern_conditions` loops while it sees
another `Token::Pattern`). Kinds: `CrowdFlow`, `AviationCorridor`,
`AviationHolding`, `IndoorHallway`, `IndoorRoom`, `IndoorTransition`,
`WaterFlow`, `Custom`.
```
PATTERN TYPE CrowdFlow
PATTERN CONFIDENCE >= 0.85
PATTERN MAX_CONSTITUENTS <= 126
```

### LINEAGE — causality traversal depth

**Syntax:** `LINEAGE DEPTH <n>` or `LINEAGE FULL` (bounded to 64 hops
internally, per the design comment — not enforced by any code today
since nothing walks lineage yet).
```
LINEAGE DEPTH 3
```

### GATE — ETL station routing

**Syntax:** `GATE <name> [, <name> ...]` or `GATE ALL`.
```
GATE DataSteward, DataCleansing
```
```
GATE ALL
```

### SATAMU — governance override

**Syntax:** `SATAMU REQUIRED` or `SATAMU BYPASS "<reason>"`.
`QueryPlan::satamu_required()` exists as a convenience reader — but same
status as everything else in this section: nothing calls it to actually
gate result release yet.
```
SATAMU REQUIRED
```
```
SATAMU BYPASS "incident response, ref INC-4471"
```

### ORBITAL — intra-day cycle time range

**Naming trap, worth stating plainly:** this is a **temporal** clause —
an intra-day cycle range, unrelated to the "orbit" the Architect means in
DubSar's Grid & Orbit 3D view or the GA/BIGRING proximity design
conversation. Don't confuse the two; there is no HeptaScript clause today
for spatial orbital-position proximity (see the companion evaluation
delivered alongside PB-218/219 for why, and what a real `NEAR`-style
addition would look like).

**Syntax:** `ORBITAL <start> .. <end>` or `ORBITAL NOW`.

**Status detail, sharper than the other nine:** ORBITAL is parsed into
`QueryPlan.orbital` (🧩 PARTIAL, same as its siblings) — but it is *also*
read directly by `orbital_range_from_query()` inside `execute_stream`, a
**second, complete executor** that pre-filters candidates by orbital
range before ever building an EAV map. The catch: `execute_stream` has
**zero callers anywhere in this workspace** (confirmed by grep) — nothing
in `enkiddb-read-server`, `enkimdb-read-server`, `enkiddb-cli`, or
`enkidb-query-server` invokes it. So ORBITAL's filtering logic is real,
tested Rust code, genuinely more built than TIER/STATE/NASH/etc. — but
identically inert on every query path that actually answers a request
today.
```
ORBITAL 100 .. 500
```

## 6. Production execution hints

Parsed (`parser.rs`'s "Production execution hints" section, after WHY
and the v2.0 filter clauses), carried on `HeptaQuery` directly (not
inside `QueryPlan`) — but with the exact same practical status as §5:
🧩 PARTIAL, and for four of the five, more inert than §5's clauses: a
direct `grep query\.anchor\|query\.stream\|query\.derive_station\|
query\.abort_scan\|query\.filter_order` across `engine.rs` — the file
containing *every* executor in this crate, including `execute_stream` —
returns exactly **one** match, `query.abort_scan` (line 164). `ANCHOR`,
`STREAM`, and `DERIVE_STATION` are copied onto `HeptaQuery`/`QueryPlan`
and never read back by anything, including `execute_stream`, despite that
function's own doc comment claiming to respect `ANCHOR`.

| Clause | Syntax | Real effect today |
|---|---|---|
| `ANCHOR` | `ANCHOR AUTO \| SURROGATE_TIME \| STATE_STATION \| E7_FIRST \| FULL_SCAN` | None anywhere — not read by any function in `engine.rs`, `execute_stream` included, despite that function's doc comment |
| `STREAM` | bare keyword, sets `HeptaQuery.stream = true` | None anywhere |
| `DERIVE_STATION` | `DERIVE_STATION <name>` (bare ident or quoted string) | None anywhere |
| `ABORT_SCAN` | `ABORT_SCAN <positive int>` | **Real**, but only inside `execute_stream` — which itself has zero callers anywhere in this workspace, so the net effect on any query you can actually run today is still none |
| `FILTER_ORDER` | `FILTER_ORDER <stage> [, <stage> ...]` — stages: `SURROGATE_RANGE`, `ORBITAL_RANGE`, `STATE`, `DERIVE_STATION`, `LANE`, `QUALITY_BYTE`, `EAV_ATTR`, `E7_REGION` | None anywhere — not even `execute_stream` reads `filter_order` |

These exist as forward-declared syntax for the `>1B particles, <1 second`
production target described in `engine.rs`'s module doc — real design
intent, not yet a real execution path. Safe to omit entirely from every
query you write against current infrastructure.

## 7. "One Tribe, multiple Tribes, or BIGRING" — the real disambiguation

Four different things in this ecosystem all touch "more than one tribe,"
and only one is fully real end-to-end. This table is the answer to "how
do I query across tribes/BIGRINGs":

| Mechanism | Where | Status | What it actually does |
|---|---|---|---|
| `WHO ... BOUND_TO ...` | HeptaScript source text | 🧩 PARTIAL | Parses a cross-tribe join binding; zero executor consumers (§3, WHO) |
| `HeptaQuery.tribes` + `SumuUkinContext::route()` | Rust API (`crates/heptascript/src/sumuukin.rs`), **not** expressible in HeptaScript source — set by the caller after parsing (`q.tribes = vec![...]`) | ✅ VERIFIED — real `std::net::TcpStream` dispatch, `AllParallel`/`FirstOnly`/`AllSerial` policies, tested against real spawned TCP servers | Fans the **same query text** out to multiple `RoutingTarget`s (host/port/tribe_id), one real TCP round-trip per target, over the `enkidb-query-server`-style wire protocol |
| `ACROSS BIGRING <name>` / `ACROSS ALL` | HeptaScript source text | 🧩 PARTIAL | Parsed, logged, never dispatched (§5, ACROSS) |
| `SumuUkinClient.route()` (Godot) | `godot/dubsar-theater/scripts/sumuukin_client.gd` | ✅ VERIFIED — real, used live by the HeptaScript Editor and Grid & Orbit view | Fans out to multiple **NODE targets** (EnkiDDB, EnkiMDB, ...) at a fixed host, by engine checkbox — a different axis (database *type*) than Tribe |

**Practical guidance today:** a single query against a single Read Node
(`WHO T.E ...`) is the only form with a real, single-hop execution story.
Genuine multi-tribe fan-out exists and works (`SumuUkinContext::route`),
but it's an operator/host-level concern — build the `RoutingTarget` list
and pick a `FanOutPolicy` in Rust (or the Godot equivalent's NODE
checkboxes), rather than expecting `BOUND_TO`/`ACROSS` inside the query
text itself to do it.

## 8. The 7D Index Stack, and what it means for query performance

Full detail: `docs/components/ANU_INDEX_STACK.md` (thirteen modules,
individually status-tagged). The short version for query-writing
purposes:

- **`surrogate` + `eav_exact_index`** (✅ VERIFIED, live): every exact-`=`
  WHERE condition you write is genuinely index-accelerated — `O(log n)`
  Xor8-fingerprint-then-binary-search, not a full scan — on **every**
  runner in §2's table (Read Node, Write Node, `enkidb-query-server`).
  This is the entire reason the Read Node constraint in §3/WHERE exists:
  the index only prunes on the *first* exact-equality condition.
- **`hepta_shell`** (HeptaShellIndex, the real "7D spatial E7-lattice"
  structure — `AnchorStrategy::E7First` / `FilterStage::E7Region` in
  §5/§6's tables) is built and tested in `enkidb-indexes`, but **not
  instantiated by any live query path** — same "not yet wired" status as
  ORBITAL's stream-only filtering. There is no HeptaScript clause today
  that reaches it.
- **`nairu_index`** (orbital-range pruning) — same status: built, tested,
  not wired to a live query.
- **`radix_spline`**, **`bloom`** — built, tested, zero consumers
  anywhere.

So "Full Production-Level HeptaScript... with all 7D Indexes Stack" is
accurate for the *parsing and core-seven-clause execution* half of that
sentence, and aspirational (but real, tested, close-to-wireable Rust) for
the 7D-spatial-index half. `hepta_shell` is the concrete next step if the
Graph Explorer / DubSar 3D work (next on the roadmap) ever needs
server-side spatial pruning instead of client-side layout math.

## 9. Full worked example — what actually runs today

Against the real deployed `enkiddb-read-server` (`192.168.122.107:7102`):
```
QUERY:WHO T.E
WHAT E[meta.title, meta.collection, link.target, link.description]
WHERE E[meta.collection] = "component"
WHY LANE != Black
HOW BY E[meta.title] ASC
HOW_MUCH LIMIT 25
```
Every clause in that query is ✅ VERIFIED and will genuinely filter, sort,
and cap the real result set.

Adding v2.1's aggregate clauses genuinely changes the result too — the
real "how many `component` documents per collection" shape, computed
server-side rather than pulled client-side and counted by hand:
```
QUERY:WHO T.E
WHERE E[meta.collection] = "component"
WHY LANE != Black
GRAVITY E[meta.collection] MAX_GROUPS 1000
MEASURE DENSE
```

Leading it with a verb keyword genuinely changes the result — unlike
every clause in the extended example below, §4's verbs are not inert:
```
QUERY:PROVE
WHO T.E
WHAT E[meta.title, meta.collection, link.target, link.description]
WHERE E[meta.collection] = "component"
WHY LANE != Black
HOW BY E[meta.title] ASC
HOW_MUCH LIMIT 25
```
returns the same matched rows, each now also carrying its full per-epoch
`history`.

The original query, extended with every v2.0/production-hint clause this
manual documents — parses, returns the *identical* result set (the
extra clauses are inert on this runner), and is the "reserved syntax"
form worth writing once DubSar's IDE or a future dispatcher starts
reading `QueryPlan`:
```
NODE EnkiDDB
ACROSS BIGRING DubSarPrime
WHO T.E
WHAT E[meta.title, meta.collection, link.target, link.description]
WHERE E[meta.collection] = "component"
WHEN AT EPOCH NOW
ORBITAL NOW
WHY LANE != Black
TIER HOT | WARM
STATE CANONICAL
NASH SCORE < 0.20
LINEAGE DEPTH 3
GATE DataSteward, DataCleansing
SATAMU REQUIRED
HOW BY E[meta.title] ASC
HOW_MUCH LIMIT 25
```

## 10. See also

- `docs/components/HEPTASCRIPT_GLOSSARY.md` — every term this manual
  uses without re-defining, HeptaScript-specific terms first.
- `docs/components/ANU_INDEX_STACK.md` — the thirteen-index W5H2
  reference this manual's §8 summarizes.
- `docs/components/KAKI_V4.md` — the Identity-Kaki/Event-Kaki byte
  layout WHERE/WHAT conditions ultimately resolve against.
- `docs/components/ENKIDB_7_TYPES.md` — the seven EnkiDB node types
  `NODE`'s `DbTarget` enum names (with the real-vs-aspirational port
  caveat repeated in §5).
- `docs/components/GEOENGINE_ALGEBRA_ARSENAL.md` — `bahyway-algebra`'s
  Clifford algebra (`Cl(7)`, wedge product), Rotor, and Pauli Exclusion
  (`enlil::pauli_check`) primitives MEASURE/GRAVITY's design rests on.
- `docs/components/BEEMDM_ETL_PIPELINE.md` — CompareEngine, including the
  hash-indexed Tribe-vs-Tribe Pauli dedup station (`pauli_dedup`) that
  shares GRAVITY's "index bounds it, algebra computes over it" design law.
- `docs/EnkiDDB_MANUAL.md`, `docs/EnkiMDB_MANUAL.md` — the real deployed
  wire protocol (`QUERY:`/`SEARCH:` prefixes, single-frame response) each
  Read Node actually speaks, one level below HeptaScript syntax itself.
- `docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` — Tribe, BIGRING, Triple-O, and
  every other ecosystem-wide term this manual uses without re-defining.
