#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — EnkiMDB Write-Node -> Read-Node Data Sync
#
# WHAT IT DOES:
#   Copies the Write Node's materialized Euphrates generation
#   (DATA_DIR/current/{entities,eav} on enkimdb-node-write) to the Read
#   Node's data volume (DATA_DIR/current on enkimdb-node-read), so
#   enkimdb-read-server's background reload picks up new catalog scans.
#
# Same pattern as scripts/enkiddb-sync-data.sh (see that script's header
# for the full rationale: two separate VMs, plain rsync/scp today,
# enkidb-replication's KAKI-sealed pipeline as the honest future hardening
# step) -- not re-explained here, just re-applied to EnkiMDB.
#
# USAGE (run ON enkimdb-node-write, pointed at enkimdb-node-read):
#   bash scripts/enkimdb-sync-data.sh <read_node_host> [remote_user] [--loop <seconds>]
#
# EXAMPLE:
#   bash scripts/enkimdb-sync-data.sh enkimdb-node-read bahaa --loop 60
#
# REQUIRES: rsync + ssh key-based access from enkimdb-node-write to
#   enkimdb-node-read. Falls back to `scp -r` if rsync isn't installed.
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

LOCAL_DATA_DIR="${DATA_DIR:-/data}/current"
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
