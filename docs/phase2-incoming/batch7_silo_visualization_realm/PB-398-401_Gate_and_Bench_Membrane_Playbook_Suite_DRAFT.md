# PB-398 → PB-401 — COMPRESSION GATE & BENCH MEMBRANE PLAYBOOK SUITE
## The Ansible playbooks that enforce GL-GOV-003 (Compression Gate) and GL-TOOL-001 (Bench Membrane)
### BahyWay.Ecosystem v4.0 · executes GL-GOV-003 · GL-TOOL-001 · binds GL-GOV-001/002 · Status: DRAFT — run = DUB.SAR's CSR-08 confirmation

*Per the absolute law: every component delivered as a numbered Ansible playbook.
These are enforcement playbooks — their acceptance test is that the forbidden
crossings and forbidden tool-uses FAIL, not merely that the allowed ones work.
Way-of-Work: Ansible HOST→VDI→EnkiDB VMs; Fedora VDI = DubSar egui/WGPU only;
no HTML in production.*

---

## Compression Gate (GL-GOV-003)

**PB-398 · `compression-gate-checkpoint`** — wire the Gate as the sole input to
the stakeholder-view renderer: the renderer reads only Gate output, never the
Simulation layer directly. Implement default-closed-for-simulation: discovered
structure flows; simulated structure requires a crossing token. A shape without a
token is undrawable in the stakeholder view (enforced by absence of code path).

**PB-399 · `gate-signature-record`** — implement the signature record (signer,
rank, evidence, timestamp) in the append-only NĀRU journal under Ed25519 seal;
the Gate emits a crossing token only against a valid record. Wire the three
conditions: reviewed, admitted-with-evidence-and-signature, provenance-bearing.
Rulings are revisable (a signature can be withdrawn; the crossing token is
revoked).

**PB-400 · `provenance-survives-compression`** — enforce the origin-class channel
on every stakeholder ribbon: `derived` (solid) / `admitted` (solid + signer mark)
/ `candidate` (dashed, provisional). Block any render that cannot carry the origin
class for a non-derived shape. Acceptance test: an unsigned simulated shape and a
provenance-stripped ribbon both FAIL to render.

---

## Bench Membrane (GL-TOOL-001)

**PB-401 · `tool-intake-membrane`** — wire the tool-intake check into the
build/CI: for every external tool, assert the two-question test — (1) does it
decide truth? (cleanse) (2) does it ship in the sovereign binary? A tool passes
CI only if both are no. Enforce at the artifact level: the shipped binary is
pure Rust; presence of any non-Rust runtime (Julia, Z3, Wolfram, a neural
runtime) in the delivered artifact FAILS the build. Bench tools are permitted
only in the design-time/research manifest, never the ship manifest. Record the
pattern-not-tool corollary: where a capability is wanted but the tool is stopped,
require a pure-Rust re-implementation validated against the tool as bench witness.

---

## Run order
PB-398 → PB-399 → PB-400 (the Gate, in order) → PB-401 (the tool membrane, wired
into CI). Each tested by attempting the forbidden action and confirming it fails.

*Recorded in the reign of Gudea 1.0. Running is DUB.SAR's confirmation under CSR-08.*
