#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — EnkiDW (Data Warehouse) Write-Node -> Read-Node Data Sync
#
# WHAT IT DOES:
#   Copies the Write Node's materialized generation
#   (DATA_DIR/current/{entities,eav}, DATA_DIR defaults to /data/dw on
#   enkidb-node-write) to the Read Node's data volume (DATA_DIR/current,
#   defaults to /data on enkidb-node-read), so enkidw-read-server's
#   background reload picks up newly-ingested/ETL'd records.
#
# Same pattern as scripts/enkiddb-sync-data.sh (see that script's header
# for the full rationale) -- not re-explained here, just re-applied to
# EnkiDW. Note the asymmetric default DATA_DIR between write (/data/dw,
# matching enkidw-write-server's own default) and read (/data, matching
# enkidw-read-server's own default + this fleet's other single-type
# read containers) -- override with the DATA_DIR / REMOTE_DATA_DIR env
# vars if your deployment differs.
#
# USAGE (run ON enkidb-node-write, pointed at enkidb-node-read):
#   bash scripts/enkidw-sync-data.sh <read_node_host> [remote_user] [--loop <seconds>]
#
# EXAMPLE:
#   bash scripts/enkidw-sync-data.sh enkidb-node-read bahaa --loop 60
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

LOCAL_DATA_DIR="${DATA_DIR:-/data/dw}/current"
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
