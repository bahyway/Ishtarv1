# ADR-014 — KAKI v4.0 Minted at Authoring Time for Playbooks and Documents

> **DubSar Help** | `Decisions > ADR-014` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-07-30"
  concept_depth:  235
  riksu_count:    4
  snapshot_epoch: "2026-07-30"

concept:          "KAKI Minting Timing Policy for Playbooks and Documents"
summary:          "Every real, numbered playbook and every promoted document gets a real Identity-Kaki minted at authoring time -- the moment it becomes a real, committed artifact -- not deferred until first execution. Renumbering, deprecation, or supersession is recorded as an APPEND (Event-Kaki) on the existing particle, per ADR-006's no-delete law, never treated as an orphan to avoid. This is the mechanism that ends 'which of these documents is current, which is deprecated' -- every artifact has one permanent identity and a queryable lifecycle from the moment it's real."
sovereign_laws:   ["§MINT-AT-AUTHORING -- Identity-Kaki is minted when a PB/document becomes real (numbered + committed / promoted), not deferred to first execution", "§SUPERSEDE-NOT-ORPHAN -- a renumbered or deprecated particle is never deleted or abandoned; its supersession is an APPEND event on its own permanent Identity-Kaki, per ADR-006"]

riksu_bindings:
  - target: "adr_006_no_delete_mandatory_partitioning.md"
    concept: "APPEND supersedes, never deletes"
    type: "GROUNDS"
  - target: "adr_013_rust_substrate_akk_heptascript_domain_split.md"
    concept: "Rust mints, HeptaScript queries"
    type: "PEER"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI identity, tribe id conventions"
    type: "GROUNDS"

orbit_tags:       ["KAKI Sovereignty", "Shakkanakku", "Uruinimgina", "Documentation Governance"]
rag_keywords:     ["KAKI", "mint at authoring", "pb_schema", "new_doc_schema", "PbEmitter", "DocumentEmitter", "RegistryEmitter", "Uruinimgina", "docpulse", "supersede", "deprecation", "DOCS_TRIBE_ID"]
-->

**Status:** Accepted and implemented 2026-07-30 — documents, playbooks, and supersession all real, tested code (see Decisions 2-4)
**Date:** 2026-07-30
**Author:** Bahaa Fadam
**Related:** ADR-013 (Rust substrate / HeptaScript domain split), ADR-006 (no-delete, APPEND supersedes), ADR-003 (KAKI sovereignty)

---

## Context

The question: when does a playbook or a document get its real KAKI identity —
at authoring time (the moment it's numbered/committed/promoted), or deferred
until it first actually runs?

The initial recommendation in this session's discussion was "defer to first
execution," reasoning that this repo has renumbered or discarded
ideation-only PB drafts more than once (the PB-255/182 numbering collision,
the "iPhone Session" draft PB-263 corrected against) — minting at authoring
time, that reasoning went, would leave "orphaned" KAKI particles for every
draft that never becomes real.

**The Architect's own correction, which this ADR adopts:** that framing was
solving the wrong problem. Per ADR-006 (no-delete, mandatory partitioning —
`APPEND` supersedes, nothing is ever deleted), a particle that gets
renumbered or deprecated after being minted is not an orphan to avoid — it's
exactly the permanent, queryable historical record that ends the actual
problem in front of us: not knowing which of several similarly-named
documents is current, which is superseded, and when that happened. Deferring
the mint doesn't prevent confusion; it just means the confusion has no
identity to resolve against. Minting at authoring time and recording
supersession as an `APPEND` event on the same Identity-Kaki is the tool this
ecosystem already built for exactly this.

## Decision

### Decision 1 — Mint at authoring time, not first execution

A playbook or document gets its real Identity-Kaki the moment it becomes a
real artifact — a numbered PB file committed into `playbooks/`, or a
document promoted into Uruinimgina's intake flow — not deferred until it
first runs or is otherwise "proven real."

### Decision 2 — Supersession is an APPEND, never an orphan — implemented

When a document is superseded by a newer version, the OLD Identity-Kaki is
never deleted and never silently abandoned. Real, tested code as of this
session:

- `enkiddb::emitter::DocumentEmitter::emit_supersession(old, new, reason)`
  — targets `old`'s own existing identity (mints nothing new), emitting
  `hist.event = "SUPERSEDED"`, `hist.superseded_by = <new Kaki, KakiPk
  link>`, `hist.reason = <the actual why>`.
- `enkiddb::writenode::WriteNode::supersede_document(old, new, reason,
  epoch)` — journals it under the new `EventCause::DocumentSuperseded`
  (`enkidb_journal::EventCause`, discriminant `0x77`).
- Wired into Uruinimgina (`crates/shakkanakku/src/docpulse.rs` stage 4): a
  durable `doc_kaki_registry.jsonl` (path → last-minted Kaki, later lines
  win) is read at the start of each pulse; when a promoted document's path
  already has a prior real Kaki on record, this run's fresh mint calls
  `supersede_document` against it automatically — `cfg.message` (the
  commit message the Architect already types for the pulse) becomes the
  `hist.reason`, at no new UI cost.
- Tests: `enkiddb::emitter::tests::emit_supersession_*` (targets the OLD
  kaki, carries event/link/reason), `enkiddb::writenode::tests::
  supersede_document_appends_to_the_old_identitys_own_history_without_minting_a_new_one`
  (end-to-end: two real documents, a real supersession, `hist.reason`
  actually queryable back out of the Journal).

"What's current, what's deprecated, when did that change, and why" is now a
Journal query (`hist.event`/`hist.reason`/`hist.superseded_by` on the old
Kaki), not something a human reconstructs by reading file timestamps or
Confluence comment threads.

For playbooks, the same law applies but no code path exercises it yet — a
renumbered PB's old KAKI would need the same `supersede_document`-shaped
treatment. Not built in this session (no renumbering event occurred to
drive it); the pattern above is the template when one is needed.

### Decision 3 — Documents: implemented this session

`enkiddb::WriteNode`/`DocumentEmitter` already minted a real Identity-Kaki
per ingested document (`ingest_document_from_path`) before this ADR was
written — `enkiddb-ingest`'s bulk CLI has treated "authoring" as "walked and
ingested" all along. This session wires the same real pipeline into
Shakkanakku's Uruinimgina tab (`crates/shakkanakku/src/docpulse.rs`, stage
4), so a document promoted through Uruinimgina gets the identical treatment
one file at a time, per pulse, instead of only in a bulk directory walk:

1. Mint a real Identity-Kaki (`enkidb_kaki::KakiMinter`, tribe
   `enkiddb::DOCS_TRIBE_ID` — moved from a private constant in
   `enkiddb-ingest/src/main.rs` to a public constant on `enkiddb` itself,
   single source of truth for both callers).
2. Journal it and materialize a real, versioned Tigris generation
   (`enkiddb::materialize_version`) — HeptaScript-queryable immediately,
   per ADR-013's query-side correction.
3. Land the promoted file into the OFFICIAL `EnkiDB` repo's
   `docs/bahyway-v4/` folder — a new, separate, gated stage (stage 5):
   skipped entirely unless a target repo is configured, commits locally on
   a named review branch that is never `main`/`master` (hard, tested
   guard), and never runs `git push` unless explicitly turned on — a PR is
   still the real merge gate even then.

**Categorization** ("AI Agent to categorize it with other already saved
documents"): manual for now — supplied as metadata at promotion time, not a
live LLM API call wired into the Rust governor. Automating that step is
future work, not a blocker for minting at authoring time today.

### Decision 4 — Playbooks: implemented

The equivalent for PBs is real, tested code as of this session:

- `enkimdb::pb::PbProfile`/`scan_pbs(repo_root, triage_doc)` — scans
  `playbooks/*.yml` for real, numbered files (`playbook_<N>_...yml`),
  cross-referenced against `docs/PLAYBOOK_EXECUTION_TRIAGE.md`'s own
  Status/Note columns when a row exists. A real bug was caught and fixed
  while building this: two different files CAN legitimately share one PB
  number (the triage doc's own fixture documents this — `playbook_90_a.yml`
  / `playbook_90_b.yml`, "different content, same number"), so the
  triage-row lookup keys on the filename, never the bare number — keying
  by number would have silently swapped one file's status/note onto the
  other whenever a collision existed. `PbProfile.file` (identity) is a
  full filename stem, never a bare number.
- `enkimdb::pb_emitter::PbEmitter` — mints one Identity-Kaki (role `Parzu`
  — "logic, template, axiom, or rule," the same role
  `RegistryEmitter::emit_error_type` uses) per real PB, emitting
  `pb.number`, `pb.file`, `pb.path`, and `pb.status`/`pb.summary` when a
  triage row exists. Mirrors `RegistryEmitter`'s exact shape.
- `enkimdb::writenode::WriteNode::ingest_pb` — journals it under the new
  `EventCause::PbRegistered` (`0x76`).
- `crates/shakkanakku/src/pb_mint.rs` — the corpus-scan trigger: every
  real Corpus-tab run (`runner.rs`) scans `playbooks/` and mints any real,
  numbered PB not yet in a durable JSONL registry
  (`<chronicle_dir>/pb_kaki_registry.jsonl`, keyed by filename, matching
  the same-number-different-file law above), then materializes a real
  Euphrates generation — quiet (no log lines, no materialize call) on the
  overwhelming majority of runs that mint nothing new. `PB_TRIBE_ID =
  0x7161` — a new, documented constant distinct from
  `enkiddb::DOCS_TRIBE_ID`.
- Consistent with the existing law that "running this playbook = CSR-08
  confirmation" (PB-263's own header): by the time a PB has a real number
  and is committed, it has already been through the Architect's own
  ratifying act — minting on corpus scan, not on a separate ceremony, is
  the correct trigger.

## Consequences

**Positive:**
- One identity per PB/document from the moment it's real — "which is
  current, which is deprecated, and why" becomes a query (`ORBIT`/`WHY`
  over the Journal's `hist.*` attributes), not a human archaeology
  exercise.
- Matches, rather than fights, ADR-006's own no-delete law — supersession
  was already the right tool, just not yet pointed at this problem.
- Both halves — documents (mint + supersession) and playbooks (mint +
  corpus-scan trigger) — are real, tested code today, not a future
  promise: 34/34 `enkimdb` tests, 92/92 `enkiddb` tests, 23/23
  `shakkanakku` tests passing, including the specific tests that prove
  the same-PB-number-different-file bug is fixed and that
  `hist.reason` actually survives a real round trip through the Journal.

**Negative:**
- A KAKI minted for a PB or document draft that turns out to be scrapped
  or heavily reworked still exists permanently, with an `APPEND` recording
  its fate — slightly more Journal volume than a defer-until-real policy,
  accepted deliberately (Decision 1's whole point).
- PB-side supersession (a renumbered PB's old KAKI recording why) follows
  the exact same pattern `supersede_document` already established for
  documents, but no code path exercises it yet — no PB was renumbered
  during this session to drive it. Template exists; not yet instantiated.

**Mitigation:**
- When a PB is renumbered for real, add the same `supersede_document`-
  shaped method to `enkimdb::WriteNode` (targets the old PB's own Identity-
  Kaki, `pb.event = "SUPERSEDED"`/`pb.reason`/`pb.superseded_by`, a new
  `EventCause` variant) rather than inventing a different mechanism.
- Keep `enkiddb::DOCS_TRIBE_ID`/`PB_TRIBE_ID`'s single-source-of-truth
  pattern for any future tribe — one documented constant, reused by every
  caller that mints into it.

## References

- ADR-006: No-Delete + Mandatory Partitioning (`APPEND` supersedes; the
  mechanism Decision 2 implements)
- ADR-013: Rust Is the Permanent Substrate (query side corrected same day;
  the mint/emitter pattern `PbEmitter`/`DocumentEmitter` both follow)
- `crates/enkimdb/src/pb.rs`, `pb_emitter.rs`: `PbProfile`/`scan_pbs`/
  `PbEmitter` — the real PB-minting pipeline
- `crates/enkimdb/src/registry_emitter.rs`: `RegistryEmitter` — the shape
  `PbEmitter` mirrors
- `crates/enkiddb/src/emitter.rs` (`emit_supersession`), `writenode.rs`
  (`supersede_document`, `ingest_document_from_path`): the real document
  mint + supersession pipeline
- `crates/shakkanakku/src/docpulse.rs`: Uruinimgina's stage 4 (real mint +
  supersession + materialize) and stage 5 (gated official-repo landing)
- `crates/shakkanakku/src/pb_mint.rs`: the real PB corpus-scan trigger,
  wired into `runner.rs`
- `crates/enkidb-journal/src/event_cause.rs`: `EventCause::PbRegistered`
  (`0x76`), `EventCause::DocumentSuperseded` (`0x77`)
