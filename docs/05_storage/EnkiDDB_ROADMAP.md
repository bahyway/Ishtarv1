# EnkiDDB (Tigris) — Implementation Roadmap

**Audience:** engineers implementing or operating EnkiDDB in Dev/Test/Prod.
**Status as of 2026-07-19 (PB-207):** Phase 0 complete, deployed, and
live-verified on real infrastructure (`eriduous-vdi`). Phase 2 (ingest CLI
+ authorship gate) is done and hardened with a second security gate and a
VDI-side ingestion path. Phase 3 (populate with production material) has
been proven end-to-end live but is not yet a repeatable, committed
pipeline — see its section below for the exact open decision. This
document tracks what's real, what's next, and what "done" means at each
step — it does not describe aspirational features as if they already
existed.

## What EnkiDDB is

EnkiDDB is one of BahyWay.Ecosystem v4.0's seven EnkiDB Types — the
**Documentation Database**, sovereign name **Tigris**. It stores documents
(architecture references, glossaries, component specs, playbook records)
as KAKI-sealed EAV particles, chunked into sections for RAG search, and
serves them through a CQRS split identical in shape to every other node
in this fleet: a Write Node with a real Journal WAL, a Read Node over
materialized, immutable Data Files (ADR-012).

Crate: `crates/enkiddb`. Servers: `bin/enkiddb-write-server` (TCP,
port 7101), `bin/enkiddb-read-server` (TCP, port 7102).

## Phase 0 — Foundation (COMPLETE, PB-176–181, PB-192)

Definition of done, all met:
- [x] `WriteNode`/`ReadNode` CQRS split, mirroring EnkiDB's own
  write/read node pattern (`crates/enkiddb/src/{writenode,readnode}.rs`).
- [x] Document parsing + section chunking
  (`DocumentParser::parse_markdown`, `DocumentStructure::sections`) — no
  paragraph or code block orphaned or dropped.
- [x] `meta.collection` taxonomy (`infer_collection`) — automatic
  classification by path convention (`playbook-record`, `glossary`,
  `architecture-reference`, numbered `docs/NN_name/` folders, etc.).
- [x] `RagIndex` — TF-IDF-style search over sections, built the
  CQRS-correct way via `RagIndex::build_from_readnode` (the Read Node
  never touches the Write Node's Journal).
- [x] Podman deployment: `deploy/podman/Containerfile.{write,read}-server`,
  `scripts/enkiddb-sync-data.sh`, `docs/05_storage/EnkiDDB_PODMAN_DEPLOYMENT.md`.
- [x] Real executable Ansible playbook
  (`playbooks/playbook_192_enkiddb_enkimdb_podman_deploy_executable.yml`)
  that builds the images and runs the containers — not just a record of
  having done so manually.
- [x] Smoke-tested as live TCP processes (PB-181): document ingested,
  materialized, synced, queried, and search-ranked correctly end to end.

## Phase 1 — Durability hardening (NOT STARTED)

**Problem:** the Write Node's Journal is in-memory only. A container
restart between `FLUSH` calls loses unflushed documents. The
Write→Read sync is plain `rsync`/`scp` on a timer
(`scripts/enkiddb-sync-data.sh`), not a sealed, verified pipeline.

**Plan:** wire in `crates/enkidb-replication` — already real, tested,
KAKI-sealed (Ed25519-signed), chain-hashed, 7-layer `HeptaSecSentinel`-
verified, and architected for exactly this Write-Pod → Broker → Read-Pod
topology. Not wired in yet because it needs a real key-management
decision (where do the Write Node and Broker's signing keys live, who
rotates them) that this roadmap deliberately does not make unilaterally.

**Definition of done:**
- [ ] Key generation/distribution decision made and documented.
- [ ] `enkidb-replication`'s broker running as a fifth container between
  Write and Read (or a documented equivalent topology).
- [ ] `scripts/enkiddb-sync-data.sh` retired or demoted to a fallback.
- [ ] A killed-mid-flush Write Node container recovers its unflushed
  documents on restart (the actual property this phase buys).

## Phase 2 — Directory-ingest CLI + team-authorship gate (DONE, PB-198)

**Problem:** `WriteNode::ingest_directory_categorized` was real and
tested but exposed nowhere — bulk-loading `docs/` meant calling that
Rust function yourself or sending one document at a time over
`enkiddb-write-server`'s TCP protocol.

**Decision (Architect, 2026-07-19):** build both entry points, not one:
- `bin/enkiddb-cli`, a local no-TCP binary, running three explicit,
  always-in-order stages (`enkiddb-cli ingest <dir>`):
  1. **SCAN** — `enkiddb::scan_markdown_directory` walks the tree, lists
     every `.md` file found.
  2. **CATEGORIZE** — each file's inferred `meta.collection`
     (`infer_collection`) plus its authorship check, reported before
     anything runs.
  3. **RUN** — only if every file passed CATEGORIZE: journal +
     materialize via `WriteNode::ingest_directory_categorized_checked`.
  `--dry-run` stops after CATEGORIZE.
- `INGEST_DIR:<path>` on `enkiddb-write-server`, mirroring
  `enkimdb-write-server`'s `SCAN_CRATES:`/`SCAN_PLAYBOOKS:` shape exactly.

**Security check, as specified:** ingestion is rejected unless the
creator is a BahyWay.Ecosystem team member. Built on git history, not a
new invented identity scheme — `crates/enkiddb/src/authorship.rs`'s
`TeamAllowlist` checks each file's last commit author
(`git log -1 --format=%ae`) against an allowlist (`--team-allowlist
<file>`, or `$ENKIDDB_TEAM_ALLOWLIST`, or a built-in seed of the
Architect's own real git identities found in this repo's history).
**Fails closed**: an undeterminable author (untracked file, no git, git
unavailable) is never authorized. The check is enforced once, in the
library (`WriteNode::ingest_directory_categorized_checked`), so both the
CLI and the server command share the same policy — the CLI can't be
bypassed by hitting the server directly.

The gate is deliberately git-identity-based, not `kupru`-signature-based
— no document format here carries a seal today, and git commit authorship
is the one piece of real, already-present provenance every tracked file
has. A future iteration could upgrade to `kupru::SovereignVerifier`
signature checks once documents actually carry seals; this is the honest
v1. It also deliberately does **not** trust this agent's own commit
identity (`noreply@anthropic.com`) by default — verified live against
this repo's real `docs/` tree, which is entirely authored under that
identity today: the CLI correctly rejects all 34 files with `RUN
skipped`, proving the gate does what it says rather than rubber-stamping
whatever the ingesting process happens to be. Ingesting this repo's
existing `docs/` for Phase 3 will need either an explicit
`--team-allowlist` the Architect opts into, or those files re-committed
under a real team identity — a decision left to the Architect, not
defaulted here.

**Definition of done:**
- [x] One command (server AND CLI, both) ingests an entire directory
  tree of markdown in one call, categorized, chunked, journaled.
- [x] Verified against this repo's real `docs/components/` (4 files):
  SCAN found them, CATEGORIZE reported their real collection + author,
  RUN journaled and materialized real Data Files (51 entities) when
  authorized, and both the CLI and the live TCP server correctly
  `ERR:ingest_dir: unauthorized creator for ...` when not.
- [x] 8 new tests (`authorship.rs`: 6, `writenode.rs`: 2), all passing;
  workspace-wide `cargo test` unaffected elsewhere.

**Hardened since PB-198, all real and shipped:**
- [x] **PB-199/200** — `/source:ro,z` bind mount added so `INGEST_DIR`
  can actually see host files; `:Z`→`:z` fix once a second container
  (`enkimdb-write`) shared the same host path (`:Z` is exclusive-per-
  container, `:z` is shared — using `:Z` on a jointly-mounted path
  silently SELinux-locks whichever container started most recently).
- [x] **PB-201/202** — `enkiddb-write-server`'s runtime image was
  missing the `git` binary (authorship checks silently returned
  "no history" for every file) and needed `git config --system --add
  safe.directory /source` (CVE-2022-24765 "dubious ownership" — the
  bind-mounted repo's UID never matches the container's). Both baked
  into `deploy/podman/Containerfile.write-server` permanently.
- [x] **PB-203** — dependency-link auto-discovery
  (`crates/enkiddb/src/links.rs`, no `regex` crate): documents that
  reference each other by markdown link or backtick path get a real
  `depends-on` particle minted automatically during directory ingest.
- [x] **PB-205** — `bin/enkiddb-cli ingest --remote <host:port>`: SCAN +
  CATEGORIZE run on the VDI's own real git checkout (correct ownership,
  git already installed), then each authorized document is sent to a
  live `enkiddb-write-server` over the wire — the container never needs
  `git` or a bind-mounted repo for this path. `bin/enkiddb-asset-server`
  (new, pure-Rust, read-only HTTP) is the sole claimant of a shared
  physical-asset volume, closing the PB-200 SELinux-sharing bug class
  structurally rather than by flag correction.
- [x] **PB-206** — a second, independent pre-ingest gate: every
  document's raw bytes are scanned for known malware/webshell/dropper
  signatures (`crates/enkiddb/src/security.rs`, wrapping the real
  `musaru-security::zip_scan` engine — the same scanner behind the
  "Nergal" visualization panel, wired into EnkiDDB for the first time
  here) before `DocumentParser` ever runs. Enforced independently by
  both `enkiddb-cli`'s CATEGORIZE stage and the server itself — a hit
  responds `ERR:security: <detail>` and nothing is parsed or journaled.
- [x] **PB-207** — `workspace/bahyway_v4/godot/dubsar-theater/project.godot`
  (previously missing entirely): the DubSar HeptaScript Notebook is now
  actually openable, points at the real deployed EnkiDDB Read Node
  (host/port corrected from a stale placeholder), and its execution log
  shows real row content (SEARCH: score + text snippet, QUERY: kaki +
  attrs) instead of just a count — the tool for visually inspecting what
  Phase 2/3 actually ingested and testing RAG search against a phrase.

## Phase 3 — Populate with production Implementation-phase material (PROVEN LIVE, not yet a repeatable pipeline)

**Problem:** EnkiDDB is a real, working documentation database; the
question is whether it holds anything of substance yet.

**What actually happened (2026-07-19, live on `eriduous-vdi`):** the
Architect ran `INGEST_DIR:` against this repo's real `docs/` tree (34
markdown files), `FLUSH`ed (460 entities materialized), then ran real
`QUERY:`/`SEARCH:` calls against the synced Read Node and got back real
results — including real component documentation and ranked RAG hits
containing real Arabic text. This is genuine end-to-end proof the
architecture works on production-equivalent infrastructure, not a
smoke-test fixture.

**Why this isn't "done" yet:** that live ingestion needed a temporary,
uncommitted opt-in of this agent's own commit identity
(`noreply@anthropic.com`) into the team allowlist, because essentially
all of this repo's real `docs/` tree is currently authored under that
identity (see Phase 2's authorship-gate note above). That opt-in was
applied by hand on the running container and was wiped by the next
resync, precisely because it is deliberately **not** committed into
`config/enkiddb_team_allowlist.txt` — extending trust to an AI commit
identity by default would defeat the point of a human-authorship gate.
This is a real, standing decision for the Architect, not an oversight:
- **Option A** — re-author (or re-commit) the production `docs/` corpus
  under a real human team identity before Phase 3's ingestion becomes
  routine.
- **Option B** — explicitly opt `noreply@anthropic.com` into
  `config/enkiddb_team_allowlist.txt` permanently, accepting that
  AI-authored commits are then a trusted source for this gate.
- **Option C** — ingest going forward only applies to genuinely
  human-authored new material (via `enkiddb-cli ingest --remote`, PB-205),
  leaving the pre-existing AI-authored `docs/` tree out of EnkiDDB by
  design.

No default has been chosen for the Architect; this roadmap will not pick
one silently.

**Definition of done:**
- [x] `RagIndex::search` returns relevant results for real engineering
  questions against the real ingested corpus — proven live, 2026-07-19.
- [ ] The above is reproducible from a clean deployment without a manual,
  uncommitted allowlist edit — blocked on the Architect's decision above.

## Phase 4 — Deferred / blocked (tracked, not scheduled)

- **ENLIL 7-index stack** and **HeptaScript v1.0 & v2.0 unification**:
  both remain blocked on the Architect supplying missing specifics — see
  `docs/08_pipeline_alaktu/OTAP_PIPELINE.md`'s own "definition of done" for the exact gap.
  Not an EnkiDDB-specific blocker; tracked here because EnkiDDB's query
  path depends on HeptaScript.
- **Multi-node / HA**: not scoped. Current topology is one Write Node,
  one Read Node. Horizontal read scaling (multiple Read Node replicas
  behind the same sync source) is a natural extension once Phase 1's
  durability work lands, not before.

## How this roadmap relates to the other 6 EnkiDB Types

This roadmap is scoped to EnkiDDB (Tigris) and EnkiMDB (Euphrates) only —
the first two of BahyWay's seven EnkiDB Types, per the agreed build
order. The other five get their own roadmap entries when their turn
comes, once their own crates/tests exist to describe honestly. This
document will not be retrofitted to cover them speculatively.
