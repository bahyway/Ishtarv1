# GL-DW-001 — The Orbits Schema (EnkiDW)
## Tribes and Orbits — a warehouse where the fact owns its dimensions

**Ecosystem:** BahyWay.Ecosystem v4.0 — EnkiDW temporal warehouse schema
**Named contrast:** Kimball, Inmon, Data Vault, Anchor — the incumbents version *rows*; the Orbits Schema versions the *membrane*.
**Consumes / relates:** GL-BRT-001 (Birth Gate), GL-ONT-002 (Non-Substitution), GL-AGE-001 (Šību), GL-TPL-001 (Pattern Minting), GL-STY-001 (StoryEngine), GL-NRG-001 (Nergal), GL-EE-001 (Enūma Eliš), GL-VIZ-001 (Nasaru Style)
**Status:** SEALED — CSR-08 confirmed by the Architect. This is the final tablet, not a draft. Its three open seams (§13) are sealed **as open** — recorded, not hidden.

---

## 1. The Schema in one sentence

> **A Tribe is a fact; its Orbits are its dimensions. Each snapshot from EnkiDB — a partition of Golden Particles — becomes a new Orbit: a temporal shell around the same Tribe. Particles never move in HeptaSpace; the membrane expands as Orbits accrete. History is a stack of Orbits read radially, not a chain of row-versions resolved by join.**

The incumbents answer "how do you record change over time?" by mutating or versioning rows (SCD-2 dimensions, satellites with load-dates, anchor timelines). The Orbits Schema refuses the premise: a particle's Hepta coordinate is immutable, so you never version the particle — you add an Orbit. Append-only history falls out for free; a point-in-time question reads one Orbit instead of resolving version chains.

## 2. Identity — one KAKI, many Orbit-states (Seam 1, sealed)

A Golden Particle is **one immutable identity (KAKI, forever) witnessed across multiple temporal Orbits.** One HeptaSpace address; many Orbit-states over time.

**Not a Data Vault satellite**, despite the visual resemblance, for two structural reasons: (1) **no links** — the particle is never decomposed into hub+satellites, so there is nothing to re-join; the satellite's link exists to repair a decomposition never performed here. (2) **living, not dead** — a satellite row is a closed, load-dated, inert record; an Orbit-state is a *living relation* whose radius-to-Tribe keeps changing as new Orbits accrete. A satellite is a photograph; an Orbit-state is a body still in motion.

## 3. The Law of Singularity — the membrane moves, not the particle

> **The Golden Particle never moves through HeptaSpace. The membrane (the Tribe field) expands as new Orbits accumulate, increasing the particle's radius-to-the-Tribe even though the particle is absolutely stationary.**

This is the **comoving-coordinate** picture, borrowed as a *structural* analogy (not the physics): in cosmology, objects hold fixed comoving coordinates while physical distances grow with the scale factor. Here: the **Hepta coordinate is the fixed comoving address** (immutable, sealed at birth); the **radius-to-Tribe is a relational standing (Šību)**, not a spatial coordinate. Space never changes; standing does. The motion is the metric's, not the particle's. (The radius law of the Nergal courts — "the sphere bulges, every Hepta anchor stays nailed" — is this same invariant applied to threat instead of time.)

## 4. Šību in Hubble form

```
Šību(particle) = r_comoving × scale_factor
scale_factor grows by a tunable coefficient per Orbit of expansion
```

Uniform expansion (one scale factor for the whole Tribe) produces **radius-proportional aging** (recession ∝ distance) — exactly Hubble's law. The same expanding membrane keeps the core living and pushes the edge into aging; the divergence is **emergent under one law, not legislated per particle.**

## 5. The two radial conditions (one law, two fates)

- **Condition 1 — short inner radius (the heart).** Expansion has little radius to multiply → ages **asymptotically slowly**, remains a living central member. Only the exact centroid is truly ageless. *Inner ages slowest, not never.*
- **Condition 2 — increasing outer radius (the edge).** Expansion multiplies a large radius → Šību climbs past threshold → the particle **juggles** → authority alert (GL-AGE-001 routing: Elder / Obsolete / Quarantine).

The **acceleration of Šību** (the W-rule / bell-on-acceleration from Nergal) is an early warning: it rings on growth-*rate* spikes (unusual ingest), stays quiet during steady aging.

## 6. The aging quantum — 7 particles = 1 PU

**7 particles = 1 Particle Unit (PU)** — Hepta-law, the ingest tick. Aging is a **deterministic logical clock**: a particle's age is a pure function of PU accumulated since birth — no wall-clock, no randomness, fully replayable. The expansion coefficient is a **tunable constant**, not fundamental (§13 seam 3).

## 7. Death by decree — never by distance

A particle is **Dead only when the authority adjudicates it terminal** (Elder-archived / Obsolete-forked / Quarantined) — never by distance alone. Birth is decreed at the Ṣīt Gate; death is decreed at the aging gate. **Expansion proposes decay; the authority confirms it.** Everything short of a death-verdict is analyzable by TDA / PH / HeptaScript.

## 8. The lifecycle valves — PU, OU, and the three orthogonal axes

- **Snapshot/partition trigger:** **N PU (load) OR cron, whichever fires first.** The load-trigger guards against a dead cron; the cron guards against low-volume staleness. The departing partition **becomes** a new EnkiDW Orbit — *the relief valve that unburdens EnkiDB is the same valve that builds EnkiDW.* Append-only never bottlenecks, because the hot region stays bounded.
- **Archive trigger — Orbit Unit (OU):** the **maximum live Orbits per Tribe.** When exceeded, the **oldest Orbits roll to the cold Archive Storage Sector** — compressed **Journal** format on SSD. Archived = **cold-but-recoverable, NEVER deleted.** OU bounds the *hot* warehouse; the archive is cold-but-retrievable-by-decree.
- **Three orthogonal axes:** **PU** governs Orbit *birth*; **OU** governs Orbit *retirement*; **Šību** governs particle *aging*. Archiving an Orbit does **not** age its particles; aging a particle does **not** archive its Orbit. Warehouse-tiering and particle-standing are separate axes.

**Interpretation rules (against skew):** Orbit count reflects *accumulated load*, not age or importance; live-Orbit count reflects *activity*, not seniority. Normalize by Tribe size before comparing across Tribes. N and OU are best expressed per-Tribe.

## 9. The archive round-trip — two doors, one guardian

The Birth Gate has **two doors:**
- **Birth door** — new records; the Ṣīt Gate adjudicates; KAKI is minted.
- **Return door** — archived particles returning (e.g. a search reaches cold data); **verify-only; the original KAKI is preserved; NEVER re-minted.**

**Nergal guards both doors: it can reject, it cannot re-identify.** A returning particle passes BeeMDM for **integrity-check (checksum / medium-degradation — near-mandatory, cheap)** and **SHAPE-reconciliation (conditional — on version-mismatch or by access-control policy)**. It may be *refused* on return (corruption / irreconcilable SHAPE → quarantine) but never *re-identified*. **Verification is a gate function, not a minting function** — this is what preserves the Law of Singularity through the archive.

*Cost shape:* hot-Orbit search is fast; archived-Orbit search pays a **thaw-and-verify cost**. Optionally keep a lightweight hot index (KAKI + coordinate + minimal metadata) of archived particles, so search can *find* them fast and pay the thaw cost only to retrieve the full body.

## 10. The query model — visual before logical

HeptaScript queries over the Orbits Schema have **no subquery, no recursion, no hierarchy** — because those are the vocabulary of relational/tree data, and this is a *space*. The natural question over a space is **"what is the shape of this region?"** — a geometric function over a simplicial complex, not relational algebra.

- **Scope-to-an-Orbit** = layer selection (which Orbit's particle-set the shape is evaluated against). The shape is the same; the Orbit is the layer.
- **Standing-at-N** = a **k-NN local-density** reading over Orbit N's complex (§13 seam 2 — centroid dissolved in favour of local density; no center required).
- **Compare-standings-across-Orbits** = drop the *same shape* on Orbit N and Orbit N+1; compare topological readings (Betti, membership, local-density). Comparison-by-overlay, not by join.

**The query surface:** the stakeholder *draws the shape* in DubSar PDM (or picks a Template from the Arsenal), and asks "which particles are in this shape, in this Orbit?" **Querying in BahyWay is visual before logical, and logical only if the visual is correct.**

## 11. The visual IS the validation — the story-validator loop

> **Logical execution is authorized only against a visually-confirmed shape.** This is the query layer's CSR-08: the stakeholder confirms by *seeing*, then the engine acts.

The loop: **draw a shape (PDM) → see the caught particles (Nasaru) → left-click a particle: it bounces and opens its StoryEngine journal → read the journals → if the facts cohere, the shape is a true pattern (mint it as a Template); if they don't, redraw.**

**Anti-rationalization guard (sealed):** the StoryEngine journals **witnessed attributes** (birth, KISPU commits, Šību standing, tomb-type — *facts*), **never generated narrative.** This is what makes the story a real test rather than a mirror: a wrong shape catches particles whose *facts* are visibly incoherent (mixed types, mixed standings, mixed Orbits), and no storytelling skill can wish that incoherence away. The system must be able to tell the human "this shape is wrong" — and it can only do that if the story is facts, not flattery.

## 12. Why each service exists — the epistemic loop

The Orbits Schema reveals that BahyWay's services are not a toolkit but **one instrument for supervised discovery** — turning the formless into the named under human sight:
- **DubSar PDM** — the hand that *draws* the hypothesis-shape.
- **Nasaru** — the eye that makes the answer *visible* before it's trusted.
- **StoryEngine** — the voice that lets caught particles *speak facts*, validating the shape.
- **Lilu VPL** — the **visualization programming language** to author a *new* visual template when no existing Template fits the real Shape of the particles in their Orbits. Reaching a **Lilu Unified Pattern is a climb** — it takes effort and iteration, not a given.
- **Templates (GL-TPL-001)** — the memory that turns a confirmed shape into reusable minted knowledge.
- **Birth Gate + Nergal** — so nothing enters unwitnessed and nothing false is admitted or re-identified.

One body, one act: *a human, guided by shape and story, deciding what is real* — the same creation arc as Enūma Eliš, expressed as a query loop.

## 13. Open seams — sealed AS open (honest record)

These are **not** closed. They are recorded so they are tracked, not forgotten:

1. **Query/analysis boundary.** Sealed direction: **spatial questions = visual-first HeptaScript queries; temporal/aggregate questions ("who crossed threshold between Orbit 40 and 90") = persistent-homology analysis over the Orbit filtration, NOT queries.** The boundary is stated; its precise HeptaScript ↔ PH handoff is not yet specified.
2. **Centroid-at-N formula — LIKELY DISSOLVED, not merely open.** The aging mechanism was framed as radius-to-a-Tribe-centroid, and the centroid was "a word, not a formula." The HeptaMap Cosmic Web court resolves this by *removing the centroid*: it measures **k-NN local density vs. expected density** (over-density ×1.54 etc.) — a **centroid-free** anomaly/standing signal. Proposed resolution: define Šību / anomaly as *local density relative to the field's expected density* (a density estimate over the simplicial complex LamassuEngine already computes), NOT as radius to a computed center. This is centroid-free, already visualized, robust to oddly-shaped/multi-modal Tribes, and consistent with the topology layer. **Recommendation: dissolve the centroid requirement; adopt k-NN local-density.** (If a centroid is ever still wanted, GL-SHP-001's κ-weighted mass-center is the fallback.)
3. **Constant calibration — OPEN, and closable ONLY by measurement, not by visualization.** N (PU snapshot), OU (archive depth), the expansion coefficient, the aging/anomaly threshold (e.g. the ×1.54 over-density cutoff) are **tunable, not fundamental.** Visualization courts make them *tunable-by-eye* (watch the anomaly panel repopulate as the threshold moves) — genuinely useful, but **"looks right" is not "calibrated."** Calibration requires labeled ground truth: run against records whose true anomaly-status is known, measure precision/recall at each threshold, choose the cutoff that optimizes the intended tradeoff. This is *data*, not depiction — no court can close it. *"Morphology proposes; the algebra proves"* (HeptaMap footer) is this seam stated exactly: the visual proposes the threshold, calibration on real data proves it. **Deferred to production calibration against the real corpus.**

## 14. Seal

```
Sealed by: DUB.SAR 𒁾  (Bahaa Fadam) — CSR-08 CONFIRMED · FINAL, NOT DRAFT
Tribes are facts; Orbits are dimensions. The membrane moves; the particle is nailed.
The visual is the validation. Three seams remain open, and are sealed as open.
```
