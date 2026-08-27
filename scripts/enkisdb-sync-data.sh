#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — EnkiSDB (Stage) Write-Node -> Read-Node Data Sync
#
# WHAT IT DOES:
#   Copies the Write Node's materialized sdb/ generation
#   (SDB_DATA_DIR/current, defaults to /data/sdb on enkidb-node-write --
#   one of THREE trees enkisdb-write-server's single process materializes,
#   see deploy/podman/Containerfile.enkisdb-write-server's header) to the
#   EnkiSDB Read Node's data volume (DATA_DIR/current, defaults to /data
#   on enkidb-node-read), so enkisdb-read-server's background reload
#   picks up newly-promoted/quarantined Stage particles.
#
#   This syncs ONLY the sdb/ tree. EnkiODB and EnkiQDB are separate Read
#   Nodes fed from the SAME write process's other two trees -- see
#   scripts/enkiodb-sync-data.sh and scripts/enkiqdb-sync-data.sh, run
#   alongside this one, not instead of it.
#
# Same pattern as scripts/enkiddb-sync-data.sh (see that script's header
# for the full rationale) -- not re-explained here, just re-applied to
# EnkiSDB's sdb/ tree.
#
# USAGE (run ON enkidb-node-write, pointed at enkidb-node-read):
#   bash scripts/enkisdb-sync-data.sh <read_node_host> [remote_user] [--loop <seconds>]
#
# EXAMPLE:
#   bash scripts/enkisdb-sync-data.sh enkidb-node-read bahaa --loop 60
#
# REQUIRES: rsync + ssh key-based access from enkidb-node-write to
#   enkidb-node-read. Falls back to `scp -r` if rsync isn't installed.
# =============================================================================
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <read_node_host> [remote_user] [--loop <seconds>]" >&2
  exit 1
fi

READ_HOST="$1"
REMOTE_USER="${2:-bahaa}"
LOOP_SECS=""
if [ "${3:-}" = "--loop" ]; then
  LOOP_SECS="${4:-60}"
fi

LOCAL_DATA_DIR="${DATA_DIR:-/data/sdb}/current"
REMOTE_DATA_DIR="${REMOTE_DATA_DIR:-/data}/current"

sync_once() {
  if [ ! -d "$LOCAL_DATA_DIR" ]; then
    echo "$(date -Iseconds) -- nothing materialized yet at $LOCAL_DATA_DIR, skipping"
    return 0
  fi

  if command -v rsync >/dev/null 2>&1; then
    rsync -az --delete "$LOCAL_DATA_DIR/" "${REMOTE_USER}@${READ_HOST}:${REMOTE_DATA_DIR}/"
  else
    ssh "${REMOTE_USER}@${READ_HOST}" "mkdir -p '$REMOTE_DATA_DIR'"
    scp -r "$LOCAL_DATA_DIR"/* "${REMOTE_USER}@${READ_HOST}:${REMOTE_DATA_DIR}/"
  fi
  echo "$(date -Iseconds) -- synced $LOCAL_DATA_DIR -> ${READ_HOST}:${REMOTE_DATA_DIR}"
}

if [ -n "$LOOP_SECS" ]; then
  echo "Looping every ${LOOP_SECS}s. Ctrl-C to stop."
  while true; do
    sync_once || echo "$(date -Iseconds) -- sync failed, will retry next tick"
    sleep "$LOOP_SECS"
  done
else
  sync_once
fi
