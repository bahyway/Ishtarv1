#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — OTAP Promotion Script (EnkiDDB rebuild pipeline)
#
# WHAT IT DOES:
#   Promotes work from one OTAP stage branch to the next. See
#   docs/OTAP_PIPELINE.md for the full stage/branch mapping. In short:
#     otap/dev -> otap/test -> otap/accept -> master (Production)
#
#   Every promotion:
#     1. Fetches both branches fresh from origin.
#     2. Checks out <to>, merges <from> into it (fast-forward only --
#        never a merge commit, never a force-push -- if it doesn't
#        fast-forward, the script stops and tells you to reconcile by hand).
#     3. Runs `cargo test --workspace` on the merged result.
#     4. Only pushes <to> if the tests pass.
#
#   Promoting INTO master additionally requires the
#   --i-understand-this-is-production flag. Without it, the script prints
#   what it *would* do and exits without touching anything -- master is
#   the one promotion that results in a real systemd redeploy on
#   enkidb-node-write/enkidb-node-read (see docs/OTAP_PIPELINE.md), so it
#   is never automatic.
#
# USAGE:
#   bash scripts/otap-promote.sh otap/dev otap/test
#   bash scripts/otap-promote.sh otap/test otap/accept
#   bash scripts/otap-promote.sh otap/accept master --i-understand-this-is-production
#
# REQUIRES: bash, git, cargo. Run from anywhere inside the repo.
# =============================================================================
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <from-branch> <to-branch> [--i-understand-this-is-production]" >&2
  exit 1
fi

FROM="$1"
TO="$2"
CONFIRM_PROD="${3:-}"

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [ "$TO" = "master" ] && [ "$CONFIRM_PROD" != "--i-understand-this-is-production" ]; then
  echo "Refusing to promote into master (Production) without the explicit flag."
  echo "This promotion results in a real redeploy on enkidb-node-write/enkidb-node-read."
  echo
  echo "If that is genuinely what you want:"
  echo "  bash $0 $FROM $TO --i-understand-this-is-production"
  exit 1
fi

echo "Fetching $FROM from origin..."
git fetch origin "$FROM"

if git ls-remote --exit-code --heads origin "$TO" >/dev/null 2>&1; then
  echo "Fetching existing $TO from origin..."
  git fetch origin "$TO"
  echo "Checking out $TO..."
  git checkout -B "$TO" "origin/$TO"

  echo "Merging $FROM into $TO (fast-forward only)..."
  if ! git merge --ff-only "origin/$FROM"; then
    echo
    echo "error: $FROM cannot fast-forward into $TO -- they've diverged."
    echo "Reconcile by hand (rebase $TO's unique commits onto $FROM, or vice"
    echo "versa) before re-running this script. Refusing to create a merge"
    echo "commit or force-push automatically."
    exit 1
  fi
else
  echo "$TO does not exist on origin yet -- this is its first promotion."
  echo "Creating $TO fresh from $FROM."
  git checkout -B "$TO" "origin/$FROM"
fi

echo "Running the workspace test suite before pushing..."
if ! (cd workspace/bahyway_v4 && cargo test --workspace); then
  echo
  echo "error: tests failed on the merged result. NOT pushing $TO."
  echo "$TO is left at its pre-promotion state on origin; your local"
  echo "checkout has the failing merge so you can investigate."
  exit 1
fi

echo "Tests passed. Pushing $TO..."
git push origin "$TO"

echo
echo "Promoted $FROM -> $TO successfully."
if [ "$TO" = "master" ]; then
  echo "NEXT STEP (not done by this script): deploy enkiddb-write-server /"
  echo "enkiddb-read-server binaries built from this commit onto"
  echo "enkidb-node-write / enkidb-node-read, per the enkidb-query-server"
  echo "precedent (playbook_34/35)."
fi
