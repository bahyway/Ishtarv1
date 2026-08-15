# EnkiMDB (Euphrates) — Operations Manual

**Audience:** engineers deploying, operating, or troubleshooting EnkiMDB.
Assumes familiarity with Rust, Podman, and this repo's KAKI/EAV/CQRS
vocabulary — see `EnkiMDB_GLOSSARY.md` if any term here is unfamiliar.

## 1. Deploy

Full build/run/volume/lifecycle instructions live in
`docs/EnkiMDB_PODMAN_DEPLOYMENT.md` — not duplicated here. Short version:

```bash
ansible-playbook playbooks/playbook_192_enkiddb_enkimdb_podman_deploy_executable.yml \
  -e enkidb_repo_root=~/Forge/EnkiDB
```

builds both images and runs both containers (write on 7201, read on
7202), idempotently — the same single playbook that deploys EnkiDDB.

**One thing EnkiDDB's deploy doesn't need:** the write container must
also bind-mount the actual repo checkout, read-only, at `/source` — it
scans real files on disk, it doesn't receive them over the wire like
EnkiDDB's document ingest does. The playbook already does this
(`-v <repo>:/source:ro,Z`); if you're running `podman run` by hand
instead, don't forget that mount.

## 2. Catalog the workspace

Send these over the TCP wire protocol (length-prefixed u32 LE frame; see
`bin/enkimdb-write-server/src/main.rs`'s module doc for the exact byte
layout) to the write container, port 7201:

- `SCAN_CRATES:/source/workspace/bahyway_v4` — walks
  `crates/*/Cargo.toml`, journals one artifact per crate found. Response:
  `OK:INGESTED:<count>`.
- `SCAN_PLAYBOOKS:/source` — walks `playbooks/*.yml`, journals one
  artifact per playbook found. Response: `OK:INGESTED:<count>`.
- `FLUSH` — force-materialize now (also happens automatically every
  `FLUSH_EVERY` ingest calls, default 50). Response:
  `OK:FLUSHED:<entity_count>`.

Verified live during PB-192: 151 real crates and 88 real playbooks
catalogued and materialized this way in this repo's own workspace.

**Re-run after every commit you want reflected.** There is no diffing —
see `EnkiMDB_ROADMAP.md`'s Phase 1/2. A re-scan after no changes is safe
but redundant, not a no-op.

## 3. Query the catalog (against the Read Node)

One request form: `QUERY:<heptascript source>`, port 7202. **Name every
attribute you want back explicitly in `WHAT`** — same placeholder-hash
limitation as EnkiDDB (`EnkiDDB_MANUAL.md` §3), verified live here too:

```
QUERY:WHO T.E
WHAT E[artifact.name, artifact.kind, artifact.path, artifact.version]
WHERE E[artifact.name] = "enkiddb"
```

returns

```json
[{"kaki":"...","attrs":[["artifact.name","enkiddb"],["artifact.kind","Crate"],
  ["artifact.path","/source/workspace/bahyway_v4/crates/enkiddb/Cargo.toml"],
  ["artifact.version","workspace"]]}]
```

`artifact.version` reads the literal string `"workspace"` (not a
resolved semver) for any crate declaring `version.workspace = true` —
which is every crate in this workspace. `enkimdb::scan::
parse_package_name_version` records that fact rather than resolving it
against the root manifest, per that function's own doc comment.

No `SEARCH` command exists here — EnkiMDB has no RAG index (§"What
EnkiMDB is" in the Roadmap explains why). If you need full-text search
over crate source or playbook YAML, that's EnkiDDB's job, not this
server's.

## 4. Keep the Read Node in sync

```bash
DATA_DIR="$(podman volume inspect enkimdb-write-data --format '{{.Mountpoint}}')" \
  bash scripts/enkimdb-sync-data.sh enkimdb-node-read bahaa --loop 60
```

Same pattern as `EnkiDDB_MANUAL.md` §4 — see that script's header for
the full rationale (not re-explained here).

## 5. Lifecycle / backup / restore

See `docs/EnkiMDB_PODMAN_DEPLOYMENT.md` §"Lifecycle" — not duplicated
here. Summary: `podman stop|start|logs -f enkimdb-write` (or `-read`);
back up the named volume's mounted directory, not the container.

## 6. Troubleshooting

| Symptom | Likely cause | Check |
|---|---|---|
| `SCAN_CRATES`/`SCAN_PLAYBOOKS` returns `OK:INGESTED:0` | Path given doesn't actually contain `crates/`/`playbooks/`, or the write container's `/source` mount is missing/wrong | Confirm the path argument matches the container's mounted path, not the host path |
| `ERR:scan_crates: ...` / `ERR:scan_playbooks: ...` | The given root directory doesn't exist inside the container | Check the `-v <repo>:/source:ro,Z` mount is present on the running container (`podman inspect enkimdb-write`) |
| `ERR:not ready` from the Read Node | No sync has run yet, or the Write Node has never `FLUSH`ed | Confirm `DATA_DIR/current` exists on the read host; run the sync script once by hand |
| `WHAT E[*]` returns garbage attribute names | Known limitation, not a bug | Name attributes explicitly in `WHAT` (§3) |
| Catalog has duplicate-looking entries for the same crate | Expected — no deduplication across re-scans (§2, Roadmap Phase 1) | Each Event-Kaki is a real, distinct journal entry; `WHERE` queries still resolve correctly, this is not data corruption |
| `podman build` fails on the builder stage | Workspace doesn't compile | Run `cargo build --workspace` locally first |
