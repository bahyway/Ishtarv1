# EnkiDW — Data Warehouse Persistence

> **DubSar Help** | `EnkiDW` | Storage

## Purpose

EnkiDW is the persistence layer for historical Orbit data. It stores the full
Jordan Chain of every particle — the immutable audit trail of all state
transitions since particle birth.

## Mechanism

- Dead particles are "frozen" into EnkiDW in their Ground State.
- Spectral Renormalization compresses petabyte-scale Tribe histories into
  Macro-Eigenvalue summaries for Hubble-Zoom queries.
- EnkiDW is append-only; no record is ever modified or deleted.

## See Also

- `05_storage/enkidb.md`
- `04_gates/shamash_gate.md`
- `09_observatory/orbital_visualization.md`
