# ADR-022 — Client Tenancy for EnkiDDB/EnkiMDB (DRAFT, unsealed)

**Status:** DRAFT — awaiting Architect ratification (CSR-08). Not a sealed
decision. Written from an incomplete record (see "Honest gap" below) —
correct or replace freely rather than treating this as settled.
**Date:** 2026-08-01
**Author:** Claude (drafting on the Architect's instruction), NOT the
Architect — this is a proposal to ratify or reject, same convention as
NL-001's own "DRAFT — awaiting AkkadianSeal" status.

## Honest gap

Earlier in this session the Architect asked (paraphrased from a
compacted summary, exact original wording not recoverable): *"Yesterday
I told you to look at the 7 Types EnkiDB databases as 2 Internal and 5
External — well they are not exactly that... EnkiDDB/EnkiMDB need
client-facing clones too."* — invoking a SQL-Server `master`/`msdb`
analogy. The full original reasoning behind that framing was lost to
context compaction before it could be written down. This ADR
reconstructs a coherent proposal from the surviving fragment plus real,
current architecture — it is a best-effort draft, not a transcript of
what was actually said. **The Architect should correct this on any point
where it doesn't match intent, not assume it's already right.**

## The real 7 Types, for reference

Per `docs/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md`: EnkiDB (7001), EnkiDW (7002),
EnkiSDB (7003), EnkiODB (7004), EnkiQDB (7005), EnkiMDB (7006), EnkiDDB
(7007) — each a CQRS Write/Read pair. This session's own real work
confirms EnkiDDB holds documentation (Markdown/playbook-doc corpus,
`enkiddb::DOCS_TRIBE_ID`) and EnkiMDB holds the crate/playbook catalog
(`SCAN_CRATES`/`SCAN_PLAYBOOKS`, real EAV `artifact.*` schema, see
PB-216).

## The reconsideration

"2 Internal (EnkiDDB, EnkiMDB) + 5 External (the rest)" is incomplete
because it treats EnkiDDB/EnkiMDB as *purely* ecosystem-internal — but a
client engagement plainly needs its own documentation corpus and its own
crate/playbook catalog too (their own PBs, their own docs, their own
artifact inventory), which are exactly what EnkiDDB/EnkiMDB hold. So
EnkiDDB/EnkiMDB can't be "internal-only" — they need a client-facing
form as much as the other 5 types do.

## Proposed reconciliation: `master`/`msdb` is about the INSTANCE, not the type

In SQL Server, every instance ships one `master` (server-wide identity/
config) and one `msdb` (job/schedule/history) database — permanently
internal to that instance, never a user database — while actual client
data lives in separate, per-tenant user databases on the same or a
different instance. The analogy that seems to fit what the Architect
described:

- **One ecosystem-internal EnkiDDB+EnkiMDB instance** (today's real
  `enkidb-node-write`/`enkidb-node-read` pair) holds BahyWay.Ecosystem's
  own sealed documentation and its own crate/playbook catalog — the
  `master`/`msdb` of the whole ecosystem. Never client-facing, Architect-
  controlled, exactly what's running today.
- **Every client gets their own separate EnkiDDB+EnkiMDB instance**
  (their own CQRS Write/Read pair, in their own OTAP), holding *their*
  documentation and *their* crate/playbook catalog — same shape as the
  other 5 types already get per client engagement, not a shared
  multi-tenant slice of the ecosystem-internal instance.

Under this reading, "2 Internal + 5 External" was never really about
*type* — it's that EnkiDDB/EnkiMDB uniquely also need one **permanent,
non-clonable ecosystem-internal instance**, in addition to the per-client
instance every type (including EnkiDDB/EnkiMDB) gets. Not 2-vs-5; all 7
are client-facing per-tenant, and 2 of them additionally have a singular
internal instance the other 5 don't need.

## Where this connects to real work already done today

The `ARCHITECT_DOCS_TRIBE_ID`/`DOCS_TRIBE_ID` split built today
(`crates/enkiddb/src/lib.rs`, threaded through `uruinimgina-cli`) solves
a **narrower, different** problem: separating the Architect's own
personal corpus from the sealed ecosystem corpus *within the same
ecosystem-internal instance*. That tribe-level split is the right tool
for two corpora sharing one trust boundary (both are the Architect's).
**It is explicitly NOT the mechanism proposed for client tenancy** — a
client is a different trust boundary entirely (see ADR-021's own tribe-
isolation law, CSR-07), so client separation should be instance-level
(their own CQRS pair, per this ADR's proposal), not tribe-level inside
the ecosystem-internal instance. Using a tribe-id split for clients
instead of a separate instance would put client data in the same
Write/Read containers as BahyWay's own sealed documentation — a real
isolation regression, not a shortcut.

## Open questions for the Architect (not decided here)

1. Does a client's own EnkiDDB+EnkiMDB instance run on infrastructure
   the client hosts, BahyWay hosts on their behalf, or either — same
   question as Q3 (Sargon Passport / Tenant Operator, see the
   companion work in this session)?
2. Is there ever a need to *aggregate* read access across all clients'
   EnkiDDB/EnkiMDB instances back into the ecosystem-internal one (e.g.
   for cross-client analytics), or must that boundary stay hard?
3. Does this same per-client-instance model apply identically to the
   other 5 types, or do any of them have their own internal-only
   exception the way EnkiDDB/EnkiMDB do?

## Status

Unsealed. Ratify, amend, or reject via a direct instruction — this
document makes no claim to be the Architect's actual intent, only a
grounded starting point.
