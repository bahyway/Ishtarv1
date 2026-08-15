# BahyWay.Ecosystem v4.0 — Final Manifesto: The Run Sequence to the BeeMDM 50-Zip Test

**Purpose:** one ordered, de-duplicated sequence — built from every approved
concept across the 28 reviewed documents plus this repo's actual code —
that ends at the BeeMDM ETL 50-zip landing-zone test. No new research below
this line. Where something is genuinely blocked, it says so and names
exactly what unblocks it, so the blockage is a single decision, not a
circle.

**Scope honesty, stated once so it doesn't need repeating:** this git
checkout (`EnkiDB`) holds 133 real crates — the *code* — but it holds
**zero playbook YAML files** except `playbook_110...yml`, and **zero
persisted data** (no `journal.bin`, no data directory). The 80,272-particle
NAJAF_CEMETERY journal, and every other "particles exist" claim in the 28
documents, lives on your eriduous-vdi Forge tree, not in this repository.
So the "check existing particles" half of your request has an honest
two-part answer: **I cannot check runtime particle counts from inside this
sandbox** — that command has to run on eriduous-vdi, and §6 below gives you
the exact query to run there. What I *can* and did check is whether the
**code path that would serve those particles fast enough actually exists**
— and that's where this manifesto earns its keep, because the answer
changes everything about what "PB-98" even means now.

---

## 0. The one finding that rewrote BLK-1 — ✅ FIXED 2026-07-07, no longer a blocker

**Status update, same day:** the gap described below is closed. `heptascript`
now depends on `enkidb-indexes`; a new `heptascript::indexed` module builds
a real `SurrogateMap` + `EavExactIndex` snapshot from one journal pass
(`build_indexes`) and resolves exact-equality `WHERE` clauses through it
(`execute_indexed`) *before* reading any particle's history, falling back to
the untouched, always-correct `execute()` whenever a query genuinely can't
be pruned that way (an `OR` in the WHERE clause, or a `WHEN` pinned to a
specific past epoch the snapshot doesn't represent). `enkidb-query-server`
now builds the snapshot once at startup and serves every query through
`execute_indexed`. A new test proves the mechanism directly: the same query
against the same 100-particle journal goes from `evaluated: 100` (full
scan) to `evaluated: 1` (indexed) for a 1%-selective WHERE clause — that gap
only widens as particle count grows. All 170 `heptascript` tests pass (164
original, untouched, + 6 new), all 54 `enkidb-indexes` tests pass, and every
crate depending on either (`enkidb-query-server`, `dubsar-ide`,
`enkidb-query`, `bahyway-cli`, `dubsar`) still builds clean. The original
diagnosis is kept below for the record, per this ecosystem's own "old state
is never erased" law — treat everything past this paragraph as history, not
a live blocker.

Every one of the 28 documents describes the KISPU/performance blocker the
same way: `enkidb-query-server/src/main.rs` calls a broken `akk_decode`
function instead of `enkidb_ingest::bridge::eav_triple_to_value`. **That
specific bug no longer exists in this repo — the file was already
rewritten.** Its own header comment says so:

> *"Rebuilt against the current index/query stack. The original version
> (PB-34/92/95/97/98)... imported `enkidb_enlil_index` / `hepta_shell_index`
> / `geo_engine` — none of which exist anymore... query execution itself
> now lives in `heptascript::execute()`, which already indexes internally.
> This server is now a thin TCP wrapper around that."*

I checked whether "already indexes internally" is true. **It is not.**
`heptascript::execute()` (`crates/heptascript/src/engine.rs:242`) does this:

```rust
pub fn execute(query: &HeptaQuery, journal: &Journal) -> QueryResult {
    let candidates = journal.all_particles();          // every particle, every query
    ...
    let mut matched = candidates.iter().filter_map(|entity| {
        let history = journal.read_particle_history(entity);  // full replay, per particle
        ...
```

The code's own comment admits it: *"Without this [LIMIT early-exit], LIMIT
100 still scans all 80,272 entries before truncating."* `heptascript`
doesn't even depend on `enkidb-indexes` (checked its `Cargo.toml` — it
depends on `bahyway-core`, `enkidb-kaki`, `enkidb-journal`, `navi-engine`;
no `enkidb-indexes`). Meanwhile `enkidb-indexes` is real and substantial —
`surrogate.rs`, `nairu_index.rs`, `eav_exact_index.rs`, `hepta_shell.rs`
all exist with working, tested code — **it's just never called.**

So: the *specific* bug named in every document was gone. The *disease* — no
index-based pruning on the hot query path, full journal replay per particle
— was fully present, just relocated from the query-server into
`heptascript` itself, undocumented anywhere until this check. **This was
the corrected BLK-1 — now fixed, same day**: `heptascript::execute()` is
untouched (still the always-correct full scan), and a new
`heptascript::indexed` module wires `enkidb-indexes` into the query path via
`build_indexes()` + `execute_indexed()`, wired into `enkidb-query-server`.
See the status line above §0's heading for the specifics. It was the
literal reason nothing downstream could trust its own performance numbers.

---

## 1. The run sequence — de-duplicated, ordered, status-tagged

Legend: **✅ real** (crate exists, checked directly) · **⏳ written,
unverified-by-execution** · **🔒 blocked on a ruling** (named below) ·
**📋 spec only, no code**.

### Phase A — Foundation (PB-37→98)
**✅ Complete, including the index-wiring fix.** KAKI, journal, ConEngine,
storage, HeptaScript v1, and now the actual ENLIL-backed query path (§0) all
exist and are exercised by real tests.

### Phase B — Test infra (PB-99→109) → the Phase-1 gate
**✅ Crates real, ⏳ test counts need a fresh count, not RM-001's table.**
Verified directly this session: `naramsin-archive` 9 tests,
`enkidb-session-registry` 6, `enkidb-con-engine` 6, `enkidb-indexes` 54,
`heptascript` 164 — none match RM-001's cited 19/12/22/14/29. Before
declaring this gate green, run `cargo nextest run --workspace` and record
the real numbers once, in one place, superseding every prior table.
`docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md` exists, committed, and is still unfilled
(no PASS/FAIL recorded) — running it for real is the actual Phase-1 gate,
not any document's claim about it.

### Phase C — PB-110 UrNammu/Nisaba/Kittu design drop
**✅ Real, committed** (`playbooks/playbook_110_...yml`), and it already
self-corrected the "Playbook 99" mislabel from the BC-SEC-001 draft.
`urnammu-attestationd` stub scaffolded per the playbook; real TPM logic is
Phase E work below.

### Phase D — PB-111→118
**Mixed.** No playbook YAML files exist for this range, but several of the
crates they were meant to produce already do: `ashnan` (real),
`esarhaddon` (real), `naramsin-archive`/`naramsin-format`/`naramsin-audit`/
`naramsin-bridge` (real). Treat this phase as *substantially delivered by
crate, undocumented by playbook* — worth writing the playbooks retroactively
to close the paper trail, but not a code blocker.

### (gap) PB-119→136
**Confirmed does not exist and was never meant to** — an unfilled roadmap
range, not lost work. Skip 118→137. (Independently confirmed via BLK-5 in
STARTPOINT-001 and the original RM-002 verification.)

### Phase E — PB-137→145
**📋 Mostly spec-only in this repo.** `NAMTAR`/`EREŠKIGAL` per Vol. I are
still design-phase; UrNammu's real attestation logic
(`urnammu-attestationd`'s stub functions), the usbguard↔AkkadiRulesEngine
contract, and Kittu Engine's actual delivery build are the concrete
remaining items. **`AkkadiRulesEngine`/`AkkadiSafeEngine`/`AkkadiCipherEngine`
do not exist as crates yet** — only as names in docs and one stale
`bahywaylab.sh` comment (fixed two batches ago). This is a real gap, not a
naming issue.

### Collection A — Algebra (PB-152–154, per PBCOLLECTIONS)
**⚠ PB-152 is contested — resolved here in favor of code.** PBCOLLECTIONS
calls PB-152 "SU(7) Lie algebra seed, 6/6 tests" — no matching code exists
anywhere in this repo or in any of the 28 documents' attachments. The
*other* PB-152 — ENLIL Tribe HotIndex — I extracted from its actual `.yml`
and compiled myself: 5/5 tests, real. **This manifesto treats PB-152 =
ENLIL Tribe HotIndex as canonical**; if SU(7) work is real, it needs a
different number when a corresponding playbook file actually surfaces.
PB-153 (Clifford multivectors) and PB-154 (rotor journal) — no corresponding
crates found under those names in this repo; unverified.

### Collection B — Triple-O Query (PB-155–157)
**Unverified in this repo.** No `enkidb-sdb`/rotor/ARS-named crates matching
these three found. HeptaScript's actual anti-SQL enforcement already exists
and is tested (164 tests) — whether that constitutes PB-155/156/157 by this
repo's naming or whether those are separate, additional work is genuinely
unclear from what's committed here.

### Collection C — EnkiDB Family Pipeline (PB-158–161)
**📋 Confirmed not built**, matching every other check this session:
no `enkidb-mdb`, no `enkidb-ddb`, no `rotor_partition.rs` in `enkidb-dw`.
EnkiMDB and EnkiDDB — the thing you've said you need most — start here.

### Collection D — BeeMDM Hepta Gates (PB-162–165)
**✅ PB-162's gate enum is real** — `bahyway_core::HeptaGate`, 7 variants,
merged pipeline functions per §2 item 1. PB-163/164/165 (fuzzy-state
machine, golden-particle supersession, per-gate latency harness) can now
be built against it — none exist yet, still genuinely new work, but no
longer blocked on a ruling.

### Collection E — Security & Verification (PB-166–170)
- PB-166 Merkle journal — 📋 not built (`enkidb-journal` has no `merkle.rs`).
- PB-167 Ring HA/DR — **✅ already real**, ahead of its own spec: 
  `enkidb-replication` exists with 1,292 lines across `broker.rs`,
  `consumer.rs`, `emitter.rs`, `event.rs` — the one item in this whole
  review where the repo is ahead of the paperwork.
- PB-168 Ed25519 KAKI signatures — 📋 not found; `kupru` crate exists
  (matches GL-001's "AkkadiCipherEngine, likely kupru" guess) but signing
  specifically wasn't checked in depth this session.
- PB-169 Z3/Lean bridge — 🔒 blocked on Z3-DEP ruling (dev-time-only vs.
  pure-Rust vs. deferred) — design already settled in an earlier session
  (MUMMU, design-time-only, download-once) per STARTPOINT-001 §1.7; treat
  as ruled, build when prioritized.
- PB-170 CSR-08 rule — 🔒 **genuinely blocked, and I checked directly**:
  `enkidb-con-engine/src/csr.rs` implements CSR-01 through CSR-07 only, as
  one function, no `rules/mod.rs` trait structure. This needs the
  Architect's CSR-08 code design (not just the governance rule already
  sealed in the glossary), or a decision to encode it the same
  single-function way as CSR-01–07.

### Collection F — Documentation Governance (PB-171)
**📋 Not built** — depends on PB-160 (EnkiDDB) existing first.

---

## 2. Rulings that unblock the most work, in priority order

1. **GATE-1 — ✅ RESOLVED 2026-07-07.** This was two questions:
   - *Sector mapping* (sealed 2026-06-18: G1 APSU→Storage...G7 ENLIL→
     Governance) — unchanged, confirmed still holds.
   - *Pipeline-step mapping* for PB-162 (GLS-001's doc-vs-SVG conflict) —
     **Architect's ruling: merge, don't choose one.** Implemented in
     `bahyway-core::hepta_gate::HeptaGate` — each gate now carries both
     `sector()` (unchanged, sealed) and a new `pipeline_function()`
     combining both source lists (e.g. G1 Doc="Security"+SVG="Identity" →
     "Identity & Security Intake"; full table in the module doc comment).
     6 new tests, all pass; full workspace (`cargo build --workspace`)
     rebuilds clean with zero errors. PB-162–165 are unblocked.
2. **CSR-08 code design** — the governance rule is sealed in prose; PB-170
   needs it in `enkidb-con-engine`'s actual trait shape.
3. **TIAMAT table edit** — confirm `NERGAL`→`ERRA` in
   `BAHYWAY_ECOSYSTEM_MANUAL_V4.md`'s alert table per GL-001's explicit
   correction (recorded, not yet applied, three batches ago).
4. **κ[8..11] and `Pattern=0x04`** — lower urgency (doesn't block the run
   sequence), but load-bearing for KAKI's byte contract whenever touched.

---

## 3. What actually gets you to the 50-zip test (MAN-001's own gate, restated as a checklist)

- [x] Phase A/B green **including the real index-wiring fix** (§0 — fixed
      2026-07-07, not just the old `akk_decode` symptom being gone).
- [ ] PB-170 / CSR-08 implemented in code.
- [ ] GATE-1's pipeline-step question ruled (§2 item 1 above).
- [ ] Landing-zone path confirmed and staged with the 50 zips.
- [ ] Latency budget table accepted as pass/fail criteria (per-gate, from
      the GA-GA doc — not located in this repo; confirm you still have it).
- [ ] KAKI-minting-only-at-`enkidb-ingest::bridge` dry run on 100 records,
      observed directly.

This is the same six-item list MAN-001 gave on 2026-07-03, with the first
item now closed. Re-run `docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md` for real numbers
against the indexed path before treating Phase A/B as fully green — the
mechanism is proven correct by unit test, but the actual 10K/1B-particle
timing claim still needs a real run, on real data, on eriduous-vdi.

---

## 4. Checking existing particles — the command for eriduous-vdi, not here

I cannot run this from inside this repo checkout — there's no `journal.bin`
here. On eriduous-vdi, against each running EnkiDB-family node, the
equivalent of "does this type have particles" is a HeptaScript query per
type:

```
WHO T.E WHAT E[*] WHERE tribe_id = <target> HOW_MUCH LIMIT 1
```
run against each of EnkiDB(7001)/EnkiSDB(7003)/EnkiODB(7004)/EnkiQDB(7005)/
EnkiMDB(7006)/EnkiDW(7002) — EnkiDDB(7007) will always return empty since
that crate doesn't exist yet anywhere. A non-empty result on any node is
your existence proof; an empty NAJAF_CEMETERY tribe result on EnkiDB would
itself be a serious, worth-reporting-back finding given every document
describes 80,272 particles already committed there.

---

**Bottom line for momentum:** the architectural fact (§0) is fixed as of
today — that was the one item purely mine to build, and it's built and
tested. What's left is two rulings (§2 items 1–2: GATE-1's pipeline-step
question, and CSR-08's code shape) plus the operational side only you can
do on eriduous-vdi (§3's remaining unchecked items). Rule those two and run
Phase-1 for real, and the 50-zip test stops being a circle.
