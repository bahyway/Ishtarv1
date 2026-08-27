# GulaFederation Suite — PB-549 … PB-554 (Manifest Tablet)

**Engine:** GulaFederationEngine — federation witness for province-wide pharmaceutical
availability, attributes, and risk. Extends the spent Gula candidacy (GL-ORG-001)
into the federation role. **All names pending CSR-08 confirmation by DUB.SAR 𒁾.**

<!-- 2026-08-24, found live during a clean-sheet open-issues audit: this
     suite was originally planned as PB-321..PB-326, then renumbered to
     PB-549..PB-554 (per ANU_GOVERNOR_PB_MANUAL.md: "Renumbered from the
     colliding PB-321-326 block" -- PB-321/322 already belong to two
     unrelated, already-landed playbooks, playbook_321_kidinnu_engine.yml
     and playbook_322_ontograph_scaffold.yml). The six playbook files
     themselves (playbook_549..554_*.yml) already carried the new
     filenames but still had the OLD numbers in their own headers/task
     names/cross-references -- fixed to match. This table was the one
     remaining place still using the retired numbering; updated to match. -->

## Run order (all FROM Fedora 44 HOST, localhost scaffolding)

| PB | Deliverable | Depends on |
|----|-------------|-----------|
| PB-549 | `gula-federation-advisory` crate — SUSA read-only advisory API, Ed25519-sealed responses, port 7011 | PB-550 read-model |
| PB-550 | `gula-synthetic-federation` crate — deterministic Baghdad province (452 nodes, ~8,000 products, batches, three price planes) | — |
| PB-551 | `gula-mobile` Godot 4.7 scaffold — three-LOD Hubble camera rig (COSMOS→CITY→FLIGHT→ANCHOR) | — |
| PB-552 | `gula-tile-bundler` — offline OSM buildings/roads bundle (no map API at runtime; ODbL attribution) | osmium-tool |
| PB-553 | `gula-advisory-verify` crate — client-side seal verification, GDExtension-ready | — |
| PB-554 | `gula-batch-audit` crate — **Inspector sub-tab backend**: R1 expiry horizon, R2 assay deviation, R3 τ price spread, R4 cold-chain slope; EnkiQDB candidate routing | PB-550 |

Recommended first pass: **550 → 549 → 554 → 553 → 551 → 552**.
Running each playbook is the CSR-08 confirmation point; debug corrections return
as numbered follow-up playbooks (PB-555+).

## Way-of-Work compliance
- Nothing targets dubsar-workstation (monitoring-only) or the EnkiDB VMs.
- The Šala HTML dive is **prototype only** (rule 5); the production body is Godot (PB-551).
- Advisory face is **advisory-only** (Namtila principle): informs, never gates care;
  anonymous queries; append-only upstream; offline-first read-model.
- Governing law intact: the production federation itself remains gated behind
  completion + testing of the existing playbook program. This suite is scaffolding
  and synthetic-workload tooling.

## Open sealing decisions for CSR-08
1. GulaFederationEngine (federation witness) — proposed here.
2. Šamaš as the public patient-advisory face name — unspent, thematically exact.
3. Whether PB-554's audit sweep is a Gula domain rite or folds under the
   decay-vs-rite abstract pattern tablet (Apkallu registry candidate).
