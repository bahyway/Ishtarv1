# EnkiMDB (Euphrates) — Implementation Roadmap

**Audience:** engineers implementing or operating EnkiMDB in Dev/Test/Prod.
**Status as of 2026-07-18 (PB-192):** Phase 0 complete, verified, deployed
as Podman containers. This document tracks what's real, what's next, and
what "done" means at each step.

## What EnkiMDB is

EnkiMDB is the second of BahyWay's seven EnkiDB Types — the **Metadata
Database**, sovereign name **Euphrates**. It catalogs BahyWay's *own*
artifacts (workspace crates, Ansible playbooks) as KAKI-sealed EAV
particles: name, kind, path, version. It is not a source-code or
document store — that is EnkiDDB's (Tigris's) job. EnkiMDB answers "what
exists, where, at what version," not "what does this file say."

Crate: `crates/enkimdb`. Servers: `bin/enkimdb-write-server` (TCP,
port 7201), `bin/enkimdb-read-server` (TCP, port 7202).

## Phase 0 — Foundation (COMPLETE, PB-192)

Definition of done, all met:
- [x] `WriteNode`/`ReadNode` CQRS split, identical shape to EnkiDDB's
  (`crates/enkimdb/src/{writenode,readnode}.rs`).
- [x] Real filesystem scanners, no bootstrap/demo data
  (`scan::scan_crates`, `scan::scan_playbooks`) — every
  `ArtifactProfile` returned corresponds to an actual file on disk at
  scan time.
- [x] `ArtifactEmitter` — mints one Identity-Kaki per artifact, emits
  real EAV particles namespaced `artifact.*`.
- [x] Versioned generations (`readnode::materialize_version`,
  `list_versions`) — multiple Euphrates catalog snapshots ("Euphrates
  v4.1", "v4.2", ...) can coexist and be compared. Verified by test:
  a query for an artifact present only in v4.2 correctly returns nothing
  against v4.1.
- [x] `bin/enkimdb-write-server`/`enkimdb-read-server` — TCP servers,
  `SCAN_CRATES`/`SCAN_PLAYBOOKS`/`FLUSH` and `QUERY` respectively.
- [x] `deploy/podman/Containerfile.enkimdb-{write,read}-server`,
  `scripts/enkimdb-sync-data.sh`, `docs/05_storage/EnkiMDB_PODMAN_DEPLOYMENT.md`.
- [x] Real executable Ansible playbook
  (`playbooks/playbook_192_enkiddb_enkimdb_podman_deploy_executable.yml`).
- [x] Smoke-tested as live TCP processes: `SCAN_CRATES` catalogued 151
  real crates, `SCAN_PLAYBOOKS` catalogued 88 real playbooks, `FLUSH`
  materialized 239 entities, and a `QUERY` against the synced Read Node
  returned the real `enkiddb` crate's full catalog row (name, kind,
  path, version) end to end.

## Phase 1 — Durability + deduplicated re-scans (NOT STARTED)

**Problem, part A (durability):** same as EnkiDDB's Phase 1 — the
Write Node's Journal is in-memory only, and Write→Read sync is plain
`rsync`/`scp`. `enkidb-replication` is the same real, unwired fix.

**Problem, part B (deduplication):** every `SCAN_CRATES`/
`SCAN_PLAYBOOKS` call journals every matching artifact again, even
unchanged ones. Harmless today (a `WHERE` query still finds the
artifact; it just accumulates redundant Event-Kakis), but not a true
delta-ingest, and will make the Journal grow unnecessarily on a
scheduled re-scan.

**Definition of done:**
- [ ] `enkidb-replication` wired in (shared decision with EnkiDDB's
  Phase 1 — the key-management/broker topology should very likely be
  the same infrastructure serving both).
- [ ] A re-scan of an unchanged workspace does not grow the catalog
  (either by diffing against the last known state, or by a documented,
  accepted tradeoff if diffing is deferred further).

## Phase 2 — Scheduled catalog refresh (NOT STARTED)

**Problem:** `SCAN_CRATES`/`SCAN_PLAYBOOKS` are triggered manually, once,
per deploy step today (see `EnkiMDB_PODMAN_DEPLOYMENT.md` §5). The
catalog goes stale the moment a new crate or playbook is added and
nobody re-triggers a scan.

**Plan:** a periodic trigger — either a `podman run --rm` sidecar on a
timer calling `SCAN_CRATES`/`SCAN_PLAYBOOKS`/`FLUSH` in sequence, or (if
Phase 1's deduplication lands first) simply running the scan on every
commit via CI, since a no-op re-scan would then be genuinely cheap.

**Definition of done:**
- [ ] A newly-added crate or playbook shows up in a `QUERY` against
  `enkimdb-read` within a bounded, documented time window, with no
  manual step.

## Phase 3 — Populate with production Implementation-phase material (NOT STARTED)

**Problem:** `SCAN_CRATES`/`SCAN_PLAYBOOKS` only catalog `crates/*` and
`playbooks/*.yml` today — the exact scope `scan.rs`'s own doc comment
states ("extend this enum only when a new scanner backs it"). The
Implementation-phase material itself (this roadmap, the manuals, the
glossaries, and any future per-Type roadmap/manual/glossary sets) is not
covered by either scanner.

**Plan:** a decision, not yet made, on whether Roadmap/Manual/Glossary
documents belong in EnkiMDB's catalog at all. The Architect's own framing
— "documentation to EnkiDDB, crate/app code to EnkiMDB" — suggests these
particular documents belong in EnkiDDB (as real, searchable text, via
its Phase 3) rather than EnkiMDB (which only ever records *that*
something exists, never its content). Flagged here rather than silently
building a third scanner that duplicates EnkiDDB's actual job.

**Definition of done:**
- [ ] Explicit Architect decision recorded: EnkiMDB catalogs only
  crates/playbooks (as now), or a defined additional artifact kind is
  added with its own real scanner.

## Phase 4 — Deferred / blocked (tracked, not scheduled)

- **Full-text/source-code search over EnkiMDB's catalogued artifacts**:
  out of scope by design (see "What EnkiMDB is," above) — this would
  duplicate EnkiDDB's `RagIndex`. If ever wanted, the correct path is
  ingesting `.rs`/`.yml` file contents into EnkiDDB, not adding search
  to EnkiMDB.
- **Multi-node / HA**: not scoped, same reasoning as EnkiDDB's Phase 4.

## How this roadmap relates to the other 6 EnkiDB Types

Scoped to EnkiMDB (Euphrates) only, alongside EnkiDDB's (Tigris's)
companion roadmap — the first two of seven EnkiDB Types, per the agreed
build order. The other five get their own roadmap entries once their own
crates/tests exist to describe honestly.
