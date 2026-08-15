# Phase 2 Glossary — DRAFT (Step 1 pass)

**Status: DRAFT.** This is a first-pass inventory of every named engine, law,
metric, and concept encountered while cataloging the 7 raw upload batches in
`docs/phase2-incoming/`. It is not yet cross-checked against the existing
`docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`, not yet deduplicated (e.g. the three
GL-NAV-001 claimants below are listed as three separate entries on purpose),
and not yet NL-001-checked for lawful naming. It will be superseded by the
consolidated `BAHYWAY_PHASE2_GLOSSARY.md` produced in Step 4, after the
conflict map (Step 2) and renumbering (Step 3) resolve collisions.

Entries are grouped by rough theme, each tagged with its source batch(es).

## CompareEngine / Jaccard (batch 1)
- **CompareEngine** — the ETL comparison station; design target for a
  two-level Jaccard similarity measure (schema-presence + value cf/df).
- **Jaccard schema-level encoding** — attribute-presence-only indicator
  vectors (Fletcher & Islam method); deliberately drops values to avoid
  false mismatches on near-equal numbers.
- **Sigmoid similarity** — the best-performing value-level metric found
  (Likavec/Lombardi/Cena), applied over EAV common/distinctive features.

## Navigation / Flight (batch 1, batch 6)
- **Nabû** — Semantic Search Engine (SSE); resolves a natural-language
  sentence into a grounded target, explicitly refusing to fabricate matches.
- **NaviEngine** — computes the descent path + neighborhood set from Nabû's
  target; plots only, never computes truth (GeoEngine remains sole truth
  source).
- **Hubble-Zooming** — the sealed six-level zoom ladder (Galaxy → Cluster →
  Orbit → Particle → Target/Event) that renders the flight.
- **Flight-to-Location** — the Nabû→NaviEngine→Hubble three-engine pipeline.
  Sealed as **GL-NAV-001** in batch 1 (`flight-to-location`) — collides
  with two later claims on the same ID (see Conflicts, below).
- **NajafEngine** — grave-location domain instance (Wādī al-Salām).
- **WPDEngine / EGDEngine** — water/electricity defect-node domain instance.
- **Hendursaga** — sensor/flight-deck engine family (batch 6); also claims
  **GL-NAV-001** via "Charter Annex A" — third collision.

## Visualization core (batch 1, 2, 5, 6, 7)
- **Šala / Shala** — the prototype rehearsal workbench. HTML-only,
  offline-sovereign, governed by `SHALA-DESIGN-CHARTER.md` — rehearsal
  scaffolding per Way-of-Work rule 5, never production.
- **BIGRING** — a torus-shaped 3D render of one orbit ring of particles.
- **StoryEngine** — per-particle event/journal history
  (`BORN → EVENT → NOW`), openable per-particle via right-click.
- **ṬĀLUKU** — the orbital motion law for living-orbit visualizations,
  ω ∝ r^(−3/2) (Kepler-style; center spins faster than rim).
- **HeptaMapSpace** — 2D-membrane "bucket" visualization of any 2 of a
  particle's 7 dimensions; independently identified (by the Architect) and
  confirmed (by the model) as literal TDA: buckets = cells of a cubical
  complex, β₀/β₁ = Betti numbers, Hubble zoom = the persistence filtration.
- **Algebra Arsenal** — the 53-member catalog of BahyWay's algebra/law/
  calculus components ("Unified Algebra Theorem"), rendered as living orbits.
- **Realm Map** — the "Visualization Realm" governance map: BENCH
  (dev-only: Julia/Pluto.jl, Podman) vs BODY (shipped, pure-Rust), separated
  by the **Bench Membrane** (`GL-TOOL-001`), with three gates: Submission
  (`GL-GOV-001`), ranked Authority (CSR-08), Compression (`GL-GOV-003`).
- **naṣāru / BWVL** — "BahyWay Visual Language," a law family
  (`GL-VIZ-000` through `GL-VIZ-006`): Visual Language, Morphological
  Discovery, Shape Verdict, Particle Monism, ColourID Lifecycle, Federation
  of BIGRINGs, Zoom-As-Necessity.

## PDM / TDA (batch 2, 7)
- **DubSar PDM (Pattern Data Modeler)** — native C++/GLSL Vulkan prototype;
  corrects a particle cluster's topological "shape" (Betti numbers) toward
  an SLA-agreed target shape, using Simplicial Complexity + a genetic
  algorithm, proved by Z3 — design-time only (MUMMU, Gate G4), never a
  runtime service.
- **LamassuEngine** — Tribe-anchored TDA orchestrator; three cadences: fast
  numeric, medium topological (Betti computation), slow GA+Z3 correction.
- **fca_engine.py** — real (non-HTML) prototype code for Formal Concept
  Analysis discovery (batch 7).

## Metrology / commercial (batch 3)
- **PU (Particles Unit)** — compute-cost metric; 1 PU = the compute to carry
  one clean reference particle to golden. Hardware-independent via a
  per-host calibration rite. Meters machine cost only, never human labor.
- **CTG** — cost-per-golden-outcome metric; folds PU in as the "metered
  compute beats" ingredient alongside human/steward costs.
- **Qishtu** — the reward-engine / commercial "Observatory" visualization:
  clients orbit at radius = CTG: clean clients orbit close, dirty ones
  drift outward.
- **Zibānītu** — the judging calculus ("the scales"): functional
  F(T‖Θ) = w_h·Δ_h + w_k·Δ_k + w_m·Δ_m + ε, axioms F1–F5, the **Fadam
  Inequality** (verdict cannot outrun the world; ε floor is a hard band).
  Named amendments: **A1** (seats PU in the unit family), **A2** (Civil
  Protection Calculus, batch 4 — Never-Averaged Theorem, Scenario-Flip
  Safety).
- **Barūtu** — omen/generator tablet + manual (`MAN_BR_001`).
- **Duru** — "walls" tablet (containment law).
- **Mashalu / Mašḫalu** — elastic membrane search foundation (`GL-MEM-001`).
- **Piqittu** — muster (backup/game-day rehearsal) blueprint (`CASE_IQ_001`).
- **Uruk / Kish** — two named physical hosts (topology introduced here,
  corroborated independently in batch 6's `PB-504_uruk_kish_weir.yml`).
  Working hypothesis: Uruk = the Architect's current bare-metal box, Kish =
  a second machine under consideration — **needs Architect confirmation**.
- **Lahmu–Lahamu** — the two-stream ledger-shipping law between Uruk and Kish.

## Civil protection / fire (batch 4)
- **Contested Sky, Fadam Verdict, Gravity of Fire, HeptaMap of Refuge,
  Kidinnu Standard** — a five-tab visualization arc applying the Zibānītu
  calculus to competing fire-scenario testimony and civilian evacuation.
- **Kidinnu** — **PROPOSED, UNSEALED** name (NL-001 naming pending Architect
  decision) for the civil-protection engine born from this arc. Do not
  treat as final.
- **Minimax directive** — the life-safety corollary: an evacuation zone's
  directive is the move whose worst case under all sealed scenarios is
  least, never a probability-weighted blend of scenarios.

## Traffic / sensor / medical (batch 6, 7)
- **Igigi Watch, Parzu Tremor Watch, Asalluhi (Station Watch), Hendursaga
  (Flight Deck)** — sensor/monitoring engine family.
- **Sila Grid / Sila Census** — Baghdad-focused traffic grid engines.
- **Karanu Vineyard** — vineyard extent/harvest domain engine.
- **Medical sector expansion** — `GL-MED-001` (Sector Charter, +Annex A
  Ninisina Engine), `GL-MED-002` (Living Anatomy), `GL-MED-003` (GOLDEN
  Medical Data Model) — cancer/gangrene diagnostic visualizations built
  on the same membrane/cluster machinery as HeptaMapSpace.
- **GulaFederation** — federation-of-BIGRINGs court/central visualization
  (batch 7's nested `GulaFederation_PB-321_326.zip`).

## Governance / schema law (batch 2, 7)
- **GL-GOV-001/002/003** — Sealed Submission Law, Law of Earned Assertion,
  Compression Gate.
- **GL-DDB-001** (batch 2, EnkiDDB SCG) and **GL-DDB-002/003/004** (batch 7,
  EnkiDDB Corpus Law +Annex B Babu Intake, PreKAKI Schema Lifecycle +Annex A
  Schema-First Client Ingest, Additive Schema Growth).
- **GL-HS3-001/002** — HeptaScript Grounded Query Grammar; Uncertainty
  Measure Epsilon (ε).
- **GL-KAKI-002** — Three KAKI Types.
- **GL-TPL-001/002, GL-TKT-001, GL-STY-001, GL-ORG-001, GL-MDM-001,
  GL-DST-002/003** — pattern-minting template, living-shape drift, ticket
  law, StoryEngine journal-event ontology, homeostasis, harmonization
  survey, Tupsimati connector wizard, Madanu court.
- **Girsu** — Vulkan classroom / extension-naming playbooks.

## Known naming/numbering conflicts (see `README.md` for full detail)
- `GL-NAV-001` claimed by batch 1, batch 2, and batch 6 — three different
  subjects.
- `PB-321` claimed by batch 4, batch 5, and batch 7 — three different
  playbooks.
- Batch 2's `PB-185`–`PB-200` collide with unrelated, already-committed
  playbooks in `playbooks/`.
