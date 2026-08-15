# Release Pipeline

> **DubSar Help** | `Pipeline > Release` | CI/CD

## Purpose

The release stage publishes a verified particle or .akk file to the EnkiDB
hot plane and EnkiStream replication log.

## Steps

1. Particle state confirmed as `Active`.
2. Events-KAKI minted and appended to Jordan Chain.
3. CrossTribe-KAKI links computed (PROBE-only; §8.3).
4. EnkiStream log entry written.
5. Orbital visualization updated (shell re-binning if HPS changed).

## See Also

- `08_pipeline_alaktu/verification.md`
- `05_storage/enkistream.md`
