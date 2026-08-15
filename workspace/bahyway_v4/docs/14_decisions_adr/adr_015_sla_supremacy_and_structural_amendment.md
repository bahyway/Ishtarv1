# ADR-015 — SLA Supremacy & Structural Amendment

> **DubSar Help** | `Decisions > ADR-015` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-15"
  concept_type:   "0x02"
  epoch:          "2026-07-31"
  concept_depth:  0
  riksu_count:    3
  snapshot_epoch: "2026-07-31"

concept:          "SLA Supremacy and Structural Amendment Governance Law"
summary:          "A discovered PDM schema (from pdm-discovery's relationship-based complex-builder) never silently overrides an existing agreed SLA/schema. Three governed moments: Moment 1 (no SLA yet) lets discovery write the first draft; Moment 2 (SLA holds) enforces conformance, classifying non-conformers FUZZY rather than rejecting them outright; Moment 3 (SLA exists but data has structurally drifted) requires discovery to propose a DUAL-sealed amendment -- never a unilateral rewrite. Modeled directly on this ecosystem's existing CSR-08 (Architect/DataSteward ratification for high-impact changes) and the death_legacy.rs precedent that TDA/discovery always advises, never rules."
sovereign_laws:   ["§SLA-SUPREMACY -- an approved SLA/schema is authoritative; a fresh discovery run's proposal is advisory input, never a silent override", "§STRUCTURAL-AMENDMENT -- when discovery detects coherent structure that conflicts with the current SLA, it proposes a DUAL-sealed amendment (both the discovery run and a human Steward/Architect ratification), never applies one unilaterally"]

riksu_bindings:
  - target: "adr_014_kaki_minted_at_authoring_time_pbs_and_docs.md"
    concept: "Model-as-Particle mint/supersede pattern this ADR's Moment 3 amendment reuses"
    type: "PEER"
  - target: "adr_016_model_as_particle.md"
    concept: "the SLA record this ADR governs is itself a Model-as-Particle"
    type: "GROUNDS"
  - target: "adr_006_no_delete_mandatory_partitioning.md"
    concept: "an amendment is an APPEND on the SLA's own Identity-Kaki, never a delete/overwrite"
    type: "GROUNDS"

orbit_tags:       ["PDM Discovery", "LamassuEngine", "bahyway-algebra", "CSR-08", "Governance"]
rag_keywords:     ["SLA supremacy", "structural amendment", "pdm-discovery", "clique_complex_persistence", "Moment 1", "Moment 2", "Moment 3", "DUAL-sealed", "CSR-08", "DubSar PDM Logical tab"]
-->

**Status:** Decision accepted 2026-07-31 — governance law only; the SLA-record/amendment machinery it governs is NOT yet built (see Consequences)
**Date:** 2026-07-31
**Author:** Bahaa Fadam
**Related:** ADR-016 (Model-as-Particle — the SLA record itself), ADR-006 (no-delete/APPEND), `crates/bahyway-core/src/death_legacy.rs` (CSR-08 precedent)

---

## Context

This session evaluated a document proposing that DubSar PDM's schema-discovery
paradigm (persistent homology over a relationship-based simplicial complex,
via `bahyway-algebra::persistence` + the new `pdm-discovery` crate, both
built this session — see ADR context below) needs a governance rule for what
happens when a *fresh* discovery run's proposed schema conflicts with an
*already-agreed* SLA/schema for the same data.

Without such a rule, "discovery" risks becoming silent authority: a stakeholder
approves a schema once, and every subsequent re-run could quietly propose a
different one with no record of why, or worse, could be wired to apply
automatically. That is exactly the failure mode this ecosystem has already
named and rejected elsewhere: `death_legacy.rs`'s own precedent is that
TDA/LamassuEngine's topological read is always advisory — "the disposition
itself is the verdict," never the algebra. CSR-08 generalizes this further:
high-impact changes always wait for Architect/DataSteward ratification: real
code, `crates/nisaba/src/autonomy.rs`, gates exactly this for NISABA's
findings today. This ADR is the same law, named for the PDM-discovery
context specifically.

## Decision

Three governed moments, keyed on whether an SLA (an approved schema-discovery
proposal, per ADR-016) already exists for the data in question:

### Moment 1 — No SLA yet: discovery writes the first draft

When no prior SLA exists, a fresh `pdm-discovery::discover_schema` run's
`SchemaProposal` is the seed for one. It still goes through the Logical tab's
review/approve step (built this session — `pdm_tab.gd`'s
`_on_logical_load_pressed`/`_on_logical_approve_pressed`) before becoming
authoritative; discovery drafts, a human approves.

### Moment 2 — SLA holds: conformance is enforced, not rejected outright

Once an SLA exists, new data is judged against it. Data that conforms passes.
Data that does not conform is **not** silently rejected or silently folded
in — it is classified FUZZY (the same three-value vocabulary
`lamassu-engine::TopologicalSignature` already uses for "a seam, not yet a
pattern"), surfaced for review, never auto-corrected and never auto-ignored.

### Moment 3 — SLA exists but data has structurally drifted: propose, never impose

When enough new data structurally disagrees with the current SLA that a
fresh discovery run detects a *coherent* alternative structure (not just
FUZZY noise), discovery's role is to **propose a structural amendment** —
never to apply one. Per §STRUCTURAL-AMENDMENT: an amendment proposal must be
DUAL-sealed — both the discovery run's own evidence (the new
`SchemaProposal`, including its relationship diffs against the current SLA)
and a human Steward/Architect ratification (CSR-08) — before the SLA's
record is updated. Consistent with ADR-006's no-delete law and ADR-014's
supersession pattern: an amendment is an `APPEND` event on the SLA
particle's own Identity-Kaki (recording old shape, new shape, and why), never
a delete-and-replace.

## Consequences

**Positive:**
- Gives PDM discovery a precise vocabulary for "discovery disagrees with what
  we agreed" that doesn't collapse into either "ignore the disagreement" or
  "let the algorithm win" — both real failure modes discovery-driven schema
  systems can fall into.
- Directly reuses two laws this ecosystem already enforces elsewhere
  (CSR-08 ratification, ADR-006 append-not-delete) instead of inventing a
  parallel governance mechanism for PDM specifically.

**Negative — named, not glossed over:**
- This ADR records the **law**, not its enforcement machinery. As of this
  session, nothing in `pdm-discovery` or DubSar PDM's Logical/Physical tabs
  actually stores an "approved SLA" as a persistent, versioned record, diffs
  a fresh `SchemaProposal` against a prior one, or drives a DUAL-sealed
  ratification flow. The Logical tab's `_approved_proposal` (this session's
  build) is in-memory only and always treated as Moment 1 — it does not yet
  know how to be Moment 2 or Moment 3.

**Mitigation / real next step:**
- Model an `SlaRecord` as a real Model-as-Particle (ADR-016): mint an
  Identity-Kaki on first approval (Moment 1), diff subsequent
  `SchemaProposal`s against the stored record on every discovery re-run, and
  when the diff is a coherent structural change rather than noise, emit a
  proposed amendment event rather than mutating the record — following
  `enkiddb::emitter::emit_supersession`'s exact shape (ADR-014, Decision 2)
  rather than inventing a new mechanism.

## References

- `crates/bahyway-algebra/src/persistence.rs`: `clique_complex_persistence`,
  `void_count` — the H2/void math this session added, which Moment 1's
  discovery drafts are built from.
- `crates/pdm-discovery/src/lib.rs`: `discover_schema`, `SchemaProposal`,
  `detect_join_keys` — the v1 relationship-heuristic complex-builder this
  ADR's Moment 1 depends on.
- `godot/dubsar-theater/scripts/pdm_tab.gd`: the Logical tab
  (`_on_logical_approve_pressed`) — today's Moment-1-only approval step.
- `crates/bahyway-core/src/death_legacy.rs`: the "advisory, never automatic"
  precedent this ADR's Moment 3 is modeled on.
- `crates/nisaba/src/autonomy.rs`: CSR-08 — real, working Architect-
  ratification gating for high-impact findings.
- ADR-006: No-Delete + Mandatory Partitioning (the APPEND mechanism Moment
  3's amendment must use).
- ADR-014: KAKI Minted at Authoring Time (the mint/supersession pattern
  `SlaRecord` amendments should mirror).
- ADR-016: Model-as-Particle (what an SLA record actually is, ontologically).
