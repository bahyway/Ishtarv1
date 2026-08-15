# BahyWay Phase 2 Glossary

**Status:** consolidated, post-reconciliation (2026-08-14). Doc IDs below
are FINAL per `docs/phase2-incoming/RENUMBERING_MAP.md` — collisions
resolved, cross-checked against `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md`
(no NL-001 naming clashes found: none of these god-names/terms were
already claimed for something else in the existing glossary). All
entries remain **DRAFT / UNSEALED** design content unless marked
SEALED-CONCEPT — see `docs/99_index/CAT-001-index.md` for per-doc status.

Entries are grouped by rough theme, each tagged with its source batch(es).

## Naming — cuneiform identity (2026-08-15, Architect decision, not from a batch)
- **Hala 𒄩𒆷** — sovereign rename of **Uruinimgina**, the docs Reform & Pulse
  engine (Shakkanakku's Tab 3 / `docpulse`), decided by the Architect for
  ease of pronunciation. Etymology: Sumerian *ḫala* — "share," "portion,"
  "lot" (as in an inheritance or a division of land) — apt for an engine
  whose job is portioning documents out of a working repo into EnkiDDB's
  sealed corpus. **Cuneiform glyph proposed: 𒄩𒆷 (HA-LA, phonetic
  two-sign spelling) — BEST-EFFORT, NOT YET VERIFIED** against a proper
  Assyriological sign list (e.g. ePSD2 or Borger's Mesopotamisches
  Zeichenlexikon). Given this glyph is meant to become a permanent,
  widely-propagated identity (documentation, UI labels, and an EnkiDDB
  attribute — see below), verify it before treating it as sealed; it is
  marked DRAFT throughout for exactly this reason, per this ecosystem's
  own "no false authority" law (`GL-DB-001`,
  `playbook_547_no_false_authority_law_seal.yml`).
  - Per the Concordance Doctrine (`docs/04_gates/law_lattice_7x7_tablets.md`,
    Clause U-3 — "no existing law ID is ever renumbered, rewritten, or
    retired"), the historical clause **7.3 Uruinimgina Reform Decree**
    and its ID are untouched; only the patron's sovereign name changed,
    recorded as an overlay note on the existing entries.
  - Per the same precedent already applied to `eriduous-vdi`→`uruk` this
    session: the **binary name** `uruinimgina-cli`, its source file
    `src/bin/uruinimgina_cli.rs`, and its config file `uruinimgina.toml`
    were deliberately **NOT** renamed — two already-sealed playbooks
    (`playbook_278_uruinimgina_fedora_w44_setup.yml`,
    `playbook_281_uruinimgina_git_recovery_and_retry.yml`) invoke them by
    exact name, and renaming would be invasive for zero functional
    benefit. Everything user/documentation-facing (the GUI tab, the web
    dashboard's Tab 3, code comments, generated commit-message text, this
    glossary) now says **Hala**; the technical plumbing underneath still
    answers to its old name. See `workspace/bahyway_v4/crates/anu-governor/
    src/{app.rs,lib.rs,docpulse.rs}` and `src/bin/web_assets/{index.html,
    app.js}` for exactly what changed (verified: `cargo check -p
    anu-governor` clean, both default and `web` features; crate path
    updated 2026-08-15 when Shakkanakku itself was renamed to AnuGovernor
    — see below).
  - **EnkiDDB attribute proposal** (schema only — no live database write
    was made from this session; no access to a running EnkiDDB instance):
    add a `cuneiform_glyph` EAV attribute (string, UTF-8, one or more
    Unicode cuneiform code points) to the KAKI identity of any particle
    representing a god-named engine (Nabû, Marduk, Shamash, Enki, Hala,
    …), populated alongside the existing name attribute, so the naṣāru
    visualization view (`GL-VIZ-000` BahyWay Visual Language family) can
    render the glyph next to the name wherever an engine identity is
    displayed, sourced from data rather than hardcoded per-view. This
    needs an actual schema decision + migration on the real EnkiDDB,
    which only runs on `uruk` — proposed here, not yet built.
- **AnuGovernor 𒀭** — sovereign rename of **Shakkanakku**, the BahyWay
  Governor (executes+halts+seals the playbook corpus), decided by the
  Architect (2026-08-15) for "more light and more precise." Etymology:
  Anu, sky god, head of the Mesopotamian pantheon. Unlike the Hala
  rename above, this one **is** a real crate/package rename (the
  Architect's explicit choice, not a sovereign-name-layer-only pattern):
  `crates/shakkanakku` → `crates/anu-governor`, package `shakkanakku` →
  `anu-governor`, lib `shakkanakku` → `anu_governor`, binaries
  `shakkanakku`/`shakkanakku-web` → `anu-governor`/`anu-governor-web`,
  config file `shakkanakku.toml` → `anu-governor.toml`. The
  `uruinimgina-cli`/`pb-catalog-cli` sub-binaries and their source files
  keep their own already-settled names, unaffected by this rename.
  Per the Concordance Doctrine, sealed law tablets that use
  "Shakkanakku" as ongoing terminology (`GL-DB-001`,
  `law_lattice_7x7_tablets.md`, `GL-DOC-001`, and others) are left
  unrewritten — this glossary entry is the single place recording the
  current name, per `GL-DOC-001`'s own Single Glossary law.
- **Elu** (elû, Akkadian "high, upper, superior") — sovereign rename of
  the index stack (`crates/enkidb-indexes`'s `SOVEREIGN_NAME`, and the
  real `crates/elu-tribe-hotindex` crate), which was itself named **Anu**
  (2026-07-13) before the Architect reclaimed "Anu" for AnuGovernor
  above and renamed the index stack a second time. See
  `docs/05_storage/ELU_INDEX_STACK.md` for the full provenance of both
  renames (ENLIL → Anu → Elu). Per NL-001's Orthography Clause (§6a),
  code identifiers use the plain-Latin "Elu"; the diacritic "elû" is
  reserved for prose.

## Golden Lifecycle (2026-08-15, Architect decision, sealed in `GL-GLD-001`)
- **Golden Particle / Golden Record** — a particle whose content is sealed as
  authoritative truth; the same claim the MDM industry names "Golden
  Record." GOLDEN is a claim about **finality of essence**, never about
  permanence of position — see `GL-GLD-001` §1-2.
- **Gravity Slop** — the real, implemented drift of a particle's *rendered
  position* on its Membrane under neighbour density and membrane tension
  (`r = s.r * taut(v) * (1 - dent)`, from `shala_layered_organism_v2.html`).
  Position-only; never touches a particle's Mandatory EAV facts, KAKI, or
  `GOLDEN/FUZZY/DEAD` state. See `GL-GLD-001` §3.
- **Aged / Decay** — the epistemic gap between a sealed fact and current
  reality, Steward-witnessed (mechanic already sealed at `GL-VIZ-004` §5;
  meaning formalized at `GL-GLD-001` §4). Never structural corruption, never
  automatic drift.
- **Golden State Debugging (= EnkiDW Analyzing)** — the named procedure for
  investigating an unexplained position/membrane shift: rule out essence
  change first, attribute to Gravity Slop mechanics, escalate only if
  Mandatory facts or state actually changed. See `GL-GLD-001` §5.
- **The Golden lifecycle loop** — GOLD → AGED → DECAY → POSITION → LOCATION
  → TIME → GOLD, a teaching device (not a new state machine) holding all
  seven ideas above in one frame. See `GL-GLD-001` §6, and
  `docs/00_codex/00_codex_government_by_sevens.md` for why this ecosystem's
  "7" recurs independently rather than being derived once.

## Phase 0 Recognizer Law (2026-08-15, sealed in `GL-ONT-002`)
- **"No External Model" law** — the Architect's own words: *"NO External
  Model, Just Pure Rust and any once Download use for ever law."*
  Production recognizers ship as pure-Rust, deterministic, offline-capable
  code — no live external model call, ever. See `GL-ONT-002` §3.
- **Once-Download-Use-Forever resource** — a lexicon/embedding/ONNX file
  fetched exactly once, cached locally, read only from disk thereafter.
  Same discipline `GL-NAV-001` already sealed for Nabû's fastembed-rs
  embeddings and cached map tiles.
- **Design-time-only ML/NLP comparison harness** — an external model may
  run *at design time only*, to find gaps in the deterministic
  recognizer's coverage — never shipped, never in a production code path.
  Mirrors the existing Z3-at-Gate-G4 discipline (`GL-ONT-001` §5).
- **Gap as motivation, not verdict** — when the harness finds something
  the deterministic recognizer misses, that gap justifies improving the
  pure-Rust algorithm, never promoting the ML model into production.

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
  Sealed as **`GL-NAV-001`** (`docs/09_observatory/GL-NAV-001-flight-to-location.md`) —
  won the number over two other claimants (see below).
- **NajafEngine** — grave-location domain instance (Wādī al-Salām).
- **WPDEngine / EGDEngine** — water/electricity defect-node domain instance.
- **Hendursaga** — sensor/flight-deck engine family (batch 6); its charter
  is filed as **`GL-NAV-001` Annex A** (Wādī al-Salām field architecture —
  the same NajafEngine domain as the base law).
- **NabuEngine (Knowledge-Graph Navigation)** — a *different* Nabû-adjacent
  concept (documentation-as-dynamic-orbits, batch 2), renumbered to
  **`GL-NAV-002`** to resolve the collision with Flight-to-Location. Its
  "PB-184 recovery" provenance claim doesn't match this repo's real
  `playbook_184` — unresolved, flagged in the doc itself.

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
- **naṣāru / BWVL** — "BahyWay Visual Language," a law family: `GL-VIZ-000`
  (Visual Language foundation), `GL-VIZ-007` (Morphological Discovery,
  renumbered from a colliding `GL-VIZ-001`), `GL-VIZ-008` (Shape Verdict,
  renumbered from a colliding `GL-VIZ-002`), `GL-VIZ-003` (Particle
  Monism), `GL-VIZ-004` (ColourID Lifecycle), `GL-VIZ-005` (Federation of
  BIGRINGs), `GL-VIZ-006` (Zoom-As-Necessity). Note: the real, load-bearing
  `GL-VIZ-001` (Bivector Orbit Encoding / BUZU chunk) and `GL-VIZ-002`
  (Orbit Witness & Isolation, batch 2) are a *separate*, older law family —
  the renumbering above exists specifically so naṣāru/BWVL doesn't collide
  with them.

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
- **OntoGraph** (`GL-ONT-001`, landed 2026-08-15, post-Phase-2) — the
  Nasaru instrument that fuses the ontological pass (EnkiDDB W5H2/Sowa
  vocabulary, GL-DDB-001) with the topological pass (LamassuEngine Betti
  classes) into one FCA formal context, computes the concept lattice via
  NextClosure, and mints it — concepts as nodes, extents as hyperedges,
  no pairwise edges — as the **Unified Pattern, sovereign-named
  Nebuchadnezzar** (NL-001 §6b Landmark Pattern Clause: an epoch-marking
  pattern may share a king's name with the era it marks; the name stays
  on the era roster too). Three-layer attribute law: KAKI stays address-
  only; Mandatory EAV (W5H2/Sowa, state class, ColourID, freshness,
  domain) is read-only spine; Optional EAV (`onto.*` discoveries,
  `dmbok.*` organizational facets) is the only layer OntoGraph writes —
  compiler-enforced via an `assert_writable` guard, not just documented.
  Promotion of a discovered concept to Mandatory is design-time only, at
  Gate G4, with Z3 proof — never at runtime. Real Rust crate at
  `workspace/bahyway_v4/crates/ontograph`, `cargo test -p ontograph`
  verified 4/4 passing during landing (one real bug found and fixed:
  `Layer` needed `#[derive(Hash)]`). PB-323 (LamassuEngine bridge) and
  PB-324 (KISPU mint + NĀRU witness) are reserved, not yet scaffolded.

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
- **Uruk / Kish** — two named physical hosts. **Confirmed by the Architect
  (2026-08-14)**: Uruk = the real bare-metal Fedora Workstation 44 box
  (now in `ansible/inventory.ini`, replacing the deprecated `eriduous-vdi`
  VDI concept for new work); Kish = a second machine, reserved but not yet
  provisioned.
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
- **GulaFederation** — federation-of-BIGRINGs court/central visualization;
  its 6-playbook suite renumbered from a colliding `PB-321`–`326` to
  `PB-549`–`554` (see `RENUMBERING_MAP.md`).

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
- **GL-PHY-001** (Physics Service Law) — a central Physics service
  (NinurtaEngine, proposed) as the sole executor of dynamics, standing
  beside GeoEngine's timeless truth; the Two-Tier Physics Law separates
  cheap presentation-side integrators from certificate-grade truth-tier
  ones, and truth-tier integrators are the ecosystem's ε source.
- **GL-PAT-001** (Foreign Pattern Quarantine & Promotion Law) — governs
  how a pattern from outside the sealed arsenal is quarantined,
  evaluated, and (if earned) promoted in.
- **GL-PAT-002** (Pattern Maturation & Template Delivery Law) — governs
  a pattern's lifecycle from quarantine through template delivery.
- **GL-DB-001** (No False Authority Law, FOUNDATIONAL) — the law this
  entire integration has been following throughout: never claim
  certainty, verification, or completion status not actually earned;
  inherited by GL-PHY-001's ε mandate, GL-ORG-001's humility clause, and
  GL-TPL-002's drift metrics.
- **GL-DOC-001** (Single Glossary Law, FOUNDATIONAL-DOC) — one glossary
  per concept, no forked/competing definitions; this document and
  `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` are both real, separate
  glossaries by scope (Phase 2 vs. core ecosystem), not a violation of
  this law, but merging them is a natural future step under it.
  - **Correction, 2026-08-15**: all five of the above existed ONLY as
    text embedded inside their own playbook's Ansible `copy:` task —
    never landed as standalone files in the Step 4 documentation pass,
    unlike their siblings in the same batch. Found during a silent-error
    audit (running the playbook would have written a duplicate, out-of-
    repo, undiscoverable copy under `$HOME/bahyway/docs/...` instead of
    failing loudly). Extracted and committed as real `docs/GL-*.md`
    files; their playbooks (`playbook_544`–`548`) converted to
    pointer-style like `playbook_301`. See
    `playbooks/playbook_555_hala_naming_correction_uruinimgina.yml`'s
    sibling reasoning and `docs/99_index/CAT-001-index.md`.

## Naming/numbering history
All collisions found during reconciliation (three claims on `GL-NAV-001`,
two on `GL-VIZ-001`/`002` vs. the real BUZU law family, three-way/two-way
`PB-321`/`PB-322` collisions across batches 4/5/7, and batch 2's `PB-185`–
`200` colliding with unrelated already-merged playbooks) are resolved —
see `docs/phase2-incoming/CONFLICT_MAP.md` (analysis) and
`docs/phase2-incoming/RENUMBERING_MAP.md` (final, applied mapping) for
full history and reasoning.
