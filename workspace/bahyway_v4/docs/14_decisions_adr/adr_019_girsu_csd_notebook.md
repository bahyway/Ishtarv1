# ADR-019 — The Girsu CSD Notebook (GL-CSD-001)

> **DubSar Help** | `Decisions > ADR-019` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-19"
  concept_type:   "0x02"
  epoch:          "2026-08-01"
  concept_depth:  0
  riksu_count:    3
  snapshot_epoch: "2026-08-01"

concept:          "The Girsu CSD Notebook -- critical slowing down as a sovereign data-science instrument"
summary:          "A Girsu notebook (.akknb) is a data-science surface for finding critical slowing down (CSD) across BahyWay's big data, governed by one inversion: the notebook cell is a sealed request, never a computation. A HeptaScript cell expresses an analysis; NinurtaEngine's Purussum Calculus executes it against the data where it lives; the cell displays the returned verdict and its tau-confidence. This ADR corrects two things found while evaluating the source tablet against the real repo: the pipeline's own cited HeptaScript WINDOW/DETREND clause and HS-EXT-001 source document do not exist (WITNESS and SYNC are real tokens; WINDOW/DETREND/OBSERVE are not, and are deferred -- see playbook_274); and the pipeline's 'ShamashEngine (statistics)' component is NinurtaEngine's own Purussum Calculus, not a separate engine (ADR-018)."
sovereign_laws:   ["§CSD-INVERSION -- the notebook cell is a sealed request, never a computation; the pure-Rust engine estate performs the analysis, the cell renders the returned verdict", "§CSD-HONESTY -- detrend before computing any indicator; never trust variance alone (population-size artifact); never alarm on a rising lambda without surrogate significance (p < 0.05); the engine emits rising instability with confidence, never a scheduled transition"]

riksu_bindings:
  - target: "adr_018_topological_engine_division.md"
    concept: "GL-TED-001 -- NinurtaEngine and the Purussum Calculus this notebook's cells request"
    type: "GROUNDS"
  - target: "adr_017_three_layer_pdm_paradigm.md"
    concept: "PH-PDM-001 -- the structural layer LamassuEngine reads, cross-checked against Ninurta's beta_1-tightening signal"
    type: "PEER"

orbit_tags:       ["Girsu IDE", ".akknb", "NinurtaEngine", "HeptaScript", "CSD", "Purussum Calculus"]
rag_keywords:     ["Girsu CSD Notebook", "GL-CSD-001", "sealed request", "critical slowing down", "restoring rate lambda", "Fourier surrogate", "tau confidence", "PIK Potsdam", "Ben-Yami Skiba Bathiany Boers", "HS-EXT-001 correction"]
-->

**Status:** Decision accepted 2026-08-01 as a design tablet — the notebook surface itself (`.akknb` cell → NinurtaEngine wiring, the HeptaScript pipeline grammar) is NOT yet built (see Consequences); the calculus it would call (`ninurta-engine::render_verdict`) is real and tested today.
**Date:** 2026-08-01
**Author:** Bahaa Fadam
**Related:** ADR-018 (Topological Engine Division — NinurtaEngine, the Purussûm Calculus), ADR-017 (Three-Layer PDM Paradigm)

---

## Context

An uploaded design tablet proposed the Girsu `.akknb` notebook as a
sovereign data-science surface for CSD analysis, grounded in Ben-Yami,
Skiba, Bathiany & Boers (Nature Communications, 2023, PIK-Potsdam — an
open-access, rigorous source, read in full while evaluating this tablet).
Two claims in the source material needed correcting against the real repo
before sealing:

1. **The cited HeptaScript `WINDOW`/`WITHIN` clause, and its cited source
   document `HS-EXT-001`, do not exist.** Checked directly against
   `crates/heptascript/src/token.rs`: `PROVE`, `WITNESS`, and `SYNC` are
   real tokens; `WINDOW`, `WITHIN`, `DETREND`, and `OBSERVE` are not, and
   no `HS-EXT-001` document exists anywhere in this repo. This is not the
   first time an unverified citation has appeared in this design-tablet
   lineage — `playbooks/playbook_160_tpl_001_section_e_corrected.yml`'s
   own header documents catching and removing an earlier fabricated
   citation ("PB-152 / ENLIL HotIndex / 2.41B-per-sec... does not exist
   anywhere in the repo"). The grammar extension this pipeline needs is
   real, buildable work — just not yet built (deferred to
   `playbooks/playbook_274_heptascript_window_detrend_gap.yml`).
2. **The pipeline's "ShamashEngine (statistics)" is corrected to
   NinurtaEngine.** Per ADR-018: Shamash already names this ecosystem's
   real Gate 4 (State Exclusion); the statistics role is NinurtaEngine's
   own Purussûm Calculus, not a fourth engine.

## Decision

### The governing principle

A Girsu notebook (`.akknb`) is a data-science surface for finding critical
slowing down (CSD) — the measurable signature of a tribe, orbit, or
BIGRING approaching a critical transition — across BahyWay's big data. Its
architecture obeys one inversion: **the notebook cell is a sealed request,
never a computation.** A HeptaScript cell expresses an analysis; the
pure-Rust engines execute it against the data where it lives; the cell
displays the returned result and its τ-confidence.

| Layer | Role |
|---|---|
| Notebook cell (`.akknb`) | Authoring + witnessing. Expresses the CSD request in HeptaScript; renders the result and its τ-score. Holds no compute. |
| HeptaScript | The sovereign request language — real tokens `WITNESS`/`SYNC`/`PROVE` today; `WINDOW`/`DETREND` deferred (see Context). |
| `NinurtaEngine` | The kernel for this notebook's requests: the Purussûm Calculus (`crates/ninurta-engine`) — detrend, λ, corroborating indicators, Fourier-surrogate significance. Calls `bahyway-algebra` (math) and `LamassuEngine` (β-topology) per ADR-018's division. |
| Seven EnkiDB types | The substrate. EnkiDW holds the time-history; the analysis reads from it; alerts `EMIT` back as particles. |

The notebook is a sealed-request surface. Saved, it is itself a Template
particle (ADR-016 / NL-MDL-001): Identity-KAKI, history, reusable — the
investigation remembered.

### The CSD indicators (what NinurtaEngine computes)

Per particle-collection, over a sliding time window on a scalar observable
(mean EAV quality, orbit radius, β₁ count, tribe density) —
`ninurta_engine::indicators`/`purussum` implement all three, real and
tested:

- **Restoring rate λ** — the preferred, robust indicator
  (`indicators::restoring_rate`). Negative for a stable state; rises
  toward 0 as a transition approaches.
- **Rising variance** — treacherous alone (`indicators::variance`); can
  shift from changing particle-count, faking a signal. Never trusted in
  isolation.
- **Rising lag-1 autocorrelation** — corroborates λ
  (`indicators::lag1_autocorrelation`).

Mapping to the EAV trichotomy (`purussum::LambdaTrichotomy`): a GOLDEN
tribe sits at stable λ≪0; λ rising toward 0 is the drift into FUZZY (the
bifurcation edge); the transition itself risks the DEAD fixed point.

### The honesty machinery (non-negotiable)

1. Detrend first (`detrend::running_mean_detrend`) — or the mean trend
   contaminates the variance.
2. Guard the population artifact — a tribe whose particle-count is
   growing shows variance changes that are artifacts of size, not
   slowing down. λ resists this; variance does not.
3. Significance by surrogates, never eyeballing
   (`surrogate::fourier_surrogates` + `surrogate_p_value`) — alarm only
   if the real λ-trend exceeds `surrogate_count` (default recommendation:
   1000) Fourier surrogates more than 5% of the time (p < 0.05,
   `purussum::SIGNIFICANCE_THRESHOLD`). The surrogate p-value IS the
   τ-score on the alert.

CSD is probabilistic evidence of destabilisation, not a schedule. The
engine emits "rising instability, confidence τ" — never "transition at
tick T." `PurussumVerdict.significant` requires BOTH the direction and the
honesty gate to agree — this is real, tested code, not aspiration.

### The dual alarm (why BIGRING is special)

A BIGRING gives two independent transition detectors, and their agreement
is the conservative, low-false-positive posture the PIK paper models: the
statistical signal (λ→0, `NinurtaEngine`, reading the ring's radius/density
observable) and the topological signal (β₁ loop tightening,
`LamassuEngine`). **Not yet built**: the actual cross-check wiring — today
`ninurta-engine` operates on a caller-supplied scalar series and does not
itself read `LamassuEngine`'s β₁ output.

## Consequences

- **Real today:** the full Purussûm Calculus (`crates/ninurta-engine`,
  23/23 tests) — anyone can call `render_verdict` directly with a real
  time series and get a real, honest verdict.
- **Not yet built:** the `.akknb` cell UI wiring a HeptaScript request to
  a `render_verdict` call; the HeptaScript `WINDOW`/`DETREND` grammar
  tokens (deferred, `playbook_274`); the LamassuEngine β₁-tightening
  cross-check wiring; the `EnkiODB` `EMIT csd_alert` write-back path.
- **Correction carried forward:** "ShamashEngine" → NinurtaEngine
  throughout; the `WINDOW`/`WITHIN` clause and `HS-EXT-001` are not real
  and must not be cited as already-sealed in future documents.

BahyWay.Ecosystem v4.0 — written by one scribe, sealed with one seal. 𒁾
