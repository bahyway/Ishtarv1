# EnkiDB — Hot Plane Storage

> **DubSar Help** | `EnkiDB` | Storage

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-05"
  concept_type:   "0x03"
  epoch:          "2026-01-01"
  concept_depth:  215
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "EnkiDB Hot Plane Storage"
summary:          "EnkiDB is the sovereign hot-plane storage layer — pure Rust Journal of INSERT-only Event-Kakis, partitioned across four KAKI axes."
sovereign_laws:   []

riksu_bindings:
  - target: "adr_006_no_delete_mandatory_partitioning.md"
    concept: "no DELETE"
    type: "CHILD"
  - target: "adr_007_mandatory_snapshot_scheduler.md"
    concept: "Index 7 snapshot B-tree"
    type: "CHILD"

orbit_tags:       ["Sovereign Storage", "KAKI Sovereignty"]
rag_keywords:     ["APPEND", "PROJECT", "TRACE", "Journal", "hot plane", "Event-Kaki", "partition", "Index 7"]
-->

## Purpose

EnkiDB is the sovereign operational memory of BahyWay. It stores particles as
Entity-Attribute-Value (EAV) triples in a block-diagonal Jordan Block layout.
There are no tables, no rows, no foreign keys.

## Mechanism

- **Hot plane**: particles in the Active and Stewardship states live in the
  hot plane (VRAM or high-speed memory).
- **Tri-Kaki indexing**: Identity-KAKI → O(1) block pointer; Events-KAKI → O(1)
  tail append; CrossTribe-KAKI → O(1) basis-transform on PROBE.
- **GPU resonance query**: Vulkan compute shader applies the Jordan Observable
  matrix across all particles in one pass — no sequential scan.
- **No third-party databases**: EnkiDB is implemented entirely in Pure Rust.

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§8.3: CrossTribe state computed on PROBE, never stored.
No PostgreSQL, SQLite, or any external database engine.

## See Also

- `05_storage/enkidw.md`
- `05_storage/enkistream.md`
- `01_mathematics/enlil_algebra.md`
