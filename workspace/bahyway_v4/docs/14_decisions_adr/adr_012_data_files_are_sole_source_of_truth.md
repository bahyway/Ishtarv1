# ADR-012 — Write Node Journals, Read Node Serves Data Files Only, via ENKWAL Streaming Replication

> **DubSar Help** | `ADR > 012` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-07-07"
  concept_depth:  240
  riksu_count:    4
  snapshot_epoch: "2026-07-07"

concept:          "Write-Node Journal / Read-Node Data Files, split by ENKWAL replication"
summary:          "The write node keeps its journal as a durable command log — it has no stakeholder interface, so replay cost there is acceptable. The read node, which is the sole stakeholder-facing interface and must answer queries over 1B+ particles in under 1 second, holds no journal at all and serves exclusively from indexed, materialized Data Files. The two are connected by enkidb-replication's already-built ENKWAL pipeline (Emitter -> Broker -> Consumer), the sovereign equivalent of PostgreSQL streaming replication."
sovereign_laws:   ["§3.6-AMEND — read-node projection reads Data Files via Index 1/Index 7, never replays", "§11.3-AMEND — read-node recovery validates Data Files, never replays", "§CQRS-SPLIT — write node owns the Journal, read node owns Data Files, ENKWAL is the only channel between them"]

riksu_bindings:
  - target: "adr_006_no_delete_mandatory_partitioning.md"
    concept: "Three-Pillar Sovereign Data Law"
    type: "PEER"
  - target: "adr_007_mandatory_snapshot_scheduler.md"
    concept: "Index 7 Snapshot sparse B-tree"
    type: "AMENDS"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI partition axes"
    type: "GROUNDS"

orbit_tags:       ["Sovereign Storage", "KAKI Sovereignty", "Performance", "Replication"]
rag_keywords:     ["Data Files", "no full scan", "Index 1", "Index 7", "ENKWAL", "streaming replication", "write node", "read node", "materialized state", "O(1) lookup"]
-->

## Status: Accepted

---

## Context

### The full-scan discovery

ADR-007 established the Three-Pillar Sovereign Data Law — No DELETE,
Mandatory Partitioning, Mandatory Snapshot Scheduler — with the explicit
goal that "the cost of reading [a particle's] life must not grow with it."
Index 7 (a sparse B-tree, `uuid_hash+epoch → cold_storage_offset`) and
Index 1 (a point-lookup index, `uuid_hash → file_offset`) were designed
for exactly this and exist today in `enkidb-indexes`. They are not wired
to anything:

- `story_engine::StoryEngine::project()` — the read path for "what is
  this particle's state right now" — **always** calls
  `Journal::read_particle_history()`, a full scan of the particle's shard
  partition, on **every** call, current-state or historical alike.
- `EnkiDb::register_particle` hardcodes `idx_identity.insert(uuid_hash, 0)`
  — Index 1 has never pointed at a real file.
- `EnkiDb::append_event` never calls `idx_snapshot.insert()` — Index 7 is
  built, tested, and populated by nothing.
- `ProjectionAlgorithm::SnapshotAccelerated`, when chosen, discards the
  snapshot state it found and still calls the unbounded
  `read_particle_history()` underneath — it currently provides zero
  actual speedup.
- `enkidb-recovery::RecoveryProcedure::run()` and
  `enkidb-persist::PersistedDb::open()` both replay the full journal on
  every restart.

At the scale this system is built for — orbits of milliards (billions) of
particles, answered to stakeholders in under one second — this is not a
performance defect to tune later. It is a structural failure mode that
will bring the system down exactly when it succeeds at scale.

### The write/read split (Architect's ruling)

The initial framing of this ADR proposed retiring the Journal everywhere.
The Architect corrected this: **the write node has no stakeholder
interface** — nothing external queries it directly — so paying journal
overhead there is free. **The read node is the only interface stakeholders
touch**, and it alone must obey the sub-second, billion-particle law. The
correct split is therefore by node role, not by blanket removal:

- **Write Node**: keeps its Journal (`enkidb-journal`, `enkidb-persist`'s
  disk-first WAL) exactly as built. It is a command log, not a query
  surface.
- **Read Node**: holds no Journal at all. It serves exclusively from
  indexed, materialized Data Files. It never replays anything, under any
  circumstances, including its own crash recovery.

### The replication mechanism already exists

The connective tissue between these two nodes does not need to be
invented. `crates/enkidb-replication` already implements a complete,
tested, cryptographically-sealed one-way pipeline — the sovereign
equivalent of PostgreSQL's WAL streaming replication:

```
Write Pod                    Broker (--network none)          Read Pod
ReplicationEmitter    ──►    ReplicationBroker         ──►    ReplicationConsumer
(KANĀKU Ed25519 seal,        (7-layer verification gate,      (2nd verify pass,
 chain-hashed .enkwal log)    write.enkwal → delta.enkwal)      apply_fn callback)
```

Frames are chain-hashed (SHA3-256, `prev_digest`), sequence-checked
(monotonic `seq`), Ed25519-sealed (KANĀKU), and epoch-freshness-checked
(5-minute window) — tamper-evidence is already stronger than a bare
Postgres WAL stream. What is missing is wiring, not mechanism: nothing
today calls `ReplicationEmitter::emit()` on commit, and
`ReplicationConsumer`'s `apply_fn` is an unwired callback stub.

---

## Decision

### Decision 1 — The Write Node's Journal is unchanged and out of scope

`enkidb-journal`, `enkidb-persist`'s disk-first WAL, and
`enkidb-recovery`'s journal-based crash recovery remain exactly as built,
**on the write node only**. Nothing in this ADR requires touching them.
The write node's job is durable, ordered command capture — it is
correctly built already.

### Decision 2 — Every write-node commit emits a replication frame

On every `EnkiDb::append_event` / KISPU commit on the write node, after
the journal write succeeds, the write node calls
`ReplicationEmitter::emit(ReplEventKind::ParticleInsert | ParticleUpdate, delta, epoch)`
with a serialized delta: `{uuid_hash, tribe_id, epoch, eav: Vec<EavTriple>}`.
This is the sole channel by which the read node learns of any change —
there is no other path, no shared database file, no second writer.

### Decision 3 — The Broker is deployed unchanged

`ReplicationBroker` already implements the correct verification gate
(frame integrity, epoch freshness, sequence monotonicity, chained digest,
KAKI match, Ed25519 seal, HeptaSecSentinel). It runs `--network none`
between the write and read pods' shared volumes exactly as documented in
`crates/enkidb-replication/MANUAL.md`. No code change — deployment wiring
only (real ansible playbook, not a design change).

### Decision 4 — The Read Node materializes Data Files from the replication stream, never from replay

`ReplicationConsumer`'s `apply_fn` is wired to a new read-node
materialization path:

1. Deserialize the delta into `(uuid_hash, tribe_id, epoch, eav)`.
2. Fold the new EAV triples onto the particle's **existing** materialized
   state (O(1) — apply just this delta, never re-derive from history).
3. Write the result as an immutable, versioned Data File:
   `{data_dir}/tribe-{tribe_id:04x}/particles/{uuid_hash:08x}/v{epoch:010}.dat`,
   atomically (write-to-temp + rename).
4. Update Index 1 (`IdentityIndex`) with the real file location of this
   latest version — replacing the current `FileOffset=0` placeholder.
5. Update Index 7 (`SnapshotIndex`, reused as the per-particle version
   index) with `(uuid_hash, epoch) → file location`. Every applied
   replication event is a version entry; the read node never needs a
   separate scheduled snapshot job to stay bounded.

The read node's `last_seq` (from `ReplicationConsumer::stats()`) is its
replication checkpoint — equivalent to a Postgres replica's replay LSN.

### Decision 5 — Read-node reads resolve through indexes, never through scans

- **Current state**: `IdentityIndex::lookup(uuid_hash)` → O(1) → read the
  latest Data File directly.
- **State at epoch T**: `SnapshotIndex::latest_before(uuid_hash, T)` →
  O(log k) → read that one Data File directly.
- `StoryEngine::project()` / `project_at()`, as used on the read node,
  are rewritten against this path and never call
  `Journal::read_particle_history()` — because the read node has no
  Journal to call it on.

### Decision 6 — Read-node crash recovery validates Data Files and resumes the replication stream; it never replays

1. Enumerate Data Files on disk; verify checksums; discard any partial
   write (prior version file is untouched and becomes current again).
2. Rebuild Index 1 and Index 7 from the surviving Data Files' own
   headers.
3. Resume `ReplicationConsumer` from its last durably-recorded `last_seq`
   — the Broker's `delta.enkwal` is re-read from that offset forward.
   This is a bounded catch-up proportional to events missed during the
   outage, not a full-journal replay, and it is the read node's *only*
   source of "recent history" — it never asks the write node for
   anything else.

### Decision 7 — CSR-03's connection audit journal is explicitly out of scope

`enkidb-con-engine::NaruJournal` (the connection-security audit trail
required by CSR-03) is unaffected by this ADR — it is a bounded
connection-event log, not particle storage, and does not scale with
particle count.

### Decision 8 — No full scan of particle orbits on the read node, ever, under any circumstance

**No code path on the read node may determine a particle's state, or any
query result over particles, by iterating over more entries than the
query's own selectivity requires.** A query that needs one particle
touches one Data File. A query that needs a range touches the indexed
range. `heptascript::indexed`'s already-isolated non-indexable fallback
remains the sole, visibly-flagged exception — never the default path for
a current-state read.

---

## W5H2

| W | Answer |
|---|---|
| **Who** | Write Node (`enkidb-write`, 192.168.122.101) and Read Node (`enkidb-read`, 192.168.122.107) of the 2-node CQRS EnkiDB deployment |
| **What** | Write node keeps its Journal unchanged. Read node holds zero Journal and serves only from Index 1/Index 7-addressed Data Files. `enkidb-replication`'s ENKWAL pipeline (Emitter → Broker → Consumer) is the sole, already-built channel between them |
| **When** | From this ADR onward. Write-node code requires no change. Read-node migration (materialization path + index wiring + replication wiring) is tracked as real playbooks (PB-174 onward), run on the actual machines, not a single blind rewrite |
| **Where** | Write side: `crates/enkidb-engine` (emit call at commit). Unchanged: `crates/enkidb-replication` (Emitter/Broker/Consumer already complete). Read side (new): `crates/enkidb-indexes` (Index 1/7 wiring), `crates/story-engine` (index-based projection), a new read-node materialization module, `crates/enkidb-recovery` (validate + resume, not replay) |
| **Why** | The write node has no stakeholder interface, so its existing replay-capable journal costs nothing. The read node is the only interface stakeholders touch and must answer over 1B+ particles in under 1 second — full scan there is a sovereign-severity failure mode, not a tuning problem |
| **How** | Write node emits a sealed ENKWAL frame per commit alongside its existing journal write. Broker verifies and forwards, unchanged. Read node's `apply_fn` materializes each delta directly into a versioned Data File and updates Index 1/Index 7 in the same step — O(1) per event, no replay ever, including at read-node crash recovery |
| **How Much** | Zero calls to `Journal::read_particle_history()`/`all_particles()` anywhere on the read node once migration completes. Read-node Index 1 offset is never `0` after a particle is first applied. Read-node recovery cost is proportional to files on disk plus the replication gap, never to total historical event volume |

---

## Consequences

### Positive

- Read-node query cost for current state is O(1); for state at a given
  epoch it is O(log k) where k = versions for that one particle — never a
  function of total system size.
- Read-node crash recovery cost is proportional to files on disk plus the
  replication gap since the outage began — never total historical event
  volume.
- The write node is untouched and loses nothing — its journal remains the
  durable command log it was always designed to be, and this ADR adds
  exactly one call (`emit`) to its commit path.
- Tamper-evidence between the two nodes is *stronger* than the read path
  it replaces: every delta the read node materializes is chain-hashed and
  Ed25519-sealed before it ever touches a Data File.

### Constraints introduced

- The read node is now only as fresh as its replication checkpoint. This
  is expected, standard, asynchronous-replica behavior (matching Postgres
  hot-standby semantics) — not a regression, since the read node never
  claimed synchronous consistency with the write node before this ADR
  either.
- Storage cost per applied delta on the read node is one full
  materialized-state version file, not a small delta record. Cold-tier
  compaction of superseded versions is an optimization, not a correctness
  requirement, and does not reintroduce replay.
- Every read-node code path currently calling
  `Journal::read_particle_history`/`all_particles` must be migrated to
  the index-resolved Data File path before this ADR is fully in force.
  Tracked crate by crate via real playbooks.

---

## Relationship to Previous ADRs

| ADR | Interaction |
|---|---|
| **ADR-006** (No DELETE + Mandatory Partitioning) | Unchanged and reinforced on both nodes — the write node's Journal and the read node's versioned Data Files are both append-only, immutable records; only the read node's physical mechanism (file per version) changes |
| **ADR-007** (Mandatory Snapshot Scheduler) | Amended only on the read node's projection algorithm (Decision 6 of ADR-007). The write node's snapshot scheduling, if any, is unaffected. The sovereign guarantee ADR-007 promises — bounded, non-degrading cost — is preserved and strengthened on the read node, which is where it actually needs to hold |
| **ADR-003** (KAKI Sovereignty) | Unchanged — Index 1/Index 7 key on `uuid_hash`, derived from the KAKI itself |

---

## Sovereign Law Statement

> **The Write Node remembers. The Read Node answers. Between them stands
> one sealed channel, and the Read Node trusts nothing it did not receive
> through it. A particle's current state, on the node stakeholders touch,
> is not a story retold from birth on every question — it is a fact,
> written once per change, found once, read once. No query, and no
> recovery, on the Read Node walks the orbit of milliards of particles to
> answer a question about one.**
>
> **This is not negotiable. This is the Law.**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | ADR-012 Accepted 2026-07-07*
