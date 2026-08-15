# GulaFederation Suite — PB-321 … PB-326 (Manifest Tablet)

**Engine:** GulaFederationEngine — federation witness for province-wide pharmaceutical
availability, attributes, and risk. Extends the spent Gula candidacy (GL-ORG-001)
into the federation role. **All names pending CSR-08 confirmation by DUB.SAR 𒁾.**

## Run order (all FROM Fedora 44 HOST, localhost scaffolding)

| PB | Deliverable | Depends on |
|----|-------------|-----------|
| PB-321 | `gula-federation-advisory` crate — SUSA read-only advisory API, Ed25519-sealed responses, port 7011 | PB-322 read-model |
| PB-322 | `gula-synthetic-federation` crate — deterministic Baghdad province (452 nodes, ~8,000 products, batches, three price planes) | — |
| PB-323 | `gula-mobile` Godot 4.7 scaffold — three-LOD Hubble camera rig (COSMOS→CITY→FLIGHT→ANCHOR) | — |
| PB-324 | `gula-tile-bundler` — offline OSM buildings/roads bundle (no map API at runtime; ODbL attribution) | osmium-tool |
| PB-325 | `gula-advisory-verify` crate — client-side seal verification, GDExtension-ready | — |
| PB-326 | `gula-batch-audit` crate — **Inspector sub-tab backend**: R1 expiry horizon, R2 assay deviation, R3 τ price spread, R4 cold-chain slope; EnkiQDB candidate routing | PB-322 |

Recommended first pass: **322 → 321 → 326 → 325 → 323 → 324**.
Running each playbook is the CSR-08 confirmation point; debug corrections return
as numbered follow-up playbooks (PB-327+).

## Way-of-Work compliance
- Nothing targets dubsar-workstation (monitoring-only) or the EnkiDB VMs.
- The Šala HTML dive is **prototype only** (rule 5); the production body is Godot (PB-323).
- Advisory face is **advisory-only** (Namtila principle): informs, never gates care;
  anonymous queries; append-only upstream; offline-first read-model.
- Governing law intact: the production federation itself remains gated behind
  completion + testing of the existing playbook program. This suite is scaffolding
  and synthetic-workload tooling.

## Open sealing decisions for CSR-08
1. GulaFederationEngine (federation witness) — proposed here.
2. Šamaš as the public patient-advisory face name — unspent, thematically exact.
3. Whether PB-326's audit sweep is a Gula domain rite or folds under the
   decay-vs-rite abstract pattern tablet (Apkallu registry candidate).
