# Architect Design — EnkiDDB & EnkiMDB Podman Topology and EAV Schemas

BahyWay.Ecosystem v4.0 — 2026-07-19

This document answers two questions directly, with facts sourced from the
real deployed infrastructure and the real source code, not from the
earlier Gemini "3-Podman Node Topology" sketch (which described a
different, never-built multi-VM/MinIO/NFS shape):

1. **Where and how do the Podman containers for EnkiDDB and EnkiMDB
   actually run** — how many hosts, how many containers, what talks to
   what?
2. **What does each database's EAV schema actually look like** — what is
   an entity, an attribute, a value; how does a document or a crate
   become rows; how does data move from a live write to a queryable
   snapshot?

Everything below is either read directly from source (`crates/`, `bin/`)
or verified live this session (PB-192, PB-205, PB-208, PB-211, PB-212,
PB-213). Where a claim is a design intent rather than a live-verified
fact, it's labeled as such.

**PB-212 UPDATE (2026-07-19):** the topology below was rewritten after
this document's first version described a single-host consolidation
(all five containers on `eriduous-vdi`). That was accurate for the
window between PB-205 and PB-212, but was never the Architect's actual
intended shape — confirmed directly against their real Virtual Machine
Manager inventory (four VMs: `eriduous-vdi`, `enkidb-node-write`,
`enkidb-node-read`, and a `dubsar-workstation` being retired) and their
explicit correction: one VM per CQRS **role**, not one VM per database.

---

## 1. Topology — the real 4-VM fleet, split by CQRS role

**Four VMs, not one.** Every EnkiDB Type's Write-role Podman container
lives on `enkidb-node-write`; every type's Read-role container lives on
`enkidb-node-read` — the same role split applied consistently across all
7 EnkiDB Types (today: EnkiDDB and EnkiMDB, the only two with real
Write/Read server binaries built; the other five join once their own
server binaries exist — explicit future work, not started). `eriduous-vdi`
runs the DubSar (Godot) IDE, is the Ansible control node every playbook
in this repo is run from, hosts whole-ecosystem monitoring, and keeps
`enkiddb-assets` (PB-205's shared static asset server — no Write/Read
split, so it doesn't belong on either node VM).

```
 enkidb-node-write            enkidb-node-read             eriduous-vdi
 192.168.122.101              192.168.122.107              192.168.122.214
┌──────────────────┐         ┌──────────────────┐         ┌──────────────────────┐
│ enkiddb-write     │         │ enkiddb-read      │         │ enkiddb-assets :7301  │
│  :7101            │         │  :7102            │         │  (shared static HTTP  │
│  Journal (in-mem  │  FLUSH  │  Data Files + RAG │         │   GET server, no      │
│  WAL), mounts     │────────▶│  index, vol:       │         │   Write/Read split)   │
│  repo /source RO  │  sync   │  enkiddb-read-data │         │                       │
│  vol: enkiddb-     │ (PB-213,│  reloads every     │         │ DubSar (Godot) IDE     │
│  write-data        │ cross- │  RELOAD_SECS       │         │ Ansible control node   │
│                     │ host)  │  (default 30s)     │         │ (runs every playbook   │
│ enkimdb-write       │        │                     │         │  in this repo, incl.  │
│  :7201              │────────▶│ enkimdb-read       │         │  the cross-host sync  │
│  Journal, mounts    │  FLUSH │  :7202              │         │  relay below)         │
│  repo /source RO    │  sync  │  Data Files (no RAG)│         │ Whole-ecosystem        │
│  vol: enkimdb-       │        │  vol: enkimdb-      │         │ monitoring             │
│  write-data          │        │  read-data          │         │                       │
└──────────────────┘         └──────────────────┘         └──────────────────────┘
```

| Container | Host | Port | Volume | Bind mount |
|---|---|---|---|---|
| `enkiddb-write` | enkidb-node-write | 7101 | `enkiddb-write-data` | repo checkout, **read-only**, at `/source:ro` — used for `INGEST_DIR:/source/...` and the git-authorship allowlist path. `enkiddb-cli --remote` (PB-205) instead runs bare-metal on `eriduous-vdi` and ships bytes over TCP to `enkidb-node-write:7101`, bypassing this mount entirely — that's the whole point of PB-205's host-side authorship check. |
| `enkimdb-write` | enkidb-node-write | 7201 | `enkimdb-write-data` | repo checkout, **read-only**, at `/source:ro` — this is what `SCAN_CRATES`/`SCAN_PLAYBOOKS` actually walks |
| `enkiddb-read` | enkidb-node-read | 7102 | `enkiddb-read-data` | none |
| `enkimdb-read` | enkidb-node-read | 7202 | `enkimdb-read-data` | none |
| `enkiddb-assets` | eriduous-vdi | 7301 | `enkiddb-assets-data` | none |

Source: `playbooks/playbook_212_deploy_cqrs_2node_split.yml` (the current
deployment playbook — supersedes PB-192, which put all four database
containers on `eriduous-vdi` alone) and `ansible/inventory.ini` (real
SSH connection details for both node VMs, sourced from the Architect's
own `09_sovereign_vm_restart.yml`).

### Why role-split VMs, not database-split or single-host

The CQRS split (Write vs Read) is a **process/data-ownership boundary**
(ADR-012's "Data Files Law": the Read Node depends only on Data Files
existing at a path, never on how they got there) that maps naturally
onto a **network/host** boundary too, once more than one EnkiDB Type
needs the same pattern: rather than a VM per database (which would mean
N databases × 2 roles = 2N VMs as more of the 7 EnkiDB Types get server
binaries), one VM per role holds every database's container of that
role. Two VMs cover all 7 types at full build-out, not fourteen.

PB-205 had briefly consolidated everything onto `eriduous-vdi` alone
(rejecting a MinIO/NFS-backed shared-storage idea from an earlier Gemini
sketch as incompatible with the sovereignty axiom), which was a real,
working state but not the Architect's actual intended shape — corrected
here once that became clear. `scripts/enkiddb-sync-data.sh` /
`enkimdb-sync-data.sh` were originally written for exactly this two-VM
cross-host shape; PB-213's Ansible sync (below) supersedes them with the
same cross-host approach plus the rootless-Podman handling PB-211 first
established.

### How data moves, end to end

1. A document/crate/playbook is ingested against the **Write** container's
   port on `enkidb-node-write` (7101 or 7201) — either directly over TCP
   (the wire protocol both `EnkiDDB_MANUAL.md` and `EnkiMDB_MANUAL.md`
   document) or via `enkiddb-cli`'s `--remote` mode (PB-205), which runs
   bare-metal on `eriduous-vdi` so its git-authorship gate can see the
   real checkout, sending over the network to `enkidb-node-write`.
2. `FLUSH` (explicit command, or automatically every `FLUSH_EVERY`
   ingests/scans — default 10 for EnkiDDB, 50 for EnkiMDB) triggers
   `materialize_fresh`: wipe `DATA_DIR/current`, then rewrite it from the
   Write Node's in-memory Journal in one O(n) pass.
3. PB-213 (an Ansible playbook run from `eriduous-vdi`, the control
   node) relays the fresh `current/` from `enkidb-node-write` to
   `enkidb-node-read`: stage it out of the Write volume with `podman
   unshare cp`, `fetch` the tarball to the control node, `copy` it to the
   Read host, then `podman unshare cp -a`/`rm -rf`/`mv` it into place —
   required, not optional, because this fleet runs **rootless** Podman,
   which remaps container-internal
   UIDs on the host filesystem (see §4 below for why this matters).
4. The Read container's own background thread notices the new
   `current/` within `RELOAD_SECS` (default 30s) and swaps it in — no
   restart needed. Before the first successful reload, it answers
   `ERR:not ready`.

---

## 2. KAKI — the entity identity shared by both databases

Every row in either database's EAV store is keyed by a **KAKI**
(`enkidb-kaki` crate): a fixed 16-byte, immutable, `Copy` value —

| Bytes | Field | Meaning |
|---|---|---|
| 0–3 | `minted_id` | numeric ID minted at creation (u32 BE) |
| 4–5 | `tribe_id` | sovereignty scope (u16 BE) |
| 6 | `kaki_type` | 0x01 Identity / 0x02 Event / 0x03 CrossTribe / 0x04 Pattern |
| 7 | `kaki_role` | 0x01 KISHIB (file/blob) / 0x02 ZIKRU (record/entity) / 0x03 PARZU (logic/template) |
| 8–11 | reserved | zeroed |
| 12–13 | `timestamp` | birth time, truncated u16 |
| 14–15 | `checksum` | CRC-16/CCITT over bytes 0–13 |

Not a UUID and not random: `minted_id` is generated by mixing
`SystemTime::now().subsec_nanos()` with a per-process counter — pure
`std`, no `uuid`/`rand` crate.

Both databases follow the same two-KAKI pattern per write:

- One **Identity-Kaki** (role KISHIB) is minted for the document, section,
  concept mention, or artifact itself — its permanent "birth certificate."
- One **Event-Kaki** (role ZIKRU) is minted for the ingest/scan action that
  wrote it, wrapping the write in a `JournalEntry`. Re-ingesting the same
  content mints a *new* Event-Kaki against the *same* Identity-Kaki if the
  caller reuses it, or an entirely new Identity-Kaki otherwise (EnkiMDB's
  re-scan behavior: see §3.2).

The governing rule, stated identically in both glossaries: **identity
lives in the KAKI; everything else — quality, state, content — lives in
EAV, never in the 16 bytes.**

---

## 3. The EAV schema itself

### 3.1 The shared row shape — `Particle` (7D EAV)

Neither database defines its own Entity/Attribute/Value type. Both reuse
one shared type from `enkidb-particles`:

```rust
pub struct Particle {
    pub entity:    IdentityKaki,  // E — which thing
    pub attribute: String,        // A — namespaced string, e.g. "meta.title"
    pub value:     AkkValue,      // V — one of 31 sovereign value variants
    pub timestamp: u64,           // T — UNIX epoch seconds
    pub shard:     u32,           // S — 0 at MVP
    pub proof_ctx: u32,           // Z — 0 = unverified
    pub mat_tag:   u32,           // M — 0 = base particle, non-zero = derived/materialized view
}
```

This is why it's called a **7D EAV row**, not a plain 3-column EAV table:
Entity, Attribute, Value, plus Timestamp, Shard, proof context (Z), and a
materialization tag (M) reserved for derived views. `Particle::base(...)`
is the constructor both databases actually call, defaulting S/Z/M to 0 —
neither database uses sharding, proof contexts, or materialized-view
tagging yet; those fields exist for future use, not dead weight removed
by either database's own code.

### 3.2 EnkiDDB (Tigris) — documents, sections, concepts

**Attribute namespace — `DocOrbit`** (a naming convention on the
`attribute: String` field, not a separate physical store):

| Orbit | Prefix | Example attributes |
|---|---|---|
| Meta | `meta.` | `meta.title`, `meta.collection`, `meta.source_path`, `meta.section_order`, `meta.concept_kind` |
| Head | `head.` | `head.h1`…`head.h6`, `head.anchor`, `head.order` |
| Body | `body.` | `body.paragraph`, `body.order`, `body.summary` (RAG key), `body.text` (RAG value) |
| Code | `code.` | `code.block`, `code.language`, `code.order` |
| Hist | `hist.` | `hist.event` (e.g. `"BIRTH"`) |
| Link | `link.` | `link.target` (a `KakiPk` back-reference), `link.description` (e.g. `"section-of"`, `"mentioned-in"`) |

**What one document becomes, per ingest:**

1. One **document-level** Identity-Kaki, carrying `meta.title`, one
   `head.h1`..`h6` + `head.anchor` + `head.order` triple per heading, one
   `body.paragraph` + `body.order` per paragraph, one `code.block` +
   `code.language` + `code.order` per fenced code block, `meta.source_path`,
   `meta.collection` (see below), and `hist.event = "BIRTH"`.
2. One **child Identity-Kaki per section** (a header-bounded chunk of the
   document — the RAG unit), carrying `meta.title` (`"<doc title> §
   <heading>"`), `meta.section_order`, `body.summary` (a non-LLM heuristic:
   title + first ~200 chars, used as the RAG search key), `body.text`
   (the section's full untouched text — the RAG search value), a
   `link.target`/`link.description="section-of"` pointer back to the
   parent document, and `hist.event = "BIRTH"`.
3. One **child Identity-Kaki per concept mention** found in the text
   (`meta.title` = concept name, `meta.concept_kind` = `"gate"` /
   `"crate"` / `"sovereign-name"`, `link.target`/`link.description=
   "mentioned-in"`, `hist.event = "BIRTH"`) — this is the concept registry
   that powers `ConceptGraph`'s exposure-preview narration.

**`meta.collection`** — assigned automatically by `infer_collection()`,
in order:
1. Path contains `playbooks/` or filename starts with `playbook_` →
   `"playbook-record"`.
2. File sits inside a subfolder of a `docs/` directory (not directly in
   its root) → that subfolder's name, numeric prefix and leading
   underscore stripped, lowercased, underscores → dashes (e.g.
   `docs/06_governance_parzu/x.md` → `"governance-parzu"`).
3. Otherwise, filename heuristics: contains `"glossary"` →
   `"glossary"`; `"architecture"` → `"architecture-reference"`;
   `"transparency"` → `"concept-law"`; else → `"general"` (the
   always-populated fallback bucket, and the safest first `QUERY:`
   collection to check against when verifying a fresh ingest).

**The RAG index (`enkiddb::rag::RagIndex`)** is EnkiDDB-only — EnkiMDB has
no equivalent. On the Read Node, it's built by running a HeptaScript query
(`WHERE E[hist.event] = "BIRTH"`, since `ReadNode::query` has no "give me
everything" primitive) to pull every section's `body.summary`/`body.text`
pair, indexes the summaries with TF-IDF + cosine similarity
(`adapa-recall`), and caches the text so `SEARCH:<k>:<phrase>` can return
ranked hits with real scores and the section's full untouched text without
re-querying per result.

### 3.3 EnkiMDB (Euphrates) — crates and playbooks, no RAG

EnkiMDB catalogs exactly two kinds of artifact —
`crates/*/Cargo.toml` and `playbooks/*.yml` — deliberately narrow (the
`ArtifactKind` enum's own doc comment: "extend only when a new scanner
backs it, no speculative variants").

**Attribute namespace — flat `artifact.*`, no orbit enum:**

| Attribute | Value | Notes |
|---|---|---|
| `artifact.name` | crate/playbook name | |
| `artifact.kind` | `"Crate"` or `"Playbook"` | |
| `artifact.path` | filesystem path | |
| `artifact.version` | crate version string | **only emitted for crates** — playbooks never get this triple. A crate declaring `version.workspace = true` records the literal string `"workspace"`, not a resolved semver. |

Every crate/playbook mints one Identity-Kaki (role KISHIB) plus one
Event-Kaki per `SCAN_CRATES`/`SCAN_PLAYBOOKS` call that touches it —
**re-scanning re-journals every artifact found, even unchanged ones, with
no deduplication.** This is documented, known v1 behavior, not a bug: if
you re-run a scan, expect duplicate-looking entries (distinguishable by
their distinct Event-Kakis and timestamps) until a dedup pass is built.

**No `SEARCH` command and no RAG index exist for EnkiMDB.** Full-text
search over crate source or playbook YAML content is explicitly out of
scope here — that's EnkiDDB's job, if such content is ever ingested there
as documents.

Live-verified count (from `EnkiMDB_MANUAL.md`): 151 real crates and 88
real playbooks catalogued and materialized from this repo's own workspace.

### 3.4 Journal → materialize → Data Files, precisely

Both write-servers implement the identical `materialize_fresh`:

```rust
fn materialize_fresh(write_node: &WriteNode, data_dir: &Path) -> io::Result<MaterializeStats> {
    let current = data_dir.join("current");
    let _ = fs::remove_dir_all(&current);   // full wipe, not incremental
    fs::create_dir_all(&current)?;
    readnode::materialize_now(write_node, current.join("entities"), current.join("eav"))
}
```

It's a full wipe-and-rebuild, not an incremental append, because
`DataFileWriter` always opens in append-only mode and never truncates —
re-materializing to the same path without wiping first would duplicate
every entity on every flush. Each materialization:

1. Groups every Journal entry by target KAKI.
2. Writes each entity's full raw journal history into `entities.data`
   (append-only records) with a sorted `entities.idx`.
3. Computes a last-write-wins EAV snapshot per entity, packs
   `(attr_hash, value_fingerprint)` posting-list keys, and writes them into
   `eav.data` with a sorted `eav.idx`.

So a live `DATA_DIR/current/` directory contains exactly four files:
`entities.data`, `entities.idx`, `eav.data`, `eav.idx` (plus transient
`.idx.staging` files during a write) — this is what you saw in the real
`podman unshare ls -la` output during PB-211's verification run.

**Durability caveat, stated explicitly in both write-servers' own module
docs:** the Journal is in-memory only in the current implementation — no
WAL persistence or replay across process restarts. Anything ingested
since the last `FLUSH` is lost on an unclean restart. `enkidb-replication`
(a KAKI-sealed append-only log) is named in the source as the real fix,
not yet wired in (task #3, still pending).

---

## 4. The rootless-Podman sync gotcha (why PB-211/PB-213 exist)

Both databases' Read Node is deliberately decoupled from the Write Node's
process — it only ever reads a `current/` directory at a path, per
ADR-012. Moving the Write volume's fresh `current/` into the Read
volume's `current/` is therefore always some kind of file transfer
between two Podman named volumes' real host mountpoints — a local copy
when Write and Read shared one host (the PB-205/PB-211 window), a
cross-host relay now that they live on separate VMs (PB-212/PB-213).

This fleet runs **rootless** Podman as the login user (`bahyway`, no
`sudo` anywhere in the deployment). Rootless Podman remaps
container-internal UIDs to a different range on the host filesystem, so
files the `enkiddb-write`/`enkiddb-read` (or `enkimdb-*`) containers wrote
into their volumes are **not** owned by the login user's literal UID —
even though that user can successfully run `podman volume inspect` on the
same volume. Plain `cp`/`rm`/`mv` as that user hit real `Permission
denied` (found live, PB-211, when Write and Read still shared a host).
`podman unshare <cmd>` runs a command inside the same user namespace
Podman itself uses for its containers, resolving the remapped ownership
correctly — Podman's own documented mechanism, not a workaround. **This
gotcha applies on BOTH ends of PB-213's cross-host transfer, not just
one** — the volume being staged OUT of on `enkidb-node-write` and the
volume being swapped INTO on `enkidb-node-read` are each independently
rootless-remapped, so `podman unshare` appears in the staging step and
the swap step, on their respective hosts (`playbooks/tasks/
enkiddb_cross_host_sync.yml`). The plain rsync/tar/copy in between —
`enkidb-node-write` to the `eriduous-vdi` control node, then to
`enkidb-node-read` — touches only plain-owned staging files, never the
volumes directly, so it needs no special handling.

### The systemd-linger gotcha (PB-212.1, found live on the first real PB-213 run)

A second, distinct rootless-Podman trap, easy to confuse with the one
above but with a different root cause and fix. PB-212 built and started
all four database containers successfully on the two node VMs — its own
`wait_for` on their ports succeeded within that same playbook run. But
by the time PB-213 ran moments later (a separate `ansible-playbook`
invocation, a new SSH session), `podman ps` on `enkidb-node-write` showed
zero containers.

Root cause: rootless Podman containers started over SSH are supervised
by that user's own systemd instance (via `conmon`). Without `loginctl
enable-linger <user>`, `systemd-logind` tears down a user's entire
systemd instance — every container it supervises included — the moment
the *last* login session for that user ends. `eriduous-vdi` never
surfaced this because a GUI session stays logged in there continuously;
`enkidb-node-write`/`enkidb-node-read` only ever get the transient SSH
session Ansible itself opens and closes, so their containers vanished as
soon as that session did.

Fixed in PB-212 (tasks 0.5/0.6 in both the Write and Read plays): check
`loginctl show-user <user> --property=Linger` and run `loginctl
enable-linger <user>` if not already set, before any container starts.
Also hardened with an explicit `podman start` after every container-run
task (3.2b/3.4b), rather than assuming "created" still means "running"
by the time a later, separate playbook checks — cheap insurance against
this same failure mode recurring for any other reason.

---

## 5. Summary table — EnkiDDB vs. EnkiMDB at a glance

| | EnkiDDB (Tigris) | EnkiMDB (Euphrates) |
|---|---|---|
| Write port | 7101 | 7201 |
| Read port | 7102 | 7202 |
| Ingests | Markdown documents | Crates (`Cargo.toml`) + playbooks (`*.yml`) |
| Ingest trigger | `INGEST_DIR:<path>` / per-doc wire command / `enkiddb-cli --remote` | `SCAN_CRATES:<path>` / `SCAN_PLAYBOOKS:<path>` |
| Attribute namespace | `DocOrbit` (`meta.`/`head.`/`body.`/`code.`/`hist.`/`link.`) | Flat `artifact.*` |
| Auto-categorization | `infer_collection()` → `meta.collection` | none — kind is explicit (`"Crate"`/`"Playbook"`) |
| RAG / `SEARCH` | Yes — `RagIndex`, TF-IDF + cosine over `body.summary`/`body.text` | No — no RAG index exists |
| Re-ingest dedup | Section/concept children re-minted per ingest, same as EnkiMDB's pattern | None — every scan re-journals every artifact found |
| `FLUSH_EVERY` default | 10 | 50 (higher — one scan call can ingest dozens of artifacts at once) |
| Pre-ingest security gate | `musaru-security::zip_scan` (byte-signature malware/webshell scan) + git-authorship allowlist | git-authorship allowlist only |
| Sovereign display name | `"tigris"` | `"euphrates"` |

---

## Sources

Real source files (`workspace/bahyway_v4/crates/{enkidb-kaki,enkidb-particles,enkiddb,enkimdb,enkidb-readnode,enkidb-datafile}/src/**`,
`workspace/bahyway_v4/bin/{enkiddb-write-server,enkiddb-read-server,
enkimdb-write-server,enkimdb-read-server}/src/main.rs`), real deployment
playbooks (`playbooks/playbook_192_*.yml`, `playbook_205_*.yml`,
`playbook_208_*.yml`, `playbook_211_*.yml`, `playbook_212_*.yml`,
`playbook_213_*.yml`), and the existing
`docs/EnkiDDB_{GLOSSARY,MANUAL}.md` / `docs/EnkiMDB_{GLOSSARY,MANUAL,
PODMAN_DEPLOYMENT}.md`, all of which remain the operational reference —
this document is the architectural map that sits above them, not a
replacement for any of them.
