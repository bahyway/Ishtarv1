# The DubSar PDM Manual
**Particle Data Modeling in Šala — Shape Operators · DataStructure Station · Orbit View**
BahyWay.Ecosystem v4.0 · rehearsal edition (matches `shala_tab_pdm_modeler_v4.html`)
Laws in force: GL-DST-001/002/003 · GL-VIZ-002 · GL-TPL-001/002 · GL-TKT-001 · GL-STY-001 · GL-MDM-001 · GL-DDB-001 · CSR-08

---

## 0 · What PDM is, in one paragraph

PDM turns a client's data into a **governed shape**: fields become vertices,
relations become edges, composites become triangles — a simplicial complex
whose health is read by TDA (β₀ islands, β₁ loops, β₂ voids), whose motion
law is a Geometric-Algebra bivector (the shape decides its own BIGRING),
and whose sealed form — the **Shape Tablet** — is the contract the
DataStructure Station enforces on every ingest. The founding tablet is a
covenant, not a photograph: the tribe *lives*, drifts, and is watched in
the Orbit View until it reaches Apsu. Three sub-tabs, one loop:

    ┌────────────────┬──────────────────┬─────────────────────────┐
    │ Shape Operators│ ⚙ Station        │ ☉ Orbit View            │
    │ author & prove │ first-ingest rite│ living shape · drift    │
    └───────┬────────┴────────┬─────────┴───────────┬─────────────┘
            └── repair ◄──────┴─── the tribe ───────┘

## 1 · Shape Operators (the Houdini-class editor)

**Reading the canvas.** Nodes are operators; wires are typed and colored:
teal = AttributeFlow, gold = ShapeFlow, purple = PatternFlow, pale =
TabletFlow. Illegal wires are refused at connect time; so are cycles —
*loops belong inside the shape, never in the graph that models it.*
Only **Seal** touches the world (CSR-08); everything upstream is pure and
deterministic (same graph → byte-identical tablet; the cook bar shows
`cook #N · ops · ms · deterministic`).

**Figure 1 — the Customer canonical, as the prototype draws it:**

    [TribeSource]──gold──[GapWatch bsn]──gold──[OrbitBivector e2^e6]
                                                       │gold
    [Seal ⚖]◄──pale──[ShapeTablet]◄──gold──[BettiProbe]┘

Walk: load the *Customer canonical* chip → the right panel reads
β = (1,0,0) HEALTHY, with one GAP (`bsn`) flagged from GapWatch → the
tablet pane composes the DRAFT contract live → press ⚖ Seal and the
CSR-08 dialog names the sealing path (HEPT → GeoEngine → Lamassu → G4 →
AAOL) before your yes.

**Figure 2 — PuhuExchange, the node that refuses unproven identity:**

    [TribeSource CustomerEN]──┐
                              ├──[PuhuExchange J=0.86·sem=1.00]──[BettiProbe]…
    [TribeSource KlantNL]─────┘        (passes: one entity, two costumes)

    [TribeSource CustomerEN]──┐
                              ├──[PuhuExchange ✖ crimson]──(no tablet forms)
    [TribeSource SensorLog]───┘   J=0.14 · sem=0.00 — witnesses disagree

Two independent witnesses (structural role:type Jaccard + the semantic
synonym bridge naam→name, stad→city) must BOTH clear threshold, or the
merge never cooks — GL-MDM-001's two-witness verdict enforced at the UI.

**Operator quick reference.** Sources: TribeSource, ShapeImport (from
Station/Orbit drift), PatternSource. Attribute ops: AttributeDefine (EAV
optional — never a KAKI byte), AttributeRelate (DERIVES · CONSTRAINS ·
CO-OCCURS · UNIT-OF · IDENTIFIES · DESCRIBES — each wire is an EnkiDDB
edge per GL-DDB-001), CompositeBind. Assembly: ShapeMerge,
CrossTribeRelate (cooks CrossTribe-KAKI mappings), PuhuExchange.
Patterns: PatternBindShape. Diagnostics: GapWatch, BettiProbe,
OrbitBivector. Outputs: ShapeTablet → Seal.

## 2 · The DataStructure Station (the first-ingest rite)

Eight steps, strictly ordered — the buttons enforce proof-before-existence:

    1 Landing → 2 Safety → 3 Infer → 4 Arsenal match → 5 Stakeholder PDM
      → 6 Gate G4 (Z3·Lean4·Algebra) → 7 SLA dual seal → 8 ETL → GOLDEN

**Figure 3 — step 3, inference on klanten_2026.csv:**

    klant_id  naam      stad        aankoop_datum  score
    8f2a…     Jansen    Amsterdam   2026-06-02     7.9
    c77d…     Bakker    Amsterdm ⚠  2026-06-03     6.1
    → fields typed from values · roles by heuristic (*_id→WHO,
      date→WHEN, city→WHERE, amount→HOWMUCH) · key: klant_id
    → draft β = (1,0,0)

Step 4 runs the arsenal's two witnesses: klanten matches the Customer
canonical **through the synonym witness** (structure agrees across
languages) → CONFIDENT. sensors.csv returns NOVEL and mints a pattern
draft (GL-TPL-001 rite governs its approval).

**Figure 4 — the discipline loop (customers_messy.csv):**

    Infer: β₀ = 2 — the notes–source pair floats as an island
    Gate G4:  Z3 ✓ ✓ · Lean4 ✓ ✓ · Algebra: “Betti ≠ HOLD” ✗
    → G4 REFUSED → rite falls back to step 5
    → Open in Shape Operators → link the island → cook →
      “Adopt cooked shape into Station” → G4 re-proves ✓

The G4-fails-into-PDM-repair loop is the best-practices discipline, not
an error path: no unproven shape can buy its way past the gate, and no
SLA can be sealed before G4, and no ETL can run before the SLA.

Step 8 animates the BeeMDM chain: records flow INGEST→STAGE→TRANSFORM→
PROVE→GOLDEN; cleanse events log (“Amsterdm → Amsterdam”); rejects
coalesce into **one ticket** (one cause, one ticket — GL-TKT-001);
survivors append into EnkiDB · 7004, projected to EnkiDW; campers remain
— which is why the Orbit View exists.

## 3 · The Orbit View (the living shape — GL-TPL-002)

**Figure 5 — the living tribe after ETL, as rendered:**

            ~ INGEST campers (pale, brightening with age) ~
         ~ STAGE campers (orange) ~
              ((( GOLDEN core — EnkiDB · 7004 )))          [REJECT
                      ☀ central glow                        colony —
                                                            crimson
                                                            island]

Lamassu's reading: founding β₀ = 1; **β₀_live = 3** — the camper band
and the reject colony are stranded islands, topology seeing drift
without reading a single record. Campers *brighten* as dwell grows:
aging rendered as luminance.

**The steward's loop, as buttons:** ⏱ Advance 10 days (campers age,
shingle-J confidence decays) → 🜚 Run Shakkanakku survey (two witnesses:
β₀_live vs founding + pattern confidence; drift confirmed only when they
agree) → Open drift cohort in PDM (the graph preloads the canonical PLUS
what the pipeline taught: `dwell_limit` CONSTRAINS, `reject_reason`) →
**Adopt as canonical v2 (append)** — v1 is never touched; the version
ledger shows the lineage and WHY note (GL-TPL-002 §5) → ⚖ Enforce:
decrees flow, campers PROMOTE, rejects REWORK or take the KISPU rite,
and the Apsu meter climbs. At 100%:

    A P S U — the engagement is complete
    the living shape has returned to its covenant ·
    every chronicle preserved beneath the gold

Apsu is the progress bar of the engagement and its contractual
completion criterion (§6). The Orbit View never retires (§1).

## Appendix A · HeptaScript (Anti-SQL — PRESENT/ORBIT/EMIT/PROVE/SYNC/WITNESS only)

Author-time — the cooked contract:

    PRESENT SHAPE Cooked
      WHAT  fields = 5 · relations = 5 · composites = 1
      HOW   betti = (1, 0, 0)
      WHERE orbit = BIVECTOR e2^e6
      WHY   gaps = bsn
    EMIT ShapeTablet SEAL AkkadianSeal

Witness-time — scopes on the living tribe (GL-VIZ-002):

    WITNESS ORBIT
      WHAT  state = FUZZY
      WHEN  dwell > 30d
      HOWMUCH window = VIEW OrbitsView
    EMIT SelectionEvent

Judgment-time — the append-only decree (GL-DST-003):

    SYNC ORBIT
      WHO      steward  = DUB.SAR 𒁾
      WHAT     verdict  = PROMOTE (APPEND state — never update)
      WHERE    home     = EnkiSDB, EnkiODB (WRITE nodes)
      WHY      note     = "campers resolved under canonical v2"
      HOWMANY  particles = 10
    EMIT DecisionTablet SEAL AkkadianSeal

Arsenal & lineage:

    PRESENT TEMPLATE
      WHAT kind = Pattern · WHO approved = DUB.SAR · WHEN release = Zagesi
    ORBIT ArsenalShelf

    WITNESS STORY
      WHO identity = κ 54bcdc2a
    PRESENT EVENTS ORBIT ChronicleRing        -- exact tier: ENLIL, O(k)

Never SELECT, FROM, JOIN, UPDATE, DELETE — the console refuses them.

## Appendix B · W5H2 tablets (per GL-DDB-001: WHO=AGNT · WHAT=THME · WHEN=PTIM · WHERE=LOC · WHY=RSON · HOW=MANR · HOWMUCH=MEAS)

A sealed shape:
- **WHO** ClientA·Klanten tribe (steward: DUB.SAR; client steward co-seal)
- **WHAT** canonical customer shape, 5 fields, 1 composite
- **WHEN** sealed 2026-08-05 · era Zagesi · canonical v1
- **WHERE** EnkiMDB · 7006 (arsenal); enforced at the ClientA Station
- **WHY** SLA GOLD contract; GL-MDM-001 conformance
- **HOW** inferred → PDM-reviewed → G4-proven (Z3·Lean4·Algebra) → dual-sealed
- **HOWMUCH** β=(1,0,0) · 150 records first batch · thresholds J≥0.5, sem≥0.5

A drift survey:
- **WHO** Shakkanakku (scheduler) · Lamassu + CompareEngine (witnesses)
- **WHAT** living-shape survey of ClientA·Klanten
- **WHEN** day 30 after first ingest
- **WHERE** across EnkiSDB/EnkiODB/EnkiQDB/EnkiDB houses
- **WHY** GL-TPL-002 §3 — covenant vs living shape
- **HOW** β₀_live=3 vs founding 1 · shingle-J 0.69 < 0.75 → DRIFT
- **HOWMUCH** 10 campers · 5 rejects · divergence D=0.31 · Apsu 89%

## Appendix C · Law cross-reference

| You do this in the prototype | The law behind it |
|---|---|
| Wire refused / cycle refused | Shape Operator spec · GL-DDB-001 edges |
| Puhu refuses to cook | GL-MDM-001 two-witness verdict (PH-002 pending) |
| G4 blocks the SLA | GL-DST-001 proof-before-existence · Z3 design-time law |
| One ticket from many rejects | GL-TKT-001 §2 coalescing |
| Decree appends, ledger grows | GL-DST-003 §2 |
| Canonical v2 appended with lineage | GL-TPL-002 §5 |
| Campers brighten, β₀_live counts islands | GL-TPL-002 §2–3 |
| Apsu at 100% | GL-DST-003 §5 · GL-TPL-002 §6 |

*Šala hub index: MARDUK v6 (witness · court · Apsu) · PDM v4 (operators ·
station · orbit view). Standing rite: every accepted law amends the
rehearsal in its next version.*

— Pressed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
