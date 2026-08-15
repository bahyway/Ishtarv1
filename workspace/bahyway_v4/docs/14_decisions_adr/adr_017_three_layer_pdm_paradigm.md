# ADR-017 — The Three-Layer PDM Paradigm (PH-PDM-001)

> **DubSar Help** | `Decisions > ADR-017` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-17"
  concept_type:   "0x02"
  epoch:          "2026-08-01"
  concept_depth:  0
  riksu_count:    3
  snapshot_epoch: "2026-08-01"

concept:          "The Three-Layer Particles Data Model Paradigm"
summary:          "Business data of any domain is modelled in three composed layers: structural (TDA + simplicial complexes over particles -- discovers entities/relationships/integrity-gaps from data shape), quantitative (EAV/baru King-plot residual/fuzzy scoring -- supplies magnitude, which topology cannot see), and semantic (Sowa conceptual graphs + AAOL orchestration, plus the human steward -- supplies meaning and sequence, which topology also cannot see). The structural layer is realized by LamassuEngine reading bahyway-algebra's persistent homology over pdm-discovery's relationship complex; every proposal carries an honest tau-confidence and is always advisory, ratified by a human steward, never applied unilaterally."
sovereign_laws:   ["§PDM-THREE-LAYER -- a complete Particles Data Model composes structural (shape), quantitative (magnitude), and semantic (meaning/sequence) readings; no single layer claims to model data alone", "§PDM-HONEST-BOUNDARY -- the structural layer's tau-confidence must never be zero always (a model claiming to capture everything is false); it states precisely what each layer leaves uncaptured"]

riksu_bindings:
  - target: "adr_015_sla_supremacy_and_structural_amendment.md"
    concept: "GL-SLA-001 governs how a structurally-discovered proposal may amend an existing agreement -- this ADR is the paradigm that proposal comes from"
    type: "PEER"
  - target: "adr_016_model_as_particle.md"
    concept: "NL-MDL-001 -- a model discovered under this paradigm is itself a KAKI-bearing particle"
    type: "PEER"

orbit_tags:       ["PDM Discovery", "LamassuEngine", "bahyway-algebra", "pdm-discovery", "TDA"]
rag_keywords:     ["Three-Layer PDM Paradigm", "PH-PDM-001", "structural layer", "quantitative layer", "semantic layer", "Betti numbers", "beta_0", "beta_1", "beta_2", "tau confidence", "baru King-plot"]
-->

**Status:** Decision accepted 2026-08-01 — the paradigm's structural layer is already real (LamassuEngine + `bahyway-algebra::persistence` + `pdm-discovery`, all built earlier this session); this ADR formally names and seals the paradigm those crates already implement a piece of.
**Date:** 2026-08-01
**Author:** Bahaa Fadam
**Related:** ADR-015 (SLA Supremacy — governs how a structural proposal may amend an SLA), ADR-016 (Model-as-Particle — a discovered model's ontology)

---

## Context

An uploaded design tablet ("The Self-Modelling Data Trilogy") proposed naming
this ecosystem's existing structural-discovery capability — persistent
homology over a relationship-based simplicial complex, reading Betti numbers
as schema — as one of three composed layers of a general Particles Data
Model (PDM) paradigm. Evaluating it against the real repo found the
structural layer's own machinery already real and tested
(`bahyway-algebra::persistence`'s `vietoris_rips_persistence`/
`clique_complex_persistence`, `lamassu-engine`'s reading of it, and
`pdm-discovery`'s relationship-heuristic complex-builder, all built this
session) but one naming error in the source material worth correcting before
sealing: the tablet's "GeoEngine" does not exist as a crate anywhere in this
repo. The name was already used and superseded once, for an unrelated
concern (`playbook_93_geo_engine.yml`'s spatial-indexing/sharding engine,
absorbed into `enkidb-indexes::hepta_shell` — see
`playbooks/playbook_93_geo_engine_reconciled.yml`). This ADR names the real
crate that actually holds the GA/simplicial-complex/persistent-homology
mathematics: `bahyway-algebra`.

## Decision

Business data of any domain is modelled as a simplicial complex over
particles, whose invariant structure is read by persistent homology, and
composed with quantitative and semantic layers to form a complete, proposed
Particles Data Model. Topology and the complex are one object seen from two
sides: the complex is the shape built from data; homology is the measure of
that shape.

### The three layers

1. **Structural layer — discovery.** Particles form a complex by proximity
   in 7D Hepta Space (`pdm-discovery`'s relationship-heuristic
   complex-builder). Persistent homology, computed once by
   `bahyway-algebra::persistence`, extracts the Betti numbers as schema:
   - β₀ — connected components → how many distinct entity-clusters
     (tribes) truly exist.
   - β₁ — loops → relational cycles: circular references, cyclic
     dependencies, structures that break naïve hierarchical schemas.
   - β₂ — voids → structural holes: missing data, absent relationships,
     gaps the client did not know were there.

   `LamassuEngine` is the reader of this layer (see ADR-018, GL-TED-001):
   it points the persistent-homology instrument at a client's data-complex
   and translates the resulting Betti numbers into a discovered schema
   proposal. These invariants are emitted as HeptaScript tribe definitions
   and structural integrity constraints.

2. **Quantitative layer — magnitude.** Topology is blind to number. EAV
   attribute distributions, thresholds, arithmetic, the `baru` King-plot
   residual (`crates/acoustic-leak-engine/src/stage1_kingplot.rs` and its
   siblings), and fuzzy-logic scoring (`fuzzy-engine`) supply what shape
   cannot see, emitted as HeptaScript `PROVE` conditions and ETL
   validation rules.

3. **Semantic / sequential layer — meaning and order.** Topology sees two
   clusters, never that one means "customers" and the other "invoices";
   it sees structure, never step-order or causality. Sowa conceptual
   graphs (in EnkiDDB) and AAOL orchestration name the structures and
   sequence the workflow — and in practice, so does the human steward at
   approval.

### What the paradigm does and does not claim

The paradigm is not a theory that models all business data by topology
alone. The honest claim is that TDA and simplicial complexes provide the
structural layer of a universal PDM — the layer that discovers entities,
relationships, and integrity constraints from data shape — which composes
with the quantitative and semantic layers to form a complete model. The
transparency-deficit calculus τ is the standing proof of this humility: a
model claiming to capture everything would have τ = 0 always, which is
false. τ measures precisely what each layer leaves uncaptured.

### The stakeholder loop

On upload, the system emits not a verdict but a proposal — "here are the
entities I found, the relationships, the integrity gaps, and my confidence
in each" — which the stakeholder approves, edits, or rejects (see ADR-015's
Moment 1/2/3 governance of exactly this proposal). What the machine proposes
is not the correct model of the organisation but the model the data
currently implies; the difference is the whole value, for it surfaces
hidden structure (three customer-clusters where one was believed) so a
human can rule on reality rather than impose an assumption. Every proposal
arrives with its τ-confidence and an honest declaration of what was too
sparse to model.

## Consequences

- **Real today:** the structural layer (β₀/β₁/β₂ via `bahyway-algebra` +
  `pdm-discovery` + `lamassu-engine`, DubSar PDM IDE's Conceptual/
  Logical/Physical tabs).
- **Real today, separately governed:** the quantitative layer's individual
  components (`fuzzy-engine`, `score-engine`, the `baru`/King-plot residual
  crates) — not yet composed together into one PDM-paradigm pipeline.
- **Not yet built:** a single orchestrated pipeline that runs all three
  layers together and emits one composed proposal with one τ-confidence
  covering all three; the semantic layer's Sowa-graph naming step.
- **Correction carried forward:** any future document referencing
  "GeoEngine" for this ecosystem's GA/simplicial-complex/persistent-homology
  mathematics should read `bahyway-algebra` instead — the name is spent
  and superseded for an unrelated (spatial-indexing) purpose.

BahyWay.Ecosystem v4.0 — written by one scribe, sealed with one seal. 𒁾
