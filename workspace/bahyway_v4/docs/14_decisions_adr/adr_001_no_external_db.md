# ADR-001 — No External Databases

> **DubSar Help** | `ADR > 001` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-01-01"
  concept_depth:  220
  riksu_count:    1
  snapshot_epoch: "2026-06-06"

concept:          "No External Databases"
summary:          "BahyWay stores all data in pure Rust sovereign crates — no PostgreSQL, SQLite, Redis, or any external database runtime."
sovereign_laws:   []

riksu_bindings:
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "Sovereign Storage"
    type: "CHILD"

orbit_tags:       ["OOO Mathematical Foundation"]
rag_keywords:     ["MINT", "sovereign storage", "pure Rust", "no external database"]
-->

## Status: Accepted

## Context

BahyWay must maintain data sovereignty. Third-party databases introduce
dependency on external schema migrations, license constraints, and network
latency between the data physics engine and the persistence layer.

## Decision

EnkiDB is implemented entirely in Pure Rust. No PostgreSQL, SQLite, Redis,
RocksDB, or any other external database engine is used or permitted.

## Consequences

- All indexing, journaling, and replication logic is owned by the BahyWay team.
- The Enlil Algebra indexing strategy (Tri-Kaki, O(1) lookups) must be
  implemented from scratch — no B-Tree library shortcuts.
- Maximum control over particle state lifecycle and KAKI sovereignty.
