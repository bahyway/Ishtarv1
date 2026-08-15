# GL-DDB-003 · ANNEX A (candidate) — SCHEMA-FIRST CLIENT INGEST
## Two Ingest Doctrines · The Schema-First Test · DEV→TEST→UAT
### BahyWay.Ecosystem v4.0 · Phase Two · extends GL-DDB-003 · Status: DRAFT — pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Two Doctrines (they must never be conflated)

The direction of the schema decides the whole sequence.

| | **A · Schema-Discovered** (public data) | **B · Schema-First** (client data) |
|---|---|---|
| Source | public online sources (MIMIC, AHD, Synthea) | client's own compressed files |
| Schema role | **output** — generated to learn the shape | **input/contract** — agreed in advance |
| Purpose | learn attribute mechanism + DMBOK compliance | ingest real client data safely |
| Schema known first? | no — Run discovers it (Pre-KAKI) | yes — it is a precondition |
| Non-conformance | shapes the draft | **data is refused** until it conforms |
| OTA reach | DEV → **TEST** (learning) | DEV → TEST → **UAT** (client acceptance) |
| Governing doctrine | GL-DDB-003 body | this annex |

The unifying safety principle (same insight as the malware/UNKNOWN split):
**never process what you have not first proven conforms.** Schema-First is
*contract before content*.

---

## 1 · The Schema-First Test (runs BEFORE any ETL)

When a client delivers compressed medical files, a pre-processing gate runs
*before* BeeMDM touches content:

    ARRIVAL (client archive) → Bābu B-1..B-3 (archive rite, Nergal AV, license)
      → SCHEMA-FIRST TEST:
          S1 · Contract load — the agreed client schema (the expected EAV shape,
               mandatory + optional attributes, DMBOK profile) is loaded as the
               reference. It is itself a sealed particle (a schema contract KAKI).
          S2 · Conformance proof — incoming structure S is tested for a
               structure-preserving simplicial map φ: S → K_contract
               (Bābu B-6 shape test; TDA names any obstruction).
          S3 · Mandatory-attribute completeness — every mandatory attribute of
               the contract must be satisfiable from S; missing ⇒ FAIL.
          S4 · License + provenance present on the client corpus.
      → [PASS] → BeeMDM ETL proceeds (DEV→TEST→UAT)
      → [FAIL] → data is REFUSED into the pipeline:
                 · structural gap  → DubSar PDM restructuring (client-side)
                 · missing attrs   → returned to client with the named deficit
                 · never silently coerced, never partially ingested

Nothing conforms-by-assumption. The test emits a **Schema-First Report**
particle (Event KAKI + per-check EAV scores + ε), sealed, append-only.

---

## 2 · DEV → TEST → UAT (why the extra tier)

Client data earns a tier public data does not: **UAT** — User Acceptance
Testing — because a real client must *accept* the result before their data is
trusted onward.

- **DEV** — the pipeline is wired and dry-run against a tiny client sample.
- **TEST** — full Schema-First Test + bounded ETL; DMBOK re-score; round-trip
  ORBIT/PROVE/WITNESS; quarantine-path check.
- **UAT** — the **client stakeholder** exercises their own data through the
  built schema and *signs acceptance* (a sealed approval, sibling of the
  GL-DDB-003 tri-approval stakeholder seal). Only UAT acceptance makes it an
  ACC/PROD candidate (a later, separate seal).

Governing law intact: promotion to real PROD workloads still waits behind the
existing playbook program + TESTING_PHASE1. This annex governs the client-data
lifecycle and its UAT tier, not a PROD action.

---

## 3 · Relationship to the Foundry (both modes, one portal)

The Šala Ingest Foundry carries both doctrines as a mode switch:
- **DISCOVER mode** (public): Run generates the Pre-KAKI schema (GL-DDB-003 body).
- **SCHEMA-FIRST mode** (client): load the contract, run the Schema-First Test
  on an uploaded client archive, see PASS/FAIL per check, then proceed to
  DEV→TEST→UAT only on PASS.

Both converge on the same KAKI promotion once conformance + approvals hold — the
difference is entirely in how the schema is obtained and proven, never in the
particle laws downstream.

## 4 · PB Placement
- PB-343 `schema-contract` — seals a client's agreed schema as a contract particle.
- PB-344 `schema-first-test` — S1..S4 gate; emits the Schema-First Report;
  routes failures to PDM/return. Runs FROM host across DEV/TEST/UAT inventories.
- PB-345 `uat-acceptance` — records the client's sealed UAT acceptance.

## 5 · Open seals for CSR-08
Annex A adoption · Schema-First Test as a named rite · UAT-acceptance quorum
(client-only vs client+steward) · PB-343..345 numbering · whether the schema
contract particle lives in EnkiMDB (metadata) or a dedicated Contract tribe.

*Recorded in the reign of Gudea 1.0, Phase Two. Nothing herein is sealed until
DUB.SAR confirms under CSR-08.*
