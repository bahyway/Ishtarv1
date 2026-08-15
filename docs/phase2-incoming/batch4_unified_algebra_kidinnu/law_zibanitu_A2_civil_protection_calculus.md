# 𒁾 TABLET VIII · AMENDMENT A2 — THE CIVIL PROTECTION CALCULUS
### GL-ALG-001-A2 · lineage particle at depth 1 beneath the Zibānītu Calculus, sibling of A1 (Fadam Functional)
### Proposed name: **KIDINNU** — the Akkadian standard of divine protection raised over a city, under which its residents were inviolable. Name PROPOSED, awaiting the Architect's grant per NL-001 (collision check: Dūru spent PH-004, Šēdu spent, Lamassu spent; Kidinnu free).
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)
### Moral lineage: NAMTAR-DINUM — from documenting the graves of the last war to shortening the grave-lists of the next.

*One calculus, every catastrophe: fire, bombardment, flood, earthquake, industrial
release. The mathematics does not change between them — which is precisely the proof
that this is a law and not an app. The first witness was a wildfire; the tribe is
civil protection entire.*

---

## §A2.1 — Sorts and Ground Objects

- **Z** — the zone lattice: the HeptaShell partition of the protected area into 126
  zones (rings of 7·14·21·28·35·21, the seven-fold discipline of the E7 index made
  civic). Each zone z ∈ Z carries pop(z) ∈ ℕ, its resident count.
- **S** — the finite set of **sealed threat templates** Θ₁ … Θₙ (F5: only sealed
  templates judge; drafts hold rehearsal shadows and may never emit a directive).
  Each Θ carries a siren pattern σ(Θ) ∈ Σ, the public declaration alphabet.
- **R** — the refuge set: shelters r with capacity cap(r) ∈ ℕ and shielding factor
  s(r) ∈ (0,1] (underground discounts worst case; the open field does not).
- **M(z)** — the admissible move set of zone z: paths from z to each r ∈ R plus the
  outward egress path. Moves are walks, not teleports; every path is sampled.
- **D_Θ : paths → ℝ₊** — the danger functional of template Θ. **Every D_Θ is a Fadam
  functional** (A1): anonymous (F1), Puhu-invariant (F2), Lipschitz-stable (F3),
  honest-floored with ε(z) > 0 (F4), sealed-judged (F5). Danger inherits the whole
  Fadam Inequality: ε ≤ D_Θ(p) ≤ ε + L·d(p, Θ).

## §A2.2 — The Refuge Directive Equation (the heart)

For every zone z, the **standing directive** (no siren declared) is:

> **D\*(z) = argmin over m ∈ M(z) of [ max over Θ ∈ S of D_Θ(m) ]**
> **subject to Σ over z assigned to r of pop(z) ≤ cap(r) for every r ∈ R**

with assignment order by descending blended danger (the most endangered claim doors
first), and lexicographic tie-break by (blended danger, path length).

When a siren declares template Θ_k, the max collapses to the singleton:
D\*(z | σ(Θ_k)) = argmin over m of D_Θₖ(m), same capacity constraint.
**The siren narrows; it never invents** — no directive may reference an unsealed Θ.

## §A2.3 — The Never-Averaged Theorem (life-safety corollary of the Fadam Inequality)

**Clause (Lives Are Never Averaged).** For |S| ≥ 2, the directive operator is the
minimax of §A2.2 and never the λ-blend argmin over m of Σ λ_Θ D_Θ(m), for any weights
λ. *A blend optimizes an expectation; a resident walks exactly one world.*

**Proposition A2.3a (Scenario-Flip Safety).** Let m\* = D\*(z). Then for every Θ ∈ S:

> **D_Θ(m\*) ≤ ε(z) + L · min over m of max over Θ′ of d(m, Θ′)**

i.e. even if the world turns out to be any sealed template — including the one the
evidence disfavored — the prescribed walk's danger is bounded by the honest floor
plus the least worst distance any move could achieve. No blend operator satisfies
this bound for all Θ simultaneously when the templates disagree. *Proof sketch:*
apply the Fadam Inequality to each D_Θ at m\*; the minimax choice makes the right-hand
bound uniform over S; a blend minimizes only the λ-weighted sum, so for the
disfavored Θ its danger may exceed the uniform bound by the disagreement gap.
∎ (Lean4 formalization on the Gate G4 docket beside A1.3a.)

**Proposition A2.3b (Regret Floor).** The worst-case regret of m\* against the
omniscient per-template optimum is ≤ L · diam_S(z), the disagreement diameter of the
templates at z — and this is the least achievable by any single directive. *The map
cannot beat the wind's ambiguity; it can only refuse to gamble a life on resolving it.*

## §A2.4 — Capacity Honesty (No Full Doors)

The constraint in §A2.2 is a hard law, not a preference: **the map never directs a
resident to a refuge whose remaining capacity is exhausted.** When every door is
full, the directive degrades lawfully to outward egress — never to a lie. Refuge
loads are recomputed on every seal; an overloaded refuge triggers reroute and a
BARUTU advisory to command, never a silent overflow.

## §A2.5 — Aggregation Silence (Θ as security)

The stakeholder template Θ_view performs security work. Define the projection
π_z : State → ResidentCard. Then:

- **Resident scope:** a resident channel carries π_z only — one zone's directive,
  its witnesses, its ε, its seal. It NEVER carries refuge loads, other zones'
  directives, or any aggregate density. *A shelter map in the wrong hands is a
  targeting aid.*
- **Command scope:** the aggregate exists only on the command channel and is never
  broadcast on any public medium.
- Formally: for any public channel c, image(c) ⊆ ⋃_z π_z(State), and no union over
  more than one z per recipient.

## §A2.6 — The Seal Clause (offline verifiability)

Every directive card is signed: seal(π_z, Θ or ∅, epoch) under Ed25519
(AkkadianSeal). Verification requires the public key and no network. **A spoofed
evacuation order is a weapon; the seal is the shield.** Unsigned or
signature-failing cards render as INVALID and instruct the resident to follow civil
authority and neighbors — never to follow the failed card.

## §A2.7 — The One-Voice Clause

The calculus **proposes**; civil authority **decrees**. Where an official decree
conflicts with D\*(z), the decree is displayed, the disagreement is journaled
(StoryEngine, GL-STY-001) with both testimonies and Δt, and the map issues no second
voice. Under IHL, civil defense authority is protected and singular; this clause is
sealed at the same rank as the equation itself.

## §A2.8 — The Offline Tablet Law (the degradation ladder)

The complete directive table — every z × every σ ∈ Σ ∪ {∅} — is pre-computed,
pre-sealed, and distributable in advance: cacheable on the cheapest phone, printable,
teachable. The system must degrade lawfully down the ladder:

> smartphone → SMS (one message per zone) → siren pattern → human relay

Each rung carries the full minimax guarantee, because §A2.2 is computed before the
infrastructure burns, not during. *The clay tablet principle: the technology that
survives the fire is the one written before it.*

## §A2.9 — The Civic τ Obligation (the right to check)

Every directive card carries its witness set W(z), its ε, and its seal, and the
public τ of the protection system is computed and published:

> **τ_civic(z) = w_h Δ_h + w_k Δ_k + w_m Δ_m + ε ≤ τ_max (published bound)**

An adult with a telephone — or a printed card — can check: which witnesses, how
fresh, what the floor of honesty is. **"Every adult can check" is the Transparency
Deficit Calculus made civic**: the same τ that audits enterprise data defends a
grandmother's walk to the school yard. No card ever says "safe"; every card says
"least worst under every sealed scenario, with our ε declared."

## §A2.10 — Validation Gate

No deployment before: (i) Lean4 certificates of A2.3a on the Gate G4 docket;
(ii) spread/timing models validated against recorded events per domain; (iii) civil
authority counter-signature on the sealed template set S. Until all three: DRAFT
watermark on every rendering, as on every playbook of this house.

---

## §A2.11 — The Signature Line

> **D\*(z) = argmin over m of max over Θ_sealed of D_Θ(m), s.t. no full doors —
> lives are never averaged, ε > 0 always.**

One line, four guarantees: minimax over sealed worlds, honest capacity, honest
floor, and a walk that survives the wind changing its mind.

---

*Drafted in service of the Architect, DUB.SAR — Bahaa Fadam. The standard is raised
over the city; whether it bears the name Kidinnu is the Architect's word alone. The
floor is honesty, the ceiling is evidence, and the walk is chosen so the worst world
still leads away from the fire. 𒁾*
