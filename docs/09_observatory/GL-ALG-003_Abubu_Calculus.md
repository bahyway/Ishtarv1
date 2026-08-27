# GL-ALG-003 — The Abūbu Calculus
## Membrane Rupture: Compliance, Critical Density, Horizon, and the Quarantine of Witnesses

**Ecosystem:** BahyWay.Ecosystem v4.0
**Domain:** GL-ALG (Algebra/Calculus family) — hosted under GL-PHY-001 truth tier (NinurtaEngine Physics Service)
**Status:** SEALED — landed by PB-327 as `crates/abubu`, 6/6 tests passing (L1-L6).
**Author:** DUB.SAR 𒁾
**Related tablets:** GL-PHY-001 (two-tier physics: presentation vs truth), GL-ORG-001 (Homeostasis, basin re-entry), GL-TPL-002 (Living Shape & Drift), GL-DST-003 (Madanu Court), GL-STY-001 (Journal), EnkiQDB definition (quarantine database), Kidinnu Standard (ε-qualified statements).

**Name:** *Abūbu* — the Deluge; the flood that breaks the wall. The calculus of the event where holding fails.

---

## 1. Purpose

Every station membrane holds attached particles while its sealed predicate works. Attachment dents the wall; crowding deepens the dent. The Abūbu Calculus answers four questions with sealed equations: **(i)** what is the membrane's elasticity coefficient and its lawful bounds; **(ii)** at what particle density does rupture occur; **(iii)** within what timeframe, under the station's breath; **(iv)** what becomes of the statistical witnesses on a membrane whose rupture risk exceeds tolerance.

Abūbu is a **calculus, not an engine**. Its runtime home is NinurtaEngine's truth tier; its presentation twin (trembling springs, tearing quads) belongs to the theater tier and is never truth (GL-DST-001).

## 2. The Compliance Coefficient κ_M

**Definition.** κ_M ∈ [0, 1] — dimensionless membrane compliance: dent produced per unit attached load, normalized. Physical ancestor: inverse flexural rigidity of a Kirchhoff–Love plate (D = Et³/12(1−ν²)); ecosystem form: a per-membrane Optional EAV attribute `phy.kappa_m`, governed.

**Bounds (sealed):**
- **κ_min = 0 — the Rigid Decree.** The BlackBox / G4-sealed membrane. It does not dent because it *refuses*: deformation zero by law, not by strength. A κ = 0 membrane can never rupture; it can only refuse at the in-gate. (Law Test L1.)
- **κ_max = 1 — the Yield Normalization.** One MAROON-band particle (m = m_MAROON) attached at range zero produces exactly the yield strain ε_y. All real membranes sit strictly inside (0, 1).

**Governance.** κ_M is a Madanu-court parameter: raising it buys sensitivity (dents reveal structure — the Membrane Courts' entire witness) at the price of rupture risk; lowering it buys safety at the price of blindness. No engine may alter κ_M at runtime.

## 3. The Dent Field and the Strain Regimes

Attached particles form a point process Φ_M on the membrane. The dent field is shot noise over it:

  **u(x) = κ_M · Σ_{i∈Φ_M} m_i · ψ(d_g(x, x_i)/σ)**

with m_i the particle load (TIAMAT-banded score), ψ the Gaussian kernel, d_g geodesic on the membrane (exact on cylinders — the wall unrolls), σ the attachment footprint.

Strain ratio: **S(x) = u(x)/u_crit**. Three sealed regimes:

| Regime | Condition | Meaning | Ecosystem twin |
|---|---|---|---|
| **Elastic** | S < ε_y | spring restores | homeostasis — basin re-entry (GL-ORG-001) |
| **Plastic** | ε_y ≤ S < 1 | permanent set | drift — founding shape will not return (GL-TPL-002) |
| **Rupture** | S ≥ 1 | the wall breaks | Abūbu event |

Default ε_y = 0.6 unless a domain tablet overrides.

## 4. The Critical Density Equation

For locally slowly-varying density ρ(x) with mean load m̄, the field integrates to u(x) ≈ κ_M · m̄ · ρ(x) · 2πσ². The rupture threshold:

  **ρ\*(x) = u_crit / (2πσ² · κ_M · m̄)**

Structural readings, sealed as clauses: ρ\* is **inversely proportional to compliance** (softer walls rupture at lower crowding) and **inversely proportional to mean load** (MAROON-heavy traffic ruptures a membrane that DILBAT traffic would never trouble). κ_M → 0 sends ρ\* → ∞: the Rigid Decree in equation form.

## 5. The Horizon Equation

The station breathes: arrivals at rate λ_a, releases at rate μ. Local density obeys dρ/dt = λ_a − μρ, hence ρ(t) = (λ_a/μ)(1 − e^{−μt}) + ρ₀ e^{−μt}.

- **Safe verdict:** if λ_a/μ ≤ ρ\*, the membrane never ruptures under this load; steady state holds below threshold.
- **Horizon verdict:** if λ_a/μ > ρ\*, rupture is predicted at

  **T\* = −(1/μ) · ln[(λ_a − μρ\*)/(λ_a − μρ₀)]**

  and the alarm condition is T\* ≤ τ for the declared horizon τ (weeks-until-ERRA grammar, as in Enbilulu).

**Probabilistic refinement (ε-humility).** One draw is not the ensemble. For Poisson Φ_M the exact tail is available through the Laplace functional ℒ_u(s) = exp(−λ∫(1−E_m[e^{−sκ_M m ψ(r)}])dr) — the Sixth Court's ℒ_I machinery verbatim. Where the closed tail is not evaluated, a seeded deterministic Monte Carlo bound stands in, and the verdict carries its ε.

## 6. The Quarantine Clause (contamination of witnesses)

Rupture censors the sample: particles leak unobserved, the point pattern becomes selection-biased, and every spatial verdict issued on that membrane loses its witness silently.

**Sealed rule:** when P(rupture within τ) > ε_Q (default ε_Q = 0.05), then:
1. all statistical verdicts on the membrane are **demoted to FUZZY**;
2. the membrane's subsequent output routes to **EnkiQDB** (quarantine database — this is its purpose);
3. the demotion and its reason are NĀRU-witnessed (GL-STY-001); nothing is deleted — witnesses are impounded, not erased;
4. release requires a **Madanu decree**: repair (lower κ_M, shed load, throttle in-gate) or retirement.

## 7. Interface

HeptaScript surface (no new verbs):
```
PROVE rupture(membrane) WITHIN horizon(τ) WITNESS naru
EMIT strain(membrane) PRESENT S_max, rho, rho_star, T_star
```
EAV attributes minted: `phy.kappa_m`, `abubu.rho_star`, `abubu.t_star`, `abubu.s_max`, `abubu.regime`, `abubu.p_rupture`, `abubu.quarantined`.

## 8. Law Tests (sealed with the calculus)

- **L1 — Rigid Decree:** κ_M = 0 ⇒ ρ\* = ∞, T\* = None, regime = Elastic forever; the membrane refuses, never ruptures.
- **L2 — Compliance monotonicity:** ρ\* strictly decreasing in κ_M and in m̄.
- **L3 — Safe breath:** λ_a/μ ≤ ρ\* ⇒ no finite T\*.
- **L4 — Horizon monotonicity:** T\* decreasing in λ_a; alarm iff T\* ≤ τ.
- **L5 — Quarantine:** p_rupture > ε_Q ⇒ verdicts FUZZY + route EnkiQDB + NĀRU witness; p ≤ ε_Q ⇒ untouched.
- **L6 — Regime ladder:** S crossing ε_y moves Elastic→Plastic; crossing 1 moves Plastic→Rupture; never skips downward without decree.

## 9. Playbook

- **PB-327** — Abūbu kernel crate (`abubu`) under NinurtaEngine's truth tier: pure-Rust, zero dependencies, functions κ-clamp, dent field, ρ\*, ρ(t), T\*, regime ladder, seeded MC tail bound, quarantine verdict; law tests L1–L6.
*(PB-326 remains held for the PB-323 collision adjudication flagged in GL-LIT-001 §9.)*

## 10. Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-27
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work.
```
