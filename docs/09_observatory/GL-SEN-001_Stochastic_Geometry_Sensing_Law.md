# GL-SEN-001 (candidate) — STOCHASTIC-GEOMETRY SENSING LAW
## Corridor infrastructure defect detection & area hazard mapping, as a new naṣāru sensing phase
### BahyWay.Ecosystem v4.0 · binds GL-DST-001 §4 · GL-MEM-001 · GL-VIZ-000/003/004/005/006 · GL-HS3-002 · Status: SEALED-CONCEPT (per CSR-08 chat confirmation, 2026-08-15)

*Drafted 2026-08-15 from a standalone Architect design conversation
(`shala-prototypes/batch8_nasaru_sensing_membrane_courts/`), evaluated
against and bound into the law already sealed/drafted in this repository
— that source conversation did not have this repository's naṣāru,
Mašḫalu, or GL-DST-001 tablets in view, so this tablet re-grounds its
proposal against the real ones rather than reproducing it verbatim.*

---

## 0 · The name and the seat

This is naṣāru's fourth phase, sitting beside the three already reserved
in `playbooks/PB-360-374_naṣāru_BWVL_Playbook_Suite_DRAFT.md`:

- **Phase G** — Grammar & medical model (PB-360–364).
- **Phase V** — the BWVL / naṣāru instrument core (PB-365–374).
- **Phase L** — the data lifecycle into EnkiDDB (PB-375–379).
- **Phase S — Stochastic-Geometry Sensing (this tablet)** — naṣāru
  watching a physical corridor or area, not a data lifecycle: the same
  particle-monist, membrane-projected, zoom-as-necessity instrument
  (Phase V), pointed at hyperspectral/multispectral/SAR/thermal cubes and
  answering, with a witnessed statistic rather than a thresholded heat
  map, whether an anomaly is real structure or coincidence.

No new geometry is invented (A-1, `GL-STD-002` §2: compose, never
invent). Every mathematical object below — Ripley's K-function, the
pair-correlation function g(r), the empty-space function F(r), RX/matched-
filter anomaly scoring, Boolean-model coverage — is standard, decades-old
stochastic geometry and signal processing, composed here into one rite
sequence and bound into the sealed instrument that already exists.

---

## 1 · The Law

**A sensed anomaly (spectral, thermal, or point-pattern) becomes a
witnessed claim only after its spatial arrangement is tested against a
null model on the surface where it actually lives — never on a flat
projection, never on brightness/threshold alone. The stage that renders
the anomaly and the instrument that tests it are two different objects,
per GL-DST-001 §4: the stage may be theater, the K(r)-against-Poisson-
null panel may never bend.**

Corollary (the honest boundary): a witnessed spatial claim is a
statement about *pattern*, never automatically a statement about
*cause* or *identity*. "This cluster's K(r) escapes the Poisson envelope
at ε" is provable from geometry alone. "This cluster is a gas leak" or
"this crater field conceals ordnance" requires the domain's own sealed
signature library and, for anything above ground-truth-blind terrain,
the two-witness calibration of §5. This law never licenses skipping that
second step.

---

## 2 · Why the surface must be Mašḫalu, not a flat plane

A pipeline is a cylinder; a bombardment sector is a terrain sheet. Both
are already the exact shape `GL-MEM-001` (Mašḫalu) describes: an elastic
membrane that particles dent under their own "gravity" — here, the
anomaly's spectral/thermal score plays the role Mašḫalu already assigns
particle-gravity, and the dent field literally is a spatial kernel-density
estimate rendered as physics, not decoration (§3 below). Flattening the
corridor to a 2D plane for rendering — the mistake the source batch's own
v1/v2 rehearsals made before self-correcting to v3/v4 — throws away
information twice:

- **Visually**, a coherent basin around one shared fault reads
  identically to isolated unrelated pricks once flattened; the membrane
  makes shared deformation vs. isolated deformation visible before any
  statistic runs.
- **Mathematically**, distance on a cylinder is *geodesic*
  (`d = √((Δs·L)² + (R·Δθ)²)`, the unrolled-cylinder metric), not the
  straight Euclidean line a flat rendering implies. A pipeline membrane
  unrolls isometrically, so the geodesic K-function computed on it is
  **exact**, not an approximation corrected for later. Terrain sheets use
  the planar metric directly, since the ground genuinely is flat at this
  scale.

This binds Phase S to `GL-VIZ-003` (Particle Monism: the membrane is
particles at a scale) and to `GL-MEM-001` §3 (particle-gravity as the
readout of the surface it deforms) rather than introducing a second,
competing rendering convention.

---

## 3 · The rite sequence

1. **Cube Ingest** — a hyperspectral/multispectral/SAR/thermal cube (or
   drone RGB photogrammetry, for crater point patterns at the centimeter
   scale) is ingested as a lattice of particles: every pixel a particle,
   its spectral vector its EAV attributes, its (x, y) its Hepta-Space
   post. Maps onto KAKI + EAV with no new primitive (`GL-VIZ-003`).
2. **Spectral Anomaly** — a background-covariance statistic (RX
   anomaly detector / matched filter / adaptive cosine estimator — all
   Mahalanobis-style, pure linear algebra, no external model) scores each
   particle against its medium's sealed signature profile (§4). This
   step alone is known to false-alarm on isolated pixels; it makes no
   claim yet.
3. **Spatial Witness (this tablet's core addition)** — anomaly particles
   attach to and dent the Mašḫalu membrane (§2). Ripley's K(r) (or g(r),
   or F(r) for the empty-space form) is computed with the surface's real
   geodesic or planar metric, against a Poisson-null population of the
   same count on the same surface, at confidence 1−ε. K_obs escaping the
   null envelope is the witnessed statement "this is structure, not
   coincidence" — GL-HS3-002's ε already governs exactly this
   uncertainty-measure vocabulary; Phase S is its first concrete
   spatial user. Rendered per GL-DST-001 §4: the membrane is stage, the
   K(r) chart beside it is the one panel that never bends.
4. **Ground Truth Calibration (two-witness)** — binds `GL-MEM-001` §3's
   two-witness integrity rule to this domain: a sparse set of real
   ground/lab samples calibrates and confirms the cube-to-hazard
   regression (this is the *only* honest path for anything the spectrum
   doesn't detect directly — heavy metals, explosive-residue byproducts —
   which bind to iron oxides/clays/organics that *do* have a spectral
   signature). The cube proposes; ground samples confirm; only then does
   the calibrated model extrapolate across the scene.
5. **Risk Field** — the calibrated, spatially-witnessed anomaly score
   becomes a continuous risk field over the surface: for corridors, a
   Boolean-model coverage computation (fraction of the easement "covered"
   by an anomaly's influence zone); for area hazard, an inhomogeneous
   point-process intensity fit to detected craters, giving a principled
   contamination-likelihood surface *between* the visible wounds — the
   interpolation a deminer or returning family actually needs.
6. **Corridor / Navigation Mint** — cost-field pathfinding over the risk
   field mints safe corridors, sampling-priority zones, and no-go
   polygons, each carrying its own ε, sealed and phone-verifiable per the
   Kidinnu civil-protection discipline already drafted in
   `shala-prototypes/batch4_unified_algebra_kidinnu/`. Geometrically this
   is the DŪRU wall law (PH-004) inverted: walls drawn around harm
   instead of around the golden store.

---

## 4 · The medium profiles (sealed signature libraries, one law, five chapters)

The rite sequence above is identical across all five; only the sealed
signature library and the physics gloss differ per medium — an
`GL-ONT-001` (OntoGraph / Nebuchadnezzar) formal-concept closure over
{spectral family, spatial-statistic verdict, corridor geometry, temporal
persistence} finds this invariant intent as a discovered Unified Pattern,
not an assumed one.

- **Water** — moisture halo: near-isotropic basin from soil-moisture
  absorption near 1.4/1.9 µm and vegetation stress above the joint.
- **Oil** — gravity run: reflectance shift in visible/NIR, the film
  running downslope from the fault — the dent field's asymmetry *is* the
  drip direction.
- **Gas** — wind-sheared plume: hydrocarbon SWIR absorption (methane/
  ethane ≈ 2.3 µm), elongated along the wind, corridor-aligned.
- **Electricity — the one honest correction.** Power lines do not leak a
  substance, so the water/oil/gas spectral chemistry does not apply.
  What occupies the identical architectural slot: thermal-band hotspots
  at joints/clamps, UV/spectral corona-discharge signatures on
  insulators, vegetation-encroachment indices along the right-of-way —
  still anomalous particles clustered along a corridor, so the rite
  sequence and the Unified Pattern survive intact; the sealed profile
  swaps chemistry for thermal/corona proxies. This law is named "corridor
  infrastructure defect detection," not "leak detection," precisely so
  this chapter is not a category error.
- **Soil / area hazard (war zones, disaster sites)** — the point-process
  chapter: hydrocarbon/fuel residue, combustion ash, disturbed-vs-
  undisturbed soil (the mass-grave and crater signature), acid/alkaline
  scars directly; heavy metals and explosive byproducts indirectly via
  §3.4's two-witness regression. Vegetation stress and post-earthquake
  liquefaction/groundwater-disruption indices are a third witness.

  **UXO LIMITATION CLAUSE (binding, not advisory):** this rite never
  detects buried unexploded ordnance directly. It detects proxies —
  crater point patterns, disturbed-soil signatures, burn scars — from
  which a hazard *probability* surface is inferred. Any rendering,
  report, or corridor mint under this law that implies clearance,
  certainty, or direct UXO detection is a violation of this tablet, not
  a permitted interpretation of it. This clause is the sworn boundary the
  Never-Averaged Theorem already models elsewhere in this ecosystem's
  civil-protection calculus: a stated refusal, not an omission.

---

## 5 · Binding to the sealed/drafted set

- **`GL-DST-001` §4 (Stage, Never Truth):** governs the stage/instrument
  split at the heart of §3.3 — already sealed, not reinterpreted here.
- **`GL-MEM-001` (Mašḫalu):** the corridor/terrain surface, its dent
  physics, and its two-witness integrity rule are Mašḫalu's, reused
  wholesale (§2, §3.4).
- **`GL-VIZ-000`/`GL-GLOSSARY (naṣāru/BWVL)`, `GL-VIZ-003/004/005/006`:**
  Particle Monism for cube ingest (§3.1), the camera-deck vocabulary
  (survey/ground/orbit/bracket/journal) for staging, Zoom-as-Necessity
  for descending from field to particle on a sensed scene exactly as on
  any other naṣāru scene.
- **`GL-HS3-002` (ε):** the uncertainty-measure vocabulary Phase S's
  spatial-witness step reports in; not a new ε concept, its first
  concrete spatial application.
- **`GL-ONT-001` (OntoGraph/Nebuchadnezzar):** the Unified Pattern claim
  of §4 is exactly an FCA closure, the same mechanism already sealed
  there.
- **PH-004 (DŪRU):** §3.6's corridor mint is the wall law inverted.
- **Kidinnu (batch4, unsealed/proposed):** the navigation-output
  verification discipline §3.6 borrows is itself not yet sealed; this
  tablet inherits that open status rather than sealing Kidinnu by proxy.

---

## 6 · Two practical honesty notes (kept, not softened, from the source conversation)

- **Resolution is a real limit, not a detail.** A 30 m satellite pixel
  sees a methane plume well and a small water seep poorly; production-
  grade area coverage is a drone-sensor cost line, not a given.
- **Compute stays inside the billion-particle budget** only if the
  stochastic-geometry pass (§3.3) runs on the anomaly subset a cube's
  RX pass already isolated, never on the raw cube globally — the same
  discipline `GL-ONT-002` already requires of Phase 0 recognizers
  (bounded, offline, no scope creep into the full corpus).

---

## 7 · Codex compliance & placement

- **A-1 compose, never invent** (`GL-STD-002` §2): every mathematical
  object in §3 is standard, pre-existing stochastic geometry / signal
  processing; the composition into one rite sequence bound to naṣāru/
  Mašḫalu/GL-DST-001 is what's new here.
- **Evidence, not proof:** `shala-prototypes/batch8_nasaru_sensing_membrane_courts/shala_membrane_courts_v4.html`
  is a synthetic-data rehearsal of the staging/instrument grammar (§2,
  §3.3), not a validated detector on real cubes — this tablet does not
  claim otherwise.
- **PB:** PB-402–409 (the naṣāru Sensing suite, see
  `playbooks/PB-402-409_Nasaru_Sensing_Stochastic_Geometry_Suite_DRAFT.md`).

## 8 · Open seals for CSR-08

The name **Phase S — Stochastic-Geometry Sensing** as naṣāru's fourth
phase · the Law of §1 (spatial witness required before an anomaly is a
claim; stage/instrument split per GL-DST-001 §4) · the geodesic-on-
Mašḫalu binding of §2 · the six-rite sequence of §3 · the five medium
profiles and the UXO limitation clause of §4 · the binding set of §5 ·
PB-402–409.

*Recorded in the reign of Gudea 1.0. The membrane dents to the anomaly's
weight; the chart beside it does not bend to match. What the surface
shows and what the instrument proves are kept two things, on purpose.*

## 9 · Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-15
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work, not fabricated here.
```
