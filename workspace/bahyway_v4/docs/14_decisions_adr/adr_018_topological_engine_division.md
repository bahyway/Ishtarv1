# ADR-018 — The Topological Engine Division (GL-TED-001)

> **DubSar Help** | `Decisions > ADR-018` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-18"
  concept_type:   "0x02"
  epoch:          "2026-08-01"
  concept_depth:  0
  riksu_count:    4
  snapshot_epoch: "2026-08-01"

concept:          "Topological Engine Division: bahyway-algebra, LamassuEngine, NinurtaEngine"
summary:          "Three engines read one topological substrate, single-purpose in duty, unified in Triple-O citizenship. bahyway-algebra owns the mathematics (GA, simplicial complexes, persistent homology -- computed once). LamassuEngine reads a STATIC complex for structure (beta_0/beta_1/beta_2 as entities/relations/voids -- the PH-PDM-001 structural layer). NinurtaEngine reads a complex IN MOTION for transition, via the Purussum Calculus (restoring rate lambda, corroborating indicators, Fourier-surrogate significance, beta_1-tightening) -- rendering a tau-scored verdict, never a scheduled transition. This ADR also seals two naming corrections found while evaluating the source design tablets: 'GeoEngine' was never a real crate (already spent, superseded, for an unrelated spatial-indexing concern); a proposed fourth 'ShamashEngine (statistics)' component collides with the already-sealed SHAMASH Gate 4 (State Exclusion) and is folded into NinurtaEngine itself, matching this ADR's own three-way division."
sovereign_laws:   ["§TED-DIVISION -- bahyway-algebra owns mathematics, LamassuEngine owns static structure, NinurtaEngine owns transition; no engine reimplements what another owns", "§TED-CITIZENSHIP -- every engine that touches shape is a Triple-O (PH-001) citizen; single-responsibility separates duty, Triple-O unifies nature", "§TED-NAMING -- GeoEngine does not exist as a crate and must not be revived under that name (spent, superseded into enkidb-indexes::hepta_shell); the Purussum Calculus is NinurtaEngine's own, never a separate fourth engine"]

riksu_bindings:
  - target: "adr_017_three_layer_pdm_paradigm.md"
    concept: "PH-PDM-001's structural layer IS LamassuEngine's reading under this division"
    type: "GROUNDS"
  - target: "adr_015_sla_supremacy_and_structural_amendment.md"
    concept: "GL-SLA-001's Moment 3 (structural drift -> amendment) is the governance law NinurtaEngine's verdict is the concrete detection mechanism for"
    type: "PEER"

orbit_tags:       ["bahyway-algebra", "LamassuEngine", "NinurtaEngine", "Purussum Calculus", "Triple-O", "TDA"]
rag_keywords:     ["Topological Engine Division", "GL-TED-001", "Purussum Calculus", "restoring rate lambda", "critical slowing down", "Fourier surrogate", "Shamash Gate 4", "GeoEngine correction", "Ninurta", "beta_1 tightening"]
-->

**Status:** Decision accepted 2026-08-01 — `ninurta-engine` built and tested this session (23/23 tests passing, real detrend/lambda-regression/Fourier-surrogate math); GL-SLA-001 Moment 3's own drift-detection wiring into `sla-engine` is NOT yet built (see Consequences).
**Date:** 2026-08-01
**Author:** Bahaa Fadam
**Related:** ADR-017 (Three-Layer PDM Paradigm), ADR-015 (SLA Supremacy, Moment 3)

---

## Context

Three uploaded design tablets ("The Self-Modelling Data Trilogy," "The Girsu
CSD Notebook," "The Topological Engine Division") proposed a critical-
slowing-down (CSD) / bifurcation-detection engine named `NinurtaEngine`,
computing a "Purussûm Calculus," sitting alongside a proposed `GeoEngine`
(math substrate) and a proposed `ShamashEngine` (statistics). Evaluating
this against the real repo found two naming problems worth correcting
before sealing anything:

1. **`GeoEngine` is not a live crate.** `playbook_93_geo_engine.yml`
   (2026-06-25) did scaffold one, under the header "GeoEngine: Sovereign
   Geometry Engine — 7 GeoLaws + 4 Shards + UnifiedParticleRegistry +
   InFlight + GeometryFanOut" — but that is a *spatial-indexing* concern
   (HeptaMap E7 zones/shards/particle-registry fanout), unrelated to GA/
   simplicial-complex/persistent-homology math. The reconciled playbook
   (`playbooks/playbook_93_geo_engine_reconciled.yml`) confirms it was
   **superseded and absorbed into `enkidb-indexes::hepta_shell`**, real
   and current. The GA/persistent-homology mathematics the tablets wanted
   to attribute to "GeoEngine" already lives, real and tested, in
   `bahyway-algebra` (`clifford.rs`, `persistence.rs`). Reusing "GeoEngine"
   for that role would be a second collision on an already-spent name.
2. **`ShamashEngine` collides with a real, sealed name.** SHAMASH is Gate
   4 of this ecosystem's 7 Primary Gates — "State Exclusion," the
   zombie/reincarnation judgment on Dead particles
   (`docs/04_gates/shamash_gate.md`) — unrelated to statistics. The Girsu
   CSD Notebook tablet's own layer table names it as a fourth pure-Rust
   engine ("ShamashEngine (statistics), GeoEngine (math truth),
   LamassuEngine (β-topology)"), which is also inconsistent with the
   Topological Engine Division tablet's own stated three-way division
   (bahyway-algebra/LamassuEngine/NinurtaEngine, no fourth engine) — the
   Purussûm Calculus (λ, surrogate significance, detrending) is already
   explicitly assigned to NinurtaEngine in that same source. This ADR
   resolves the inconsistency the tablets themselves contained: the
   statistics role is NinurtaEngine's own, not a fourth engine.

`Ninurta` and `Purussum` were checked directly against
`crates/naming-registry`'s full seeded list (every name currently in use in
this ecosystem, per NL-001) before sealing — both unspent, no collision.

## Decision

### Founding clause — Triple-O citizenship

Before any division of labour: every engine that touches shape is a citizen
of Triple-O (PH-001). Single-responsibility separates *what each engine
does*; Triple-O governs *what everything is*. These operate at different
levels of the constitution, so there is no contradiction — the engines are
single-purpose in function and identical in citizenship.

### The division — one substrate, three readers

| Engine | Owns | Reads / Question |
|---|---|---|
| `bahyway-algebra` | the mathematics | Single source of mathematical truth: geometric algebra (`clifford.rs`), simplicial-complex construction and persistent-homology computation (`persistence.rs`). Computed once, correctly, in one place. Neither reader below reimplements it — both call it. |
| `LamassuEngine` | structure | Reads a STATIC complex → discovers structure. β₀/β₁/β₂ as entities/relational loops/voids. Question: *what shape does this data have?* The PDM structural-discovery layer (ADR-017) IS a Lamassu reading. |
| `NinurtaEngine` | transition | Reads a complex IN MOTION → detects change of regime, via the **Purussûm Calculus**: restoring rate λ, variance and lag-1 autocorrelation as corroborators, detrending, Fourier-surrogate significance testing, β₁-tightening cross-check. Question: *is this shape about to break?* Renders a τ-scored verdict — real, tested code as of this ADR: `crates/ninurta-engine`. |

Lamassu and Ninurta read the same tools — GA, simplicial complexes, TDA,
β₁ — on different objects for different purposes: Lamassu asks *what is
the shape*, Ninurta asks *is the shape about to break*. Two engines sharing
one mathematical library, never one engine doing both.

### The fusion — a self-monitoring data model

`bahyway-algebra` builds the client's data-complex over particles and
computes its homology once, as mathematical truth. `LamassuEngine` reads
the static complex and proposes the PDM (ADR-017's structural layer).
`NinurtaEngine` watches that SAME tribe's complex evolve and warns — via
the Purussûm Calculus — when its structure is destabilising. This is the
concrete mechanism behind ADR-015's (GL-SLA-001) Moment 3: "detect that the
SLA may need renegotiation." Ninurta is the organ that detects structural
drift before it breaks the sealed agreement — not yet wired to
`crates/sla-engine` (see Consequences), but the detection math itself is
real.

### The standing humility

TDA reads structure and its change — powerful, but blind (per ADR-017) to
magnitude, sequence, and meaning. Ninurta's verdict is structural/
dynamical only: it must compose with the quantitative layer (actual λ
magnitudes, `baru` residuals) and is always τ-scored (the Fourier-surrogate
p-value), never presented as the whole truth of why a model destabilises.
`ninurta_engine::purussum::PurussumVerdict.significant` is `true` only when
BOTH the direction (λ rising) and the honesty gate (p < 0.05) agree — this
is load-bearing in the real code, not a documentation aspiration.

## Consequences

- **Real today:** `crates/ninurta-engine` — detrending, λ-regression
  (OLS), lag-1 autocorrelation, Fourier-surrogate generation (direct DFT,
  zero external dependencies) and significance testing, the composed
  `render_verdict` pipeline, the GOLDEN/FUZZY/DEAD trichotomy mapping.
  23/23 tests passing, including a deterministic destabilizing-series
  fixture (lambda trending toward 0) that correctly triggers a positive
  `lambda_trend`, and a stable-series fixture that correctly does not.
- **Not yet built:** the β₁-tightening cross-check against `LamassuEngine`
  (`ninurta-engine` today operates on a caller-supplied scalar time
  series; wiring it to read `LamassuEngine`'s own β₁ readings over time
  is real follow-on work); the HeptaScript pipeline surface (`WITNESS` /
  `SYNC` are real tokens, but `WINDOW`/`DETREND` are not — see
  `playbooks/playbook_274_heptascript_window_detrend_gap.yml`); the
  Girsu `.akknb` notebook cell → `render_verdict` wiring; the ADR-015
  Moment 3 drift-detection wiring into `crates/sla-engine`.
- **Naming corrections carried forward:** "GeoEngine" → `bahyway-algebra`
  everywhere in this ecosystem's design documents; "ShamashEngine
  (statistics)" → `NinurtaEngine`'s own Purussûm Calculus, never a
  separate engine.

BahyWay.Ecosystem v4.0 — written by one scribe, sealed with one seal. 𒁾
