#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — EnkiDDB Write-Node -> Read-Node Data Sync
#
# WHAT IT DOES:
#   Copies the Write Node's materialized Tigris generation
#   (DATA_DIR/current/{entities,eav} on enkidb-node-write) to the Read
#   Node's data volume (DATA_DIR/current on enkidb-node-read), so
#   enkiddb-read-server's background reload picks up new writes.
#
# WHY THIS ISN'T A THIRD PODMAN CONTAINER:
#   enkidb-write-server and enkidb-read-server run on two SEPARATE VMs.
#   A container can only see its own host's volumes -- bridging two
#   remote hosts needs either a shared network filesystem (heavier ops
#   burden, new infra) or a network copy. This repo's own established
#   convention (see docs/08_pipeline_alaktu/OTAP_PIPELINE.md's citation of playbook_145) is
#   already scp between machines -- this script is that same pattern,
#   just automated and run on a loop, not a new mechanism.
#
#   `enkidb-replication` (crates/enkidb-replication) already implements a
#   more rigorous version of this exact bridge -- KAKI-sealed
#   (Ed25519-signed), chain-hashed, 7-layer-verified. It is real, tested,
#   and designed for precisely this Write-Pod -> Broker -> Read-Pod
#   topology, but wiring it in (key generation/distribution, the
#   HeptaSecSentinel firewall gate) is real additional work not done in
#   this pass -- this script is the honest, working v1; swapping in
#   enkidb-replication later is the natural hardening step, not a redesign.
#
# USAGE (run ON enkidb-node-write, pointed at enkidb-node-read):
#   bash scripts/enkiddb-sync-data.sh <read_node_host> [remote_user] [--loop <seconds>]
#
# EXAMPLE:
#   bash scripts/enkiddb-sync-data.sh enkidb-node-read bahaa --loop 60
#
# REQUIRES: rsync + ssh key-based access from enkidb-node-write to
#   enkidb-node-read (the same access playbook_145's scp sync convention
#   already assumes). Falls back to `scp -r` if rsync isn't installed.
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
