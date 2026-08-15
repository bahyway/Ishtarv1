# EnkiDDB (Tigris) — Podman Deployment Guide

**Sealed:** 2026-07-12. Deploys `enkiddb-write-server`/`enkiddb-read-server`
onto the Architect's existing `enkidb-node-write`/`enkidb-node-read` VMs as
Podman containers — no systemd units, plain `podman run`/`stop`/`start`.
**Updated 2026-07-19 (PB-199):** the Write Node now also bind-mounts the
repo checkout read-only at `/source`, matching EnkiMDB's write container —
without it, PB-198's `INGEST_DIR:<path>` command has nothing to scan.

## Prerequisites

- Podman installed on both `enkidb-node-write` and `enkidb-node-read`.
- The current `master`-promoted commit (see `docs/OTAP_PIPELINE.md`) present
  on both VMs at `~/Forge/EnkiDB` (same scp sync convention as everything
  else in this repo — `playbook_145`).
- SSH key-based access from `enkidb-node-write` to `enkidb-node-read` (for
  the data sync step) — the same access the existing scp workflow assumes.

## 1. Build both images (on each VM, or build once and `podman save`/`podman load` across)

```bash
cd ~/Forge/EnkiDB
podman build -t enkiddb-write-server:latest \
  -f deploy/podman/Containerfile.write-server workspace/bahyway_v4
podman build -t enkiddb-read-server:latest \
  -f deploy/podman/Containerfile.read-server workspace/bahyway_v4
```

## 2. Run the Write Node (on `enkidb-node-write`)

```bash
podman volume create enkiddb-write-data
podman run -d --name enkiddb-write \
  -p 7101:7101 \
  -v enkiddb-write-data:/data:Z \
  -v ~/Forge/EnkiDB:/source:ro,z \
  -e TRIBE_ID=0x7160 \
  -e FLUSH_EVERY=10 \
  -e ENKIDDB_TEAM_ALLOWLIST=/source/config/enkiddb_team_allowlist.txt \
  --restart=always \
  enkiddb-write-server:latest
```

## 3. Run the Read Node (on `enkidb-node-read`)

```bash
podman volume create enkiddb-read-data
podman run -d --name enkiddb-read \
  -p 7102:7102 \
  -v enkiddb-read-data:/data:Z \
  -e RELOAD_SECS=30 \
  --restart=always \
  enkiddb-read-server:latest
```

At this point the Read Node answers `ERR:not ready` — it has no data
until the first sync.

## 4. Start the sync loop (on `enkidb-node-write`)

```bash
DATA_DIR="$(podman volume inspect enkiddb-write-data --format '{{.Mountpoint}}')" \
  bash scripts/enkiddb-sync-data.sh enkidb-node-read bahaa --loop 60
```

Run this under `podman run -d` too if you want it container-managed
(any minimal image with `rsync`/`ssh` + this script mounted in works —
deliberately left as a plain script, not baked into its own image, since
it's the piece most likely to be swapped for `enkidb-replication`'s
KAKI-sealed pipeline once that's wired in), or as a background process,
or just re-run it by hand whenever you want to push new documents through.

## 5. Ingest a document (against the Write Node)

Any TCP client sending the wire protocol works. Example with the plain
frame format (`<collection>\n<markdown>`, length-prefixed u32 LE) — see
`bin/enkiddb-write-server/src/main.rs`'s module doc for the exact frames.

**Security-scanned before anything else (PB-206):** every document,
regardless of which path it arrives by, is scanned by
`enkiddb::scan_document` (wrapping `musaru_security::zip_scan` — the real
engine behind the "Nergal" visualization panel) before `DocumentParser`
ever runs. A known malware/webshell/dropper byte signature gets
`ERR:security: <detail>` and is never parsed or journaled.

**Bulk-load a whole directory (PB-198):** `INGEST_DIR:/source/docs` —
categorizes and chunks every `.md` file under `/source/docs` in one call,
gated by `config/enkiddb_team_allowlist.txt` (mounted in at step 2). Add
real team-member emails to that file and restart the container to change
who's trusted; no image rebuild needed. Follow with `FLUSH` to
materialize immediately rather than waiting for the next
`FLUSH_EVERY`-document auto-flush.

**Bulk-load from the VDI instead (PB-205, preferred for client-facing
ingest):** `enkiddb-cli ingest <dir> --remote <write-node-host>:7101` runs
SCAN + CATEGORIZE (git-authorship check) on the VDI's own real checkout —
correct host-side ownership, git already installed — then sends each
authorized document to the container over the same plain wire command
`INGEST_DIR` uses under the hood. The container never sees a directory
path and never runs `git` for this path; `INGEST_DIR` and the
Containerfile's git/`safe.directory` fix remain available for local/dev
use, this is an additional path, not a replacement.

## 5b. Central Shared Asset Storage (PB-205)

`enkiddb-asset-server` (`deploy/podman/Containerfile.asset-server`, port
7301) is a dedicated, pure-Rust, read-only HTTP file server for physical
assets (markdown/scripts/images/SVGs) that Write and Read nodes fetch by
pointer path instead of bind-mounting the host directory themselves. It
is the sole claimant of its own volume (`:Z`, not `:z`) — with no second
container ever mounting that path, PB-200's class of bug (two containers
racing for an exclusive SELinux label on one shared mount) has no second
claimant to race with. See `playbooks/playbook_205_asset_server_and_remote_ingest_3podman_vdi.yml`
for the deploy steps.

## 6. Query / search (against the Read Node)

- `QUERY:WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.collection] = "component"`
- `SEARCH:5:KAKI sovereign identity`

## Lifecycle — the operations the Architect specifically wanted

```bash
podman stop enkiddb-write        # / enkiddb-read
podman start enkiddb-write
podman logs -f enkiddb-write
```

**Backup:** back up the volume, not the container — the volume is where
state lives; the container is disposable.

```bash
podman volume inspect enkiddb-write-data --format '{{.Mountpoint}}'
tar czf enkiddb-write-backup-$(date +%Y%m%d).tar.gz -C <that path> .
```

**Restore:** stop the container, replace the volume's contents from a
backup tarball, start the container again.

## Honest scope of this v1

- **Durability**: the Write Node's Journal is in-memory only; the volume
  only has what survived the last flush (`FLUSH_EVERY` documents, or an
  explicit `FLUSH` request). A container restart between flushes loses
  unflushed writes. See `enkiddb-write-server`'s own doc comment.
- **Sync**: plain rsync/scp on a timer, not `enkidb-replication`'s
  KAKI-sealed, chain-hashed, 7-layer-verified pipeline — that crate exists,
  fully built and tested, and is the natural hardening step, but wiring it
  in (key management, `HeptaSecSentinel`) is separate, not-yet-started work.
- **ENLIL 7-index stack / HeptaScript v2.0 unification**: not part of this
  deployment — both remain blocked on the Architect supplying the missing
  specifics (see `docs/OTAP_PIPELINE.md`'s "definition of done").
