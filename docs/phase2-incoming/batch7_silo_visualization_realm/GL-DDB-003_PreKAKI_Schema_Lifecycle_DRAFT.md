# GL-DDB-003 (candidate) — PRE-KAKI SCHEMA LIFECYCLE
## Medical-Department Ingest · Draft Schema → Tri-Approval → KAKI Promotion → OTA TEST Build
### BahyWay.Ecosystem v4.0 · Phase Two (GL-STD-002) · extends GL-DDB-002 · Status: DRAFT — pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Principle

Schema is born unnamed. When a medical department is ingested, the system first
generates a **Pre-KAKI schema** — a complete EAV model whose components are
*draft particles* with NO identity yet (no KAKI). Identity is not granted by the
machine; it is granted by **three human approvals**. Only then are KAKI v4.0
minted, and only then is the schema physically built and tested in the **OTA
TEST** environment. Nothing reaches production unnamed, unapproved, or untested.

Lineage of states (each a genuine ontological rung):
    PRE-KAKI (draft, no identity) → [tri-approval] → KAKI PARTICLE (named)
    then downstream: UNKNOWN / GOLDEN / FUZZY / DEAD (runtime states)

---

## 1 · The Run — Pre-KAKI Schema Generation

Clicking **Run** for a chosen department (Cardiology, Laboratory, Radiology,
Pharmacy, Oncology, Surgery, …) generates a complete EAV schema:

- **Draft entities** (Pre-KAKI): Corpus → Document → **Chunk** → ConceptNode.
  Each is particle-shaped but identity-less: it carries a `draft_id`, not a KAKI.
- **Essential EAV** per chunk (from GL-DDB-002 §1): Body, Provenance, License,
  Simtu facets, Classification codes (MeSH/ICD/ATC/SNOMED refs), Quality/State,
  and **Eṭemmu (embedding) refs** — the "embroiderers": vector projections that
  adorn each chunk, carrying NO KAKI of their own (spirit points to body).
- **Concept edges** (Pre-KAKI CrossTribe drafts): same-concept, depends-on,
  translates (ar↔en), part-of — the Graph-RAG structure, still un-sealed.
- **DMBOK pre-score**: the §2 GL-DDB-002 scorecard computed on the draft, so the
  architect sees quality *before* approval, not after.

Output artifact: **`<department>.prekaki.schema.json`** — the draft tablet.

---

## 2 · Tri-Approval Gate (identity is granted, never assumed)

The Pre-KAKI schema becomes real only with THREE sealed approvals, each an
Event particle in an append-only journal:

| Approver | Verifies |
|---|---|
| **Data Architect** | structure, template correctness, Gate G4 satisfiability |
| **Data Steward** | facets, classification codes, license, DMBOK score ≥ threshold |
| **Authorized Client Stakeholder(s)** | domain truth — does this model the real department? |

Rules: approvals are Ed25519-sealed; any one rejection returns the schema to
draft with a reasoned note (DubSar PDM restructuring if structural — the
simplicial-map shape test of Bābu B-6); partial approval never promotes;
the three seals together are the promotion authority (CSR-08 delegation aside).

---

## 3 · KAKI Promotion (the naming)

On tri-approval, the promoter mints KAKI v4.0 for every constituent:

- **Chunks** → KAKI role Identity (κ[6]=0x01), tribe = department.
- **ConceptNodes** → KAKI Identity in the Concept tribe.
- **Concept edges** → KAKI role CrossTribe (κ[6]=0x03).
- **Embeddings (embroiderers)** → **NO KAKI**; promoted as sealed EAV refs on
  their chunk (locked law — quality/projection never in KAKI bytes, never a
  particle of its own).

Each promotion is a sealed Event; the draft_ids are retained in provenance
(the tablet remembers it was once unnamed — StoryEngine scar).

---

## 4 · OTA TEST Build (physical, tested, sandboxed)

The promoted schema is **physically built** in the OTA **TEST** environment
(OTA tiers: DEV → **TEST** → ACC → PROD). Never in PROD, never in ACC first.

- Instantiate the seven-Enki tribe/collections for the department in TEST.
- Load a bounded sample (MIMIC-IV-demo / AHD subset / Synthea) through the
  Bābu gate → Nergal → BeeMDM → into the new schema.
- Run the TESTING rite: structural validation, DMBOK re-score, round-trip
  ORBIT/PROVE/WITNESS queries, β/τ/ε sanity, quarantine-path check.
- Only a passing TEST build becomes an ACC candidate (a later, separate seal).

Governing law intact: the existing playbook program + TESTING_PHASE1 gate the
promotion of any of this to real workloads; GL-DDB-003 is the lifecycle law and
its TEST-tier exercise, not a PROD action.

---

## 5 · PB Placement
- PB-331 `enkiddb-corpus-schema` (GL-DDB-002) gains a `--prekaki` mode: emits
  draft particles with draft_ids instead of KAKI.
- PB-340 `prekaki-tri-approval` — records the three sealed approvals; on the
  third, invokes promotion.
- PB-341 `kaki-promotion` — mints KAKI for chunks/concepts/edges; embeddings → EAV refs.
- PB-342 `ota-test-build` — instantiates the schema in the TEST environment and
  runs the TESTING rite. (Way of Work: Ansible from host → VDI → EnkiDB VMs.)

## 6 · Open seals for CSR-08
GL-DDB-003 adoption · sovereign name for the Pre-KAKI/draft state
(Ṭuppu ṣābitu / lā šumšu candidates) · tri-approval quorum rule (all three vs
architect+steward+any-one-stakeholder) · confirmation that "embroiderers" =
embeddings · PB-340…342 numbering.

*Recorded in the reign of Gudea 1.0, Phase Two. Nothing herein is sealed until
DUB.SAR confirms under CSR-08.*
