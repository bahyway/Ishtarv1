# EriduOS — Runtime, Scheduler, Supervisor (§11)

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
Verified against real source and `cargo test` output on 2026-07-21.**

---

## What EriduOS is

"EriduOS" names the three crates that give BahyWay.Ecosystem v4.0 its
runtime layer — a synchronous cooperative task executor, a tick-based
job scheduler, and a lifecycle/health supervisor over both. Named for
Eridu, the city in the naming law's "structural/geographic" tier
(`docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md` §8.1) — not a
literal operating system; no kernel, no process isolation, just the
in-process coordination layer §11 of the architecture reference refers
to.

| Crate | Role | Real exports | Tests |
|---|---|---|---|
| `eridu-runtime` | Synchronous cooperative task executor + `SchedulerLoop` | `EriduRuntime`, `Task`, `TaskResult`, `RuntimeState`, `SchedulerLoop`, `TickOutcome` | ✅ 10 passing |
| `eridu-scheduler` | Tick-based, interval job scheduling — no wall-clock dependency, fully testable via `tick(n)` | `EriduScheduler`, `ScheduledJob`, `DueJob`, `JobKind`, `VALIDATION_SWEEP_JOB`, `VALIDATION_SWEEP_DEFAULT_TICKS` | ✅ 9 passing |
| `eridu-supervisor` | Lifecycle + health management over runtime and scheduler; v4.0.1 added `HardwareHealthReport` (Shedu telemetry) | `EriduSupervisor`, `HealthStatus` (`Healthy`/`Degraded`/`Down`), `HardwareHealthReport` | ✅ 10 passing |

## `SchedulerLoop` — a real, in-process SDB→ODB/QDB pipeline

`eridu-runtime::SchedulerLoop` integrates `EriduScheduler` with the
SDB/ODB/QDB pipeline directly, in-process (not over a network):

1. Advance `EriduScheduler` by `n` ticks via `tick(n)`.
2. For each `DueJob` with `JobKind::ValidationSweep`:
   a. Run `ValidationSweep` on the `SdbStore`.
   b. Drain promoted particles from SDB into ODB.
   c. BlackBox Station scans quarantined particles and routes each to
      its final jail: confirmed-harmful → Storage Sector (terminal),
      fuzzy/unknown → EnkiQDB (pending Data Steward review).
   d. Journal every transition with the correct `EventCause`.

`SchedulerLoop` owns the `SdbStore`, `OdbStore`, `QdbStore`,
`StorageSector`, and `Journal` directly — a single point of coordination,
no external locking needed.

**Honest note on overlap:** this session separately built
`enkisdb-write-server` (see `docs/PB-221_SCALE_BENCHMARK_FINDINGS.md`
and `playbook_222_enkisdb_odb_qdb_dw_deploy.yml`), a standalone TCP
daemon that runs the *same* real LandingZone→SdbPipeline→
ValidationSweep→OdbStore/QdbStore shape, but as its own process behind a
binary wire protocol, for the real 2-VM CQRS deployment. `SchedulerLoop`
and `enkisdb-write-server` are two independent, real implementations of
the same pipeline logic at two different deployment granularities
(in-process vs. networked daemon) — not yet unified into one. Which one
a given caller should use depends on whether it needs networked CQRS
(`enkisdb-write-server`) or single-process embedding (`SchedulerLoop`,
e.g. inside `dubsar-theater`'s own process). This duplication is a real,
open architectural question, not resolved by this document.

## Verify it yourself

```
cargo test -p eridu-runtime -p eridu-scheduler -p eridu-supervisor
```
