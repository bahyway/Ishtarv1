# The 7 EnkiDB Types

**Standalone component reference. Follows `docs/08_pipeline_alaktu/TRANSPARENCY_STANDARD.md`.
Every crate name below was checked to actually exist as a directory
and a workspace member on 2026-07-11 — this pass found and corrects a
naming error that had propagated into this session's own prior
Architecture Reference document.**

---

## Correction made in this pass

Two turns ago, `docs/15_howto/BAHYWAY_V4_ARCHITECTURE_REFERENCE_2026-07-11.md`'s
EnkiDB Types table cited `enkidb-dw`, `enkidb-sdb`, `enkidb-odb`, and
`enkidb-qdb` as real crates (alongside the correct `enkidw`, `enkisdb`,
`enkiodb`, `enkiqdb`). **Those four hyphenated names do not exist —
checked directly: `ls -d enkidb-dw enkidb-sdb enkidb-odb enkidb-qdb`
returns "No such file or directory" for all four, and none appear in
the workspace `Cargo.toml` members list.** They were carried forward
from an older naming convention in the pre-2026-07-01 Glossary/Roadmap
without being re-checked against the current directory tree. The real
crates are the no-hyphen names only: `enkidw`, `enkisdb`, `enkiodb`,
`enkiqdb`. This is exactly the kind of error the transparency standard
exists to catch — including in documents this session itself produced.

---

## The 7 types

**✅ VERIFIED** — every crate below confirmed to exist as a directory,
a workspace member, and a passing test suite on 2026-07-11.

| Port | Name | True Role | Real Crate | Test count (2026-07-11) |
|---|---|---|---|---|
| 7001 | EnkiDB | Golden Store — final, permanent destination. Journal-replay-on-open today. | `enkidb-engine`, `enkidb-persist`, `enkidb-storage` | `enkidb-engine`: 4 |
| 7002 | EnkiDW | Data warehouse — full ETL (LandingZone, ZipEngine, WayCompiler) + analytics, receives retired EnkiODB particles | `enkidw` | 59 |
| 7003 | EnkiSDB | Stage/landing DB — Musarû pre-scan, 15-minute (or 900-tick, sources disagree on the exact interval — not resolved here) validation sweep | `enkisdb` | 14 |
| 7004 | EnkiODB | Operational DB — active validated particles; state changes are new inserts, never mutations | `enkiodb` | 5 |
| 7005 | EnkiQDB | Quarantine — permanent append-only store for particles that failed validation or were Musarû-flagged, **fuzzy/unknown only, never confirmed-malicious** (that's Storage Sector's job) | `enkiqdb` | 5 |
| 7006 | EnkiMDB | Metadata Database — BahyWay's own crates, playbooks, and services as KAKI-sealed EAV particles | `enkimdb` | 7 |
| 7007 | EnkiDDB | Documentation Database — BahyWay documents (internal + client) as KAKI-sealed EAV particles | `enkiddb` | 13 |

**🧩 PARTIAL, timeline note:** EnkiMDB and EnkiDDB are the two most
recently completed types. Every document dated through 2026-07-07
(the 28-document review, the Manifesto's own "Collection C —
confirmed not built") records both as absent. They exist now,
confirmed by the crate directories and their exactly-matching
`Cargo.toml` descriptions above — real progress in the four days
between that review and this document, not a standing gap.

## Naming-collision history — now resolved, kept for the record

**📄 DOCUMENTED**, per this ecosystem's "old state is never erased" law.
Before EnkiMDB/EnkiDDB existed, two unrelated, real crates squatted on
their ports without matching their true role:
- **`enkidb-quantdb`** (still exists, still real, still unrelated) —
  "EnkiDB Quantitative Time-Series Store — sovereign tick/OHLC archive
  for asset particles." Was previously the only thing registered at
  port 7006; is not EnkiMDB and never claimed to be once the
  mislabeling was caught in 2026-07-01.
- **`enkidb-recovery`** (still exists, still real, still unrelated) —
  "Crash Recovery and Abrupt-Termination handler." Was previously the
  only thing at port 7007; is not EnkiDDB.

Both remain real, separate, useful crates. Neither collides with
EnkiMDB/EnkiDDB anymore now that those exist under their own names.

## Post-KAKI Tier-Transition Pipeline

**✅ VERIFIED, real, tested in `eridu-runtime::SchedulerLoop`.**

```
EnkiSDB (staged, Pending)
    v ValidationSweep
    +-- pass ------------------------> EnkiODB (Active) -> EnkiDW (retire) / EnkiDB (Golden Store)
    +-- fail (Quarantined) -> BlackBox Station scan
                                 +-- malware_flag=true  -> Storage Sector (terminal jail, one-way)
                                 +-- malware_flag=false -> EnkiQDB (fuzzy, pending Data Steward review)
                                                                v Data Steward resolves
                                                     +-- clean -> requeued into EnkiSDB (Pending)
                                                     +-- confirmed harmful -> Storage Sector
```

`storage-sector` and `blackbox-station` are ✅ VERIFIED real, separate
crates (built 2026-07-01). `Storage Sector`'s sealing is a one-way
door — the crate exposes no method to remove, read back, or requeue a
sealed particle.

## Pre-KAKI Structural Pipeline (BeeMDM)

**🧩 PARTIAL** — see `docs/08_pipeline_alaktu/BEEMDM_ETL_PIPELINE.md` for the
full, dedicated treatment. Summary: DataStructure → DataCompare →
DataCleansing, each a real crate (`data-structure-station`,
`compare-tribe-schema`, `data-cleansing-station` + `pii-vault`), but no
single crate implements this exact three-station sequence as one
pipeline object the way the post-KAKI pipeline above is implemented in
`SchedulerLoop`.

## Open items

1. **Validation sweep interval** — the Glossary and this session's
   Architecture Reference both say "every 900 ticks"; `enkisdb`'s own
   Cargo.toml description says "15-minute... validation sweep." Not
   reconciled here — if 900 ticks and 15 minutes are meant to be the
   same thing (900 ticks = 15 min at 1 tick/sec), that's consistent
   and just stated two ways; if not, this is a real discrepancy.
2. **EnkiDB's real WAL + data-file storage engine** — still deferred.
   `enkidb-engine`/`enkidb-persist`/`enkidb-storage` are real but
   journal-replay-on-open remains the actual mechanism at port 7001;
   the "Golden Store, >1B particles" goal depends on this being built.
