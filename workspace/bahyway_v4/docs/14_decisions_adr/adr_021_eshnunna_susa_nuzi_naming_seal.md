# ADR-021 — Eshnunna, Susa, and Nuzi: Naming Seal

**Status:** Accepted
**Date:** 2026-08-01
**Author:** DUB.SAR -- Bahaa Fadam

## Context

Three city names surfaced this session, each carrying a real, distinct
architectural role diagnosed across earlier daily-work sessions
(`docs/__DialyWorks/Tus20260706`, referenced from the Architect's own saved
transcripts) but never sealed in `naming-registry` or built as live code
until now:

1. **Eshnunna** -- named for its personal significance (its remains sit in
   the Architect's own neighborhood) and its historical fame: the Laws of
   Eshnunna, one of the earliest codified legal systems, predating
   Hammurabi by roughly two centuries. Diagnosed as the fix for a real,
   proven performance bug: `enkidb-indexes::SurrogateMap` prunes queries
   to a small surrogate set in O(log N), but until this ADR there was no
   columnar data file to fetch the actual particle *values* from by that
   surrogate -- retrieval fell back to an O(n) journal walk, which is why
   HeptaScript queries degraded from ~5 seconds at 100 particles to hours
   at 10,000. `crates/eshnunna-engine` closes that gap: surrogate u32 ->
   fixed byte offset -> mmap'd column value, one read, no scan.

2. **Susa** -- per NL-001 (unsealed draft), "outward gateway." This is not
   a new concept to name: it is the existing, already-built, already-tested
   `crates/susa-engine` ("the sovereign import THRESHOLD... the guardian of
   the boundary," 9/9 tests passing, zero dependents yet), which was never
   formally entered into `naming-registry`. This ADR closes that gap by
   registration only -- no code change.

3. **Nuzi** -- per NL-001 (unsealed draft), "inward archive." Famous in the
   historical record for thousands of Hurrian-period family, legal, and
   administrative tablets (adoptions, land transfers, loans, inheritance)
   -- individually retrieved records, not a bulk repository. Reserved here
   as the **retrieval/query side** of the client-document archive: looking
   up one record and proving its provenance/lineage. Paired with
   `SipparStore` (already reserved, 2026-08-01, for the Ebabbar
   temple-library association -- a bulk accumulated repository), which is
   correspondingly narrowed to the **ingest/write side**: where documents
   land and get persisted. Same shape as the CQRS split already used by
   every one of the 7 Types EnkiDB (Write=Journal, Read=Datafiles).

## Decision

- **Eshnunna** is sealed `SealedByLaw`, `system_role: Engine`,
  `crate_path: crates/eshnunna-engine/src/lib.rs`, citing this ADR.
- **Susa** is sealed `SealedByLaw`, `system_role: Engine`,
  `crate_path: crates/susa-engine/src/lib.rs`, citing this ADR (the code
  predates this ADR; the ADR formalizes the registry entry).
- **Nuzi** is `Reserved`, `system_role: Suite`, no `crate_path` yet
  (nothing built for the retrieval side), paired explicitly with
  `SipparStore` via `domain_tags`.
- `SipparStore`'s existing entry is narrowed in its blurb from "the
  client-document archive" to specifically "the ingest/write side" of it,
  to remove the overlap with Nuzi's now-sealed retrieval-side role.

## Consequences

- No collision between Nuzi and Sippar: they name the two sides of one
  CQRS pair, not the same slot twice.
- Susa entering the registry closes an open question from earlier this
  session ("does a new name replace SusaEngine for client-document
  ingestion?") -- no new name is needed; `SusaEngine` already is that
  engine.
- Eshnunna's retrieval fix (`crates/eshnunna-engine`) is sealed but not
  yet wired into the live read path (`enkidb-indexes` / the Read Node) --
  that wiring is follow-up work, tracked separately, not claimed as done
  by this ADR.
