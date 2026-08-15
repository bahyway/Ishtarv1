#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#  𒁾  BahyWay.Ecosystem — Invoke-CodeFix.sh
#  Sovereign Code Fixer — grep finds, sed fixes, tee logs everything
#
#  Architecture:
#    Each FIX is a declarative rule:
#      PATTERN  → what to search for  (grep)
#      REPLACE  → what to replace with (sed)
#      SCOPE    → which files/dirs to scan
#      CATEGORY → type of fix (font | import | deprecation | etc.)
#
#  USAGE:
#    ./Invoke-CodeFix.sh                         # run ALL rules
#    ./Invoke-CodeFix.sh --category font         # only font fixes
#    ./Invoke-CodeFix.sh --category deprecated   # only rand deprecations
#    ./Invoke-CodeFix.sh --category import       # unused import cleanup
#    ./Invoke-CodeFix.sh --dry-run               # preview, no changes
#    ./Invoke-CodeFix.sh --verify                # verify only
#
#  WORKSPACE (pick one):
#    WORKSPACE_PATH=/path/to/ws ./Invoke-CodeFix.sh   # env var
#    ./Invoke-CodeFix.sh --workspace /path/to/ws      # flag
#    (defaults to ../../bahyway_v4 relative to script)
#
#  𒁾  DUB.SAR — BahyWay.Ecosystem v4
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
set -euo pipefail

# ── Args ──────────────────────────────────────────────────────────────────
CATEGORY=""
DRY_RUN=false
VERIFY_ONLY=false
WS_OVERRIDE=""

i=1
while [[ $i -le $# ]]; do
    arg="${!i}"
    case $arg in
        --category)
            i=$(( i + 1 )); CATEGORY="${!i}" ;;
        --category=*)
            CATEGORY="${arg#*=}" ;;
        font|deprecated|import|type|bevy|all)
            CATEGORY="$arg" ;;
        --dry-run)
            DRY_RUN=true ;;
        --verify)
            VERIFY_ONLY=true ;;
        --workspace)
            i=$(( i + 1 )); WS_OVERRIDE="${!i}" ;;
        --workspace=*)
            WS_OVERRIDE="${arg#*=}" ;;
    esac
    i=$(( i + 1 ))
done

# ── Paths ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Priority: --workspace flag > WORKSPACE_PATH env var > default relative path
if [[ -n "$WS_OVERRIDE" ]]; then
    WORKSPACE="$WS_OVERRIDE"
elif [[ -n "${WORKSPACE_PATH:-}" ]]; then
    WORKSPACE="$WORKSPACE_PATH"
else
    WORKSPACE="$(cd "$SCRIPT_DIR/../../bahyway_v4" 2>/dev/null && pwd \
                 || echo "$SCRIPT_DIR")"
fi

SRC_DIR="$WORKSPACE"
CRATES_DIR="$WORKSPACE/crates"
LOG_DIR="$WORKSPACE/logs"
mkdir -p "$LOG_DIR"

TS=$(date '+%Y%m%d_%H%M%S')
LOG_FILE="${LOG_DIR}/codefix_${TS}.log"

# ── Colours ───────────────────────────────────────────────────────────────
CYAN='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
RED='\033[0;31m';  DIM='\033[2m';      BOLD='\033[1m';  NC='\033[0m'

# ── Counters ──────────────────────────────────────────────────────────────
TOTAL_RULES=0
RULES_APPLIED=0
RULES_SKIPPED=0
RULES_ALREADY_CLEAN=0
FILES_MODIFIED=0
declare -A MODIFIED_FILES=()

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# LOGGING
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
log()       { echo -e "$@" | tee -a "$LOG_FILE"; }
log_plain() { echo "$@" >> "$LOG_FILE"; }
log_section() {
    local title="$1"
    local line; line=$(printf '━%.0s' {1..68})
    log ""; log "${CYAN}  ${line}${NC}"
    log "${CYAN}  ${title}${NC}"
    log "${CYAN}  ${line}${NC}"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CORE FIX ENGINE
# apply_fix CATEGORY DESCRIPTION SCOPE GREP_PATTERN SED_EXPRESSION...
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
apply_fix() {
    local cat="$1" desc="$2" scope="$3" grep_pat="$4"
    shift 4
    local sed_exprs=("$@")

    TOTAL_RULES=$(( TOTAL_RULES + 1 ))

    if [[ -n "$CATEGORY" && "$CATEGORY" != "all" && "$CATEGORY" != "$cat" ]]; then
        RULES_SKIPPED=$(( RULES_SKIPPED + 1 ))
        return
    fi

    log ""; log "  ${BOLD}[RULE]${NC} ${YELLOW}${cat}${NC} — ${desc}"

    local files
    files=$(grep -rln "$grep_pat" "$scope" --include="*.rs" 2>/dev/null || true)

    if [[ -z "$files" ]]; then
        log "  ${GREEN}[CLEAN]${NC} No matches — already fixed or not present"
        RULES_ALREADY_CLEAN=$(( RULES_ALREADY_CLEAN + 1 ))
        return
    fi

    log "  ${DIM}  Affected files:${NC}"
    local file_count=0
    while IFS= read -r file; do
        file_count=$(( file_count + 1 ))
        local rel="${file#$WORKSPACE/}"
        log "  ${DIM}    [$file_count] ${rel}${NC}"
        grep -n "$grep_pat" "$file" 2>/dev/null | while IFS= read -r match_line; do
            log "  ${DIM}        BEFORE → ${match_line}${NC}"
        done
    done <<< "$files"

    if $DRY_RUN || $VERIFY_ONLY; then
        log "  ${YELLOW}[DRY-RUN]${NC} Would apply: ${sed_exprs[*]}"
        RULES_SKIPPED=$(( RULES_SKIPPED + 1 ))
        return
    fi

    local sed_args=()
    for expr in "${sed_exprs[@]}"; do sed_args+=("-e" "$expr"); done

    while IFS= read -r file; do
        sed -i "${sed_args[@]}" "$file"
        local rel="${file#$WORKSPACE/}"
        MODIFIED_FILES["$rel"]=1
        if grep -q "$grep_pat" "$file" 2>/dev/null; then
            log "  ${YELLOW}[PARTIAL]${NC} Some matches remain in: ${rel}"
        else
            log "  ${GREEN}[FIXED]${NC}  ${rel}"
        fi
    done <<< "$files"

    RULES_APPLIED=$(( RULES_APPLIED + 1 ))
    FILES_MODIFIED=$(( FILES_MODIFIED + file_count ))
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# VERIFY ENGINE
# verify_clean CATEGORY DESCRIPTION SCOPE GREP_PATTERN
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
verify_clean() {
    local cat="$1" desc="$2" scope="$3" grep_pat="$4"
    if [[ -n "$CATEGORY" && "$CATEGORY" != "all" && "$CATEGORY" != "$cat" ]]; then
        return
    fi
    local hits
    hits=$(grep -rn "$grep_pat" "$scope" --include="*.rs" 2>/dev/null || true)
    if [[ -z "$hits" ]]; then
        log "  ${GREEN}[✓ CLEAN]${NC}  ${desc}"
    else
        log "  ${RED}[✗ DIRTY]${NC}  ${desc}"
        echo "$hits" | while IFS= read -r line; do
            log "  ${RED}           ${line}${NC}"
        done
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# HEADER
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║  𒁾  BahyWay.Ecosystem — Sovereign Code Fixer Log"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    echo "║  Script    : Invoke-CodeFix.sh"
    echo "║  Workspace : ${WORKSPACE}"
    echo "║  Category  : ${CATEGORY:-ALL}"
    echo "║  Dry Run   : ${DRY_RUN}"
    echo "║  Verify    : ${VERIFY_ONLY}"
    echo "║  Started   : $(date '+%Y-%m-%d %H:%M:%S')"
    echo "║  Log File  : ${LOG_FILE}"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
} | tee "$LOG_FILE"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RULE DEFINITIONS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_section "CATEGORY: FONT — Monospace size fixes"

apply_fix "font" "monospace(9.0) → monospace(14.0)"   "$SRC_DIR" \
    "FontId::monospace(9\.0)" \
    's/FontId::monospace(9\.0)/FontId::monospace(14.0)/g'

apply_fix "font" "monospace(10.0) → monospace(14.0)"  "$SRC_DIR" \
    "FontId::monospace(10\.0)" \
    's/FontId::monospace(10\.0)/FontId::monospace(14.0)/g'

apply_fix "font" "proportional(8.0) → proportional(13.0)" "$SRC_DIR" \
    "proportional(8\.0)" \
    's/proportional(8\.0)/proportional(13.0)/g'

apply_fix "font" ".size(9.0) → .size(13.0)" "$SRC_DIR" \
    "\.size(9\.0)" \
    's/\.size(9\.0)/.size(13.0)/g'

apply_fix "font" ".size(8.0) → .size(13.0)" "$SRC_DIR" \
    "\.size(8\.0)" \
    's/\.size(8\.0)/.size(13.0)/g'

log_section "CATEGORY: DEPRECATED — rand 0.9 API renames"

apply_fix "deprecated" "rand::thread_rng() → rand::rng()" "$SRC_DIR" \
    "rand::thread_rng()" \
    's/rand::thread_rng()/rand::rng()/g'

apply_fix "deprecated" ".gen_range( → .random_range(" "$SRC_DIR" \
    "\.gen_range(" \
    's/\.gen_range(/.random_range(/g'

apply_fix "deprecated" ".gen::<T>() → .random::<T>()" "$SRC_DIR" \
    "\.gen::<" \
    's/\.gen::/.random::/g'

apply_fix "deprecated" "rng.gen() → rng.random()" "$SRC_DIR" \
    "rng\.gen()" \
    's/rng\.gen()/rng.random()/g'

log_section "CATEGORY: BEVY — Bevy 0.14 API fixes"

apply_fix "bevy" "dynamic_linking in release → remove" "$CRATES_DIR" \
    'features = \["dynamic_linking"\]' \
    's/features = \["dynamic_linking"\]/version = "0.14"/g'

apply_fix "bevy" ".scale() → to_scale_rotation_translation()" "$SRC_DIR" \
    "gtransform\.scale()" \
    's/gtransform\.scale()\.x/{ let (sc,_,_) = gtransform.to_scale_rotation_translation(); sc }.x/g'

log_section "CATEGORY: TYPE — Type cast fixes"

apply_fix "type" "result.total += usize → as u64 cast" "$SRC_DIR" \
    "+= result\.total;" \
    's/+= result\.total;/+= result.total as u64;/g'

apply_fix "type" "::egui::Color32 → egui::Color32" "$SRC_DIR" \
    "::egui::Color32" \
    's/pub use ::egui::Color32;/use egui::Color32;/g' \
    's/use ::egui::Color32;/use egui::Color32;/g'

log_section "CATEGORY: IMPORT — Remove unused imports"

apply_fix "import" "Remove unused PrimitiveTopology import" "$SRC_DIR" \
    "use bevy::render::mesh::PrimitiveTopology;" \
    '/use bevy::render::mesh::PrimitiveTopology;/d'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# VERIFICATION PASS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
log_section "VERIFICATION — Checking no old patterns remain"

verify_clean "font"       "No monospace(9.0) remaining"   "$SRC_DIR" "FontId::monospace(9\.0)"
verify_clean "font"       "No monospace(10.0) remaining"  "$SRC_DIR" "FontId::monospace(10\.0)"
verify_clean "font"       "No .size(8.0) remaining"       "$SRC_DIR" "\.size(8\.0)"
verify_clean "font"       "No .size(9.0) remaining"       "$SRC_DIR" "\.size(9\.0)"
verify_clean "deprecated" "No thread_rng() remaining"     "$SRC_DIR" "rand::thread_rng()"
verify_clean "deprecated" "No .gen_range() remaining"     "$SRC_DIR" "\.gen_range("
verify_clean "deprecated" "No .gen::<> remaining"         "$SRC_DIR" "\.gen::<"
verify_clean "bevy"       "No dynamic_linking in crates"  "$CRATES_DIR" 'features = \["dynamic_linking"\]'
verify_clean "type"       "No ::egui::Color32 remaining"  "$SRC_DIR" "::egui::Color32"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SUMMARY
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    echo "║  𒁾  CODE FIX SUMMARY"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    printf "║  %-20s : %-52s ║\n" "Total Rules"      "$TOTAL_RULES"
    printf "║  %-20s : %-52s ║\n" "Applied"          "$RULES_APPLIED"
    printf "║  %-20s : %-52s ║\n" "Already Clean"    "$RULES_ALREADY_CLEAN"
    printf "║  %-20s : %-52s ║\n" "Skipped (filter)" "$RULES_SKIPPED"
    printf "║  %-20s : %-52s ║\n" "Files Modified"   "${#MODIFIED_FILES[@]}"
    printf "║  %-20s : %-52s ║\n" "Dry Run"          "$DRY_RUN"
    printf "║  %-20s : %-52s ║\n" "Workspace"        "$WORKSPACE"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    if [[ ${#MODIFIED_FILES[@]} -gt 0 ]]; then
        echo "║  Modified Files:"
        for f in "${!MODIFIED_FILES[@]}"; do
            printf "║    %-72s ║\n" "• $f"
        done
        echo "╠════════════════════════════════════════════════════════════════════════════╣"
    fi
    printf "║  %-20s : %-52s ║\n" "Log" "$(basename "$LOG_FILE")"
    printf "║  %-20s : %-52s ║\n" "Finished" "$(date '+%Y-%m-%d %H:%M:%S')"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    echo ""
    if $DRY_RUN; then
        echo "  ⚠  DRY RUN — no files modified. Remove --dry-run to apply."
    else
        echo "  Next: bash bahyway.sh --build"
    fi
    echo ""
} | tee -a "$LOG_FILE"
