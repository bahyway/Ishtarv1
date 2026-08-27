# ADR-007 — Mandatory Snapshot Scheduler in EnkiDB and UrOS

> **DubSar Help** | `ADR > 007` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-06-05"
  concept_depth:  230
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "Mandatory Snapshot Scheduler"
summary:          "Mandatory snapshot scheduler with Index 7 sparse B-tree enables O(log k) projection at exabyte scale."
sovereign_laws:   ["§7.1 — snapshot interval derived from Markov steady-state"]

riksu_bindings:
  - target: "adr_006_no_delete_mandatory_partitioning.md"
    concept: "Three-Pillar Sovereign Data Law"
    type: "PEER"
  - target: "enkidb.md"
    concept: "Index 7 snapshot B-tree"
    type: "GROUNDS"

orbit_tags:       ["Sovereign Storage", "KAKI Sovereignty"]
rag_keywords:     ["PROJECT", "FORECAST", "Index 7", "snapshot", "sparse B-tree", "O(log k)"]
-->

## Status: Accepted

---

## Context

### The Eternal Journal Problem

ADR-006 established that DELETE does not exist in EnkiDB. Every particle
accumulates an append-only Journal of Event-Kakis from birth. In a system
with no deletion, journals grow without bound.

The StoryEngine (§3.6) projects a particle's current state by replaying its
entire Journal from the beginning:

```
Cost of projection = O(total events in journal)
```

For a particle with 10 events this is trivial. For a civil registry particle
with 10,000 events, or a sensor stream particle with 1,000,000 events, full
replay becomes operationally unacceptable — the cost grows linearly with
the age of the system.

### The Power-Outage Risk (§11.6)

BahyWay deployments in the Iraqi sovereign context face a specific
operational constraint: frequent power outages. Without a periodic snapshot
committed to durable storage, a long-running Journal that loses power
mid-session requires full replay from the beginning at restart — which can
take minutes or hours for high-volume particles. This is unacceptable for
operational continuity.

### The Three-Pillar Sovereign Law

ADR-006 established two pillars:
- **No DELETE** — data is eternal
- **Mandatory Partitioning** — eternal data remains queryable

This ADR establishes the third and final pillar:
- **Mandatory Snapshot Scheduler** — eternal journals remain performant

These three pillars are inseparable. A system that never deletes, never
partitions, and never snapshots will eventually become unqueryable. All
three must be present for the sovereign guarantee to hold at scale.

---

## Decision

### Decision 1 — Snapshots Are Mandatory, Not Optional

Every EnkiDB and EnkiDW instance **must** have at least one
`SnapshotJob` registered and running. Instances with no active snapshot
schedule are not permitted in production.

A snapshot is not a special database operation. It is a **normal
Event-Kaki** (kaki_type = 0x02) that updates three mandatory universal
EAV attributes on the particle:

| Attribute | Attribute Hash | Content |
|---|---|---|
| `Snapshot_Date` | `0x9A1D` | Epoch of the snapshot |
| `Snapshot_State` | `0x7E32` | Full projected state at that epoch |
| `Snapshot_Frequency` | `0x4B0F` | VectorId of the governing SnapshotJob |

Because a snapshot is a normal Event-Kaki:
- It obeys all LineageChain rules — it is appended, never modified
- It is part of the particle's immutable history
- It cannot be retroactively removed or altered
- It is visible in audit trails exactly as any other event

### Decision 2 — The SnapshotJob Is a VectorId Mechanism, Not a Particle

The `SnapshotJob` is identified by a `VectorId` in `Default.Jobs` — it
is **not** a particle and does not have a KAKI PK. This is a deliberate
design: the scheduler is an operational mechanism of the system, not a
data entity within it.

```
Default.Jobs registry:
    VectorId(snapshot_hourly)   → SnapshotJob { interval: 3600s }
    VectorId(snapshot_sensor)   → SnapshotJob { interval: 100 events }
    VectorId(validation_sweep)  → ValidationSweep { interval: 900 ticks }
```

### Decision 3 — Four Sovereign Schedule Policies

The `SnapshotSchedule` enum defines four sovereign policies, each matching
a real deployment pattern:

| Policy | Trigger | Sovereign Use Case |
|---|---|---|
| `Never` | Never | Archive-only particles that never change |
| `EventCount(n)` | Every N events | Sensor streams (n=100), registries (n=10,000) |
| `TimeBased(duration)` | Every N seconds/hours | Iraqi deployment (hourly), financial ledgers (daily) |
| `OnEveryTransition` | Every state change | High-churn real-time particles |

**Built-in sovereign presets (from §11.6 and §3.6):**

```rust
SnapshotSchedule::iraqi_deployment()  // hourly — power-outage resilience
SnapshotSchedule::civil_registry()    // every 10,000 events — low-churn sovereign data
SnapshotSchedule::sensor_stream()     // every 100 events — high-churn IoT data
```

### Decision 4 — UrOS Runs the Scheduler (eridu-scheduler + eridu-runtime)

The `EriduScheduler` in `crates/eridu-scheduler` is the sovereign job
dispatcher. It uses **logical ticks** (not wall-clock time) — making it
fully deterministic and testable without real-time dependency.

```
EriduRuntime → scheduler_loop
    │ tick(n) every sovereign epoch
    ▼
EriduScheduler::tick_typed(n)
    │ returns Vec<DueJob> { name, kind }
    ▼
Dispatch by JobKind:
    JobKind::ValidationSweep → run SDB validation (§12.1, every 900 ticks)
    JobKind::SnapshotJob     → run SnapshotJob::run() for assigned particles
    JobKind::Generic         → domain-specific job handler
```

The `VALIDATION_SWEEP_DEFAULT_TICKS = 900` constant (15 minutes at 1
tick/second) is a sovereign default — administrators may reconfigure via
`EriduScheduler::set_job_interval()` but cannot set interval to 0.

### Decision 5 — Index 7: The Snapshot Index Is a Sovereign Native Index (§9.3)

The seven native sovereign indexes of EnkiDB are embedded in the storage engine
itself — they are not optional and they are not user-created. Index 7 is the
**Snapshot Index**:

| Index | Structure | Purpose |
|---|---|---|
| Index 1 | Identity B-tree | Lookup by KAKI PK (`κ[0..15]`) |
| Index 2 | Sovereignty B-tree | Lookup by tribe + domain |
| Index 3 | Type+Role bitmap | Filter by kaki_type + kaki_role |
| Index 4 | Temporal LSM | Time-ordered Journal access |
| Index 5 | ColorID R-tree | Spatial queries on 7D quality vector |
| Index 6 | EAV inverted index | Attribute-value lookup across all particles |
| **Index 7** | **Snapshot sparse B-tree** | **Jump directly to the nearest snapshot before a target epoch** |

Index 7 is a **sparse** B-tree — it only stores one entry per snapshot
Event-Kaki, not one entry per Journal Event-Kaki. This sparsity is the
exabyte-scale guarantee:

```
Without Index 7:
    StoryEngine must scan the full Journal to find whether a snapshot exists.
    At 1,000,000 Event-Kakis per particle, this scan itself is O(n) before
    projection even begins.

With Index 7 (sparse B-tree):
    A particle with 10,000 Event-Kakis and a snapshot every 100 events
    contributes ~100 entries to Index 7 — not 10,000.
    At billions of particles, this is the difference between feasible and
    infeasible.
    StoryEngine calls: Index7::nearest_snapshot_at_or_before(kaki_pk, epoch)
    → returns SnapshotRecord or None in O(log k) where k = number of snapshots
    → then replays only Journal entries after the snapshot epoch
```

Index 7 is built and maintained automatically by the storage engine. Every
Snapshot Event-Kaki written to the Journal is automatically indexed. No
administrator action is required.

### Decision 6 — Projection Algorithm Chooses Automatically

The `ProjectionAlgorithm` enum governs how the StoryEngine starts
projection for any particle at any point in time:

```
StoryEngine::project(particle, at_epoch):
    → find most recent SnapshotRecord at or before at_epoch
    → if found:  SnapshotAccelerated { start from snapshot_state }
    → if absent: FullReplay { start from Journal beginning }

Cost:
    FullReplay:           O(total events)      — unbounded growth
    SnapshotAccelerated:  O(events since snap) — bounded by schedule interval
```

A particle with a daily snapshot and 1,000 events per day will never
require replaying more than ~1,000 events regardless of how old it is.
This is the sovereign performance guarantee of mandatory snapshotting.

---

## W5H2

| W | Answer |
|---|---|
| **Who** | Every EnkiDB instance, every EnkiDW instance, UrOS runtime (eridu-scheduler + eridu-runtime), and every data steward configuring a tribe's Template |
| **What** | A mandatory scheduled `SnapshotJob` that periodically appends a Snapshot Event-Kaki to each particle's Journal, capturing full projected state. This bounds projection cost to O(events since last snapshot) regardless of total journal length |
| **When** | From EnkiDB v4.0 onwards — eternal, never revised. Schedule frequency is configurable per tribe/template; the existence of a schedule is not |
| **Where** | `crates/enkidb-snapshot` (SnapshotRecord, ProjectionAlgorithm), `crates/snapshot-job` (SnapshotJob, SnapshotSchedule), `crates/eridu-scheduler` (EriduScheduler, JobKind), `crates/eridu-runtime` (scheduler_loop) |
| **Why** | Without snapshots, StoryEngine projection cost grows as O(total events) — unbounded in a system with no DELETE. Mandatory snapshots bound this cost. In power-outage environments (§11.6), snapshots provide restart continuity without full-journal replay |
| **How** | SnapshotJob runs on EriduScheduler ticks. For each particle with new events since last snapshot: project current state via StoryEngine, append a Snapshot Event-Kaki (normal kaki_type=0x02) updating Snapshot_Date, Snapshot_State, Snapshot_Frequency. ProjectionAlgorithm automatically selects SnapshotAccelerated when a snapshot exists |
| **How Much** | 3 mandatory EAV attributes per particle · 4 schedule policies · 3 built-in presets · 7 native sovereign indexes (Index 7 = Snapshot sparse B-tree) · VALIDATION_SWEEP_DEFAULT_TICKS = 900 · Zero wall-clock dependency in scheduler · 100% deterministic and testable |

---

## The Three-Pillar Sovereign Law (Complete)

```
┌─────────────────────────────────────────────────────────────────────┐
│              EnkiDB / EnkiDW Sovereign Data Law                     │
├─────────────────────┬───────────────────────┬───────────────────────┤
│   ADR-006 Pillar 1  │   ADR-006 Pillar 2    │   ADR-007 Pillar 3   │
│                     │                       │                       │
│   NO DELETE         │   MANDATORY           │   MANDATORY           │
│                     │   PARTITIONING        │   SNAPSHOT            │
│                     │                       │   SCHEDULER           │
├─────────────────────┼───────────────────────┼───────────────────────┤
│ Guarantee:          │ Guarantee:            │ Guarantee:            │
│ No particle ever    │ Eternal data stays    │ Eternal journals      │
│ disappears          │ queryable             │ stay performant       │
│                     │                       │                       │
│ Mechanism:          │ Mechanism:            │ Mechanism:            │
│ INSERT/UPDATE/MERGE │ Epoch + Lane +        │ SnapshotJob on        │
│ only — DELETE has   │ Domain + State        │ EriduScheduler ticks  │
│ no grammar entry    │ partition dimensions  │ — O(delta) projection │
└─────────────────────┴───────────────────────┴───────────────────────┘

Without Pillar 1: data can be erased — lineage trust broken
Without Pillar 2: eternal data degrades query performance — operationally fails
Without Pillar 3: eternal journals degrade projection performance — operationally fails

All three must be present. All three are mandatory. None is optional.
```

---

## Consequences

### Positive

**Performance:**
- StoryEngine projection cost is bounded to O(events since last snapshot)
  regardless of the particle's total age or journal length.
- At steady state, high-frequency particles (sensors, financial ticks) never
  require replaying more than `EventCount(n)` events regardless of system age.

**Operational resilience:**
- Power outages in Iraqi deployments cause at most `interval` events of
  re-computation on restart — not full-journal replay from birth.
- The `iraqi_deployment()` preset (hourly) means maximum 1 hour of events
  must be replayed on unexpected restart.

**Audit integrity:**
- Snapshots are normal Event-Kakis — they appear in the LineageChain and
  cannot be distinguished from other events by an auditor.
- A snapshot cannot be retroactively inserted before another event — the
  Journal is append-only and timestamps are sovereign epoch values.
- An attacker cannot fake a "clean" snapshot at an earlier point to hide
  intervening events — the chain hash would break.

**UrOS integration:**
- The scheduler uses logical ticks — it runs identically in production,
  test, and simulation environments.
- `set_job_interval()` allows administrators to tune schedules without
  restarting the system or recompiling code.

### Constraints Introduced

- **Every tribe Template must declare `Snapshot_Frequency`** at creation.
  Templates without a snapshot frequency inherit `Never` — acceptable only
  for truly static archive particles.
- **Storage grows for snapshots too.** Each snapshot appends
  `sizeof(projected_state)` bytes to the Journal. High-frequency
  snapshotting of large particles requires storage tiering planning
  (satisfied by ADR-006 Mandatory Partitioning).
- **Snapshot state encoding must be stable.** `story_engine::projection::encode_state()`
  must produce deterministic output across software versions — a format
  change requires a snapshot migration job before the old format snapshots
  become unreadable.

---

## Relationship to Previous ADRs

| ADR | Interaction |
|---|---|
| **ADR-001** (No External DB) | The SnapshotJob, EriduScheduler, and StoryEngine are pure Rust — no external cron daemon, no PostgreSQL trigger, no Redis TTL. The scheduler is sovereign. |
| **ADR-003** (KAKI Sovereignty) | A Snapshot Event-Kaki carries the same KAKI identity as any other event. The snapshot is part of the particle's sovereign identity record. |
| **ADR-004** (BeeMDM 4-lane pipeline) | Snapshot frequency is set per quality lane — GEM and TRIBE particles may have more frequent snapshots than FUZZY or DEAD particles whose journals rarely change. |
| **ADR-005** (Enterprise Data Fabric) | `bahyway-fabric` pipeline runs trigger snapshot-worthiness checks. A particle that passes through multiple pipeline stages in one epoch may qualify for an immediate snapshot via `OnEveryTransition`. |
| **ADR-006** (No DELETE + Mandatory Partitioning) | Snapshots are the third pillar of the sovereign data law. No DELETE makes data eternal. Mandatory Partitioning makes eternal data queryable. Mandatory Snapshot Scheduler makes eternal journals performant. |

---

## Sovereign Law Statement

> **A particle's journal is its life. As that life grows longer — through
> thousands of events, through years of state transitions, through power
> outages and restarts — the cost of reading that life must not grow with
> it. The Snapshot Scheduler is the sovereign covenant that makes this
> possible: at every configured interval, the system takes a photograph
> of the present and seals it into the journal. From that moment, the
> past is not forgotten — it is summarised. The future reads from the
> summary, not from the entire history.**
>
> **In UrOS and in every EnkiDB and EnkiDW instance: there is no
> production database without a Snapshot Scheduler. This is not
> negotiable.**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | ADR-007 Accepted 2026-06-05*
