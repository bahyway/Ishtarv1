# ADR-016 — Model-as-Particle

> **DubSar Help** | `Decisions > ADR-016` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-16"
  concept_type:   "0x02"
  epoch:          "2026-07-31"
  concept_depth:  0
  riksu_count:    3
  snapshot_epoch: "2026-07-31"

concept:          "A discovered/approved PDM schema model is itself a KAKI-bearing particle"
summary:          "An approved discovered-schema proposal (pdm-discovery's SchemaProposal, once a stakeholder approves it in DubSar PDM's Logical tab) is not a transient UI value -- it becomes a particle: it gets a real Identity-Kaki, is saved as a Template in EnkiMDB (Templates are already particles in this ecosystem), is HeptaScript-queryable like any other particle, and every later amendment (per ADR-015's Moment 3) is an Event-Kaki APPEND on that same identity, never a silent overwrite. This is what makes a self-populating, portable 'template library' of discovered schemas possible across ecosystem instances -- the same ontological move this ecosystem already made for documents and playbooks (ADR-014), applied to discovered models."
sovereign_laws:   ["§MODEL-AS-PARTICLE -- an approved PDM schema model is minted a real Identity-Kaki at approval time and stored as an EnkiMDB Template, not left as transient application state", "§AMENDMENT-IS-APPEND -- every later revision to an approved model (ADR-015 Moment 3) is an Event-Kaki on the model's own Identity-Kaki, per ADR-006's no-delete law, never a new unrelated record"]

riksu_bindings:
  - target: "adr_015_sla_supremacy_and_structural_amendment.md"
    concept: "the SLA record ADR-015 governs is this ADR's particle"
    type: "PEER"
  - target: "adr_014_kaki_minted_at_authoring_time_pbs_and_docs.md"
    concept: "mint-at-authoring-time and supersede-as-APPEND pattern this ADR reuses verbatim for models"
    type: "GROUNDS"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI identity conventions this ADR's model particles follow"
    type: "GROUNDS"

orbit_tags:       ["PDM Discovery", "Templates", "EnkiMDB", "KAKI Sovereignty", "Model-as-Particle"]
rag_keywords:     ["model as particle", "SchemaProposal", "Template", "EnkiMDB", "Identity-Kaki", "Event-Kaki", "template library", "pdm-discovery", "DubSar PDM Logical tab"]
-->

**Status:** Decision accepted 2026-07-31 — the ontological law only; minting/storage code is NOT yet built (see Consequences)
**Date:** 2026-07-31
**Author:** Bahaa Fadam
**Related:** ADR-015 (SLA Supremacy — the lifecycle this particle is governed by), ADR-014 (the mint/supersede pattern this reuses), ADR-003 (KAKI sovereignty)

---

## Context

This session's evaluation of a proposed TDA+PDM data-modeling paradigm
(persistent homology, via `bahyway-algebra::persistence`, read over a
relationship-based simplicial complex built by the new `pdm-discovery` crate)
raised a specific ontological question: once a stakeholder approves a
discovered schema in DubSar PDM's Logical tab, what *is* that approved
object? Left as an in-memory UI value (which is exactly what this session's
`pdm_tab.gd::_approved_proposal` still is), it has no identity, no history,
and no way to be queried, compared across tenants, or reused. This ecosystem
already answered the equivalent question for documents and playbooks in
ADR-014: a real artifact gets a real Identity-Kaki the moment it becomes real,
and its later revisions are `APPEND` events on that same identity, never
orphaned or overwritten. This ADR is that same law applied to discovered
PDM models specifically — credited as the user's own insight in this
session's evaluation.

## Decision

### Decision 1 — An approved model gets a real Identity-Kaki at approval time

The moment a `SchemaProposal` is approved (DubSar PDM's Logical tab,
"Approve Selected"), it is minted a real Identity-Kaki — following
ADR-014 Decision 1's exact precedent (mint at the moment an artifact becomes
real, not deferred to first use).

### Decision 2 — Saved as a Template in EnkiMDB

Templates are already particles in this ecosystem (established prior to this
session). An approved `SchemaProposal` — its table list, its approved
relationships, and the persistence diagram's `component_count`/`void_count`
evidence behind it — is saved as an EnkiMDB Template under that Identity-Kaki,
not as a bespoke new storage shape.

### Decision 3 — HeptaScript-queryable

Because it is a Template particle, an approved model is queryable through
HeptaScript exactly like any other particle (`WHO`/`WHAT`/`PROVE`) — no
separate query surface is invented for "discovered models" specifically.

### Decision 4 — Amendments are Event-Kaki history, never overwrites

Per ADR-015's Moment 3 and ADR-006's no-delete law: when a model is amended
(a structural-drift proposal is DUAL-sealed and ratified), the amendment is
an `APPEND` — an Event-Kaki on the model's own Identity-Kaki, recording the
prior structure, the new structure, and why — following
`enkiddb::emitter::emit_supersession`'s exact shape (ADR-014 Decision 2)
rather than a new mechanism invented for models.

### Consequence of Decisions 1-4 together: a portable template library

Because an approved model is a KAKI-bearing, HeptaScript-queryable Template
with a real amendment history, it can in principle be compared across
ecosystem instances the same safe way `lamassu-engine::compare_readings`
already compares two `TribeReading`s without leaking raw KAKI bytes — a real,
buildable seed for a self-populating "discovered-schema template library"
across tenants, not a new federation mechanism invented here.

## Consequences

**Positive:**
- Gives discovered models the same permanent, queryable identity documents
  and playbooks already have (ADR-014), instead of a third, bespoke
  "how do we remember this" mechanism.
- Makes ADR-015's Moment 2/3 (conformance checking, amendment proposals)
  possible at all — you cannot diff a fresh discovery run against "the SLA"
  if the SLA has no persistent identity to diff against.

**Negative — named, not glossed over:**
- Nothing built this session actually mints an Identity-Kaki for an approved
  `SchemaProposal`, saves it as an EnkiMDB Template, or exposes it to
  HeptaScript. `pdm_tab.gd`'s "Approve Selected" button (this session's
  build) only populates an in-memory `_approved_proposal` Dictionary
  consumed by the Physical tab in the same running session — it is lost on
  close, has no identity, and cannot yet be compared across tenants or
  amended per ADR-015.

**Mitigation / real next step:**
- Add a mint step to `_on_logical_approve_pressed`'s real counterpart once a
  GDExtension bridge (or an intermediate CLI/server call, mirroring
  `bin/pdm-discover`'s own subprocess pattern) exists: call the same
  `KakiMinter`/`Template`-saving path `enkimdb::pb_emitter::PbEmitter` and
  `enkiddb::emitter::DocumentEmitter` already use (ADR-014), rather than
  inventing a third emitter shape for models.

## References

- `crates/pdm-discovery/src/lib.rs`: `SchemaProposal` — the value this ADR
  says must become a particle.
- `godot/dubsar-theater/scripts/pdm_tab.gd`: `_approved_proposal` — today's
  transient, non-particle stand-in.
- `crates/lamassu-engine/src/lib.rs`: `compare_readings`/`ShapeComparison` —
  the existing safe-comparison primitive a future template-library
  comparison would extend.
- ADR-014: KAKI Minted at Authoring Time (the mint/supersede pattern this
  ADR reuses in full).
- ADR-015: SLA Supremacy and Structural Amendment (the lifecycle this
  particle is governed by once it exists).
- ADR-006: No-Delete + Mandatory Partitioning (the APPEND mechanism
  Decision 4 requires).
- ADR-003: KAKI Sovereignty (identity/tribe conventions this ADR's model
  particles must follow once minted).
