# EnkiStream — Replication

> **DubSar Help** | `EnkiStream` | Storage

## Purpose

EnkiStream handles real-time replication of particle state changes across
EnkiDB nodes. It uses the Events-KAKI as its unit of replication.

## Mechanism

- Each Events-KAKI that passes all four Pauli Gates is written to the
  EnkiStream log before being applied to the hot plane.
- Replication consumers read the log and apply the same Jordan Chain append
  on their local node.
- Divergence detection: if a consumer's Jordan Block hash does not match
  the primary, a reconciliation PROBE is triggered.

## See Also

- `05_storage/enkidb.md`
- `02_identity/events_kaki.md`
