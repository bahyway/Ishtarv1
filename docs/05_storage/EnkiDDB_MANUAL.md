# EnkiDDB (Tigris) — Operations Manual

**Audience:** engineers deploying, operating, or troubleshooting EnkiDDB.
Assumes familiarity with Rust, Podman, and this repo's KAKI/EAV/CQRS
vocabulary — see `EnkiDDB_GLOSSARY.md` if any term here is unfamiliar.

## 1. Deploy

**To bring up the whole environment from scratch, run one command from
`eriduous-vdi` (PB-215, 2026-07-20):**

```bash
ansible-playbook playbooks/playbook_215_full_environment_bootstrap.yml
```

This chains, in order, everything a real bring-up needs: PB-212 (build +
run all four database containers on the real 2-VM CQRS split —
`enkiddb-write`/`enkimdb-write` on `enkidb-node-write`,
`enkiddb-read`/`enkimdb-read` on `enkidb-node-read`), PB-213 (cross-host
FLUSH/sync/verify), and PB-208 (full-corpus ingestion). See PB-215's own
header for the few one-time prerequisites it doesn't automate (repo
checked out on all three VMs, SSH already working, the team allowlist
populated). No manual multi-step sequence to remember — this is now the
canonical bring-up path, replacing the old single-host
`playbook_192_*.yml` (kept in the repo as historical record; superseded,
not deleted).

If PB-215 finishes but QUERY/SEARCH still report `ERR:not ready`, run
`ansible-playbook playbooks/playbook_214_diagnose_read_node_not_ready.yml`
next — a real, single-command diagnostic playbook (retries the query
properly, checks SELinux if it's still stuck), not a set of manual
commands to copy-paste.

## 2. Ingest a document

Any TCP client sending the wire protocol works — length-prefixed u32 LE
frame, one request/response per connection (see
`bin/enkiddb-write-server/src/main.rs`'s module doc for the exact byte
layout). The request body is:

```
<collection>
<markdown content...>
```

- `<collection>` — a free-text label you choose for this call (e.g.
  `component`, `roadmap`). It is separate from, and does not override,
  the automatic `meta.collection` tag `infer_collection` would assign
  from the document's real file path — this protocol-level `<collection>`
  line exists because a raw TCP ingest has no file path to infer from.
- Response: `OK:<kaki_hex>:<section_count>` on success,
  `ERR:<message>` on failure. As of PB-206, every body is scanned for
  known malware/webshell/dropper byte signatures
  (`enkiddb::scan_document`, wrapping `musaru_security::zip_scan` — the
  real engine behind the "Nergal" visualization panel) before
  `DocumentParser` ever runs; a hit responds `ERR:security: <detail>`
  and nothing is parsed or journaled. This check runs on every ingestion
  path (`INGEST_DIR`, this plain command, `enkiddb-cli` local and
  `--remote`) since it lives inside `WriteNode::ingest_document_from_path`
  and this handler both, not just one entry point.

Special request: `FLUSH` (no body) — force-materializes the current
Journal state to `DATA_DIR/current/{entities,eav}` immediately, instead
of waiting for the next `FLUSH_EVERY`-document auto-flush. Response:
`OK:FLUSHED:<entity_count>`.

**Bulk directory-ingest (PB-198):**

```
INGEST_DIR:<path-inside-this-container>
```

Walks `<path>` for every `.md` file, categorizes and chunks each one
(`WriteNode::ingest_directory_categorized_checked`), and journals them
all in one call — response `OK:INGESTED:<count>`. Gated by the same
git-based team-authorship check `enkiddb-cli` enforces (below): a file
whose last commit author isn't on the allowlist aborts the whole call
with `ERR:ingest_dir: unauthorized creator for <path>: <author>` instead
of ingesting anything. Set `ENKIDDB_TEAM_ALLOWLIST=/path/to/team.txt`
(one git author email per line) in the container's environment before
relying on this in anger — the built-in seed only knows the Architect's
own real git identities.

For local, no-TCP bulk loads (e.g. from Eriduos-vdi directly, not
through the container), use `bin/enkiddb-cli` instead:

```
enkiddb-cli ingest <dir> [--data-dir <dir>] [--team-allowlist <file>] [--dry-run]
```

It runs the same three stages explicitly and in order — SCAN (list every
`.md` file), CATEGORIZE (report each file's collection + authorship
verdict, before anything runs), RUN (journal + materialize, only if
every file passed CATEGORIZE) — so a rejected file is visible before any
write happens, not discovered after. `--dry-run` stops after CATEGORIZE.

### 2a. Recommended production workflow (PB-205/206) — step by step

The workflow above materializes Data Files **locally**, disconnected
from any running server. For ingesting real client/team documents into
an actually-deployed Write Node (the case that matters once EnkiDDB is
sold as a service), use `enkiddb-cli`'s `--remote` mode instead — SCAN
and CATEGORIZE run exactly as above, but RUN sends each authorized,
scanned document over the wire to a live `enkiddb-write-server`, so
authorship checking happens on your own real git checkout (correct
ownership, git already installed) and the container never touches `.git`
for this path.

```bash
# 1. SCAN + CATEGORIZE + RUN against the live Write Node.
#    Every file is checked TWICE before it can reach the Journal:
#    (a) git-authorship (crates/enkiddb/src/authorship.rs) — commit
#        author must be on the team allowlist;
#    (b) security scan (crates/enkiddb/src/security.rs, PB-206) — raw
#        bytes must not match a known malware/webshell/dropper signature.
#    A file failing either check aborts the WHOLE batch (fail closed) --
#    nothing partial gets journaled.
enkiddb-cli ingest ./docs --remote <write-node-host>:7101

# 2. Force an immediate materialize instead of waiting for the next
#    FLUSH_EVERY auto-flush (send FLUSH over the wire protocol, e.g.
#    with the frame_client.py-style helper, or just wait).

# 3. Let the sync step (§4) carry the new Data Files to the Read Node
#    (or wait for the next scheduled sync run).

# 4. Verify with a real query or search against the Read Node (§3) --
#    or, for a visual check, open the DubSar HeptaScript Notebook
#    (godot/dubsar-theater/project.godot, PB-207): press Ctrl+E once
#    the theater scene is running, the NODE picker already defaults to
#    EnkiDDB, type QUERY:... or SEARCH:<k>:<phrase>, and Run -- the log
#    shows each real row's content (kaki+attrs, or score+text), not
#    just a count.
```

A rejected file shows exactly why, before RUN ever starts:

```
[2/3] CATEGORIZE
  OK       widget.md         -> general   author=you@example.com
  REJECTED dropper.md        -> general   security=EICAR standard antivirus test file
  REJECTED unsigned.md       -> general   author=<unknown -- no git history>

  1 authorized, 2 rejected
  RUN skipped -- 2 file(s) failed the team-authorship or security-scan check.
```

## 3. Query / Search (against the Read Node)

Two request forms:

- `QUERY:<heptascript source>` — a full HeptaScript query. Always name
  the attributes you want back explicitly in `WHAT` — a bare `WHAT E[*]`
  returns hash-keyed placeholder attribute names, not the real ones (no
  reverse hash map exists over materialized Data Files; see PB-181's
  record for the discovery). Example:

  ```
  QUERY:WHO T.E
  WHAT E[meta.title, meta.collection]
  WHERE E[meta.collection] = "components"
  ```

  (`"components"`, plural — `infer_collection`'s real output for
  `docs/components/*.md`; verified live while authoring PB-208 that the
  singular form silently returns zero rows, not an error, since the
  WHERE clause is a real exact-match filter.)

- `SEARCH:<top_k>:<query text>` — RAG search over document sections.
  Returns up to `top_k` hits, each `{"kaki":..,"score":..,"text":..}`,
  ranked by relevance, with the section's full untouched text attached
  (`RagIndex::fetch_value_from_readnode`).

Both return `ERR:not ready -- Data Files not synced from the Write Node
yet` if the Read Node hasn't loaded a materialized generation yet — check
that the sync step (§4) has actually run.

## 4. Keep the Read Node in sync

**Correction (PB-208, 2026-07-19):** `scripts/enkiddb-sync-data.sh` was
written for a two-VM topology (`enkidb-node-write` --SSH/rsync-->
`enkidb-node-read`, this repo's older sibling VMs). The topology this
repo actually deploys and has live-verified
(`playbooks/playbook_192_...yml`) is different: `enkiddb-write` and
`enkiddb-read` are two Podman **containers on the same host**
(`eriduous-vdi`), each with its own named volume. Don't pass a container
name to that script expecting SSH semantics — it isn't a reachable host.
`scripts/enkiddb-sync-data.sh` remains correct and useful for an actual
two-VM deployment, if that topology is ever used instead — it just isn't
what's running today.

**Use `playbooks/playbook_211_flush_sync_and_verify_enkiddb.yml`, not a
manual script (PB-211, 2026-07-19).** Earlier revisions of this section
showed a hand-copied shell snippet (`podman volume inspect` + `podman
unshare cp/rm/mv`) — that snippet itself went stale once (an even earlier
version used plain `cp`/`rm`/`mv` without `podman unshare`, then briefly
`sudo rsync`), because the same procedure was being maintained by hand in
two places at once and one of them was forgotten. It's now consolidated
into a single shared Ansible task file
(`playbooks/tasks/enkiddb_flush_sync_verify.yml`), imported by both
PB-208 (full-corpus ingestion, as its last phase) and PB-211 (this
standalone playbook, for syncing/verifying without re-ingesting
anything). Run:

```bash
ansible-playbook playbooks/playbook_211_flush_sync_and_verify_enkiddb.yml
```

Read PB-211's own header comment for the full story: rootless Podman
remaps container-internal UIDs to a different range on the host
filesystem, so files the containers wrote into these volumes aren't
owned by your literal UID even though you can `podman volume inspect`
them — plain `cp`/`rm`/`mv` hit real `Permission denied` there. `podman
unshare <cmd>` runs inside the same user namespace Podman itself uses
for its containers, resolving the remapped ownership correctly (Podman's
own documented mechanism, not a workaround). That failure is also
**silent and dangerous** if you don't check for it: a `cp`/`rm` that
fails without stopping the rest of a script leaves the Read Node serving
stale data while later queries still look like they "worked." PB-211
defaults `enkiddb_search_phrase` to something you can override (`-e
enkiddb_search_phrase="..."`) to a phrase you know is only in the batch
you just ingested, so a real hit is proof, not just a plausible-looking
old result.

The Read Node's background thread reloads `DATA_DIR/current` every
`RELOAD_SECS` (default 30) automatically once new data lands there — no
read-side restart needed; PB-211 already waits for this.

## 5. Categorization reference

`infer_collection(path)` (`crates/enkiddb/src/ingest.rs`) assigns
`meta.collection` automatically for directory-based ingestion, by these
rules in order:
1. Path contains `playbooks/` or filename starts with `playbook_` →
   `playbook-record`.
2. Path is `docs/<subfolder>/<file>` (at least one level deep) → the
   subfolder name, digit/underscore-prefix stripped, lowercased,
   underscores to dashes (e.g. `docs/06_governance_parzu/x.md` →
   `governance-parzu`).
3. Otherwise, by filename: contains `glossary` → `glossary`; contains
   `architecture` → `architecture-reference`; contains `transparency` →
   `concept-law`; else → `general`.

Applied to this Implementation-phase material once Phase 2 lands:
`EnkiDDB_GLOSSARY.md`/`EnkiMDB_GLOSSARY.md` → `glossary` (rule 3);
`EnkiDDB_ROADMAP.md`/`EnkiDDB_MANUAL.md`/their EnkiMDB counterparts →
`general` (no subfolder, no filename match) — worth knowing when writing
a `WHERE E[meta.collection] = ...` query against them later.

## 6. Lifecycle / backup / restore

See `docs/05_storage/EnkiDDB_PODMAN_DEPLOYMENT.md` §"Lifecycle" — not duplicated
here. Summary: `podman stop|start|logs -f enkiddb-write` (or `-read`);
back up the named volume's mounted directory, not the container.

## 7. Visualize / inspect (DubSar Notebook, PB-207)

`workspace/bahyway_v4/godot/dubsar-theater/project.godot` is a real,
openable Godot 4.3 project — the "DubSar HeptaScript Notebook." Open it,
press Play, then **Ctrl+E** for the notebook overlay. The NODE target
picker defaults to EnkiDDB at the real deployed Read Node's host/port.
Type `QUERY:...` or `SEARCH:<top_k>:<phrase>` (§3's exact syntax) and
press Run — the log shows each real row's content (score + text snippet
for `SEARCH:`, kaki + attrs for `QUERY:`), not just a count, so you can
directly see whether semantic search is finding the right documents.
The PDM tab renders the same results as a GEM/ORBIT/FUZZY classification
grid. The theater's 3D particle ring is atmospheric only — not yet wired
to live query results (no GDExtension exists for that today).

## 8. Troubleshooting

| Symptom | Likely cause | Check |
|---|---|---|
| `ERR:not ready` from the Read Node | No sync has run yet, or the Write Node has never `FLUSH`ed | Confirm `DATA_DIR/current` exists on the read host; run the sync script once by hand |
| `WHAT E[*]` returns garbage attribute names | Known limitation, not a bug | Name attributes explicitly in `WHAT` (§3) |
| Documents ingested before a restart are gone | Journal is in-memory only (Phase 1 not done) | Call `FLUSH` more often (`FLUSH_EVERY`), or wait for Phase 1's durability hardening |
| `SEARCH` returns nothing for an obviously-relevant query | RAG index built before the document was ingested, or Read Node hasn't reloaded yet | Wait up to `RELOAD_SECS`, or restart the read container |
| `podman build` fails on the builder stage | Workspace doesn't compile | Run `cargo build --workspace` locally first — the Containerfile does the same build, just inside `rust:1-slim` |
| `ERR:security: <detail>` on ingest | PB-206's malware-signature scan matched the raw bytes (EICAR test string, PE/MZ header, PHP/JS probe) | Check the flagged file's actual content — this is a real hit, not a false positive from formatting |
| `INGEST_DIR`/authorship check rejects everything, even known-good files | Container image missing `git`, or "dubious ownership" (PB-201/202) | Rebuild from current `Containerfile.write-server`; or prefer `enkiddb-cli ingest --remote` (§2a), which never needs git inside the container at all |
| Godot notebook connects to nothing when "EnkiDDB" is selected | `enkiddb-read-server` isn't actually running, or you're on an older checkout with the pre-PB-207 port (7007) | `podman ps \| grep enkiddb-read`; confirm `scripts/enki_engines.gd` has EnkiDDB on port 7102 |
