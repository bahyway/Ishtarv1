#!/usr/bin/env bash
# =============================================================================
# 𒁾 BahyWay — Smart Branch Consolidation Script v3.0
#
# WHAT IT DOES:
#   Finds ALL remote claude/* branches, merges them into master one by one,
#   auto-resolves the most common conflicts (Cargo.toml, ROADMAP.md, Cargo.lock),
#   then deletes the merged branches from remote.
#
# USAGE:
#   bash scripts/consolidate-branches.sh
#
# LOG:
#   Every run writes a timestamped log to:
#     scripts/logs/consolidate-YYYY-MM-DD_HH-MM-SS.log
#
# WHAT IT HANDLES AUTOMATICALLY:
#   - Cargo.toml conflicts   → keeps the branch version (new crates preserved)
#   - ROADMAP.md conflicts   → keeps the branch version (new entries preserved)
#   - Cargo.lock conflicts   → keeps the branch version (lock file regenerated)
#   - Already-merged branches → skipped silently
#   - Branches with 0 ahead  → skipped (nothing to merge)
#   - Any other conflict     → stops, shows exact fix commands, safe to re-run
#
# AFTER A CONFLICT (if script stops):
#   1. git status                          (see the conflicting file)
#   2. git checkout --theirs <file>        (take the branch version)
#   3. git add <file>
#   4. git merge --continue
#   5. git push origin master
#   6. bash scripts/consolidate-branches.sh   (re-run, skips done branches)
#
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
REMOTE="origin"
MASTER="master"
BRANCH_PREFIX="claude/"

# Files that are always auto-resolved using --theirs (branch wins)
AUTO_RESOLVE_FILES=(
    "workspace/bahyway_v4/Cargo.toml"
    "workspace/bahyway_v4/Cargo.lock"
    "workspace/bahyway_v4/ROADMAP.md"
    "workspace/bahyway_v4/SOVEREIGN_MANUAL_STEP1.md"
    "workspace/bahyway_v4/SOVEREIGN_MANUAL_PART2.md"
)

# ── Log setup ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
TIMESTAMP="$(date +%Y-%m-%d_%H-%M-%S)"
LOG_FILE="$LOG_DIR/consolidate-$TIMESTAMP.log"

# Mirror all output to log file AND terminal
exec > >(tee -a "$LOG_FILE") 2>&1

# ── Colour helpers (Git Bash supports ANSI) ────────────────────────────────────
RED='\033[0;31m';  GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; BOLD='\033[1m';     RESET='\033[0m'

log()     { echo -e "[$(date +%H:%M:%S)] $*"; }
info()    { echo -e "\n${BLUE}  ℹ  $*${RESET}"; }
ok()      { echo -e "${GREEN}  ✓  $*${RESET}"; }
warn()    { echo -e "${YELLOW}  ⚠  $*${RESET}"; }
err()     { echo -e "${RED}  ✗  $*${RESET}"; }
divider() { echo -e "\n${BOLD}══════════════════════════════════════════════════════${RESET}"; }
step()    { divider; echo -e "\n${BOLD}STEP $1 — $2${RESET}\n"; }

# ── Banner ────────────────────────────────────────────────────────────────────
clear
divider
echo -e "${BOLD}"
echo "   𒁾  BahyWay Branch Consolidation Script v3.0"
echo "   Started : $(date '+%Y-%m-%d %H:%M:%S')"
echo "   Log     : $LOG_FILE"
echo -e "${RESET}"
divider

# ── Step 1: Safety check ───────────────────────────────────────────────────────
step 1 "Safety checks"

# Must be in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    err "Not inside a git repository. cd into your EnkiDB folder first."
    exit 1
fi
ok "Inside git repository."

# Must not have an active merge or rebase in progress
if [ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ]; then
    err "A merge is already in progress!"
    echo ""
    echo "   Run these to finish it first:"
    echo "   git checkout --theirs workspace/bahyway_v4/Cargo.toml"
    echo "   git add workspace/bahyway_v4/Cargo.toml"
    echo "   git merge --continue"
    echo "   git push origin master"
    echo "   Then re-run this script."
    exit 1
fi

if [ -d "$(git rev-parse --git-dir)/rebase-merge" ] || \
   [ -d "$(git rev-parse --git-dir)/rebase-apply" ]; then
    err "A rebase is in progress!"
    echo ""
    echo "   Run this to cancel it cleanly:"
    echo "   git rebase --abort"
    echo "   Then re-run this script."
    exit 1
fi
ok "No merge or rebase in progress."

# Must be on master
CURRENT=$(git branch --show-current)
if [ "$CURRENT" != "$MASTER" ]; then
    warn "Currently on '$CURRENT'. Switching to master..."
    git checkout "$MASTER"
fi
ok "On branch: $MASTER"

# ── Step 2: Fetch ─────────────────────────────────────────────────────────────
step 2 "Fetch all remote branches"
git fetch --all --prune
ok "Fetch complete."

# ── Step 3: Update master ─────────────────────────────────────────────────────
step 3 "Update local master from remote"
git pull "$REMOTE" "$MASTER"
MASTER_TIP=$(git rev-parse HEAD)
ok "master tip: $MASTER_TIP"

# ── Step 4: Discover branches ─────────────────────────────────────────────────
step 4 "Discover all remote $BRANCH_PREFIX* branches"

# Get all remote claude/* branches, sorted by committer date (oldest first = safest merge order)
mapfile -t ALL_BRANCHES < <(
    git branch -r \
        | grep "$REMOTE/$BRANCH_PREFIX" \
        | sed "s|.*$REMOTE/||" \
        | while read -r b; do
            # Get the date of the latest commit on this branch
            DATE=$(git log -1 --format="%ct" "$REMOTE/$b" 2>/dev/null || echo 0)
            echo "$DATE $b"
          done \
        | sort -n \
        | awk '{print $2}'
)

if [ ${#ALL_BRANCHES[@]} -eq 0 ]; then
    ok "No $BRANCH_PREFIX* branches found on remote. Nothing to do."
    echo ""
    echo "   master is clean and up to date."
    exit 0
fi

echo ""
echo "   Found ${#ALL_BRANCHES[@]} branch(es):"
for B in "${ALL_BRANCHES[@]}"; do
    AHEAD=$(git rev-list --count "$MASTER..$REMOTE/$B" 2>/dev/null || echo "?")
    BEHIND=$(git rev-list --count "$REMOTE/$B..$MASTER" 2>/dev/null || echo "?")
    printf "   %-45s behind=%-4s ahead=%s\n" "$B" "$BEHIND" "$AHEAD"
done

# ── Step 5: Merge each branch ─────────────────────────────────────────────────
step 5 "Merge each branch into master"

MERGED=()
SKIPPED=()
FAILED=()

for BRANCH in "${ALL_BRANCHES[@]}"; do
    divider
    info "Branch: $BRANCH"

    # ── Count commits ahead ───────────────────────────────────────────────────
    AHEAD=$(git rev-list --count "$MASTER..$REMOTE/$BRANCH" 2>/dev/null || echo 0)

    if [ "$AHEAD" -eq 0 ]; then
        ok "0 commits ahead of master — nothing to merge. Skipping."
        SKIPPED+=("$BRANCH (0 ahead)")
        continue
    fi

    # ── Check if already merged ───────────────────────────────────────────────
    BRANCH_TIP=$(git rev-parse "$REMOTE/$BRANCH" 2>/dev/null || echo "")
    if [ -n "$BRANCH_TIP" ] && git merge-base --is-ancestor "$BRANCH_TIP" HEAD 2>/dev/null; then
        ok "Already merged into master — skipping."
        SKIPPED+=("$BRANCH (already merged)")
        continue
    fi

    log "  Commits to merge: $AHEAD"

    # ── Attempt merge ─────────────────────────────────────────────────────────
    info "Merging $BRANCH into master (--no-ff, no rebase)..."

    if git merge --no-ff "$REMOTE/$BRANCH" \
        -m "Merge branch '$BRANCH' into master

Merged by scripts/consolidate-branches.sh on $(date '+%Y-%m-%d %H:%M:%S')
Commits merged: $AHEAD"; then

        ok "Clean merge — no conflicts."
        MERGED+=("$BRANCH ($AHEAD commits)")

    else
        # ── Auto-resolve known conflict files ─────────────────────────────────
        info "Conflict detected. Attempting auto-resolution..."
        UNRESOLVED=()

        for CF in "${AUTO_RESOLVE_FILES[@]}"; do
            if git status --porcelain | grep -qE "^(UU|AA|DD) $CF"; then
                git checkout --theirs "$CF"
                git add "$CF"
                ok "Auto-resolved: $CF (kept branch version)"
            fi
        done

        # Check if any conflicts remain
        REMAINING=$(git status --porcelain | grep -E "^(UU|AA|DD)" || true)

        if [ -z "$REMAINING" ]; then
            # All conflicts resolved — complete the merge
            git commit -m "Merge branch '$BRANCH' into master (auto-resolved conflicts)

Auto-resolved files: ${AUTO_RESOLVE_FILES[*]}
Merged by scripts/consolidate-branches.sh on $(date '+%Y-%m-%d %H:%M:%S')
Commits merged: $AHEAD"
            ok "Merge completed after auto-resolution."
            MERGED+=("$BRANCH ($AHEAD commits, auto-resolved)")

        else
            # Unknown conflict — stop and guide the user
            err "Unresolved conflict — manual fix required."
            echo ""
            echo -e "${RED}  ╔══════════════════════════════════════════════════════════╗${RESET}"
            echo -e "${RED}  ║  MANUAL FIX NEEDED                                        ║${RESET}"
            echo -e "${RED}  ║  Branch: $BRANCH"
            echo -e "${RED}  ╠══════════════════════════════════════════════════════════╣${RESET}"
            echo    "  ║                                                            ║"
            echo    "  ║  1. See conflicting files:                                 ║"
            echo    "  ║       git status                                           ║"
            echo    "  ║                                                            ║"
            echo    "  ║  2. For each conflicting file:                             ║"
            echo    "  ║       git checkout --theirs <file>                         ║"
            echo    "  ║       git add <file>                                       ║"
            echo    "  ║                                                            ║"
            echo    "  ║  3. Complete the merge:                                    ║"
            echo    "  ║       git merge --continue                                 ║"
            echo    "  ║       git push origin master                               ║"
            echo    "  ║                                                            ║"
            echo    "  ║  4. Re-run this script (skips already-merged branches):   ║"
            echo    "  ║       bash scripts/consolidate-branches.sh                 ║"
            echo    "  ║                                                            ║"
            echo    "  ║  To ABANDON merging this branch:                           ║"
            echo    "  ║       git merge --abort                                    ║"
            echo    "  ║       bash scripts/consolidate-branches.sh                 ║"
            echo -e "${RED}  ╚══════════════════════════════════════════════════════════╝${RESET}"
            echo ""
            FAILED+=("$BRANCH")
            log "STOPPED. Branches merged so far: ${MERGED[*]:-none}"
            log "Log: $LOG_FILE"
            exit 1
        fi
    fi
done

# ── Step 6: Push master ───────────────────────────────────────────────────────
step 6 "Push master to remote"

if [ ${#MERGED[@]} -gt 0 ]; then
    git push "$REMOTE" "$MASTER"
    ok "master pushed successfully."
else
    ok "Nothing new merged — push not needed."
fi

# ── Step 7: Delete merged branches ────────────────────────────────────────────
step 7 "Delete merged remote branches"

DELETED=()
for BRANCH in "${MERGED[@]}"; do
    # Extract branch name from "name (N commits)" format
    BNAME=$(echo "$BRANCH" | sed 's/ (.*//')
    if git ls-remote --exit-code --heads "$REMOTE" "$BNAME" > /dev/null 2>&1; then
        git push "$REMOTE" --delete "$BNAME"
        ok "Deleted remote branch: $BNAME"
        DELETED+=("$BNAME")
    fi
done

# ── Summary ───────────────────────────────────────────────────────────────────
divider
echo ""
echo -e "${BOLD}  𒁾  CONSOLIDATION COMPLETE${RESET}"
echo "  Finished : $(date '+%Y-%m-%d %H:%M:%S')"
FINAL_TIP=$(git rev-parse HEAD)
echo "  master   : $FINAL_TIP"
echo ""

echo -e "${GREEN}  ── Merged & Deleted ──────────────────────────────────${RESET}"
if [ ${#MERGED[@]} -eq 0 ]; then
    echo "    (none — master was already up to date)"
else
    for M in "${MERGED[@]}"; do echo -e "${GREEN}    ✓  $M${RESET}"; done
fi

echo ""
echo -e "${YELLOW}  ── Skipped ────────────────────────────────────────────${RESET}"
if [ ${#SKIPPED[@]} -eq 0 ]; then
    echo "    (none skipped)"
else
    for S in "${SKIPPED[@]}"; do echo -e "${YELLOW}    –  $S${RESET}"; done
fi

echo ""
echo "  ── Log ─────────────────────────────────────────────────"
echo "    $LOG_FILE"
echo ""
divider
echo ""
