# Tablet of Ḫendursaĝa — Charter of the ḪendursagaEngine
## GL-NAV-001 (candidate) · Navigation Domain · Annex A: Wadi al-Salam Field Architecture
## DRAFT — unsealed, awaiting Architect run & CSR-08 confirmation

---

## I. Identity

**Logical name (god layer, NL-001):** ḪendursagaEngine
**Physical name (Ḫubullu gloss, GL-NAM-002):** Navigation Engine (NAV)
**Sector:** Navigation & Pathways · dual terrain
**Era:** Zagesi · proposed 2026-08-11
**Naming rationale:** Ḫendursaĝa, herald of the streets, the god who walks *before* the traveler with his staff — patron of safe passage and of pathways at night. He governs two terrains under one name: paths in Hepta space (the ecosystem flight deck, camera rites BIRD / ORBIT / LANDING / FOLLOW / REPLAY) and paths on Earth (the offline field navigator). The herald who walks before travelers also walks before mourners, guiding them to their dead; no duty in the ecosystem fits the god of pathways more exactly than Wadi al-Salam.

The name **NaviEngine** from the source proposal is retired at the threshold: acronyms do not name engines. The proposal's engineering substance is adopted; its name is not.

---

## II. Mission of Annex A

Provide centimetre-grade, fully offline navigation and georeferencing across Wadi al-Salam (Najaf) — the world's largest cemetery, millions of graves over ~10 km² — as the spine of a grave-survey and grave-detection capability for the client of record **Najaf Cemetery Authority (SLA §4.2 — Data Custody & Repair)**. The cemetery is a NUZI made physical: the archive city of lineage. Every capability in this annex serves three acts: *find the path, fix the position, seal the record.*

---

## III. The Three Layers (one god, three garments)

### Layer 1 — Field Core (pure Rust, headless, adopted from the source proposal)
- **Sensors:** u-blox ZED-F9P GNSS with **RTK** corrections (RTCM from a base station at the cemetery office) → centimetre accuracy; Bosch BNO08x-class IMU; barometer; magnetometer.
- **Fusion:** error-state Kalman filter (INS/GNSS); **dead-reckoning fallback is mandatory, not optional** — mausoleum corridors produce multipath that blinds GNSS, and the survey must not stop.
- **Time:** PPS from GNSS; every sample stamped. Timestamps are the hinge on which Layer 2 turns.
- **Maps:** the survey's own drone-photogrammetry orthophoto → tiles → **MBTiles on industrial SD** (rusqlite reads it natively). No online tiles, ever. OSM is deleted from the tile-source list, not merely deselected.
- **Serving:** axum HTTP server for the local UI + tile endpoint; **WebSocket feed at `ws://:9000/position`** streaming `{lat, lng, alt, heading, speed, t}` — this contract is constitutional: the field UI, the Šala flight-deck prototype, and DubSar Theater's Godot rig all drink from the same stream.
- **Resilience:** hardware watchdog, journaled writes, append-only telemetry (NĀRU discipline in the field), auto-restart, start-up self-checks with safe-fail.
- **Power & environment:** LiFePO₄ + solar charge controller + DC-DC; wide-temperature industrial SBC (CM4 industrial carrier class); dust-sealed enclosure. Najaf summers reach 50 °C; the hardware doctrine of the source proposal is adopted as written.

### Layer 2 — Sensing (georeferencing of the detectors)
- Ground-penetrating radar (GPR) and/or magnetometry ride the survey cart; the Field Core's RTK+PPS stamps **every trace with a position and a time**.
- A georeferenced radar anomaly is **minted as a particle**: KAKI identity, OrbitalPosition carrying the survey coordinates, state **FUZZY** — a candidate grave is a hypothesis, never a fact.
- Interpretive honesty clause: Wadi al-Salam holds multi-level vaulted burials stacked over centuries; radar returns are layered and ambiguous. Therefore **no anomaly auto-promotes**. The Analysis-to-Solution Law governs: DETECT (trace) → PROVE (steward/archaeologist confirmation, two witnesses where possible: GPR + magnetometry or GPR + registry record) → then and only then the state may leave FUZZY.

### Layer 3 — Registry (BahyWay.Ecosystem v4.0)
- Cemetery **sections and family plots = tribes**; grave records = particles; burial permits, photographs, and scanned registers ingest through **EnkiDDB** under the Simtu/Šasû/Eṭemmu laws; lineage and provenance rest in **NUZI**; contested identifications go to the **Madanu Court**; AsalluhiEngine advises on anomalous or corrupted records.
- Every record sealed **Ed25519**; every change a journal entry; nothing overwritten, only succeeded — in a cemetery registry, the append-only doctrine is not a technical choice but a moral one.

---

## IV. Field UI Doctrine (the borrowed vessel)

- **Leaflet JS, self-hosted, serving only local tiles** — the borrowed browser-side vessel, permitted because the engine runs headless and the UI is optional (the source proposal's own rule, adopted). The unpkg CDN references are replaced by vendored files on the device.
- **MapLibre GL JS (self-hosted) is the approved upgrade path** when the landing-camera feel is wanted in the field: native bearing and pitch, offline vector tiles. Leaflet for the MVP; MapLibre for the flight deck.
- HTML remains prototype/field-UI only per the Way of Work; the sovereign production theater remains DubSar Theater, fed by the same `ws://:9000/position` stream.

---

## V. Ethics & Sovereignty Clause (non-negotiable)

Burial data is religious data, family data, grief data. Therefore:
1. **Offline-first is mandatory**, not preferred. The device functions with zero connectivity; nothing syncs without an explicit, sealed export rite.
2. Access governed by **AkkadiRulesEngine ABAC**; family-plot records readable only by roles the Authority decrees.
3. Anomaly candidates (possible unmarked graves) are treated with the same dignity as confirmed records: FUZZY particles are prune-exempt while any investigation is open (EnkiQDB custody discipline).
4. No third-party map service, analytics, or telemetry touches the data. The Transparency axiom holds: the edge where transparency ends is the edge this engine does not cross.

---

## VI. Invariants

1. Engine core pure Rust; browser JS confined to the optional UI vessel; no CDN at runtime in the field.
2. KAKI v4.0 byte layout untouched; anomalies enter as FUZZY and leave FUZZY only through the PROVE rite.
3. The `ws://:9000/position` message schema is versioned and sealed; consumers adapt to it, never it to consumers.
4. RTK base-station coordinates are surveyed, sealed, and journalled; a moved base without a re-seal invalidates the day's fixes (and the playbook says so loudly).
5. Every provisioning act, fix, and calibration is a numbered playbook; running it is the Architect's CSR-08 confirmation.
6. Ḫubullu: every sealed term in this tablet carries its plain gloss (§VIII).

---

## VII. Playbook Registry — PB-420 series (numbering pending registry confirmation)

| PB | Name | Duty |
|---|---|---|
| PB-420 | field core scaffold | Cargo workspace: hendursaga-core, gnss, imu, fusion-ekf, navcore, tileserver, feed |
| PB-421 | tile pipeline | orthophoto → tiles → MBTiles, checksums, dual seal, copy to industrial SD |
| PB-422 | RTK base station | ZED-F9P base config, surveyed position seal, RTCM caster service |
| PB-423 | field SBC image | hardened OS, watchdog, journaled logging, power monitors, autostart units |
| PB-424 | feed + UI deploy | position feed service on :9000, vendored Leaflet UI, local-tiles-only guard |
| PB-425 | GPR anomaly minting | trace georeferencing, FUZZY particle minting to EnkiSDB, Ed25519 seal |
| PB-426 | registry bridge | Najaf tribes schema (EnkiMDB), EnkiDDB permit ingestion, NUZI lineage, ABAC decrees |

All stubs delivered beside this tablet as DRAFT Ansible; none is law until run and confirmed.

---

## VIII. Ḫubullu Glossary

| Sealed term | Plain gloss |
|---|---|
| ḪendursagaEngine | Navigation Engine (NAV) |
| Field Core | the offline navigation computer on the survey cart |
| RTK | correction technique giving centimetre GPS accuracy |
| dead-reckoning | keeping position by motion sensors when GPS is blocked |
| MBTiles | a single-file offline map database |
| anomaly particle | a possible unmarked grave, recorded as a hypothesis |
| FUZZY | "detected but not yet confirmed" |
| NUZI | the lineage archive — here, the cemetery registry itself |
| feed contract | the fixed message format every display must accept |

---

*Provenance: DRAFT, grade P0. Source engineering adopted from the external NaviEngine proposal with its name retired at the gate and its OSM fallback deleted. The Fadam Floor applies. Nothing herein is law until the Architect runs the rites and confirms.*
