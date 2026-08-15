#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — Documentation Location Collision Report
#
# WHAT IT DOES:
#   You have documentation scattered across multiple local folders (e.g. the
#   Architect's four Forge trees). This script does NOT move or delete
#   anything -- it only reads. It walks every location you give it, finds
#   every Markdown file, and reports:
#     - files whose NAME appears in more than one location, split into:
#         IDENTICAL   -- same sha256 everywhere it appears (safe to just
#                         keep one copy; no content decision needed)
#         DIFFERS     -- same filename, different content in at least one
#                         location (needs a human to pick a winner or merge)
#     - files that exist in only ONE location (nothing to reconcile, just
#         needs moving into the canonical structure)
#
#   This is meant to run BEFORE consolidating several documentation folders
#   into one canonical layout (e.g. EnkiDB's docs/00_codex, docs/02_identity,
#   ... numbered taxonomy) -- so you know exactly what's a real duplicate,
#   what has actually diverged, and what's simply new, before moving a
#   single file.
#
# USAGE:
#   bash scripts/diff-doc-locations.sh <location1> <location2> [location3 ...]
#
# EXAMPLE (the Architect's four Forge trees):
#   bash scripts/diff-doc-locations.sh \
#     /home/bfadam/Forge/bahyway_v4/docs \
#     /home/bfadam/Forge/EnkiDB/workspace/bahyway_v4/docs \
#     /home/bfadam/Forge/EnkiDB/docs \
#     /home/bfadam/Forge/EnkiDB/lab_creation_docs
#
# OUTPUT:
#   Printed to stdout AND written to scripts/logs/doc-collisions-<ts>.txt
#   (relative to wherever this script itself lives, not your cwd).
#
# REQUIRES: bash, find, sha256sum (or shasum on macOS -- auto-detected).
#   No other dependencies -- deliberately plain POSIX-ish tooling so it
#   runs anywhere without installing anything.
# =============================================================================
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <location1> <location2> [location3 ...]" >&2
  echo "Need at least two locations to compare." >&2
  exit 1
fi

SHA_CMD="sha256sum"
if ! command -v sha256sum >/dev/null 2>&1; then
  if command -v shasum >/dev/null 2>&1; then
    SHA_CMD="shasum -a 256"
  else
    echo "error: need sha256sum or shasum on PATH" >&2
    exit 1
  fi
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
TS="$(date +%Y-%m-%d_%H-%M-%S)"
LOG_FILE="$LOG_DIR/doc-collisions-$TS.txt"

LOCATIONS=("$@")
for loc in "${LOCATIONS[@]}"; do
  if [ ! -d "$loc" ]; then
    echo "error: not a directory: $loc" >&2
    exit 1
  fi
done

# TSV: filename<TAB>location<TAB>full_path<TAB>sha256
MANIFEST="$(mktemp)"
trap 'rm -f "$MANIFEST"' EXIT

for loc in "${LOCATIONS[@]}"; do
  while IFS= read -r -d '' f; do
    hash="$($SHA_CMD "$f" | awk '{print $1}')"
    printf '%s\t%s\t%s\t%s\n' "$(basename "$f")" "$loc" "$f" "$hash" >> "$MANIFEST"
  done < <(find "$loc" -type f -name '*.md' -print0)
done

{
  echo "Documentation Location Collision Report — $TS"
  echo "Locations compared:"
  for loc in "${LOCATIONS[@]}"; do echo "  - $loc"; done
  echo

  names="$(cut -f1 "$MANIFEST" | sort -u)"

  echo "=== IDENTICAL across locations (safe to dedupe, no content decision) ==="
  identical_count=0
  while IFS= read -r name; do
    [ -z "$name" ] && continue
    rows="$(awk -F'\t' -v n="$name" '$1==n' "$MANIFEST")"
    count="$(printf '%s\n' "$rows" | wc -l)"
    [ "$count" -le 1 ] && continue
    uniq_hashes="$(printf '%s\n' "$rows" | cut -f4 | sort -u | wc -l)"
    if [ "$uniq_hashes" -eq 1 ]; then
      identical_count=$((identical_count + 1))
      echo "  $name"
      printf '%s\n' "$rows" | awk -F'\t' '{print "      " $3}'
    fi
  done <<< "$names"
  echo "  ($identical_count identical filename(s))"
  echo

  echo "=== DIFFERS across locations (needs a human decision) ==="
  differs_count=0
  while IFS= read -r name; do
    [ -z "$name" ] && continue
    rows="$(awk -F'\t' -v n="$name" '$1==n' "$MANIFEST")"
    count="$(printf '%s\n' "$rows" | wc -l)"
    [ "$count" -le 1 ] && continue
    uniq_hashes="$(printf '%s\n' "$rows" | cut -f4 | sort -u | wc -l)"
    if [ "$uniq_hashes" -gt 1 ]; then
      differs_count=$((differs_count + 1))
      echo "  $name  ($uniq_hashes distinct version(s))"
      printf '%s\n' "$rows" | awk -F'\t' '{print "      [" $4 "]  " $3}'
    fi
  done <<< "$names"
  echo "  ($differs_count file(s) genuinely diverged)"
  echo

  echo "=== UNIQUE to one location (just needs moving, nothing to reconcile) ==="
  unique_count=0
  while IFS= read -r name; do
    [ -z "$name" ] && continue
    rows="$(awk -F'\t' -v n="$name" '$1==n' "$MANIFEST")"
    count="$(printf '%s\n' "$rows" | wc -l)"
    if [ "$count" -eq 1 ]; then
      unique_count=$((unique_count + 1))
      printf '%s\n' "$rows" | awk -F'\t' '{print "  " $3}'
    fi
  done <<< "$names"
  echo "  ($unique_count file(s) unique to a single location)"

} | tee "$LOG_FILE"

echo
echo "Full report saved to: $LOG_FILE"
