# GL-MRD-003 — Orbit Spectral Diagnostics: GUE/Poisson Spacing Statistics
**Status:** SEALED (concept + real code). Runtime signal only —
implementation is a standalone Rust crate (`orbit-spectral-engine`) that
does not depend on Theater or the Nēberu Slicer existing yet; it is
ready the moment either produces real orbit return-time data.
**Domain:** MardukEngine (DPMVAS) → Nabû Calculus, sibling to GL-MRD-002.
**Origin:** answers a direct question — can the mathematics surrounding
the Riemann Hypothesis (never RH itself, which this law explicitly does
not depend on) help TOP Algebra's orbit calculus? Checked against this
repo's real state, not assumed: MUMMU already exists as this ecosystem's
real design-time-only placement tier alongside Z3; GL-MRD-002's Nēberu
Slicer already collects exactly the Poincaré-section crossing data this
law's diagnostic needs; `bahyway-algebra::enbilulu`'s real, tested Têrtu
fingerprinting is already the founding precedent GL-MRD-002 §7's
Analysis-to-Solution Law cites for DETECT→PROVE→PREDICT→PRESCRIBE;
`lamassu-engine` is real and does topology (shape) only — this law's
diagnostic is deliberately the complementary axis (rhythm), not a
duplicate.

## 1. What Riemann Hypothesis Mathematics Does NOT Give Us
The Riemann Hypothesis itself — zeros of ζ(s) on the critical line
Re(s)=1/2 — is an open conjecture. Nothing in this ecosystem may depend
on it being true. No RiemannCalculus playbook, crate, or gate exists or
is proposed here for that reason.

## 2. What the Surrounding Mathematics DOES Give Us — Three Real Imports

**2.1 Dynamical zeta functions — primes are closed orbits.** The
Artin–Mazur / Ruelle zeta function transplants ζ(s)'s "product over
primes" into dynamical systems as a "product over prime closed orbits"
(periodic orbits not traversable as repetitions of a shorter one). The
Prime Orbit Theorem then counts closed orbits in a flow the way the
Prime Number Theorem counts primes: π(T) ~ e^{hT}/hT, with topological
entropy h in place of log. This gives BIGRING a conserved counting law —
the census of prime orbits below a given period is a computable
invariant of a healthy system; drift in that census is a principled
anomaly signal, not a heuristic. **Design-time only** (zeta
regularization, entropy estimation from raw trajectory data) — belongs
with MUMMU/GeoEngine, same placement as Z3 (Gate G4): never in the
shipped binary.

**2.2 Trace formulas — the Nēberu Slicer is already the collection
apparatus.** The Selberg/Gutzwiller trace formulas state that an
operator's spectrum and a space's closed-orbit lengths determine each
other (the rigorous, dynamical-systems version of the Hilbert–Pólya
vision — zeros as eigenvalues). The natural computational entry point to
periodic-orbit data is a Poincaré section — which is exactly what
GL-MRD-002's Nēberu Slicer already is. Once built, the Slicer isn't just
a visualization tool; its SECTION crossings (`PRESENT SECTION OF ORBIT
<id> AT PLANE ... WINDOW THICKNESS <delta>`) are the raw orbit-length /
return-time data this law's §3 diagnostic consumes directly — no new
collection mechanism needed. **Design-time only** (trace-formula
derivation itself) — same MUMMU/Z3 placement as §2.1.

**2.3 GUE statistics — the cheap, runtime diagnostic payoff.**
Montgomery–Odlyzko showed the Riemann zeros' spacings follow Gaussian
Unitary Ensemble (GUE) statistics — the same level-repulsion signature
real chaotic quantum/dynamical systems show. This is the one piece of
the three that is cheap enough to actually run at runtime: compute the
nearest-neighbor spacing distribution of an orbit's return times through
a Nēberu section, and compare it against the closed-form GUE Wigner
surmise CDF and the Poisson (uncorrelated) CDF via a Kolmogorov-Smirnov
distance. Healthy chaotic dynamics show GUE-like repulsion (no
near-zero gaps, level repulsion); Poissonian clumping or crystalline
rigidity signals decoupling or pathological locking. **This is what
ships** — see §3.

## 3. What Actually Ships — `orbit-spectral-engine`
Real, tested, zero-dependency Rust crate
(`workspace/bahyway_v4/crates/orbit-spectral-engine`), sealed alongside
this law:
- `spacings(returns: &[f64]) -> Vec<f64>` — mean-normalized
  nearest-neighbor spacings from a raw (need not be pre-sorted) sequence
  of orbit return/crossing times.
- `cdf_poisson(s)` — the standard exponential(1) CDF for an uncorrelated
  point process.
- `cdf_gue(s)` — a closed-form CDF derived here (not numerically
  integrated at runtime) from the standard GUE (β=2) Wigner surmise PDF
  p(s) = (32/π²) s² exp(-4s²/π); uses a local, dependency-free `erf`
  approximation (Abramowitz & Stegun 7.1.26). The derivation is verified
  by the crate's own `cdf_gue_derivative_matches_pdf` test — the
  numerical derivative of the CDF is checked against the analytic PDF
  directly, not just asserted plausible.
- `ks_distance(sample, reference_cdf)` — the standard two-sided
  one-sample Kolmogorov-Smirnov statistic.
- `classify(returns) -> SpectralClassification` — the DETECT-phase
  entry point: computes both KS distances and returns `GueLike`,
  `PoissonLike`, or `Ambiguous` (fewer than `MIN_SPACINGS_FOR_CLASSIFICATION`
  = 5 spacings — a refusal to guess on insufficient data, not a forced
  verdict), plus both raw KS distances so a WITNESS event can show its
  work.

Real tests include synthetic fixtures for both hypotheses — a genuine
inverse-transform-sampled Exponential(1) sequence (real Poisson
nearest-neighbor law, not an approximation) correctly classifies
`PoissonLike`; a jittered evenly-spaced sequence (genuinely
non-Poissonian rhythm — no near-zero-gap clustering, no long tail, but
not literally sampled from the GUE CDF either, to avoid a circular test)
correctly classifies `GueLike`. Both fixtures are explicitly labeled
synthetic in the crate's own test names and comments — never presented
as live BIGRING data, because no live Nēberu Slicer data exists yet.

## 4. Integration into GL-MRD-002 §7's Analysis-to-Solution Law
This law's diagnostic is a DETECT-phase signal, slotting into the
already-sealed chain without inventing new plumbing:
```
DETECT   — orbit-spectral-engine::classify() on a Nēberu section's
           real return-time crossings (once the Slicer is built).
PROVE    — the classification is a certified statistic (KS distance
           against a named reference law), never eyeballed — same
           standard GL-MRD-002 §2 already sets for Betti numbers.
PREDICT  — a PoissonLike verdict on a previously GueLike orbit implies
           a real dynamics-law change (decoupling, locking) the Nabû
           covariant derivative can extrapolate forward, same as any
           other certified pattern class per GL-MRD-002 §7.
PRESCRIBE — solution candidates follow the same EMITted, advisory-only,
           WITNESSed pattern GL-MRD-002 §7 already mandates (Namtila/
           NINSUN authority boundary — this law grants no new authority).
```

## 5. Relationship to LamassuEngine — Complementary, Not Duplicate
`lamassu-engine` is real (`crates/lamassu-engine`) and answers a
different question entirely: it samples a Tribe's particles into a
point cloud and asks `bahyway_algebra::persistence` for real persistent
homology, returning GOLDEN/FUZZY/DEAD — **shape**. This law's diagnostic
never touches point-cloud topology; it operates on a 1-D sequence of
orbit return times and asks only about their spacing statistics —
**rhythm**. A healthy orbit can, in principle, show a clean GOLDEN
persistent-homology shape while its spacing rhythm degrades toward
Poissonian clumping (or vice versa) — the two signals are genuinely
independent evidence, not two views of the same computation.

## 6. Cadence & Governance
Per the same three-cadence framing GL-MRD-002 already establishes: the
spectral classification computes on the medium topological cadence (it
needs enough real crossings accumulated to be meaningful — see
`MIN_SPACINGS_FOR_CLASSIFICATION`); any census-law ratification (§2.1's
prime-orbit counting, if ever promoted from design-time theory to a
runtime invariant) is a slow-cadence, Architect-ratified decision, not
automatic. All prescriptions remain ADVISORY-ONLY under the standing
Namtila/NINSUN pattern — no solution particle from this law ever holds
blocking power.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0, sealed 2026-07-27.
