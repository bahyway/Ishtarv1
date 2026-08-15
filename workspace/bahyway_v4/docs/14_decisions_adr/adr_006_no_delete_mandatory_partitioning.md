# ADR-006 — No DELETE in EnkiDB/EnkiDW + Mandatory Partitioning

> **DubSar Help** | `ADR > 006` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-06-05"
  concept_depth:  240
  riksu_count:    3
  snapshot_epoch: "2026-06-06"

concept:          "No DELETE and Mandatory Partitioning"
summary:          "DELETE does not exist in EnkiDB — state is superseded by INSERT of a new Event-Kaki; four mandatory partition axes enforced."
sovereign_laws:   ["§3.1 — DML is INSERT and READ only", "§9.1 — four canonical partition axes"]

riksu_bindings:
  - target: "adr_007_mandatory_snapshot_scheduler.md"
    concept: "Three-Pillar Sovereign Data Law"
    type: "PEER"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI partition axes"
    type: "PEER"
  - target: "enkidb.md"
    concept: "Journal INSERT"
    type: "GROUNDS"

orbit_tags:       ["Sovereign Storage", "KAKI Sovereignty"]
rag_keywords:     ["APPEND", "TRACE", "no DELETE", "mandatory partitioning", "Event-Kaki", "Journal"]
-->

## Status: Accepted

---

## Context

### The DELETE Problem

Every major relational database supports a DELETE command. This creates a
fundamental security and auditability flaw that no amount of audit logging
can fully resolve:

- An administrator with sufficient privilege can delete data **and** delete
  the audit trail that recorded the deletion.
- A malicious insider can suppress evidence of fraud, error, or tampering
  by removing the records that prove it existed.
- "Soft delete" patterns (an `is_deleted` flag) are conventions — they can
  be bypassed by any developer with direct database access.
- Even with triggers and audit tables, the audit mechanism is **separate**
  from the data it audits. Separate means bypassable.

In the BahyWay sovereign model, every data particle has an immutable KAKI
identity and an append-only LineageChain. A DELETE command is
**architecturally incompatible** with these guarantees — because deletion
would sever a KAKI particle from its lineage, producing an orphaned chain
segment that proves tampering occurred but cannot reconstruct what was lost.

### The Growth Problem

If data is never deleted, it grows eternally. Without a structural mechanism
to manage this growth, query performance degrades, storage costs escalate,
and the operational burden of managing billions of records becomes
unsustainable.

The answer is not DELETE. The answer is **Partitioning** — not as an
optional performance optimisation, but as a mandatory sovereign structural
layer of every EnkiDB and EnkiDW instance.

Partitioning and No-DELETE are therefore two sides of the same sovereign law:
- **No DELETE** guarantees that no particle ever disappears.
- **Mandatory Partitioning** guarantees that eternal data remains queryable,
  manageable, and economically viable.

---

## Decision

### Decision 1 — No DELETE Command

**The DELETE command is not implemented in EnkiDB or EnkiDW.**

This is not a restriction, not a permission setting, not a configurable
policy. It is an absence at the grammar level of the data language.

The canonical operations in EnkiDB and EnkiDW are:

| Operation | Status | Meaning |
|---|---|---|
| `INSERT` | **Always permitted** | A particle is born — KAKI assigned, LineageChain starts; or a new Event-Kaki supersedes prior state |
| `READ / PROJECT` | **Always permitted** | StoryEngine replays the Journal to project current state |
| `UPDATE` (SQL sense) | **Does not exist** | Replaced by INSERT of a superseding Event-Kaki — the prior Event-Kaki remains permanently in the Journal |
| `DELETE` | **Does not exist** | Has no grammar entry — the concept of erasure was never built into the system |

What appears in other systems as UPDATE is mechanically, in EnkiDB, an INSERT
of a new Event-Kaki whose epoch supersedes the prior one. The prior Event-Kaki
is never modified, never removed — it remains in the Journal forever, proving
what the state was before the change and who changed it.

To make a record invisible to application queries, a new Event-Kaki is INSERTed
setting the state attribute to SUSPENDED. To re-activate it, a further INSERT
sets state to ACTIVE. In both cases, the original particle and its complete
history remain permanently in the system.

An internal attacker attempting to erase evidence cannot find a locked door —
they find that the concept of erasure **was never built into the system**.
Every action they take to hide data instead creates a new permanent record
proving that they acted, when they acted, and what state the data was in
before they touched it.

### Decision 2 — Partitioning Is Mandatory, Not Optional

Every EnkiDB and EnkiDW database instance **must** declare a partitioning
strategy at creation time. Unpartitioned instances are not permitted in
production.

Partitioning in EnkiDB/EnkiDW is sovereign — it does not use external
partitioning libraries. The four mandatory partition axes are geometrically
embedded in the KAKI byte layout itself (§9.1):

| Partition Axis | KAKI Bytes | Key | Purpose |
|---|---|---|---|
| **By Tribe** | `κ[4..5]` `tribe_id` | u16 tribe identifier | Cross-domain isolation — a query for Domain A never scans Domain B partitions |
| **By KAKI Hash** | `κ[0..3]` `uuid_hash` | u32 hash prefix | Distributes load across shards; prevents hot-partition skew in high-volume tribes |
| **By Time** | `κ[12..13]` `timestamp` | u16 sovereign epoch | Time-range queries without full-scan; epoch boundaries are coarse partition keys |
| **By State** | EAV `state` attribute | hot / warm / cold | Routes particles to correct storage tier — active particles stay hot, dead particles move to cold |

The partition axes are not metadata attached to a particle — they are the KAKI
bytes themselves. The particle **is** its own partition key. This means:
- Partition routing requires zero extra index lookups
- Partition boundaries are provably stable — they are determined at KAKI
  creation time and cannot drift as the particle's state evolves
- An attacker cannot forge a KAKI that routes to a different partition without
  invalidating the checksum at `κ[14..15]`

Partitioning serves the following sovereign purposes:

1. **Query isolation** — active, healthy particles are queried without
   scanning the full eternal archive.
2. **Storage tiering** — DEAD and QUARANTINE lane partitions can be moved
   to cold storage without deleting them.
3. **Regulatory compliance** — GDPR right-to-erasure is satisfied by
   moving a subject's partitions to a SUSPENDED state — data becomes
   inaccessible to processing while the lineage remains intact for audit.
4. **Performance at eternal scale** — without partitioning, a system that
   never deletes will degrade. With mandatory partitioning, query plans
   target only the relevant epoch and lane slices.
5. **Security boundary enforcement** — partition boundaries map to istar
   domain rules. A user with access to Domain A's partitions cannot
   accidentally or maliciously reach Domain B's partitions.

---

## W5H2

| W | Answer |
|---|---|
| **Who** | Every developer, DBA, and data steward working with EnkiDB or EnkiDW |
| **What** | DELETE is not a command in EnkiDB/EnkiDW. Partitioning by Epoch, Lane, Domain, and State is mandatory at instance creation |
| **When** | From EnkiDB v4.0 onwards — eternal, never revised |
| **Where** | `crates/enkidb-engine`, `crates/enkidb-storage`, `crates/enkidb-dw`, `crates/enkidb-qdb` (Quarantine), `crates/permanent-storage` |
| **Why** | DELETE is incompatible with KAKI immutable identity and append-only LineageChain. Mandatory partitioning is the sovereign answer to eternal data growth — it replaces deletion as a data lifecycle tool |
| **How** | State transitions via INSERT of superseding Event-Kaki replace deletion. Partitioning by sovereign KAKI-embedded axes (Tribe κ[4..5], Hash κ[0..3], Time κ[12..13], State EAV) routes particles to appropriate storage tiers without removing them |
| **How Much** | Zero DELETE commands in the entire codebase. Zero UPDATE (SQL sense) commands. Every EnkiDB/EnkiDW instance declares 4 mandatory partition axes embedded in the KAKI byte layout at creation |

---

## Consequences

### Positive

**Security:**
- An internal attacker has no mechanism to erase data or evidence — the
  grammar of the system prevents it, not a policy that can be overridden.
- Every state change by any actor — including administrators — produces a
  new permanent LineageHop proving who acted and when.
- Audit trails are inseparable from data because the lineage IS the record
  structure, not a separate table.

**Compliance:**
- GDPR Article 17 (Right to Erasure) is satisfied via state change to
  SUSPENDED — data is inaccessible to processing without physical deletion.
- Financial regulations (Basel III, MiFID II, DAMA-DMBOK) requiring
  7–10 year data retention are satisfied by design — nothing is ever lost.
- Regulators auditing the system find a complete, unbroken record of every
  particle from birth to present state.

**Architecture:**
- The LineageChain guarantee holds absolutely — no chain is ever orphaned
  by a deletion event.
- KAKI particle identity is provably eternal — once assigned, a KAKI PK
  references a particle that will always be findable in the system.
- Partitioning provides O(1) partition-level routing, keeping query
  performance stable as the total data volume grows without bound.

### Constraints Introduced

- **Developers must think in Events, not mutations.** Any feature request
  that says "delete this record" must be expressed as an INSERT of a new
  Event-Kaki transitioning state to SUSPENDED. Any "update this field" must
  be expressed as an INSERT of a new Event-Kaki superseding the prior value.
  This is a design discipline, not a limitation.
- **Storage grows eternally.** Cold storage tiering for DEAD and QUARANTINE
  partitions must be planned for every deployment. The sovereign answer to
  storage cost is tiering, not deletion.
- **GDPR erasure requests require a defined procedure.** The procedure is:
  INSERT a new Event-Kaki setting the subject's personal attribute fields to
  a cryptographic tombstone value, INSERT another Event-Kaki transitioning
  state to SUSPENDED, partition moves to cold tier. The KAKI identity record
  — which contains no personal data — is retained. The tombstone Event-Kakis
  are themselves permanent evidence that erasure was performed.

---

## State Lifecycle — The Sovereign Replacement for DELETE

```
Particle BORN
    │ INSERT Event-Kaki → KAKI assigned, LineageChain starts
    ▼
State: ACTIVE          ← visible to all queries
    │ INSERT superseding Event-Kaki (state → SUSPENDED)
    ▼
State: SUSPENDED       ← invisible to application queries
    │                     prior Event-Kakis untouched, GDPR-compliant
    │ INSERT superseding Event-Kaki (state → ACTIVE)
    ▼
State: ACTIVE          ← re-activation Event-Kaki in Journal
    │ INSERT superseding Event-Kaki (quality degrades, B11 < 60)
    ▼
State: DEAD            ← B11 < 60, routed to enkidb-qdb (Quarantine)
    │                     permanent archive, never processed again
    │                     but eternally readable for audit
    ▼
Quarantine Partition   ← cold storage, lineage intact, KAKI valid
```

No step in this lifecycle requires DELETE.
No step in this lifecycle is invisible.
Every step produces a permanent record of who triggered it and when.

---

## Relationship to Previous ADRs

| ADR | Interaction |
|---|---|
| **ADR-001** (No External DB) | EnkiDB's No-DELETE grammar is only possible because we own the entire storage engine. PostgreSQL and SQLite cannot remove DELETE from their grammars. |
| **ADR-003** (KAKI Sovereignty) | KAKI particles are eternal by this ADR — their identity persists in the system after any state change, forever. This ADR is the storage guarantee of ADR-003's identity promise. |
| **ADR-004** (BeeMDM 4-lane pipeline) | The four lanes (GEM/TRIBE/ACTIVE/FUZZY/DEAD) map directly to the mandatory partition Quality Lane dimension. Lane assignment IS partition assignment. |
| **ADR-005** (Enterprise Data Fabric) | `bahyway-fabric` LineageChain append-only guarantee is enforced by this ADR — no Fabric stage can DELETE a record it has already logged in the chain. |

---

## Sovereign Law Statement

> **In EnkiDB and EnkiDW, a particle born into the system remains in the
> system eternally. Its state may change. Its quality lane may degrade. Its
> partition may move to cold storage. But its KAKI identity, its birth
> record, and its complete LineageChain are permanent facts of the sovereign
> universe — as immutable as the laws of physics that govern the particle
> orbits above them.**
>
> **Partitioning is not a performance feature. It is the sovereign
> infrastructure that makes eternal data economically and operationally
> viable. Every EnkiDB and EnkiDW instance is partitioned. This is not
> negotiable.**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | ADR-006 Accepted 2026-06-05*
