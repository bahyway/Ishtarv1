# EnkiDDB (Tigris) — Glossary

Terms an engineer needs to read `crates/enkiddb`, its servers, and its
deployment docs without cross-referencing five other files first.
Foundational BahyWay terms are included (not just EnkiDDB-specific ones)
since this is meant to stand alone.

## Foundational (shared across the Ecosystem)

**KAKI** — the 16-byte sovereign primary key format (`enkidb-kaki`
crate). Every particle, document, and artifact in this Ecosystem is
identified by one. Structure: κ[0..4] minted_id, κ[4..6] tribe_id,
κ[6] kaki_type, κ[7] kaki_role, κ[12..14] timestamp, κ[14..16] checksum.

**Identity-Kaki** — a KAKI of `kaki_type = Identity` (0x01): the "birth
certificate" of a document or particle. `IdentityKaki` in code.

**Event-Kaki** — a KAKI of `kaki_type = Event` (0x02): an immutable
record of one state-transition. Every document ingest in EnkiDDB mints
one.

**EAV / Particle** — Entity-Attribute-Value. `enkidb_particles::Particle`
is one EAV row: which entity (Identity-Kaki), which attribute (a
namespaced string like `meta.title`), what value (an `AkkValue`).

**Journal** — the real, append-only Write-Ahead Log
(`enkidb_journal::Journal`). Holds every `JournalEntry` (an Event-Kaki
plus its EAV triples) since the process started. In-memory only in this
crate's current implementation — see the Roadmap's Phase 1.

**Data Files / materialize** — the Read path's on-disk format
(`enkidb-datafile`, `enkidb-readnode`, per ADR-012). `materialize_now`
performs one O(n) pass over a Journal, writing two files (`entities`,
`eav`) that a `ReadNode` can then open in O(1) without ever touching the
Journal. A full wipe-and-rebuild each time, not an incremental append —
`DataFileWriter` never truncates.

**CQRS** — Command Query Responsibility Segregation: the Write Node
(Journal, mutation-adjacent) and Read Node (Data Files, query-only) are
separate processes with no shared mutable state, connected only by
materialize + sync.

**Epoch** — a monotonically increasing counter tagging when a
`JournalEntry` was written, relative to a single `WriteNode`'s lifetime.
Not a wall-clock timestamp.

**Generation** — one named, versioned materialization
(`readnode::materialize_version`), e.g. "Tigris v4.1". Lets multiple
snapshots of the catalog coexist and be compared, listed via
`list_versions`.

## EnkiDDB-specific

**Tigris** — EnkiDDB's sovereign name (`readnode::SOVEREIGN_NAME`). Does
not rename the crate or type — same relationship "GeoEngine" has to
`bahyway-algebra`.

**DocOrbit** — the six-way structural taxonomy every document particle
is tagged with (`crates/enkiddb/src/orbit.rs`): `Meta` (title/language/
format), `Head` (h1-h6 headers), `Body` (paragraphs/lists/blockquotes),
`Code` (code blocks), `Hist` (version history), `Link` (cross-references).
Orthogonal to `meta.collection`.

**meta.collection** — the document-KIND taxonomy (distinct from
`DocOrbit`): `component`, `glossary`, `architecture-reference`,
`playbook-record`, `concept-law`, `general`, or a `docs/<subfolder>/`
name. Assigned automatically by `infer_collection`, or supplied
explicitly over the wire protocol — see `EnkiDDB_MANUAL.md` §2, §5.

**Section** — one chunk of a parsed document (`document::Section`),
grouped at header boundaries so nothing is orphaned. What `RagIndex`
actually indexes and searches — not whole documents.

**RagIndex** — the search index (`crates/enkiddb/src/rag.rs`) built over
a Read Node's sections. `RagIndex::build_from_readnode` is the CQRS-
correct construction path (reads only materialized Data Files, never a
Journal). `RagHit` is one search result: KAKI, relevance score, section
reference.

**WriteNode / ReadNode** — `crates/enkiddb/src/{writenode,readnode}.rs`.
The two halves of EnkiDDB's CQRS split. `WriteNode::ingest_document`/
`ingest_document_categorized`/`ingest_directory_categorized` are the real
ingestion entry points; `ReadNode::query`/`RagIndex::search` are the real
read entry points.

**FLUSH_EVERY** — `enkiddb-write-server` env var: auto-materialize after
this many ingested documents (default 10). Also triggerable on demand
via the `FLUSH` wire command.

**RELOAD_SECS** — `enkiddb-read-server` env var: how often the
background thread re-opens `DATA_DIR/current` and rebuilds the RAG index
(default 30 seconds).

**ADR-012** — the architecture decision establishing the materialized-
Data-Files Read Node pattern this crate reuses rather than inventing a
new one. Referenced throughout `enkidb-readnode`'s own doc comments.

**TeamAllowlist** — the authorship gate's set of trusted git author
emails (`crates/enkiddb/src/authorship.rs`). A directory ingest checks
every file's `git log -1 --format=%ae` against it and fails closed
(rejects the whole batch) on an unauthorized or undeterminable author.
Loaded from `--team-allowlist <file>` / `$ENKIDDB_TEAM_ALLOWLIST`, or a
built-in seed of the Architect's own real git identities. Deliberately
excludes this agent's own commit identity (`noreply@anthropic.com`) by
default — see the Roadmap's Phase 3 for what that means for ingesting
this repo's own `docs/` tree today.

**Musarû (`musaru-security`)** — the real security-check crate behind
this ecosystem's "Nergal" visualization panel (the panel itself has no
scan logic of its own — see `dubsar-visualizer::panels::nergal`). Its
`zip_scan` module does pure byte-signature pattern matching, applied by
EnkiDDB (PB-206, `crates/enkiddb/src/security.rs::scan_document`) to
every document's raw bytes before parsing — a second, independent gate
alongside `TeamAllowlist`. A hit responds `ERR:security: <detail>` and
nothing is journaled.

**`enkiddb-cli --remote`** — the VDI-side ingestion mode (PB-205):
`enkiddb-cli ingest <dir> --remote <host:port>` runs SCAN/CATEGORIZE
locally, then sends each authorized document over the wire to a live
`enkiddb-write-server`, instead of journaling into a disposable local
`WriteNode`. The recommended production path — see Manual §2a.

**`enkiddb-asset-server`** — Central Shared Asset Storage (PB-205): a
small, pure-Rust, read-only HTTP server (port 7301) for physical assets
(markdown/scripts/images) that Write/Read nodes fetch by pointer path
instead of bind-mounting a shared host directory themselves — the sole
claimant of its own volume, which is what actually closes the SELinux
`:Z`/`:z` mount-sharing bug class (PB-200) structurally.

**DubSar Notebook** — the Godot 4 HeptaScript Notebook
(`workspace/bahyway_v4/godot/dubsar-theater/`, PB-207): a real,
now-openable UI (`project.godot`) for running `QUERY:`/`SEARCH:`
commands against a live EnkiDDB Read Node and seeing real row content
(not just counts) in a log and a GEM/ORBIT/FUZZY classification grid.
See Manual §7.
