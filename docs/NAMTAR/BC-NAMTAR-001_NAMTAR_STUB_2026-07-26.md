# BC-NAMTAR-001 — NAMTAR Sacred Burial Intelligence
## BahyWay.Ecosystem v4.0 | Status: SCAFFOLD (crate: IMPLEMENTED, tests passing) | 2026-07-26

### Deity Reference
NAMTAR — Mesopotamian messenger of fate, attendant of Ereškigal,
queen of the underworld. Guardian between the living and the dead.

### Domain
Sacred burial site mapping, preservation status monitoring,
archaeological risk assessment, cultural heritage provenance.

### Particle Classes
| Class       | kaki_type | kaki_role | Description |
|---|---|---|---|
| SiteRecord  | 0x01 | KISHIB | Site identity (GPS, heritage tier) |
| StatusEvent | 0x02 | ZIKRU  | Preservation state change |
| ThreatSignal| 0x02 | PARZU  | Risk event (looting/conflict/construction) |
| ExternalRef | 0x03 | CrossTribe | UNESCO/govt/satellite imagery reference |

### TIAMAT Index
PHI (Preservation Health Index), stub formula implemented in
`namtar-kaki::compute_phi()` — full Mamdani FIS still pending.

### Tribe ID
`namtar_iraq` (primary), `namtar_regional` (extension, not yet built)

### Status
Crate `namtar-kaki` is real: 4 tests passing, registered in the
workspace. Full BC-NAMTAR-001 ADR (Mamdani FIS, satellite imagery
adapter, UNESCO CrossTribe feed) still pending.
