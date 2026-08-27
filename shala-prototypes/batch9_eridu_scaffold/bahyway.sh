#!/usr/bin/env bash
# ============================================================================
# bahyway.sh — DailyWorks structure + status tracking for BahyWay.Ecosystem
# ----------------------------------------------------------------------------
# Philosophy (matches your sealed laws):
#   - project/  = the DURABLE home. One home per thing (Hepta uniqueness).
#   - journal/  = time-ordered NOTES, append-only (Sedimentation, GL-STY-001).
#                 daily → weekly → monthly → yearly roll-ups.
#   - STATUS.md = the ONE ledger that answers "where do I stand?" (GL-DOC-001).
#                 Folders can't tell you an engine's % done; this file can.
#
# Usage:
#   ./bahyway.sh init            # create the structure (safe to re-run)
#   ./bahyway.sh today           # drop today's journal note (idempotent)
#   ./bahyway.sh week            # drop this week's roll-up note
#   ./bahyway.sh month           # drop this month's roll-up note
#   ./bahyway.sh year            # drop this year's roll-up note
#   ./bahyway.sh status          # print "where I stand" from STATUS.md
#   ./bahyway.sh                 # = init + today + status  (your morning command)
# ============================================================================
set -euo pipefail

# --- root: change this to wherever you want it (default: ~/BahyWay) ----------
ROOT="${BAHYWAY_ROOT:-$HOME/BahyWay}"
DATE="$(date +%Y-%m-%d)"
DOW="$(date +%A)"
WEEK="$(date +%Y-W%V)"
MONTH="$(date +%Y-%m)"
YEAR="$(date +%Y)"

# ----------------------------------------------------------------------------
init_structure() {
  # DURABLE project home — flat, one home per thing. Create only what earns a folder.
  mkdir -p "$ROOT/project/laws"          # GL-*.md and PB-*.yml, flat, sorts by number
  mkdir -p "$ROOT/project/glossary"      # CAT-001 (GL-DOC-001 single glossary)
  mkdir -p "$ROOT/project/engines"       # one subfolder per engine WHEN it has content
  mkdir -p "$ROOT/project/prototypes"    # Šala hub .html
  mkdir -p "$ROOT/project/infra"         # ZFS / VM / backup playbooks
  mkdir -p "$ROOT/project/triple-o"      # philosophy / theory / theses
  mkdir -p "$ROOT/project/docs"          # manuals, whitepaper drafts

  # JOURNAL — time-ordered notes, four cadences.
  mkdir -p "$ROOT/journal/daily"
  mkdir -p "$ROOT/journal/weekly"
  mkdir -p "$ROOT/journal/monthly"
  mkdir -p "$ROOT/journal/yearly"

  # INBOX — unsorted, deal-with-later. Zips, downloads.
  mkdir -p "$ROOT/inbox"

  # STATUS ledger — created once, then you edit it by hand as work progresses.
  [ -f "$ROOT/STATUS.md" ] || cat > "$ROOT/STATUS.md" <<'EOF'
# BahyWay.Ecosystem v4.0 — STATUS LEDGER
# The ONE place that answers "where do I stand?"
# Edit the % and status of each line as you work. `bahyway.sh status` reads this.
#
# status values: idea | designed | building | testing | done | blocked
# Format (keep the pipes):  name | kind | status | percent | note
#
# --- ENGINES ----------------------------------------------------------------
EnkiDB          | engine | designed | 5   | CQRS core; PB-160 gate not closed
EnkiSDB         | engine | designed | 5   | ingest store
EnkiODB         | engine | designed | 5   | staging store
EnkiQDB         | engine | designed | 5   | query-gate projection
EnkiDW          | engine | designed | 5   | warehouse projection
EnkiMDB         | engine | designed | 5   | arsenal / templates
EnkiDDB         | engine | designed | 5   | documents
GeoEngine       | engine | designed | 3   | truth source; equations
NinurtaEngine   | engine | idea     | 0   | physics/dynamics (GL-PHY-001)
LamassuEngine   | engine | designed | 3   | persistent homology
NabuEngine      | engine | designed | 3   | knowledge graph
Shakkanakku     | engine | building | 15  | governor; chronicle proven earlier
DubSarTheater   | engine | designed | 5   | Godot runtime/stage
DubSarPDM       | engine | designed | 8   | Godot shape editor
Girsu           | engine | building | 20  | VSCodium workbench provisioned
AkkadianAOL     | engine | designed | 5   | orchestration language + codegen
#
# --- FOUNDATIONS (laws sealed = design done; implementation separate) --------
Laws_Sealed     | laws   | done     | 100 | ~50 GL/PB concept-sealed
PB160_Gate      | milestone | blocked | 0 | THE gate. Nothing runs until closed.
Vertical_Slice  | milestone | idea  | 0   | one tribe -> GOLDEN -> rendered
#
# --- INFRA ------------------------------------------------------------------
ZFS_ReadNode    | infra  | idea     | 0   | recordsize/atime datasets
ZFS_Backup      | infra  | idea     | 0   | daily snapshot+send rite
Source_Backup   | infra  | idea     | 0   | restic ~/BahyWay to external (DO THIS)
EOF

  echo "✓ structure ready at $ROOT"
}

# ----------------------------------------------------------------------------
note_daily() {
  local f="$ROOT/journal/daily/$DATE.md"
  if [ -f "$f" ]; then echo "• today's note already exists: $f"; return; fi
  cat > "$f" <<EOF
# $DATE ($DOW)

## Done today
-

## Decisions / rulings
-

## Sealed (laws / PBs)
-

## Links into project/
-

## Blocked / needs next
-

## Tomorrow's one thing
-
EOF
  echo "✓ created $f"
}

note_rollup() {
  local dir="$1" key="$2" label="$3"
  local f="$ROOT/journal/$dir/$key.md"
  if [ -f "$f" ]; then echo "• $label note already exists: $f"; return; fi
  cat > "$f" <<EOF
# $label — $key

## What advanced this $label
-

## % movement (from STATUS.md)
-

## What stalled / stayed blocked
-

## The one thing that matters next $label
-
EOF
  echo "✓ created $f"
}

# ----------------------------------------------------------------------------
show_status() {
  local s="$ROOT/STATUS.md"
  [ -f "$s" ] || { echo "no STATUS.md yet — run: $0 init"; return; }
  echo ""
  echo "  BahyWay — where I stand ($DATE)"
  echo "  ─────────────────────────────────────────────"
  # parse the pipe lines, skip comments/blanks, compute a simple bar + averages
  awk -F'|' '
    /^[[:space:]]*#/ {next}
    /^[[:space:]]*$/ {next}
    NF>=4 {
      name=$1; status=$3; pct=$4;
      gsub(/^[ \t]+|[ \t]+$/,"",name);
      gsub(/^[ \t]+|[ \t]+$/,"",status);
      gsub(/^[ \t]+|[ \t]+$/,"",pct);
      p=pct+0;
      # build a 10-cell bar
      filled=int(p/10); bar="";
      for(i=0;i<10;i++){ bar = bar (i<filled ? "█" : "·") }
      printf "  %-16s %s %3d%%  %s\n", name, bar, p, status;
      sum+=p; n++;
      if(status=="done") done++;
      if(status=="blocked") blk++;
    }
    END{
      printf "  ─────────────────────────────────────────────\n";
      if(n>0) printf "  overall: %d items · avg %.0f%% · %d done · %d blocked\n", n, sum/n, done+0, blk+0;
      printf "  (edit STATUS.md to update; this reads it — no guessing, GL-DB-001)\n";
    }
  ' "$s"
  echo ""
}

# ----------------------------------------------------------------------------
case "${1:-morning}" in
  init)    init_structure ;;
  today)   init_structure >/dev/null; note_daily ;;
  week)    init_structure >/dev/null; note_rollup weekly  "$WEEK"  "WEEK"  ;;
  month)   init_structure >/dev/null; note_rollup monthly "$MONTH" "MONTH" ;;
  year)    init_structure >/dev/null; note_rollup yearly  "$YEAR"  "YEAR"  ;;
  status)  show_status ;;
  morning) init_structure >/dev/null; note_daily; show_status ;;
  *) echo "usage: $0 {init|today|week|month|year|status|morning}"; exit 1 ;;
esac
