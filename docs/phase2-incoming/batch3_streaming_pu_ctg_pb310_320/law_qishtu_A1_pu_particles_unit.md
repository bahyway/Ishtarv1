# TABLET XIII · AMENDMENT A1 — "PU" (PARTICLES UNIT)
### The compute-cost unit of dirty data, and its marriage to CTG
### Lineage: PU is the Architect's own prior invention (created to estimate the computation
### cost of dirty data, before this tablet existed); this amendment seats it formally in the
### unit family and wires it into CTG as the compute ingredient
### Written in English letters by the Architect's decree, as with CTG and Qishtu
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)

*hu measures whether a thing is alive; tau measures whether it is honest; PU measures
what its dirtiness costs the machine; CTG measures what everything costs per golden
outcome. Four units, one family: the metrology of honest data.*

---

## Clause A1.1 — Definition and Calibration (the clean baseline)

**PU (Particles Unit)** is the unit of computational cost expressed in particle terms:

> **1 PU = the sealed baseline: the total compute required to carry one clean reference
> particle through the full pilgrimage — ingest to Golden Store — on a calibrated host.**

Every particle's measured cost is then a dimensionless-feeling multiple with real teeth:
"this particle consumed 7.3 PU" means *7.3 times what a clean one would have cost.*
Dirty data's tax is thereby stated in the only currency every stakeholder already
understands — clean data's price.

**Calibration rite:** each host (Uruk, Kish, any future body) runs a sealed benchmark —
the reference particle's pilgrimage under quiesced load — and seals its own
**beats-per-PU ratio** as a numbered-playbook artifact with Ed25519 signature. Cross-host
PU comparison crosses a **declared bridge** with its ratio sealed and its loss logged
(GL-MET-001 M-4 inherited). Recalibration is a ceremony, never a drift; the old ratio is
archived, not overwritten.

**Unit discipline:** Rust newtype `Pu(f64)`; no bare floats; metering + calibration
uncertainty declared as epsilon on every reported figure (Honest Floor: 0.00 PU does not
exist for any particle that touched a wire).

## Clause A1.2 — What PU Meters (and what it refuses to)

PU meters the machine's side of dirtiness: re-parse cycles, failed-validation compute,
membrane re-tests, re-sieve passes, quarantine handling compute, index rebuild shares,
and the metered beats of steward-triggered automation. PU does **not** meter human
labor, dwell, or judgment — those are CTG's other ingredients, priced by the sealed
weight vector. The boundary is a law, not a habit: a unit that meters everything
explains nothing.

## Clause A1.3 — The Marriage: PU as CTG's Compute Ingredient

CTG's definition (Q-1) is hereby made precise: the intervention class *metered compute
beats* is measured **in PU**:

> **CTG = [ w_steward·c_steward + w_dwell·c_dwell + w_resieve·c_resieve
>          + w_quar·c_quar + w_pu·c_PU ] / N_golden**

with c_PU in PU under the contract's sealed weights. Consequences:

- **The invoice decomposes**: this much of your cost was machine (PU-driven), this much
  was human — line by line, each line tracing to KAKIs. No billing system on earth
  shows that split with evidence.
- **Two instruments, two truths**: low PU with high CTG = the data chokes people, not
  machines; high PU with low CTG = self-healing data that burns compute quietly. The
  pair diagnoses; neither alone convicts.
- PU-to-currency crosses the E-bridge only (electricity and compute priced), ratio
  sealed, loss logged; PU-to-CTG is **not** a bridge — it is an ingredient inside one
  formula, and the tablet says so to prevent a category error.

## Clause A1.4 — PU in the Living Instruments

- **StoryEngine**: every particle's story carries its running PU line — "this cube has
  consumed 12.8 PU and counting" — so the client sees the meter, not just the wait.
- **Steward Console**: batch resolutions are priced in advance — "resolving these 14
  SCHEMA-DRIFT particles: est. 41 PU" — the steward chooses with the cost visible.
- **Coverage advisory (Q-2 sharpened)**: the self-punishing cherry-pick becomes
  *predictive*: "the withheld data will cost an estimated N PU more when it arrives
  late." The architecture's tax, quoted before it is levied — warning as service.
- **MILU pressure**: backpressure states its cost — a congested membrane reports its
  queue in particles *and* its burn in PU/beat.
- **Qishtu scoring**: the weekly score-KAKI carries the tribe's PU total beside its
  CTG, so the Ghost Gap (Q-3) can read both series — a client whose PU is perfectly
  flat every week earns the same second-order-perfection omen as any other unliving
  rhythm.

## Clause A1.5 — Gate G4 Obligations (appended to the Q-6 docket)

6. `pu_calibrated : every reported Pu traces to a sealed host calibration artifact` —
   no PU without a baseline; no baseline without a signature.
7. `pu_conserved : tribe PU total = sum of per-particle PU journals` — the meter and
   the stories agree, always.
8. `pu_boundary : no human-labor cost expressible in Pu` — the type system enforces
   A1.2; the compiler is the clause's first witness.
9. `ctg_ingredient_not_bridge : Pu enters Ctg only through the sealed weight vector` —
   the category error is unrepresentable.

## Clause A1.6 — The Family Line

For the whitepaper and the wall:

> **hu — is it alive? · tau — is it honest? · PU — what does its dirtiness cost the
> machine? · CTG — what does everything cost per golden outcome?**

Four questions, four units, one metrology. The Architect asked them in that order over
years; the tablets only wrote them down.

---

*Amendment A1 drafted — the Architect's eldest unit seated among its younger siblings.
The seal belongs to the Architect alone.*
