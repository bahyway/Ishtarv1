#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — EnkiODB (Operational Database) Write-Node -> Read-Node Data
# Sync
#
# WHAT IT DOES:
#   Copies enkisdb-write-server's materialized odb/ generation
#   (ODB_DATA_DIR/current, defaults to /data/odb on enkidb-node-write --
#   one of THREE trees that single process materializes, see
#   deploy/podman/Containerfile.enkisdb-write-server's header) to the
#   EnkiODB Read Node's data volume (DATA_DIR/current, defaults to /data
#   on enkidb-node-read), so enkiodb-read-server's background reload
#   picks up particles `OdbStore::drain_from_sdb` just promoted.
#
#   There is no separate EnkiODB write process to run this against --
#   the source tree lives on the SAME host/volume as EnkiSDB's, produced
#   by enkisdb-write-server. Run alongside scripts/enkisdb-sync-data.sh
#   and scripts/enkiqdb-sync-data.sh, not instead of them.
#
# Same pattern as scripts/enkiddb-sync-data.sh (see that script's header
# for the full rationale) -- not re-explained here, just re-applied to
# EnkiODB's odb/ tree.
#
# USAGE (run ON enkidb-node-write, pointed at enkidb-node-read):
#   bash scripts/enkiodb-sync-data.sh <read_node_host> [remote_user] [--loop <seconds>]
#
# EXAMPLE:
#   bash scripts/enkiodb-sync-data.sh enkidb-node-read bahaa --loop 60
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

LOCAL_DATA_DIR="${DATA_DIR:-/data/odb}/current"
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
