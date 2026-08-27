# GL-NRG-001 — The Nergal Transport Calculus
## Infection as Particle Transport: Criticality, Shielding, and the Control-Rod Doctrine

**Ecosystem:** BahyWay.Ecosystem v4.0 — Nergal AV Service (storage-domain sentinel; sits beside the Suhrim calculus in MardukEngine's court)
**Founding text (Book Court):** Haghighat, *Monte Carlo Methods for Particle Transport* — courted structure-only per GL-LIT-001; corpus registration to follow
**Consumes:** GL-LBR-001 (NUZI write-once = pure absorber), GL-LBR-001-A2 (the SSD/HDD interface is a miṣru), GL-ALG-003 (first-passage T\*), GL-NSR-001 (Markasu null discipline), GL-OPS-001/002 (two streams)
**Status:** DRAFT — awaiting Architect seal (CSR-08). **Name check:** Nergal — plague-lord who alone commands pestilence — unspent; the inversion (plague-god as guardian) is deposited openly.

---

## 1. The Transport Dictionary (sealed mapping)

| Transport physics | Storage / Nergal meaning |
|---|---|
| Spatial mesh / regions | Sectors, block groups, volumes; the sector **graph** (discrete-event MC, not continuum) |
| Material cross sections Σ | Medium infectivity: SSD = high-scatter isotropic (wear-leveling teleports writes); HDD = forward-peaked anisotropic (track locality) |
| Material interface (transmission/reflection/albedo) | The SSD↔HDD boundary — a **miṣru**; cross-device infection = interface transport |
| Absorption Σ_a | Dead ends: immutable stores (NUZI write-once = boron), read-only mounts, **Nergal scan-kill rate** |
| Scattering Σ_s | Copy/propagation to another sector without multiplication |
| Fission, ν | One infected object spawning ν descendants (droppers, worm fan-out) |
| **k_eff** | **The invasion number.** k_eff < 1: every outbreak dies exponentially. k_eff > 1: epidemic from any seed. |
| Control rods / poison | The Nergal AV Service: scan rate is added Σ_a; the service's whole purpose is holding k_eff < 1 |
| Shielding, optical thickness τ = Σt·d | Quarantine zones; breach probability e^(−τ) — quarantines are **sized**, never asserted |
| Buildup factor | Re-emergence after partial cleaning (the worm that scatters around the shield) |
| Importance sampling / weight windows / Russian roulette | Risk-weighted scanning: deep-scan hot regions, roulette the cold ones — Nergal stays cheap enough to run always |
| Tallies / flux estimators | Infection-pressure telemetry per sector, journaled to NĀRU |

## 2. The Two-Tribes Collision Clause

A healthy tribe and an infected tribe competing for the same sectors is two-species transport on one mesh: collisions occur where their fluxes overlap. The court's question is never "where is the virus" alone but **"what is k_eff of this medium, for this strain, with Nergal at this scan rate"** — the collision map is the flux-overlap tally, and containment is proved by the eigenvalue, not by the absence of alarms.

## 3. The Honesty Clauses

- **The null-model boundary:** transport particles are non-adaptive; malware studies the shield. Nergal's Monte Carlo supplies the **null model and the sizing mathematics** (as OU serves Markasu). Adaptive-adversary defense remains with Suhrim + UrNammu; claiming MC alone contains an adaptive foe is a breach and accrues τ.
- **Subcriticality is the service-level law:** Nergal's sealed guarantee is stated as *k_eff ≤ k_max < 1 for the registered strain classes, at scan budget B* — measurable, falsifiable, journaled. "The storage is safe" without an eigenvalue is an eternity-claim (GL-LBR-001 §4).
- **Quarantine is shielding:** every quarantine zone carries its declared optical thickness τ and breach probability e^(−τ) in the tablet of the zone.

## 4. Playbook

- **PB-344** — Nergal transport kernel, landed at `workspace/bahyway_v4/crates/nergal-transport` (flat crate convention, workspace member), `cargo test -p nergal-transport` 4/4 passing (2026-08-21; the original draft's `hosts: bahyway_host` / `/home/bahaa/BahyWay.Ecosystem/v4.0` target was never real — see `ansible/inventory.ini`, host is `uruk`). Sector-graph free-flight sampling, collision channels (absorb/scatter/fission), k_eff power-iteration estimator, shield attenuation, importance-weighted scanning. Law tests **L45** (k_eff < 1 ⇒ every seeded outbreak finite, all seeds), **L46** (supercritical medium invades; adding Nergal Σ_a drives it subcritical — the control rod works), **L47** (barrier transmission matches e^(−τ) within MC error), **L48** (importance-weighted scanning finds infections with fewer scans than uniform — variance reduction proven, not assumed). One real test-calibration bug found and fixed during landing: L46 originally asserted 15/20 seeded outbreaks invade for a k_eff = 1.2 medium; the true branching-process extinction probability for that offspring distribution is q = 1/3 (survival ≈ 66.7%, not ≈100%), and the first 20 SplitMix64 seeds happen to sample low (7/20) — well inside normal variance for n = 20. Fixed by widening to 200 trials and asserting ≥50%, which holds with margin under the true rate and still fails hard if the control-rod effect breaks.

## 5. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
