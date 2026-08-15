# PB-390 → PB-393 — SEALED SUBMISSION PLAYBOOK SUITE
## The Ansible playbooks that enforce GL-GOV-001 — the ranked submission gate
### BahyWay.Ecosystem v4.0 · executes GL-GOV-001 · binds CSR-08 · Status: DRAFT — run = DUB.SAR's CSR-08 confirmation

*Per the absolute law: every component delivered as a numbered Ansible playbook.
The gate is guided (attribute-exploration), not freehand. Way-of-Work:
Ansible HOST→VDI→EnkiDB VMs; Fedora VDI = DubSar PDM egui/WGPU only; no HTML in
production. The DubSar PDM correction IDE is the surface; these playbooks wire
its gate.*

---

**PB-390 · `submission-gate-roles`** — register the High Authorized Group and its
two axes: Architect (rank 1, truth-sovereign, CSR-08), Steward (rank 2, truth /
data-quality corrections), Administrator (last in visualization, first in
automation/infra). Encode axis-routing: truth corrections → Steward→Architect;
operation/infra changes → Administrator. Bind to the AkkadiRulesEngine (ABAC).

**PB-391 · `attribute-exploration-loop`** — deploy the guided correction loop:
PROPOSE (stakeholder frames the correction as a claim) → WITNESS-REQUIRED (extent
for an added relation; counterexample object for a refuted implication — no
counterexample, no refutation) → route by axis for RANKED APPROVAL. Refuse any
proposal lacking a witness. This is the propose→confirm/refute-with-example
discipline, in code.

**PB-392 · `naru-witness-seal`** — on admission or refusal, append a NĀRU journal
entry (proposer, approver, rank, evidence, timestamp) under Ed25519 seal; the
journal is append-only and witnessed. Nothing enters without its seal.

**PB-393 · `derived-asserted-provenance`** — tag every edge/relation as
**derived** (FCA-computed from data) or **asserted** (expert-admitted with
witness); enforce the invariant that asserted ≠ derived ≠ GOLDEN, and that an
asserted relation is never promoted without proof. Wire simplicial higher-order
relations (3+ entities) as simplices carrying the asserted tag; wire GA to render
their geometry only after admission.

---

## Run order
PB-390 (roles/axes) → PB-391 (guided loop) → PB-392 (witness seal) → PB-393
(provenance). Each tested before the next. Governing law holds: the ranked gate
runs wherever a stakeholder can propose a correction; the core pipeline and
PB-98 (BLK-1) keep sequencing priority.

*Recorded in the reign of Gudea 1.0. Running is DUB.SAR's confirmation under CSR-08.*
