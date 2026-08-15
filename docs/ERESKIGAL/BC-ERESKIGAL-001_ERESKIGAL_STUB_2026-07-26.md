# BC-ERESKIGAL-001 — EREŠKIGAL Informal Settlement Intelligence
## BahyWay.Ecosystem v4.0 | Status: SCAFFOLD (crate: IMPLEMENTED, tests passing) | 2026-07-26

### Deity Reference
EREŠKIGAL — Queen of the Great Below, ruler of the underworld,
sister of Inanna. Sovereign over those who dwell in the margins.

### Domain
Informal settlement (slum/IDP camp) infrastructure mapping,
service access monitoring (water/electricity/sanitation),
population flow tracking, humanitarian needs assessment.

### Particle Classes
| Class          | kaki_type | kaki_role | Description |
|---|---|---|---|
| SettlementSite | 0x01 | KISHIB | Settlement identity (GPS boundary, population) |
| ServiceEvent   | 0x02 | ZIKRU  | Service delivery event |
| NeedsSignal    | 0x02 | PARZU  | Unmet need signal |
| ExternalReport | 0x03 | CrossTribe | UNHCR/OCHA/govt report |

### TIAMAT Index
SVI (Settlement Vulnerability Index), stub formula implemented in
`ereskigal-kaki::compute_svi()`.

### Tribe ID
`ereskigal_iraq` (primary), `ereskigal_regional` (extension, not yet built)

### Status
Crate `ereskigal-kaki` is real: 4 tests passing, registered in the
workspace. Full BC-ERESKIGAL-001 ADR (UNHCR CrossTribe adapter,
satellite settlement boundary ingestion) still pending.
