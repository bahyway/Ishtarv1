# GL-NJF-001 — The Najaf Grave Court
## Capacity, Lawful Vacancy, and the Protection of Cherished Graves — Wadi al-Salam Chapter of the Sensing Law

**Ecosystem:** BahyWay.Ecosystem v4.0 — ENKI-TERRA humanitarian line
**Domain:** GL-NJF (Najaf) — chapter of the corridor/area sensing family; consumes GL-ALG-003 (Abūbu), GL-ALG-001-A2 (civil protection / Kidinnu), GL-STY-001 (Journal)
**Status:** SEALED — landed by PB-328 as `crates/najaf-grave-court`, 5/5 tests passing (capacity monotone in lambda; erosion angle fixed; buffer never becomes vacancy; DEAD unissuable; accuracy-ladder ranges).
**Author:** DUB.SAR 𒁾

---

## 1. Purpose

Wadi al-Salam is a point process of profound density and profound meaning. This chapter seals three instruments over it: **(i)** grave density and sector capacity with a planning horizon; **(ii)** lawful vacancy — where a new grave may be established, by dimension, orientation, and preference; **(iii)** the location and protection of old, dilapidated, cherished graves at the highest honest accuracy, with a refusal clause that guards dignity.

## 2. The Capacity Equation (Abūbu inverted)

Grave density ρ(x) is kernel intensity over the sovereign EnkiDB grave layer. The threshold ρ\* is not rupture but **sanctity capacity**: the per-sector density beyond which access paths vanish, new interments disturb older ones, and maintenance fails. ρ\* is sealed per sector (ground condition, historical layering, multi-level burial practice), never globally.

Because graves do not release, the breath equation loses its μ term (μ = 0) and the horizon becomes a pure countdown:

  **T\*_sector = (ρ\* − ρ₀) / λ**

with λ the sector burial rate. Alarm when T\* ≤ τ (planning horizon, default τ = 10 years). Verdicts carry ε from the intensity estimate.

## 3. The Qibla-Erosion Rule (lawful vacancy)

A location admits a new grave iff the **grave rectangle fits lawfully** at that point. Sealed construction:

1. Dilate every existing grave and every protective buffer (§5) by the required clearance; take the complement — the raw vacancy.
2. **Erode the vacancy by the grave footprint at its lawful orientation**: the rectangle's long axis lies **normal to the Qibla bearing** (≈ 192° true at Najaf), so the deceased, laid on the right side, faces the Qibla. Orientation is a constant of the erosion, never a free parameter.
3. Subtract suitability layers: minimum walkway corridor to the nearest path (Boolean corridor coverage), slope/drainage, and sector rules.
4. What survives is minted as **discrete lawful plots**, each carrying its ε, its distance-to-preference scores (shrine axis, revered sections, family adjacency), and a Kidinnu seal (Ed25519, phone-checkable, offline-degradable). The output is a plot mint, never a bare green heat map — allocation must be verifiable because allocation is disputed.

## 4. The Accuracy Ladder (honest sensing)

- **±2–5 cm** — drone photogrammetry with RTK/PPK GNSS + ground control: every grave with any surface expression, including subsidence hollows, broken curbs, collapsed mounds (a dilapidated grave is a centimeter-scale depression signature in the elevation model). This meets the ≤10 cm target for surface-expressed graves.
- **±10–50 cm** — ground-penetrating radar for erased graves: detection strong, position honest only to decimeters.
- **Never at this chapter's scale** — satellite optical/hyperspectral (≥30 cm–30 m): admissible as a *proposing* witness only.
The verdict always states which rung produced it. Claiming finer accuracy than the rung permits is a breach.

## 5. The Refusal Clause (dignity twin of the UXO clause)

Every located old grave is **two-witness** before minting: a remote signature (photogrammetry/GPR) plus either a ground survey or an archive record. States:
- **GOLDEN** — documented and surface-verified (two witnesses).
- **FUZZY** — detected, single witness; ground survey queued.
- **DEAD — this court refuses to issue it.** A grave is never declared absent, erased, or reclaimable by remote sensing. An unverifiable detection becomes a **protective buffer**: excluded from vacancy, excluded from allocation, held until witnesses arrive or the Madanu court decrees with the family's standing. Nothing cherished is overwritten by an algorithm.

## 6. Journal and custody

Every density verdict, plot mint, anchor, and buffer is NĀRU-witnessed (GL-STY-001). Grave records remain in the sovereign EnkiDB layer; the map surface (Leaflet/OSM in rehearsal, Godot in production per the NaviEngine note) carries pointers only.

## 7. Playbook

- **PB-328** — Najaf Grave Court kernels: per-sector intensity + T\* countdown, Qibla-erosion plot mint, accuracy-ladder tagging, refusal-buffer propagation into vacancy; law tests (capacity monotone in λ; erosion angle fixed; buffer never becomes vacancy; DEAD unissuable).

## 8. Seal

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
