# EnkiMDB (Euphrates) — Podman Deployment Guide

**Sealed:** 2026-07-18. Deploys `enkimdb-write-server`/`enkimdb-read-server`
as Podman containers — no systemd units, plain `podman run`/`stop`/`start`,
same convention as `docs/05_storage/EnkiDDB_PODMAN_DEPLOYMENT.md`.

EnkiMDB (sovereign name: **Euphrates**) is the metadata catalog — it
records every workspace crate and playbook as a real, KAKI-sealed EAV
particle (name/kind/path/version), scanned directly off disk, never
hand-typed or bootstrapped. It is not a source-code blob store: EnkiDDB
(Tigris) is where full document/markdown text lives; EnkiMDB is the
sovereign index of *what exists*.

## Prerequisites

- Podman installed on the VM(s) running these containers.
- The current commit of this repo checked out at a known path (e.g.
  `~/Forge/EnkiDB`) — this is what gets bind-mounted read-only into the
  Write Node container so it has something to scan.
- SSH key-based access from the write host to the read host (for the data
  sync step), same as EnkiDDB's deployment.

## 1. Build both images

```bash
cd ~/Forge/EnkiDB
podman build -t enkimdb-write-server:latest \
  -f deploy/podman/Containerfile.enkimdb-write-server workspace/bahyway_v4
podman build -t enkimdb-read-server:latest \
  -f deploy/podman/Containerfile.enkimdb-read-server workspace/bahyway_v4
```

## 2. Run the Write Node

Two volumes: `/data` (this node's own state) and a **read-only** bind
mount of the actual source checkout at `/source` — the Write Node scans
that path on request, it does not embed or copy the source into the image.

```bash
podman volume create enkimdb-write-data
podman run -d --name enkimdb-write \
  -p 7201:7201 \
  -v enkimdb-write-data:/data:Z \
  -v ~/Forge/EnkiDB:/source:ro,z \
  -e TRIBE_ID=0xFF01 \
  -e FLUSH_EVERY=50 \
  --restart=always \
  enkimdb-write-server:latest
```

## 3. Run the Read Node

```bash
podman volume create enkimdb-read-data
podman run -d --name enkimdb-read \
  -p 7202:7202 \
  -v enkimdb-read-data:/data:Z \
  -e RELOAD_SECS=30 \
  --restart=always \
  enkimdb-read-server:latest
```

At this point the Read Node answers `ERR:not ready` — it has no data
until the first sync.

## 4. Start the sync loop (on the write host)

```bash
DATA_DIR="$(podman volume inspect enkimdb-write-data --format '{{.Mountpoint}}')" \
  bash scripts/enkimdb-sync-data.sh enkimdb-node-read bahaa --loop 60
```

Same honest v1 as EnkiDDB's: plain rsync/scp on a timer, not
`enkidb-replication`'s KAKI-sealed pipeline — see that script's header.

## 5. Catalog the workspace (against the Write Node)

Any TCP client sending the wire protocol works (length-prefixed u32 LE
frame; see `bin/enkimdb-write-server/src/main.rs`'s module doc):

- `SCAN_CRATES:/source/workspace/bahyway_v4` — walks
  `crates/*/Cargo.toml`, journals one artifact per crate found. Returns
  `OK:INGESTED:<count>`.
- `SCAN_PLAYBOOKS:/source` — walks `playbooks/*.yml`, journals one
  artifact per playbook found. Returns `OK:INGESTED:<count>`.
- `FLUSH` — force-materialize now (also happens automatically every
  `FLUSH_EVERY` ingest calls). Returns `OK:FLUSHED:<entity_count>`.

Re-run `SCAN_CRATES`/`SCAN_PLAYBOOKS` after every commit you want
reflected in the catalog — each call journals a fresh Event-Kaki per
artifact found at that moment; it does not diff against what's already
catalogued (a re-scan after no changes simply re-records the same
artifacts under new epochs, which is safe but not deduplicated in v1).

## 6. Query the catalog (against the Read Node)

**Name every attribute you want back explicitly in `WHAT` — a bare
`WHAT E[*]` returns hash-keyed placeholder attribute names, not the real
ones** (the same materialized-Data-Files limitation
`docs/05_storage/EnkiDDB_PODMAN_DEPLOYMENT.md` and PB-181 already documented; there is no
reverse hash map). Verified live during PB-192's smoke test:

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

`artifact.version` reads `"workspace"` (not a literal semver) whenever the
crate declares `version.workspace = true`, which is every crate in this
workspace — the real value lives in the root `Cargo.toml`, and
`enkimdb::scan::parse_package_name_version` records that fact rather than
resolving it, per that function's own doc comment.

## Lifecycle

```bash
podman stop enkimdb-write        # / enkimdb-read
podman start enkimdb-write
podman logs -f enkimdb-write
```

**Backup:** back up the volume, not the container.

```bash
podman volume inspect enkimdb-write-data --format '{{.Mountpoint}}'
tar czf enkimdb-write-backup-$(date +%Y%m%d).tar.gz -C <that path> .
```

**Restore:** stop the container, replace the volume's contents from a
backup tarball, start the container again.

## Honest scope of this v1

- **Durability**: same in-memory-Journal limitation as EnkiDDB — a
  container restart between flushes loses catalog entries ingested since
  the last `FLUSH`. Re-running `SCAN_CRATES`/`SCAN_PLAYBOOKS` after a
  restart repopulates them from disk, since the scan itself is always a
  fresh read of the real filesystem, not a diff against prior state.
- **Sync**: plain rsync/scp on a timer, identical honest scope to
  EnkiDDB's — `enkidb-replication` is the real hardening step, not wired
  in here.
- **No deduplication across re-scans**: each `SCAN_CRATES`/
  `SCAN_PLAYBOOKS` call journals every matching artifact again, even if
  nothing changed since the last scan. Harmless (the Read Node just sees
  more Event-Kakis for the same Identity-Kaki, and a `WHERE` query still
  finds the artifact), but not the same as a true delta-ingest.
- **No source-code text**: EnkiMDB catalogs *that* a crate/playbook
  exists (name, kind, path, version) — it does not embed the crate's
  actual `.rs` source or the playbook's YAML body. Ingesting the actual
  text content of those files (for search, not just cataloging) is
  EnkiDDB's job, via its existing directory-upload path — pointing that
  at `workspace/bahyway_v4/crates/**/*.rs` and `playbooks/*.yml` is a
  separate, not-yet-run step, not something this deployment does.
