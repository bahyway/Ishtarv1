# Runbook — EnkiDB Node Recovery

> **DubSar Help** | `Runbooks > Node Recovery` | Runbooks

## Trigger

An EnkiDB node goes offline. EnkiStream replication lag exceeds threshold.

## Steps

1. Confirm node failure via `nabu tribe <tribe_id>` — expect timeout or error.
2. Check EnkiStream log for last committed Events-KAKI before failure.
3. Restore the Jordan Block from the most recent EnkiDW snapshot.
4. Replay EnkiStream events from the snapshot timestamp to now.
5. Verify Jordan Block hash matches the primary node.
6. Re-enable the node in the replication topology.

## See Also

- `05_storage/enkistream.md`
- `05_storage/enkidw.md`
