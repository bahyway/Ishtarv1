# ADR-013 — Rust Is the Permanent Substrate; `.akk` / HeptaScript Are the Domain Layer, Phased In Post-v4.0

> **DubSar Help** | `Decisions > ADR-013` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-07-30"
  concept_depth:  235
  riksu_count:    3
  snapshot_epoch: "2026-07-30"

concept:          "Engine Implementation Language Policy"
summary:          "New BahyWay.Ecosystem engines are written in pure Rust before, during, and after v4.0. There is no version boundary at which engines stop being Rust. What changes post-v4.0 is where domain logic is expressed: governance rules and sovereign operations move into .akk and HeptaScript source files, interpreted or compiled by Rust engines that are themselves phased in as HeptaScript's own crates (heptascript-engine, parzu-engine, alert-engine, etc.) are built out."
sovereign_laws:   ["§ENGINE-LANG — every engine crate that executes .akk or HeptaScript is written in Rust, with no expiration date", "§DOMAIN-SPLIT — governance law and sovereign-algebra operations are authored in .akk/HeptaScript; process orchestration, I/O, and runtime plumbing remain Rust"]

riksu_bindings:
  - target: "adr_010_heptascript_language_design.md"
    concept: "HeptaScript sovereign vocabulary and crate ownership"
    type: "PEER"
  - target: "heptascript_design.md"
    concept: "crate ownership table (which HeptaScript verbs are Not built)"
    type: "GROUNDS"
  - target: "akk_format.md"
    concept: ".akk grammar (governance law, not a systems language)"
    type: "GROUNDS"

orbit_tags:       ["Language Policy", "HeptaScript Sovereign Language", "Engine Architecture"]
rag_keywords:     ["Rust", "AAOL", ".akk", "HeptaScript", "engine language", "substrate", "self-hosting", "domain split", "v4.0", "heptascript-engine"]
-->

**Status:** Accepted (corrected 2026-07-30, same day)
**Date:** 2026-07-30
**Author:** Bahaa Fadam
**Related:** ADR-010 (HeptaScript Language Design), `docs/07_file_formats/akk_format.md`, `docs/09_languages/heptascript_design.md`

---

## CORRECTION (same day)

This ADR's original Context section 2 claimed HeptaScript's execution engines
were mostly "Not built," citing `docs/09_languages/heptascript_design.md`'s
crate-ownership table. That table was never checked against the actual
`crates/heptascript` source — it is **wrong**. Checked directly:
`crates/heptascript` is a real, mature query engine — a lexer (`token.rs`),
parser (`parser.rs`), execution engine (`engine.rs`), and two indexing modules
(`indexed.rs`, `modular_index.rs`), **7,147 lines**, **227 passing tests**
(`cargo test -p heptascript`), and actually imported by all seven EnkiDB
read-server binaries (`enkiodb-`, `enkiqdb-`, `enkimdb-`, `enkiddb-`,
`enkidb-`, `enkisdb-`, `enkidw-read-server`), `enkidb-query-server`,
`bahyway-cli`, `dubsar-ide`, and `enkiddb` itself (`writenode`/`rag`/`topics`/
`exposure`). It is versioned **v2.0/v2.1**: the W5H2 grammar (`WHO`/`WHAT`/
`WHERE`/`WHEN`/`WHY`/`HOW`/`HOW_MUCH`) plus cross-database v2.0 clauses
(`NODE` — targets `EnkiMDB | NARUDU | ENKI_PATTERN | ALL`, `ACROSS`, `TIER`,
`STATE`, `NASH`, `PATTERN`, `LINEAGE`, `GATE`, `SATAMU`, `ORBITAL`) and v2.1
aggregate clauses (`MEASURE`, `GRAVITY`). **Querying is a now capability, not
a future one.**

One nuance survives, checked just as directly: `crates/heptascript/src/operations.rs`
defines exactly five verbs (`Orbit`, `Emit`, `Prove`, `Sync`, `Witness`), and
`lib.rs`'s own doc comment states "all five stay strictly read-only" —
there is no `MINT`/`APPEND`/write verb in this crate. So the *query* half of
Decision 2's table below is real and available today; the *mint/write* half
(creating a new particle in the first place) genuinely still requires a
Rust-side emitter (`enkimdb::RegistryEmitter`, `enkiddb::DocumentEmitter` —
see `crates/shakkanakku/src/docpulse.rs`'s EnkiDDB stage for a live example),
not HeptaScript syntax. Decision 2's table and the Negative/Mitigation
consequences below are corrected accordingly; Decisions 1 and 3 (Rust as the
permanent substrate, no version-boundary cutover) are unaffected — a mature,
real HeptaScript is still implemented *in* Rust, indefinitely.

**Separately, and still open:** `docs/09_languages/heptascript_design.md`
(canonicalized by ADR-010) describes a *different* vocabulary entirely —
`PROJECT`/`MINT`/`ORBIT`/`PROBE`/`ASSESS`/`WATCH`/`SEAL`/`FORECAST`/`TRACE`/
`DIVERGE`/`RANK`/`ENTROPY`, plus Akkadian primitives `DUB`/`ME`/`RIKSU`/
`KIŠIB`/`ZIKRU`/`PARZU` — with almost no overlap against what's actually in
`crates/heptascript` (W5H2 + `NODE`/`ACROSS`/`TIER`/etc.). Two designs share
the name "HeptaScript"; the docs describe the one that isn't the one
actually shipped and wired in everywhere. Reconciling ADR-010 itself (is it
superseded, is the real crate due for a v3 that adds its vocabulary, are
these two intentionally-separate languages under one name) is the
Architect's call, not resolved by this correction.

---

## Context

The question on the table: should the ecosystem split its tool-building policy
at the v4.0 boundary — **before/during v4.0: build new engines in pure
Rust**, **after v4.0: build new engines in AAOL, the `.akk` language** — as a
clean generational handover?

Before accepting that framing, two things in this repo needed checking
against it:

1. **What `.akk` actually is today.** Per `docs/07_file_formats/akk_format.md`,
   `.akk` is a small declarative grammar for governance law:
   ```
   akk_file := header rule*
   rule     := "on" event "when" condition "do" action
   ```
   This is a rule file for the PARZU/MARDUK governance gate — not a
   general-purpose or systems language. It has no constructs for process
   control, I/O, threading, or subprocess orchestration — the kind of work
   an engine (e.g. `docpulse.rs`'s reform/audit/pulse pipeline) actually
   does.

2. **What HeptaScript actually is today.** `docs/09_languages/heptascript_design.md`
   (canonicalized by ADR-010) is the real "sovereign language" — the
   orbit-based particle algebra. But its own crate-ownership table lists
   most of its execution engines (`heptascript-engine`, `graph-engine`,
   `ammas-engine`, `alert-engine`, `parzu-engine`) as **Not built**, and the
   ones that exist (`kaki-core`, `enkidb-journal`) are Rust crates.
   HeptaScript is *interpreted or compiled by* Rust — nothing in its design
   describes it (or `.akk`) as self-hosting, i.e. capable of authoring the
   engine that executes itself.

The practical consequence: "stop building engines in Rust after v4.0" isn't
a policy that `.akk` or HeptaScript can currently support. Something has to
keep parsing, compiling, and executing `.akk`/HeptaScript source — and that
something is a Rust crate, indefinitely. A version-boundary cutover away
from Rust would either stall (no engine to run the new language) or quietly
smuggle Rust back in as "the language that builds the language."

## Decision

### Decision 1 — Rust remains the substrate, with no expiration date

Every engine crate that parses, compiles, or executes `.akk` or HeptaScript
— the runtime, the governance-gate evaluator, the sovereign-algebra
operations, process/I/O orchestration — is written in Rust. This is not
revisited at v4.0 or any later version boundary. `docpulse.rs`,
`heptascript-engine`, `parzu-engine`, and their future siblings are Rust
crates by law, not by current convenience.

### Decision 2 — The real split is domain logic vs. runtime, and query vs. mint — not a version boundary

What legitimately moves, and when:

| Layer | Expressed in | When |
|---|---|---|
| Governance rules (promote/demote/steward/archive on a tribe) | `.akk` | Already true today |
| **Querying** particles (read/filter/aggregate across any of the 7 EnkiDB types) | HeptaScript (`crates/heptascript`, v2.0/v2.1: W5H2 + `NODE`/`ACROSS`/`TIER`/`STATE`/`NASH`/`PATTERN`/`LINEAGE`/`GATE`/`SATAMU`/`ORBITAL`/`MEASURE`/`GRAVITY`) | **Already true today** — real, tested, wired into all 7 read-servers |
| **Minting** a new particle (creating a document, PB, or any other Identity-Kaki-bearing record in the first place) | Rust emitter (`enkimdb::RegistryEmitter`, `enkiddb::DocumentEmitter`, and future siblings e.g. a `PbEmitter`) | Always — HeptaScript's own five verbs (`Orbit`/`Emit`/`Prove`/`Sync`/`Witness`) are deliberately read-only; there is no write verb to phase in |
| Process orchestration, subprocess/git/network I/O, threading, UI | Rust | Always |

A new engine crate is legitimately "queried via HeptaScript" the moment it
writes real particles through a Rust emitter into any of the 7 EnkiDB
types — that half needs no further phase-in, it works today. Its *minting*
path stays a Rust emitter indefinitely, by HeptaScript's own read-only
design, not because anything is unbuilt.

### Decision 3 — No hard cutover at v4.0

There is no `if version >= 4.0 { forbid new Rust engines }` rule. The
migration of domain logic from hand-written Rust into `.akk`/HeptaScript is
gated on HeptaScript's own crate-ownership table (`heptascript_design.md`),
not on a calendar version. An engine whose domain logic has no HeptaScript
verb yet (because the backing crate is "Not built") is written in Rust
directly, exactly as `docpulse.rs` was.

## Consequences

**Positive:**
- Avoids a stall where a version boundary arrives but no `.akk`/HeptaScript
  runtime exists yet to execute anything.
- Matches what `docpulse.rs` and every existing Shakkanakku module already
  do: orchestration in Rust, governance rules in `.akk` where they exist
  (PARZU).
- Keeps the migration path honest against `heptascript_design.md`'s own
  "Not built" column instead of an arbitrary date.

**Negative:**
- Less clean-sounding than a hard "Rust before, `.akk` after" story — the
  split is a layer boundary (domain vs. runtime, query vs. mint), not a
  timeline.
- `docs/09_languages/heptascript_design.md`/ADR-010 describe a different
  vocabulary than the real, shipped `crates/heptascript` — that divergence
  needs the Architect's own reconciliation (see CORRECTION above), separate
  from this ADR.

**Mitigation:**
- When a new engine is proposed: querying its particles across any EnkiDB
  type is already available today via the real `crates/heptascript`
  (v2.0/v2.1) — no phase-in needed. Minting them is a Rust emitter,
  mirroring `enkimdb::RegistryEmitter`/`enkiddb::DocumentEmitter` (see
  `docpulse.rs`'s EnkiDDB stage for a live example wiring both together:
  Rust mints via `enkiddb::WriteNode`, the result is immediately
  HeptaScript-queryable once materialized). `.akk` stays the governance-law
  layer it already is. Do not block a new engine on any of this — write the
  Rust orchestration directly, wire in a real emitter, and it's queryable
  the same day.

## References

- `crates/heptascript`: the real, shipped v2.0/v2.1 W5H2 query engine (227 tests, wired into all 7 EnkiDB read-servers) — the actual ground truth this correction is based on, not `heptascript_design.md`'s stale table
- `crates/enkimdb/src/registry_emitter.rs`, `crates/enkiddb/src/emitter.rs`/`writenode.rs`: the real Rust mint/emit pattern every new particle type follows
- `crates/shakkanakku/src/docpulse.rs`: worked example wiring both together — Rust mints via `enkiddb::WriteNode`, HeptaScript queries the result once materialized
- ADR-010: HeptaScript Language Design (canonical vocabulary; crate ownership table — now known to describe a different design than `crates/heptascript`, unreconciled)
- `docs/07_file_formats/akk_format.md`: `.akk` grammar — governance law, not a systems language
- `docs/09_languages/heptascript_design.md`: the HeptaScript spec ADR-010 canonicalized — its crate-ownership table does not describe `crates/heptascript`'s real state
