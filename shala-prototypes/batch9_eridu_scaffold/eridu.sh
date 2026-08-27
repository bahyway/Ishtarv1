#!/usr/bin/env bash
# ============================================================================
# eridu.sh — EriduScaffold
#   The first ground. A scaffolder-and-scribe (structure, not engine).
#   NL-001: city-name → structure. GL-DOC-001: named because USED daily.
# ----------------------------------------------------------------------------
# ROLE (kept narrow, complementary to Claude-Code's infra PBs):
#   Eridu LAYS and TRACKS ground. It does NOT provision databases or
#   containers — those are the CQRS/Podman infra PBs. Eridu builds the
#   OTA-INTERNAL folder structure and keeps an append-only record of how
#   that ground EVOLVES over time (Sedimentation, GL-STY-001).
#
# FLAT-FILE FIRST (this version):
#   - zero dependencies: filesystem only. Runs even when EnkiDB is down.
#   - evolution tracked in journal/evolution.log (append-only) + a
#     snapshot each run, diffed against the previous snapshot.
#   - an EnkiDB-emit hook is STUBBED (emit_to_enkidb) for the graduation
#     step — off by default, switched on later. A scaffolder must never
#     hard-depend on the thing it scaffolds for.
#
# GRADUATION TIMELINE (written into the tool, see `eridu.sh roadmap`):
#   Stage 0  flat-file scaffold + evolution log            ← YOU ARE HERE
#   Stage 1  emit OTA structure as particles to EnkiSDB (landing)
#   Stage 2  BeeMDM ETL: SDB→ODB→QDB gates over the structure feed
#   Stage 3  proven shape reaches EnkiDB GOLDEN store (7004)
#   Stage 4  self-hosting: Eridu reads its own OntoGraph back from GOLDEN
#
# Usage:
#   ./eridu.sh init                 # create OTA-internal structure (safe re-run)
#   ./eridu.sh scan                 # snapshot the ground; append evolution delta
#   ./eridu.sh evolution            # show how the ground has changed over time
#   ./eridu.sh roadmap              # print the graduation timeline + where you are
#   ./eridu.sh emit                 # (stub) emit structure to EnkiDB — Stage 1+
#   ./eridu.sh                      # = init + scan + evolution  (daily ground pass)
# ============================================================================
set -euo pipefail

ROOT="${ERIDU_ROOT:-$HOME/BahyWay}"
OTA="${ERIDU_OTA:-$ROOT/OTA}"
STAMP="$(date +%Y-%m-%dT%H:%M:%S)"
DATE="$(date +%Y-%m-%d)"

SNAP_DIR="$OTA/.eridu/snapshots"
EVO_LOG="$OTA/.eridu/evolution.log"
LAST_SNAP="$OTA/.eridu/last.snapshot"

# --- EnkiDB emit toggle (Stage 1+). Off by default: flat-file is self-sufficient.
ERIDU_EMIT_ENKIDB="${ERIDU_EMIT_ENKIDB:-0}"

# ----------------------------------------------------------------------------
# The OTA-INTERNAL layout. Lean — every folder earns its place; no empty
# 45-node skeletons. This is the ground work LANDS on inside an OTA env.
ota_dirs() {
  cat <<'EOF'
inbox
staging/landing
staging/inferred
staging/proven
playbooks
prototypes
docs
docs/glossary
evolution
EOF
}

init_structure() {
  mkdir -p "$SNAP_DIR"
  while read -r d; do
    [ -n "$d" ] && mkdir -p "$OTA/$d"
  done < <(ota_dirs)

  # a README that states the ground's contract, so it's self-describing
  [ -f "$OTA/README.md" ] || cat > "$OTA/README.md" <<EOF
# OTA environment — internal ground (scaffolded by Eridu)
Laid: $STAMP

This is the OTA-internal structure. Eridu builds and tracks it; it does
NOT provision the databases or containers (those are infra PBs).

- inbox/              unsorted arrivals
- staging/landing/    raw structure feed (→ EnkiSDB at Stage 1)
- staging/inferred/   after inference gate
- staging/proven/     after proof gate (→ GOLDEN at Stage 3)
- playbooks/          OTA-scoped PBs
- prototypes/         Šala rehearsals under test in OTA
- docs/               OTA docs + glossary
- evolution/          human-readable evolution reports

Graduation: this flat ground evolves into the BeeMDM ETL pipeline
(SDB→ODB→QDB) until the proven shape reaches EnkiDB GOLDEN. See:
    ./eridu.sh roadmap
EOF
  echo "✓ OTA ground laid at $OTA"
}

# ----------------------------------------------------------------------------
# SCAN: snapshot the current ground (path + file-count + total-bytes per dir),
# diff against the last snapshot, append the delta to the evolution log.
take_snapshot() {
  local out="$1"
  # for each tracked dir: path | files | bytes  (excluding the .eridu meta dir)
  while read -r d; do
    [ -z "$d" ] && continue
    local p="$OTA/$d"
    [ -d "$p" ] || continue
    local files bytes
    files=$(find "$p" -type f 2>/dev/null | wc -l | tr -d ' ')
    bytes=$(find "$p" -type f -printf '%s\n' 2>/dev/null | awk '{s+=$1} END{print s+0}')
    printf '%s|%s|%s\n' "$d" "$files" "$bytes"
  done < <(ota_dirs) | sort > "$out"
}

scan() {
  init_structure >/dev/null
  local snap="$SNAP_DIR/$STAMP.snapshot"
  take_snapshot "$snap"

  local new_dirs=0 grew=0 added_files=0
  if [ -f "$LAST_SNAP" ]; then
    # compare per-dir file counts
    while IFS='|' read -r d files bytes; do
      local prev
      prev=$(grep -E "^$d\|" "$LAST_SNAP" 2>/dev/null | head -1 || true)
      if [ -z "$prev" ]; then
        new_dirs=$((new_dirs+1))
      else
        local pf; pf=$(echo "$prev" | cut -d'|' -f2)
        if [ "$files" -gt "$pf" ]; then
          grew=$((grew+1)); added_files=$((added_files + files - pf))
        fi
      fi
    done < "$snap"
  else
    new_dirs=$(wc -l < "$snap" | tr -d ' ')
  fi

  # append-only evolution log line (never rewritten — Sedimentation)
  printf '%s | new_dirs=%s grew=%s +files=%s | snapshot=%s\n' \
    "$STAMP" "$new_dirs" "$grew" "$added_files" "$(basename "$snap")" >> "$EVO_LOG"

  cp -f "$snap" "$LAST_SNAP"

  echo "✓ ground scanned: $new_dirs new dir(s), $grew grew, +$added_files file(s)"

  # optional Stage-1 emit
  if [ "$ERIDU_EMIT_ENKIDB" = "1" ]; then emit_to_enkidb "$snap"; fi
}

# ----------------------------------------------------------------------------
show_evolution() {
  echo ""
  echo "  Eridu — how the OTA ground has evolved"
  echo "  ─────────────────────────────────────────────"
  if [ -f "$EVO_LOG" ]; then
    # last 15 evolution lines, most recent last
    tail -n 15 "$EVO_LOG" | while IFS= read -r line; do echo "  $line"; done
  else
    echo "  (no evolution yet — run: $0 scan)"
  fi
  echo "  ─────────────────────────────────────────────"
  # current ground summary
  if [ -f "$LAST_SNAP" ]; then
    local tot_files tot_bytes
    tot_files=$(awk -F'|' '{s+=$2} END{print s+0}' "$LAST_SNAP")
    tot_bytes=$(awk -F'|' '{s+=$3} END{print s+0}' "$LAST_SNAP")
    printf "  current ground: %s files, %s bytes across %s tracked dirs\n" \
      "$tot_files" "$tot_bytes" "$(wc -l < "$LAST_SNAP" | tr -d ' ')"
  fi
  echo "  (append-only log; flat-file; no EnkiDB dependency — GL-STY-001)"
  echo ""
}

# ----------------------------------------------------------------------------
show_roadmap() {
  cat <<'EOF'

  Eridu → BeeMDM → GOLDEN : the graduation timeline
  ═════════════════════════════════════════════════════════════

  Stage 0  [YOU ARE HERE]  flat-file scaffold + append-only evolution log
           · Eridu lays OTA ground, tracks its evolution on the
             filesystem only. Zero dependencies. Runs anytime.

  Stage 1  ○ emit OTA structure as particles → EnkiSDB (landing)
           · switch ERIDU_EMIT_ENKIDB=1. Each scan emits the ground
             snapshot as landing particles into EnkiSDB (7001).
           · the structure becomes DATA in the running database.

  Stage 2  ○ BeeMDM ETL over the structure feed
           · SDB → ODB → QDB gate chain runs on the emitted ground.
           · inference gate  → staging/inferred
           · proof gate (G4) → staging/proven
           · this is the BeeMDM pipeline exercised on real, self-
             generated data (BahyWay's own ground as first tribe).

  Stage 3  ○ proven shape reaches EnkiDB GOLDEN store (7004)
           · the OTA ground, proven through the gates, lands in the
             GOLDEN store. BahyWay's own structure is now a GOLDEN
             tribe — the first citizen of its own database.

  Stage 4  ○ self-hosting: Eridu reads its OntoGraph back from GOLDEN
           · Eridu queries GOLDEN (HeptaScript) to render the OTA
             ground's OntoGraph FROM the database it helped fill.
           · the loop closes: BahyWay models itself, in itself.
             THIS is the investor proof — built by its own tools.

  ═════════════════════════════════════════════════════════════
  Rule held at every stage: Eridu SCAFFOLDS and RECORDS; it never
  provisions DBs/containers (infra PBs do). Flat-file always works;
  EnkiDB emission is a feature switched ON, never a dependency.

EOF
}

# ----------------------------------------------------------------------------
# STUB — Stage 1+. Deliberately inert until EnkiDB emission is wired.
# Kept here so the graduation path is visible in the tool, not just docs.
emit_to_enkidb() {
  local snap="$1"
  echo "  … [emit stub] would land $(wc -l < "$snap" | tr -d ' ') ground-particles into EnkiSDB (7001)"
  echo "  … [emit stub] Stage 1 not yet wired — flat-file remains source of truth"
  # Future: pipe snapshot rows to enkidb-ingest::bridge as landing KAKIs.
}

# ----------------------------------------------------------------------------
case "${1:-pass}" in
  init)       init_structure ;;
  scan)       scan ;;
  evolution)  show_evolution ;;
  roadmap)    show_roadmap ;;
  emit)       ERIDU_EMIT_ENKIDB=1; scan ;;
  pass)       init_structure >/dev/null; scan; show_evolution ;;
  *) echo "usage: $0 {init|scan|evolution|roadmap|emit|pass}"; exit 1 ;;
esac
