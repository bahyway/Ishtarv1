# 𒁾 TABLET XI — PH-004 "DŪRU" 𒂦 — The Walls of Revolution
### Constraint membranes in orbit space: shells, wedges, cells, crossings, and the creep gauge
### Sibling of PH-003 MAŠKU (the skin that spans honestly; the wall that separates lawfully)
### Proposed seat: NIPPUR 1.7 (Ontology reserve) or lineage particle beneath PH-002 Puhu — the Architect assigns
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)

*dūru: the city wall. Cities did not build walls to make symmetric shapes; they built them
to isolate sectors of law. The revolution is not the point — it is merely the cheapest
lawful way to erect a wall. Glossary check performed: "Dūru" unclaimed.*

---

## Clause D-1 — The Wall

Fix a tribe with nucleus N and axis A; coordinates around A are (ρ, φ, v): radius,
azimuth, axial height (in Hepta Space: the profile sweeps a hypersurface; the theater
shows the 3-slice). A **wall** is a pair

> **W = (r, Σ)** , r : [v₀, v₁] → ℝ₊ the profile, Σ ⊆ 𝕊¹ the sweep,

realized as the membrane **W = { (ρ, φ, v) : ρ = r(v), φ ∈ Σ, v ∈ [v₀,v₁] }.**

- Σ = 𝕊¹ (full revolution) → a **shell wall**: radial isolation, constraint bands.
- Σ ⊊ 𝕊¹ (partial revolution) → a **wedge wall**: angular isolation, sector regimes.

**The wall is the equality; the law is the inequality.**

## Clause D-2 — The Cell (the constraint regime)

A **cell** is a conjunction of wall-inequalities:

> **C = { p : r⁻(v(p)) ≤ ρ(p) ≤ r⁺(v(p)), φ(p) ∈ Σ_C, v(p) ∈ [v₀,v₁] }**

with membership operator χ_C(p) ∈ {0,1}. Shells alone give nested bands; shells crossed
with wedges give the **cellular constraint lattice**: every cell a regime, every particle
in exactly the cell whose constraints it satisfies.

**Axiom D-2a (Rendering, not physics).** The geometry displays the law; the predicate
decides it. For every cell there is a sealed predicate P_C over EAV attributes, and the
wall is faithful only if **χ_C(p) = P_C(p)** for all p — an equality proven at Gate G4,
never assumed. Particles do not bounce off membranes; ledgers change jurisdiction at
them. The picture may persuade; only the predicate convicts.

## Clause D-3 — Crossing Is Custody (the BV Ledger)

For a trajectory p(t), a **crossing** occurs at t* where χ_C(p(t)) flips for some cell.
Every crossing cuts a KAKI recording (from-cell, to-cell, adannu, cause-class). Hence:

> **the custody history of a particle = the total variation of its membership vector**
> ‖χ(p(·))‖_BV = number of crossing KAKIs,

and a particle whose membership vector has unbounded variation in bounded time is
itself an omen (thrash between regimes — a BĀRÛTU cell waiting to be minted).

Drift classes become **sectors bounded by skins**: aging = slow outward dρ/dt through
the shells; **reject** = expulsion past the outermost wall; **arkû** = entry through a
declared **gate** G ⊆ W (an aperture where inward crossing is lawful without ceremony);
**quarantine** = the shell whose wall is crossed inward only under Architect seal.
A gate is part of the wall's definition, not an exception to it.

## Clause D-4 — The Partition Obligation

The declared walls must partition orbit space:

> **Σᵢ χ_{Cᵢ}(p) = 1 for all p** — full cover, no double jurisdiction —

or every failure must be *declared*: an uncovered region is a **sacred void of
constraint-space** (drawn dark, honored empty); an overlap is a **double-jurisdiction
fault** and blocks the seal. In Hepta Space, where profiles sweep hypersurfaces and
careless drawing gaps easily, this clause is the difference between a legal map and a
rumor of one. Soft variant: a partition of unity with declared overlap width flowing
into ε.

## Clause D-5 — The Creep Gauge (MAŠKU turned inward)

Between two **sealed** rims, the honest wall is minimal: H = 0 + ε_disc. Therefore for
any wall W between sealed constraints:

> **Creep(W) = mean |H(W)| + ε_disc**

is the **constraint-creep gauge**: a bulging wall is a wall enforcing something nobody
declared — scope creep, rendered as curvature, priced in the same geometric τ, read by
the same view-invariant instrument (Puhu holds for the gauge). Verdict bands as always:
SEALED CLEAR / ADVISORY / PARZU 0x03.

## Clause D-6 — Sectoral Instruments

Each cell carries its own Zibānītu instrument: per-cell dials Sₙ over its resident
particles, per-cell τ against the cell's own sealed Θ, per-cell census in URUK units.
The lattice of cells is thereby a lattice of jurisdictions **and** a lattice of gauges —
the wall does not only separate law; it separates measurement, which is the same thing
said twice.

## Clause D-7 — The Equation of the Walls

The tablet's one-liner, fit for the door:

> **C = { r⁻ ≤ ρ ≤ r⁺ , φ ∈ Σ } · Σᵢ χ_{Cᵢ} ≡ 1 · Δχ ⇒ KAKI · H(W | sealed rims) = 0 + ε**

*Jurisdiction is a revolution of a profile; law is the inequality between walls; every
change of custody is a ledger event; and an honest wall between sealed rims carries no
curvature it can't confess.*

## Clause D-8 — Gate G4 Obligations

1. `wall_faithful : ∀ p C, χ C p = P_C (attrs p)` — geometry equals predicate, per cell.
2. `partition : ∀ p, Σᵢ χ_{Cᵢ} p = 1 ∨ declared_void p` — cover without double
   jurisdiction, voids declared.
3. `crossing_kaki : ∀ trajectory, Δχ ⇒ emits_kaki` — no silent change of custody;
   composes with L-7's conservation (a crossing is a custody *transition*, never a loss).
4. `creep_floor : sealed_rims W → Creep W ≤ ε_tolerance` — walls between sealed
   constraints are minimal up to declared ε.
5. `gate_lawful : ∀ inward_crossing quarantine, bears_seal CSR-08 ∨ through_gate G` —
   the quarantine wall admits by ceremony or by declared aperture, and nothing else.

---

*Eleventh tablet drafted. MAŠKU spans; DŪRU separates; together they are the firmament
and the city. The seal belongs to the Architect alone. 𒁾*
