# GL-MRD-002 — Nēberu Slicer: Orbit Section & 7D Wave Law
**Status:** SEALED (concept), amended to Rev. 2. Implementation deferred
until BeeMDM ETL General/Specific testing completes.
**Domain:** MardukEngine (DPMVAS) → Nabû Calculus.
**Naming:** "Nēberu Slicer" — PROPOSAL, pending Architect
confirmation. Nēberu = Marduk's star, "the crossing point." Note: the
cross-terrain solar-farm defect calculus considered "Neburu" as a name
too, but that was superseded — the Architect sealed **IgigiEngine** for
that domain instead, so no collision exists.

## 1. Problem Statement
Dense orbits (e.g., BIGRING at 13M+ particles) become
unreadable blurs under magnification. Zooming (the "Hubble
limit") cannot resolve patterns created by millions of
windings. Resolution is not the bottleneck; superposition is.

## 2. Law Statement — Section, Don't Zoom
The Nēberu Slicer applies the Poincaré section principle:
a plane P is cut through Hepta Space, and only particle
crossings of P (within WINDOW thickness δ) are presented.
Superposed windings collapse into readable structure:
- Closed curves on the section → invariant tori (stable law)
- Island chains → resonances between dimensions
- Scattered seas → chaotic regimes
- Betti numbers of the section (β₀, β₁) certify the pattern
  class via PROVE — pattern claims are proofs, not eyeballing.

## 3. Second Stage — 7D Wave Decomposition
Every section can be re-expressed as WAVEBANDs: a harmonic
decomposition of the crossing distribution along each of the
seven Hepta axes, weighted by the Nabû metric
g = diag(w₁…w₇). Hidden periodicities invisible in spatial
view appear as sharp harmonic peaks in specific dimensions.
Section view answers WHERE structure lives; waveband view
answers WHICH dimension is producing it.

## 4. HS-EXT-004 (proposal) — Nouns Only, No New Verbs
New geometric NOUNS: SECTION, PLANE, WAVEBAND, HARMONIC.
Reused: WINDOW clause (section thickness), existing verbs
PRESENT / ORBIT / EMIT / PROVE / SYNC / WITNESS. Canonical
query form:

```
PRESENT SECTION OF ORBIT <orbit-id>
  AT PLANE NORMAL(D<k>) OFFSET <r>
  WINDOW THICKNESS <delta>
  EMIT WAVEBAND BY DIMENSION D1..D7 METRIC g
  PROVE BETTI
  WITNESS
```

Anti-SQL absolute: no SELECT/WHERE/JOIN ever. The section
predicate lives in the PRESENT clause geometry, not in a
filter expression.

## 5. Execution & Rendering
- ŠUMU-UKIN plans SECTION queries against the ENLIL index
  stack; crossing detection uses HeptaShellIndex zones to
  prune non-intersecting shells before the plane test.
- Runtime is pure sovereign Rust; target obeys the 1-billion-
  particles-under-1-second law (sections prune, not scan).
- DubSar Theater gains a third lens — SECTION — beside
  ORBIT 3D and GRID. Slice plane is draggable; waveband
  panel renders the 7 harmonics with ColourID B11 hues
  (240-scale, per Plimpton 322 law).
- Every presented section is WITNESSed: plane parameters,
  δ, metric weights, and Betti certificate are inscribed as
  an immutable KAKI event — reproducible science, DUB.SAR
  style: the observation is a tablet.

## 6. Stakeholder & Pipeline Note
Section studies performed for analysis campaigns route
through the standard design-time pipeline when they define
new schema: PDM simplicial complex → Z3 at Gate G4
(Pre-Template, design-time only) → AAOL Core generates and
consumes the .akk tablet → HeptaScript + EAV schema.
ROLE_PARTICLE_MODELER (fifth Θ stakeholder) is the natural
owner of the SECTION lens.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.

<!-- BEGIN PB-168 ANALYSIS-TO-SOLUTION LAW -->

## 7. Analysis-to-Solution Law (Rev. 2, sealed 2026-07-11)

**Law statement:** Observation that does not emit prediction
is an incomplete act. Every WITNESSed pattern must flow
through the full chain:

```
DETECT → PROVE → PREDICT → PRESCRIBE
```

1. **DETECT** — SECTION reveals structure (islands, tori,
   chaotic seas, β-signatures) that superposition hid.
2. **PROVE** — the pattern class is certified (Betti
   numbers, harmonic peaks per dimension), never eyeballed.
3. **PREDICT** — the certified class implies a dynamics
   law; the Nabû covariant derivative ∇ extrapolates it
   forward on the g = diag(w₁…w₇) manifold: trajectories,
   horizons, commitment boundaries.
4. **PRESCRIBE** — the law is inverted to generate solution
   candidates: perturbations to metric weights, attribute
   interventions, or boundary conditions that dissolve a
   harmful resonance or move a horizon. Domain instances:
   fire → suppression actions that contract the horizon;
   Šazu → controls that close a fraud loop; Addu cyclone →
   evacuation timing that beats the commitment boundary;
   Enbi → Têrtu diagnosis → Mīlu alert (the founding
   precedent of this law — real and tested in
   `bahyway-algebra::enbilulu` as of 2026-07-10).

**Solution particles:** every prescription is EMITted as an
immutable KAKI particle carrying (a) the PROVE certificate
of the pattern it answers, (b) κ residual, (c) measurement
uncertainty ε, and (d) an advisory flag (NINSUN pattern).
A solution is a witnessed tablet, never a floating opinion.

**Authority boundary:** all prescriptions are ADVISORY-ONLY.
The system proposes; the Architect or domain commander
decides. No solution particle ever holds blocking power or
cryptographic authority (Namtila/NINSUN law).

**HeptaScript form (HS-EXT-004 nouns, no new verbs):**

```
PRESENT SECTION OF ORBIT <orbit-id>
  AT PLANE NORMAL(D<k>) OFFSET <r>
  WINDOW THICKNESS <delta>
  EMIT WAVEBAND BY DIMENSION D1..D7 METRIC g
  PROVE BETTI
  EMIT SOLUTION CANDIDATES WITH RESIDUAL κ, UNCERTAINTY ε
  WITNESS
```

— Amendment inscribed for DUB.SAR 𒁾, sealed 2026-07-11.

<!-- END PB-168 ANALYSIS-TO-SOLUTION LAW -->
