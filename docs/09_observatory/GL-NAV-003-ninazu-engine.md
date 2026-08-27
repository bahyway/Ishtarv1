# GL-NAV-003 — The Ninazu Engine
## Sovereign Offline Navigation of the Narrow Roads (Wadi-us-Salam, Najaf)

**Ecosystem:** BahyWay.Ecosystem v4.0 — navigation over a heptagonal membrane field
**Name:** *Ninazu* — the oldest Sumerian underworld god, Nergal's predecessor: "custodian of the earth / steward of the underworld," father of Ningišzida. Gloss (Ḫubullu Law): **"the steward of the narrow roads."** Naming register (deity → engine) is the Architect's CSR-08 decision.
**Consumes:** GL-GEO-002 (hyperbolic heptagonal index), GL-VSL-001 (Lilu), GL-ONT-003 (non-substitution — each grave an individual), GL-BRT-001 (Birth Gate — graves are born Particles), GL-STY-001 (StoryWay)
**Status:** SEALED — landed by PB-350 as `crates/ninazu`, 4/4 tests passing (L69-L72), see §4 Playbook for the full landing note.

**Landing note (2026-08-21):** this tablet arrived in the Mon20260817 delivery bearing the ID GL-NAV-001 and was renumbered to GL-NAV-003 on landing — GL-NAV-001 and GL-NAV-002 are already sealed in this repo (`docs/09_observatory/GL-NAV-001-flight-to-location.md`, `GL-NAV-001-AnnexA-hendursaga-charter_DRAFT.md`, `GL-NAV-002-knowledge-graph-navigation.md`). See `docs/mon20260817-incoming/README.md` for the original, unmodified draft and the full collision record.

**Rename note (2026-08-21, later same day):** the engine itself was renamed **Ningišzida → Ninazu** per the Architect's explicit call in the Tus20260818 Master Glossary review — *Ningišzida is difficult to pronounce*, and the glossary had independently already flagged this correction, reserving **Ningišzida as a future child sub-engine name** (e.g. a path-tracer beneath Ninazu). The GL-NAV-003 number is unaffected — only the engine's name and its crate (`workspace/bahyway_v4/crates/ninazu`, package `ninazu`) changed. See `docs/13_changelog/TUS20260818_NINAZU_RENAME_2026-08-21.md`.

---

## 1. Purpose & Dignity Clause

The Ninazu Engine helps a **living visitor find a specific grave** in Wadi-us-Salam — one of Earth's densest burial grounds — by resolving the **narrow roads** between densely packed grave-blocks that flat indices merge away. Its purpose is dignity: every grave is a **named individual, non-substitutable** (GL-ONT-003), and the engine's job is to make each one *findable as itself*, never collapsed into a neighbor. The engine navigates roads for the living; it makes no claim over the dead.

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

- **PB-350** — Ninazu Engine, landed at `workspace/bahyway_v4/crates/ninazu` (flat crate convention, workspace member), `cargo test -p ninazu` 4/4 passing (2026-08-21; host `uruk`, not the draft's `bahyway_host`). Offline tile-store ingest (daily snapshot, Birth-Gate adjudication of tiles), heptagonal georeferenced index, Lilu F4 cinematic frontend (birdfly / land / landmark / route). Law tests **L69** (navigation reads only the local snapshot — no live-service call at use time), **L70** (a route never steps through a grave cell, only road cells), **L71** (each grave is one distinct index cell — no two graves share a cell), **L72** (Lilu F4 honors the Verb Law — route proposes to bench, no apply). One real fixture bug found and fixed during landing: the L70b "no route through solid graves" test asked for a route between two corner grave cells, (0,0) and (4,4), that have no road-adjacent neighbour at all in the 5×5 test snapshot — they are graph-disconnected from the road network by construction, so `navigate` correctly returns `None` and the test's own `is_some()` assertion was unsatisfiable. Fixed by routing between (1,1) and (3,3), graves that do each border the road cross.

## 6. Seal

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
