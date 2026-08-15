# Tri-Kaki Enlil Index

> **DubSar Help** | `Math > Tri-Kaki Index` | Mathematics

## Purpose

The Tri-Kaki Enlil Index replaces all B-Tree and hash-table indexes in EnkiDB
with three algebraically direct lookup strategies, each O(1).

## The Three Strategies

### 1 — Identity-KAKI Index (Shamash Primary)
- Maps Identity-KAKI → Jordan Block memory address via spectral hash.
- Combined with Bloom Filter (ADAD Gate), the system knows instantly whether
  the particle exists and which Tribe it belongs to.
- Complexity: O(1).

### 2 — Events-KAKI Index (ADAD Temporal)
- Tracks only the tail of the Jordan Chain (most recent event).
- New Event-KAKI appends to the LRU tail; indexing = one pointer update.
- Old events fall off via nilpotency — no cleanup job needed.
- Complexity: O(1) append.

### 3 — CrossTribe-KAKI Index (ANU Relational)
- Stores basis-transformation matrices P and P⁻¹ between Tribe Jordan Blocks.
- Relationship traversal = one matrix multiplication (SIMD-accelerated).
- Materialised only on PROBE (§8.3), never stored permanently.
- Complexity: O(1) matrix multiply.

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
No full-table scans. No secondary indexes consuming disk space.

## See Also

- `01_mathematics/enlil_algebra.md`
- `02_identity/kaki_triad.md`
- `05_storage/enkidb.md`
