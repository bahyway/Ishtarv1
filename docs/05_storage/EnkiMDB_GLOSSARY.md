# EnkiMDB (Euphrates) — Glossary

Terms an engineer needs to read `crates/enkimdb`, its servers, and its
deployment docs without cross-referencing five other files first.
Foundational BahyWay terms are repeated from `EnkiDDB_GLOSSARY.md` (not
just cross-referenced) since this is meant to stand alone.

## Foundational (shared across the Ecosystem)

**KAKI** — the 16-byte sovereign primary key format (`enkidb-kaki`
crate). Every particle, document, and artifact in this Ecosystem is
identified by one.

**Identity-Kaki** — a KAKI of `kaki_type = Identity`: the "birth
certificate" of an artifact profile. One minted per catalogued crate or
playbook.

**Event-Kaki** — a KAKI of `kaki_type = Event`: an immutable record of
one state-transition. Every artifact ingest mints one.

**EAV / Particle** — Entity-Attribute-Value. `enkidb_particles::Particle`
is one EAV row: which entity (Identity-Kaki), which attribute (a
namespaced string like `artifact.name`), what value (an `AkkValue`).

**Journal** — the real, append-only Write-Ahead Log
(`enkidb_journal::Journal`). In-memory only in this crate's current
implementation — see the Roadmap's Phase 1.

**Data Files / materialize** — the Read path's on-disk format (ADR-012).
`materialize_now`/`materialize_version` perform one O(n) pass over a
Journal into two files (`entities`, `eav`) a `ReadNode` opens in O(1).

**CQRS** — Command Query Responsibility Segregation: the Write Node
(Journal) and Read Node (Data Files) are separate processes, connected
only by materialize + sync.

**Epoch** — a monotonically increasing counter tagging when a
`JournalEntry` was written, relative to a single `WriteNode`'s lifetime.

**Generation** — one named, versioned materialization
(`readnode::materialize_version`), e.g. "Euphrates v4.1". Lets multiple
catalog snapshots coexist and be compared, listed via `list_versions`.

## EnkiMDB-specific

**Euphrates** — EnkiMDB's sovereign name (`readnode::SOVEREIGN_NAME`).
Does not rename the crate or type.

**ArtifactProfile** — one catalogued artifact, ready to be minted into
KAKI-sealed particles (`crates/enkimdb/src/artifact.rs`): `name`, `kind`,
`path`, `version` (`None` for playbooks, which have no version).

**ArtifactKind** — `Crate` (a workspace crate with its own `Cargo.toml`)
or `Playbook` (a `.yml` file under `playbooks/`). Deliberately narrow —
"extend this enum only when a new scanner backs it," per its own doc
comment. See `EnkiMDB_ROADMAP.md`'s Phase 3 for the open question of
whether a third kind (documentation) should ever be added here, versus
belonging to EnkiDDB instead.

**scan_crates / scan_playbooks** — the real filesystem scanners
(`crates/enkimdb/src/scan.rs`) that produce `ArtifactProfile`s. Every
result corresponds to an actual file on disk at scan time — no
bootstrap/demo data. `scan_crates` takes a *workspace* root (the
directory containing `crates/`); `scan_playbooks` takes a *repo* root
(the directory containing `playbooks/`) — these are different paths in
this repo's own layout (`workspace/bahyway_v4` vs. the repo root).

**ArtifactEmitter** — mints one Identity-Kaki per artifact and emits its
profile as EAV particles namespaced `artifact.*`
(`crates/enkimdb/src/emitter.rs`).

**WriteNode / ReadNode** — `crates/enkimdb/src/{writenode,readnode}.rs`.
`WriteNode::ingest_artifact` is the real ingestion entry point;
`ReadNode::query` (HeptaScript) is the real read entry point. No search/
RAG index exists — see the Roadmap's "What EnkiMDB is."

**SCAN_CRATES / SCAN_PLAYBOOKS** — `enkimdb-write-server`'s wire
commands triggering a filesystem scan + ingest of everything found,
against a path inside the container (the bind-mounted `/source`).

**FLUSH_EVERY** — `enkimdb-write-server` env var: auto-materialize after
this many ingest calls (default 50; higher than EnkiDDB's default 10,
since one `SCAN_CRATES` call can itself ingest dozens of artifacts).

**RELOAD_SECS** — `enkimdb-read-server` env var: how often the
background thread re-opens `DATA_DIR/current` (default 30 seconds).

**ADR-012** — the architecture decision establishing the materialized-
Data-Files Read Node pattern this crate reuses from EnkiDDB/EnkiDB
rather than inventing a new one.
