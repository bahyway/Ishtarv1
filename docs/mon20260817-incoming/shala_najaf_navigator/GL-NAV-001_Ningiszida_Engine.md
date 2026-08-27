# GL-NAV-001 — The Ninĝišzida Engine
## Sovereign Offline Navigation of the Narrow Roads (Wadi-us-Salam, Najaf)

**Ecosystem:** BahyWay.Ecosystem v4.0 — navigation over a heptagonal membrane field
**Name:** *Ninĝišzida* — the Mesopotamian guide/wayfinder. Gloss (Ḫubullu Law): **"the guide of the narrow roads."** Naming register (deity → engine) is the Architect's CSR-08 decision.
**Consumes:** GL-GEO-002 (hyperbolic heptagonal index), GL-VSL-001 (Lilu), GL-ONT-002 (non-substitution — each grave an individual), GL-BRT-001 (Birth Gate — graves are born Particles), GL-STY-001 (StoryWay)
**Status:** DRAFT — awaiting Architect seal (CSR-08)

---

## 1. Purpose & Dignity Clause

The Ninĝišzida Engine helps a **living visitor find a specific grave** in Wadi-us-Salam — one of Earth's densest burial grounds — by resolving the **narrow roads** between densely packed grave-blocks that flat indices merge away. Its purpose is dignity: every grave is a **named individual, non-substitutable** (GL-ONT-002), and the engine's job is to make each one *findable as itself*, never collapsed into a neighbor. The engine navigates roads for the living; it makes no claim over the dead.

## 2. The Five Principles

1. **Sovereign & offline.** The engine depends on **no live external service** — not Google Earth, not OSM live tiles. Imagery (drone and/or satellite; Google Earth *imagery* may be used but never the live *service*) is ingested **once per day** into a sovereign tile store (NUZI), and all navigation runs against that local snapshot. No query leaves the machine at use time.
2. **Heptagonal index (GL-GEO-002).** The graveyard is indexed by a hyperbolic {7,3} heptagonal grid, not flat hexagons. Each grave is a distinct heptagon cell; the narrow roads between blocks are **preserved as open cell-boundaries**, never merged. This is the anti-fragmentation guarantee applied to physical space.
3. **Membrane field.** The terrain is a curved membrane field (per the Šala courts): the caging-confinement mechanism pins each grave in its cell so it cannot be absorbed by a neighbor, even millimeters away.
4. **Lilu F4 — the cinematic frontend.** A new Lilu frontend compiles `.lilu` navigation scenes into a cinematic flow: **bird-fly** low along the narrow roads, **land-from-sky** onto the target grave, and **landmark focus** on nearby famous locations (shrines, notable tombs). The Verb Law holds: `rehearse birdfly`, `rehearse land`, `focus landmark`, `propose route -> bench`; no `apply`.
5. **Falsifiable route claim.** A computed route is a claim: *"this path of open road-cells reaches the target grave without crossing a grave cell."* It is testable — a route that steps through a grave cell (rather than a road cell) is an invalid route, rejected.

## 3. The Offline Tile Store

Daily ingest: drone/satellite (or Google Earth imagery) → SUSA gateway → Birth-Gate adjudication (imagery tiles that are corrupt/misaligned refused) → NUZI sovereign tile store, georeferenced to the heptagonal index. Use-time navigation reads only NUZI. A stale snapshot is explicitly dated; the engine never silently depends on freshness it does not have.

## 4. Lilu F4 Cinematic Grammar

- `witness field of NAJAF` — bind the heptagonal membrane field over the daily tile snapshot.
- `rehearse birdfly <road-path>` — camera follows the road waypoints at low altitude (the "bird flying the narrow roads").
- `rehearse land <target>` — the sky-descent onto the target grave (easeInOut from high pitch to ground).
- `focus landmark <name>` — highlight and label a nearby famous location.
- `propose route -> bench` — the computed route is proposed, never auto-applied.

## 5. Playbook

- **PB-350** — Ninĝišzida Engine: offline tile-store ingest (daily snapshot, Birth-Gate adjudication of tiles), heptagonal georeferenced index, Lilu F4 cinematic frontend (birdfly / land / landmark / route). Law tests **L69** (navigation reads only the local snapshot — no live-service call at use time), **L70** (a route never steps through a grave cell, only road cells), **L71** (each grave is one distinct index cell — no two graves share a cell), **L72** (Lilu F4 honors the Verb Law — route proposes to bench, no apply).

## 6. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
