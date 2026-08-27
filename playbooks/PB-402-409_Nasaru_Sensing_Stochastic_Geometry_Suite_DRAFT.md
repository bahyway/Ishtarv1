# PB-402 → PB-409 — naṣāru SENSING · STOCHASTIC-GEOMETRY PLAYBOOK SUITE
## The Ansible playbooks that build naṣāru's fourth phase — corridor infrastructure defect detection & area hazard mapping
### BahyWay.Ecosystem v4.0 · executes GL-SEN-001 · binds GL-DST-001 §4 · GL-MEM-001 · GL-VIZ-000/003/004/005/006 · GL-HS3-002 · Status: DRAFT — run = DUB.SAR's CSR-08 confirmation

*Per the absolute law: every component is delivered as a numbered Ansible
playbook. Way-of-Work: (1) Ansible from HOST to eriduous-vdi; (2) Ansible
from eriduous-vdi to EnkiDB VMs; (3) Ubuntu VDI = pure-Rust monitoring;
(4) Fedora VDI = DubSar Visualizer egui/WGPU only, never HTML; (5) no
HTML dashboards in production. `shala_membrane_courts_v4.html`
(`shala-prototypes/batch8_nasaru_sensing_membrane_courts/`) is scratch
rehearsal only, per this same rule — this suite is what makes it real.*

Runs after Phase V (`PB-360-374_naṣāru_BWVL_Playbook_Suite_DRAFT.md`,
PB-365–374) is built and tested: Phase S reuses the naṣāru generator,
camera-deck vocabulary, and Mašḫalu membrane rendering wholesale (see
`GL-SEN-001` §2, §5) rather than re-implementing them.

---

## Phase S — Stochastic-Geometry Sensing (GL-SEN-001)

**PB-402 · `sensor-cube-ingest`** — ingest a hyperspectral/multispectral/
SAR/thermal cube (or drone RGB photogrammetry for crater point patterns)
as a lattice of particles: pixel → KAKI + EAV spectral vector + Hepta-
Space post. GL-SEN-001 §3.1.

**PB-403 · `spectral-anomaly-rx`** — pure-Rust RX anomaly detector /
matched filter / adaptive cosine estimator against a medium's sealed
signature profile (PB-408); background-covariance linear algebra, no
external model, no dependency. GL-SEN-001 §3.2.

**PB-404 · `geodesic-spatial-witness`** — attach anomaly particles to the
Mašḫalu membrane (cylinder for corridors: exact geodesic metric
`d = √((Δs·L)² + (R·Δθ)²)`; plane for terrain sheets); compute Ripley's
K(r) / g(r) / F(r) against a same-count Poisson null on that surface at
confidence 1−ε; render the flat, never-bending instrument panel per
GL-DST-001 §4. This is the load-bearing playbook of the suite — the
step that turns a brightness threshold into a witnessed spatial claim.
GL-SEN-001 §3.3.

**PB-405 · `ground-truth-calibration`** — the two-witness rite
(GL-MEM-001 §3 reused): register a sparse set of real ground/lab samples;
calibrate and confirm the cube-to-hazard regression before any
extrapolation across the scene is permitted. Required for any target the
spectrum only indirectly detects (heavy metals, explosive-residue
byproducts). GL-SEN-001 §3.4.

**PB-406 · `risk-field-mint`** — continuous risk field over the surface:
Boolean-model easement coverage for corridors; inhomogeneous point-
process intensity fit to detected craters for area hazard, yielding a
contamination-likelihood surface between visible wounds. GL-SEN-001 §3.5.

**PB-407 · `corridor-navigation-mint`** — cost-field pathfinding over the
risk field; mints safe corridors / sampling-priority zones / no-go
polygons, each carrying its own ε, Ed25519-sealed and phone-verifiable
per the (unsealed/proposed) Kidinnu civil-protection discipline. Refuses
to mint anything the UXO limitation clause (GL-SEN-001 §4) forbids
implying. GL-SEN-001 §3.6.

**PB-408 · `medium-signature-profiles`** — load and seal the five medium
profiles (water/oil/gas/electricity/soil-war-zone) PB-403 and PB-406
depend on, including the electricity thermal/corona correction and the
UXO limitation clause text itself, so it ships as data, not something a
caller can omit by forgetting a comment. GL-SEN-001 §4.

**PB-409 · `membrane-court-render`** — the DubSar egui/WGPU production
twin of `shala_membrane_courts_v4.html`: five courts, one grammar,
dents-are-physics/glow-is-threshold, full naṣāru camera deck (⌖/✛/⟳),
Ṣabātu particle-picking with StoryEngine journal — reusing PB-365–374
(naṣāru core) and PB-380–389 (Mašḫalu) rather than a second rendering
path. GL-SEN-001 §2, §5.

---

## Run order

PB-402 (ingest) → PB-403 (spectral anomaly) → PB-404 (spatial witness) →
PB-405 (ground-truth calibration) → PB-406 (risk field) → PB-407
(corridor/navigation mint), with PB-408 (signature profiles) available
to PB-403/406 from the start and PB-409 (render) buildable in parallel
once PB-404's membrane/instrument split is proven — the render depends
on naṣāru's generator (PB-365) and Mašḫalu (PB-380–389), not on PB-405–
407's calibration/mint logic. Each playbook tested before the next.
Governing law still holds: PB-98 (KISPU/ENLIL bridge) remains the BLK-1
blocker for the core pipeline; this suite specifies what the pipeline
must feed and runs in parallel with it, same standing as Phase V/L.

*Recorded in the reign of Gudea 1.0. Running is DUB.SAR's confirmation
under CSR-08.*
