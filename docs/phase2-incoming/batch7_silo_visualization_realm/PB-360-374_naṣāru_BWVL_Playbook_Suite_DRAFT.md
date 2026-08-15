# PB-360 → PB-374 — naṣāru / BWVL PLAYBOOK SUITE
## The Ansible playbooks that build the Symbolic Visualization Programming Language and its data lifecycle
### BahyWay.Ecosystem v4.0 · executes GL-VIZ-003/004/005/006 · GL-KAKI-002 · GL-MED-003 · GL-HS3-001 · Status: DRAFT — run = DUB.SAR's CSR-08 confirmation

*Per the absolute law: every component is delivered as a numbered Ansible playbook.
Way-of-Work: (1) Ansible from HOST to eriduous-vdi; (2) Ansible from eriduous-vdi
to EnkiDB VMs; (3) Ubuntu VDI = pure-Rust monitoring; (4) Fedora VDI = DubSar
Visualizer egui/WGPU only, never HTML; (5) no HTML dashboards in production. The
Šala tabs are scratch rehearsal only.*

---

## Phase G — Grammar & medical model (already drafted, listed for the run-order)

**PB-360 · `heptascript-grounded-grammar`** — install the ASK/PROVE/GHOST/WITNESS
grammar (GL-HS3-001) into the EnkiDDB query layer; wire the four-outcome honesty
contract (FACT/WEAK/GHOST/NONE) to GOLDEN retrieval.

**PB-361 · `four-outcome-render-binding`** — bind the four outcomes to particle
render attributes (brightness/spread) at the WGPU layer spec.

**PB-362 · `ghost-surprisal-scorer`** — deploy the Eṭemmu surprisal scorer
(−log P(claim|GOLDEN corpus)) for the GHOST outcome.

**PB-363 · `convergence-query-engine`** — the symptom-first convergence engine
(GL-MED-003): observed signs → weighted disease ranking → discriminating sign.

**PB-364 · `image-junction-attach`** — attach diagnostic IMAGE particles at
disease junctions (D5/D6 of the cascade).

---

## Phase V — the BWVL / naṣāru instrument (GL-VIZ / GL-KAKI)

**PB-365 · `bwvl-generator`** — the runtime generator (GL-VIZ-003 §4): query →
GOLDEN retrieval → particle scene where nodes/edges/labels are ALL particle
assemblies. Arranges, never builds. The product, not any example.

**PB-366 · `particle-dissolve`** — local per-element zoom: clicking/entering ANY
element re-queries GOLDEN for that element's constituent particles and renders
them, recursively to the essential particle. No solid node-spheres; no overlay
edge-lines.

**PB-367 · `event-kaki-journal`** — mint one Event-KAKI per state/ColourID change;
append-only NĀRU journal; StoryEngine reads the chain to tell history and reveal
the immutable Birth Root Shade.

**PB-368 · `crosstribe-kaki-derive`** — derive CrossTribe-KAKI from 7D Hepta
near-adjacency across tribes; feed the federation layout.

**PB-369 · `colourid-eav-tribe`** — register ColourID as a Mandatory EAV attribute
on Entity=Tribe across the seven EnkiDB types; assign Birth Root Shade (root hue +
unique per-particle shade-degree) at ingest.

**PB-370 · `golden-transition-bounded`** — enforce the bounded GOLDEN colour
transition (paler/yellowish, never brown/black; Birth Root Shade preserved);
wire Steward-governed Aging/Decay via append-alert.

**PB-371 · `federation-layout`** — CrossTribe-driven placement of tribe BIGRINGs
in Multi-HeptaSpace (position earned from 7D proximity, not assigned).

**PB-372 · `bigring-tribe-render`** — render each tribe as a BIGRING in its root
colour-band with per-particle shade-degrees (WGPU compute; Niagara-technique
ported sovereign).

**PB-373 · `hubble-descent`** — the continuous zoom-in dissolve (field → region →
cluster → particle → KAKI+EAV); the reachability motion.

**PB-374 · `birdseye-focus-camera`** — the enigmatic bird's-eye focus-camera
fly-to-region; the whole-field reading motion. PB-373 + PB-374 implement
GL-VIZ-006 Zoom-as-Necessity.

---

## Phase L — the data lifecycle into EnkiDDB (this deliverable)

**PB-375 · `beemdm-medical-ingest`** — BeeMDM ETL: download public medical API
data; birth particles (Identity-KAKI + Birth Root Shade); register states as
Event-KAKI through processing stations; promote to GOLDEN. StoryEngine upstream.

**PB-376 · `golden-store-partition`** — save GOLDEN particles to EnkiDB (OLTP,
7004); partition/snapshot to EnkiDW (OLAP, 7005).

**PB-377 · `enkiddb-category-schema`** — EnkiDDB (7007) receives GOLDEN particles
and builds its OWN category schemas (tribes) for the medical scenario; seals each
schema as a Major Template.

**PB-378 · `heptascript-notebook`** — deploy the HeptaScript Notebook surface:
runnable ASK cells that simulate and visualize the internal data lifecycle as a
naṣāru scene (the Šala tab is the rehearsal; production = DubSar Theater notebook
cell + WGPU render).

**PB-379 · `nasaru-lifecycle-sim`** — the end-to-end lifecycle simulation:
source → BeeMDM → EnkiDB/EnkiDW → EnkiDDB category schemas → notebook query →
naṣāru federation render, driven entirely from GOLDEN.

---

## Run order
Phase G (PB-360–364) → Phase V (PB-365–374) → Phase L (PB-375–379). Each phase
tested before the next. Governing law still holds: finish + test ALL playbooks
before building new components; PB-98 (KISPU/ENLIL bridge) remains the BLK-1
blocker for the core pipeline — the naṣāru visualization research runs in
parallel and *specifies what the pipeline must feed*, but the internal
I→P→S→O pipeline + GOLDEN scoring keep their sequencing priority.

*Recorded in the reign of Gudea 1.0. Running is DUB.SAR's confirmation under CSR-08.*
