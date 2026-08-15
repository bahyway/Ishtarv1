#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#  𒁾  BahyWay.Ecosystem — compare_versions.sh
#  Diff two workspace versions for a given crate name (keyword)
#  Usage:
#    ./compare_versions.sh v352 v353               # compare all crates
#    ./compare_versions.sh v352 v353 kupru         # filter by keyword
#    ./compare_versions.sh v352 v353 kupru --files # also show file diffs
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

set -euo pipefail

VER_A="${1:-}"
VER_B="${2:-}"
FILTER="${3:-}"
SHOW_FILES="${4:-}"

CYAN='\033[0;36m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'
RED='\033[0;31m'; DIM='\033[2m'; NC='\033[0m'; MAGENTA='\033[0;35m'

info()  { echo -e "${CYAN}[𒁾]${NC} $*"; }
ok()    { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[⚠]${NC} $*"; }

if [[ -z "$VER_A" || -z "$VER_B" ]]; then
    echo -e "${YELLOW}Usage: ./compare_versions.sh <verA> <verB> [keyword] [--files]${NC}"
    echo ""
    echo "  Examples:"
    echo "    ./compare_versions.sh v352 v353"
    echo "    ./compare_versions.sh v352 v353 kupru"
    echo "    ./compare_versions.sh v352 v353 kaki --files"
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

resolve_ws() {
    local ver="$1"
    local candidates=(
        "$SCRIPT_DIR/bahyway-codex-$ver"
        "$SCRIPT_DIR/$ver"
        "$SCRIPT_DIR"
    )
    for c in "${candidates[@]}"; do
        if [[ -f "$c/Cargo.toml" ]]; then echo "$c"; return 0; fi
    done
    echo "$SCRIPT_DIR"
}

WS_A=$(resolve_ws "$VER_A")
WS_B=$(resolve_ws "$VER_B")

echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  𒁾  BahyWay Version Comparison${NC}"
echo -e "${CYAN}  ${VER_A} → ${VER_B}  ${FILTER:+(filter: $FILTER)}${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# ── List crates in each workspace ─────────────────────────────────
list_crates() {
    local ws="$1"
    local filter="${2:-}"
    if [[ -f "$ws/Cargo.toml" ]]; then
        # Extract member crate names from workspace Cargo.toml
        grep -E '^\s+"[^"]+",?\s*$' "$ws/Cargo.toml" 2>/dev/null | \
            sed 's/[" ,]//g' | sed 's/^\s*//' | \
            { [[ -n "$filter" ]] && grep -i "$filter" || cat; } | sort
    else
        echo "(workspace not found: $ws)"
    fi
}

CRATES_A=$(list_crates "$WS_A" "$FILTER")
CRATES_B=$(list_crates "$WS_B" "$FILTER")

# ── Side-by-side comparison ───────────────────────────────────────
echo -e "${DIM}  ┌─────────────────────────────────────┬─────────────────────────────────────┐${NC}"
printf "${DIM}  │${NC} %-37s ${DIM}│${NC} %-37s ${DIM}│${NC}\n" \
    "  $VER_A Crates" "  $VER_B Crates"
echo -e "${DIM}  ├─────────────────────────────────────┼─────────────────────────────────────┤${NC}"

# Combine and align
ALL_CRATES=$(echo -e "$CRATES_A\n$CRATES_B" | sort -u | grep -v '^$')

while IFS= read -r crate; do
    [[ -z "$crate" ]] && continue
    in_a=$(echo "$CRATES_A" | grep -c "^${crate}$" 2>/dev/null || echo 0)
    in_b=$(echo "$CRATES_B" | grep -c "^${crate}$" 2>/dev/null || echo 0)

    if   [[ "$in_a" -gt 0 && "$in_b" -gt 0 ]]; then
        printf "  ${DIM}│${NC} ${CYAN}%-37s${NC} ${DIM}│${NC} ${CYAN}%-37s${NC} ${DIM}│${NC}\n" "$crate" "$crate"
    elif [[ "$in_a" -gt 0 && "$in_b" -eq 0 ]]; then
        printf "  ${DIM}│${NC} ${RED}%-37s${NC} ${DIM}│${NC} ${DIM}%-37s${NC} ${DIM}│${NC}\n" "$crate" "(removed/renamed)"
    elif [[ "$in_a" -eq 0 && "$in_b" -gt 0 ]]; then
        printf "  ${DIM}│${NC} ${DIM}%-37s${NC} ${DIM}│${NC} ${GREEN}%-37s${NC} ${DIM}│${NC}\n" "(new in $VER_B)" "$crate"
    fi
done <<< "$ALL_CRATES"

echo -e "${DIM}  └─────────────────────────────────────┴─────────────────────────────────────┘${NC}"

# ── File diff (optional) ──────────────────────────────────────────
if [[ "$SHOW_FILES" == "--files" && -n "$FILTER" ]]; then
    echo ""
    info "File diff for '$FILTER' between $VER_A and $VER_B:"
    echo ""
    # Find src dirs
    SRC_A=$(find "$WS_A" -type d -name "src" 2>/dev/null | grep -i "$FILTER" | head -1)
    SRC_B=$(find "$WS_B" -type d -name "src" 2>/dev/null | grep -i "$FILTER" | head -1)

    if [[ -n "$SRC_A" && -n "$SRC_B" ]]; then
        diff -rq --exclude="*.lock" "$SRC_A" "$SRC_B" 2>/dev/null || true
    elif [[ -z "$SRC_A" ]]; then
        warn "No src/ found for '$FILTER' in $VER_A ($WS_A)"
    elif [[ -z "$SRC_B" ]]; then
        warn "No src/ found for '$FILTER' in $VER_B ($WS_B)"
    fi
fi

# ── Count summary ─────────────────────────────────────────────────
COUNT_A=$(echo "$CRATES_A" | grep -c . 2>/dev/null || echo 0)
COUNT_B=$(echo "$CRATES_B" | grep -c . 2>/dev/null || echo 0)
echo ""
echo -e "  ${CYAN}$VER_A${NC}: ${COUNT_A} crates  →  ${CYAN}$VER_B${NC}: ${COUNT_B} crates  \
(${GREEN}+$((COUNT_B - COUNT_A < 0 ? 0 : COUNT_B - COUNT_A))${NC} added)"
echo ""
