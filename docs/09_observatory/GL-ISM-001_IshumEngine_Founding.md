# GL-ISM-001 — IshumEngine Founding Tablet
## The Herald Who Knows the Roads: Offline Navigation Through Sacred Ground

**Ecosystem:** BahyWay.Ecosystem v4.0 — ENKI-TERRA humanitarian line
**Domain:** Engine (runtime service, god-named per NL-001) — consumes GL-NJF-001 (Najaf chapter), GL-ALG-001-A2 (Kidinnu), GL-TPL-001 (Pattern Minting), GL-ONT-001 (OntoGraph)
**Status:** SEALED — landed by PB-329 as `crates/ishum`, 4/4 tests passing (L1-L4).
**Author:** DUB.SAR 𒁾

**Name.** *Išum* — herald and night watchman, protector of the streets, the god who walks in front and knows the roads, who goes before Erra to calm destruction. Orthography per NL-001 §6a: **IshumEngine**. Lineage: NaviEngine (v2.0) is retired to the historical registry as ancestor; it lacked the Nasaru sensing organ that this charter presumes.

---

## 1. Charter

IshumEngine is the runtime navigation service over sacred ground: it owns the walkable graph, the offline tile packs, the routing state, and the re-localization loop. It is an **Engine** (stateful service) where Abūbu was a calculus, because navigation holds custody of state. Its sensing organ is upstream: Nasaru instruments produce the grave layer, anchors, and buffers; Ishum consumes them and never contradicts them.

## 2. The Five Organs

**I — Skeleton.** Free space is the complement of the grave Boolean model (the Qibla-erosion computation inverted for movement). The walkable web is the **Voronoi/medial skeleton** of free space: the narrow way between two clan plots *is* the Voronoi edge between them, weighted by clearance = 2 × distance to nearest grave. Source of truth is drone orthomosaic at 2–3 cm GSD (GL-NJF-001 §4); satellite is change-detection witness only, flagging sectors for re-flight. OSM carries the approach; it never carries the interior.

**II — Filtration.** "Can a traveler of width w pass?" is a sublevel question on the clearance field: erode free space by w/2 and test H₀ connectivity. The merge structure over w yields, for every origin–destination pair, the exact **disconnect width** w†. Connectivity is monotone: once severed at w, severed for all w′ > w (sealed as Law Test L1). Named profiles are thresholds on one structure: LONE VISITOR 0.6 m · WHEELCHAIR 0.9 m · BIER BEARERS 1.5 m.

**III — Constellation.** GNSS is untrustworthy amid dense grave fields (multipath). Re-localization is by the ground's own stars: the local point pattern of surrounding graves is a fingerprint (k-nearest constellation, rotation-normalized), unique in the spirit of the Hepta Space Uniqueness Law. A camera seeing five neighbors relocates without GNSS. Owner identity rides the anchor's KAKI with Optional EAV (archive record, inscription OCR, headstone geometry hash, constellation signature) under the **two-witness rule** — no owner is ever asserted from imagery alone.

**IV — Pack.** Offline by construction: per-sector sealed packs (skeleton + clearances + anchors + fingerprints), sharded by KAKI tribe_id, Ed25519-verifiable, Kidinnu degradation ladder for stale packs. Millions of graves route in milliseconds on-device via contraction hierarchies over the skeleton.

**V — Route.** Routes prefer clearance, honor sanctity constraints (paths that must not cross named plots), and treat **refusal buffers as walls** (GL-NJF-001 §5 — a protective buffer is never a passage, sealed as Law Test L2). Every issued route carries its profile, its ε, and its pack seal.

## 3. The Unified Pattern Clause (Nebuchadnezzar over cemeteries)

Courted grounds — Wadi al-Salam, Cairo's Qarafa (habitation layer: free space contested by dwellings), Delhi's burial grounds, lawn cemeteries — enter one OntoGraph context. The invariant intent at ⊤ is the sacred-ground navigation pattern: *occupied Boolean set + skeleton + clearance filtration + two-witness anchors + refusal buffers + Kidinnu pack*. Domain profiles vary (Qibla constant, sanctity sets, habitation); the pattern does not. Cultural honesty clause: predominantly cremation traditions (much of Hindu practice) receive a modified sacred-site wayfinding profile, not a burial-ground claim.

## 4. Playbook

- **PB-329** — Ishum skeleton kernel (pure Rust, zero deps): chamfer clearance transform, H₀ components (union–find), monotone connectivity + disconnect-width bisection, constellation fingerprint; law tests L1 (monotone severance), L2 (buffers block), L3 (bisection = scan), L4 (fingerprints distinguish neighbors).

## 5. Seal

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
