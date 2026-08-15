# GL-ADU-002 — Addu Cyclone Extension Law
**Status:** SEALED (concept). Implementation deferred until BeeMDM ETL
General/Specific testing completes.
**Domain:** MardukEngine → Addu Calculus (climate).
**Authority note:** Addu (climate calculus) is DISTINCT from
ADAD (metadata authority). Advisory-only (Namtila pattern).

## 1. Law Statement
The Wave→Horizon→Orbit Law sealed for wildfire and vertical
(tower) fire generalizes without structural change to tropical
cyclones. One law, three domains, differing only in the metric
weights g = diag(w₁…w₇) of Hepta Space and in the EAV mandatory
attribute set. A hurricane is not merely representable as an
orbit structure — it IS one:

| Cyclone phenomenon            | Triple-O construct                    |
|-------------------------------|---------------------------------------|
| Outer rain bands              | Particle waves (pre-orbit layer)       |
| Eyewall                       | Composed BIGRING orbit ring            |
| Eye                           | Intrinsic β₁ = 1 topology              |
| Concentric eyewall cycle      | β₁ = 2 (double ring) — replacement law |
| Spawned tornado / vortex      | β₀ jump in wave layer                  |
| Cone of uncertainty           | FUZZY probability gradient             |
| Landfall commitment boundary  | Horizon zone (irreversibility band)    |
| Forecast-track error          | κ residual (barû pattern, per Enbi)    |
| Ocean sensor blind spots      | Gap(n) = Θ_E7(n) − Θ_found(n)          |
| Steering flow + Coriolis      | Anisotropic wave-field driver          |

## 2. Horizon Semantics
- INSIDE horizon: landfall/strike committed. Predicted particle
  states transition FUZZY → GOLDEN. PROVE-grade statements only.
- OUTSIDE horizon: genuinely open. Evacuation and mitigation
  decisions still matter. States remain FUZZY with gradient.
- Horizon derivative = control indicator: contracting horizon
  (weakening/recurving storm) vs expanding horizon (intensifying
  or accelerating storm). Leading indicator, not lagging.
- HEALTH remains a continuous EAV attribute H (orbit radius +
  ColourID B11 = round(H(P) × 240)); it is never a state.

## 3. Topological Alarms (Betti)
- β₀ jump ahead of the wave front: spawned tornado, secondary
  vortex, or detached convective cluster — the discontinuous
  hazard class, detected before orbit composition.
- β₁ = 1 stable: mature eye. β₁ = 2: eyewall replacement cycle
  in progress (intensity forecast inflection point).
- β₁ > 0 over land sectors: encircled population zones —
  direct evacuation-routing alarm.

## 4. Design-Time Pipeline (unchanged, per AAOL-as-Core)
Sector contents (population, infrastructure, surge exposure)
→ simplicial complex in PDM
→ Z3 proves the stakeholder equation Θ (emergency management,
  population, insurers, ROLE_PARTICLE_MODELER as fifth) at
  Pre-Template stage, Gate G4 (MUMMU) — design-time only
→ sealed equation handed to AAOL Core, which GENERATES the
  .akk tablet, then CONSUMES the same tablet to emit
  HeptaScript (ORBIT / EMIT / PROVE / WITNESS over wave,
  horizon, and orbit layers) plus the EAV schema
→ NUZI genealogy links every regenerated tablet to its
  ancestor; re-migration through AAOL re-proves at Gate G4
  (old proof, old tablet; new proof, new tablet).

**Implementation status note (added on landing, 2026-07-11):** AAOL
(`crates/aaol`) is real and substantial, but as of this date it only
CONSUMES `.akk` (Lexer→Parser→Semantic→CodeGen to Rust/Python/JSON/PS/
XML/HeptaScript). No code path GENERATES a `.akk` tablet from a
Z3-proven PDM equation yet — that "write the tablet" direction, and Z3
integration itself (no `z3` dependency exists in any `Cargo.toml`
today), remain design intent, not built. `MUMMU`/Gate G4 is real —
`bahyway_core::HeptaGate::Mummu = 4`.

## 5. Transparency Telos
All advisories carry visible κ residual and measurement
uncertainty ε per the τ equation. The system shows WHY it
predicts. It never holds decision authority.

## 6. Unified Addu Family (as of this seal)
One Wave→Horizon→Orbit Law, three instantiations:
1. Wildfire (horizontal, wind-dominated, chaotic anisotropy)
2. Tower fire (vertical, buoyancy-dominated, discrete descent)
3. Tropical cyclone (rotational, steering-flow anisotropy,
   intrinsic orbital topology)

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
