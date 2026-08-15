# GeoEngine — `bahyway-algebra` + the Algebra Arsenal

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
Verified against real source and `cargo test` output on 2026-07-21 — no
claim below is asserted without a crate name and a passing test count.**

---

## What "GeoEngine" is

"GeoEngine" is a concept-document name (at least four sealed documents —
HS-EXT-002, GL-MRD-002, GL-DST-001, TPL-001 — use it), not a separate
crate. Confirmed against `bahyway-algebra`'s own `lib.rs`: **GeoEngine =
`bahyway-algebra`**, the ecosystem's single mathematical truth source.
Every GeoLaw below is enforced there, or in a crate `bahyway-algebra` is
the truth source for. `algebra-arsenal` (`crates/algebra-arsenal`) is a
second, separate crate that indexes/re-exports the primitives below from
wherever they actually live — the one-command sanity check for any of
this is `cargo test -p algebra-arsenal -p bahyway-field`.

## The Arsenal — every primitive, real home, verified status

| Concept | Real home | Status |
|---|---|---|
| Field (abstract, ℝ) | `bahyway-field::RealField` | ✅ real, 7 axiom tests |
| Zmod240 (B11 ring) | `bahyway-field::Zmod240` | ✅ real — a **ring**, not a field (ℤ/240 is composite; 2 has no inverse mod 240) |
| Vector Space (ℝ⁷) | `kinetic-engine::Vec7D` | ✅ real Add/Sub/Mul<f64>, dot/magnitude/normalize |
| Inner Product Space, weighted | `hepta-score` (`health_score`/`weighted_distance`) | ✅ real — H(P) = 1/(1+√Σwᵢ(Pᵢ−Tᵢ)²) |
| Simplex / Simplicial Complex | `najaf-engine::topology` | ✅ real — barycentric 7D membership, Gaussian-elimination ghost reconstruction |
| Eigenvalue / Eigenvector | `ea-agent-algebra::matrix::SovereignMatrix` | ✅ real — exact 2×2 including complex, power-iteration spectral radius |
| Jordan Normal Form | `ea-agent-algebra::jnf` + `jordan.rs` | ✅ complete for symmetric matrices (Jacobi eigendecomposition — spectral theorem guarantees size-1 blocks). General defective-matrix JNF explicitly **not built** |
| Clifford Algebra Cl(7) | `bahyway-algebra::clifford` | ✅ real, full 128-blade Cl(7,0) |
| Bivector | `Multivector::grade(2)` | ✅ real |
| Rotor | `bahyway-algebra::rotor` | ✅ real, 7 tests |
| Spinor | — | ⚠️ partial — only the single-plane Rotor form exists; general multi-plane spinor does not |
| Octonions | `bahyway-algebra::octonion` | ✅ real — Cayley-Dickson doubling (ℝ→ℂ→ℍ→𝕆), verified via norm multiplicativity and confirmed loss of associativity |
| Manifold / Geodesic / Covariant Derivative / Riemannian Curvature | `vgca-engine::riemannian` | ✅ real — general metric-tensor machinery, verified against a known analytic case (sphere, K=1/r²) |
| Shannon Entropy | `vgca-engine` (`bfv`, `fsv`) | ✅ real — two implementations (byte + text) |
| KL Divergence | `vgca-engine::kl_divergence` | ✅ real — fixed ε=1e-10 smoothing per Sovereign Rule |
| Directed Graph / PageRank / Betweenness / SCC | `graph-engine` | ✅ real — SCC≥3 wired to `alert-engine` (AML ring detection) |
| Markov Chain / Steady-State π / Mean First Passage Time | `ammas-engine::markov` | ✅ real — verified against closed-form 2-state analytic results |
| Enbilulu Calculus (Φ_Enbi, TIAMAT bands, horizon, Têrtu, Milu) | `bahyway-algebra::enbilulu` | ✅ real, 14 tests, consumed (not duplicated) by `wpd-engine::junction` |

**Not re-verified this pass, carried from `ALGEBRA_GLOSSARY.md`'s
2026-06-05 table:** DomainCentroid, Orbits Calculus, Particles Algebra,
Tribe Algebra, Modular Form, Theta Series, Eisenstein Series E₂. Full
detail and provenance: `workspace/bahyway_v4/ALGEBRA_GLOSSARY.md` Part
VI.

## The 7 GeoLaws

| GeoLaw | Name | What it enforces |
|---|---|---|
| GeoLaw-01 | E7 Lattice | E7 is the ONLY orbital zone decomposition — 126 kissing neighbours |
| GeoLaw-02 | Plimpton 322 | B11 = round(H(P) × 240). 240 is the ONLY valid divisor, never 255 |
| GeoLaw-03 | Betti Numbers | β₀/β₁/β₂ computed ONLY in GeoEngine |
| GeoLaw-04 | Gap(n) | Gap(n) = Θ_E7(n) − Θ_found(n). Positive = HIDDEN_PATTERN. Negative = INTRUDER_CORRUPT |
| GeoLaw-05 | Jordan Normal Form | Orbit stability: eigenvalues inside unit disc = stable — `ea-agent-algebra::jordan::JordanAnalyzer` |
| GeoLaw-06 | VGCA∆ Tribe Algebra | Governs tribe interactions via `VgcaRegistry` |
| GeoLaw-07 | Pauli Exclusion | No two KAKIs at the same (r,θ,φ,tribe_id) |

## Verify it yourself

```
cargo test -p algebra-arsenal -p bahyway-field -p bahyway-algebra \
  -p ea-agent-algebra -p vgca-engine -p graph-engine -p ammas-engine \
  -p kinetic-engine -p hepta-score -p najaf-engine
```
