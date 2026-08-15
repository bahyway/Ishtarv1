# Nabu CLI Reference

> **DubSar Help** | `nabu` | Tooling

## Purpose

Nabu is the command-line interface for BahyWay. It translates HeptaScript
queries into Jordan Observable matrices and dispatches them to EnkiDB.

## Key Commands

```
nabu probe  <kaki>          Query a single particle by KAKI.
nabu tribe  <tribe_id>      List particles in a Tribe with HPS scores.
nabu ingest <source>        Submit a new signal through all four Pauli Gates.
nabu audit  <kaki>          Show the full Jordan Chain (storytelling journal).
nabu zoom   <tribe_id> <k>  Render shell k of a Tribe in the Observatory.
```

## Sovereign Constraints

`nabu probe` triggers CrossTribe-KAKI computation on demand (§8.3). The result
is displayed but never written back to EnkiDB.

## See Also

- `05_storage/enkidb.md`
- `01_mathematics/tri_kaki_index.md`
