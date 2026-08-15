#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#  𒁾  BahyWay.Ecosystem — build_Final.sh  v3 (Linux/pure-cargo edition)
#
#  WORKSPACE:
#    ./build_Final.sh v4 --workspace release
#    ./build_Final.sh v4 enkidb release
#
#  STANDALONE:
#    ./build_Final.sh --standalone vaultEngine release
#    ./build_Final.sh --standalone vaultEngine release open
#
#  BUILD ALL STANDALONES:
#    ./build_Final.sh --all-standalones release
#
#  RUN A STANDALONE:
#    ./build_Final.sh --run vaultEngine
#    ./build_Final.sh --run vaultEngine release
#
#  FLAGS: release | debug | open | --fix
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
set -euo pipefail

CYAN='\033[0;36m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'
RED='\033[0;31m';  DIM='\033[2m';       NC='\033[0m'
info()    { echo -e "${CYAN}[𒁾]${NC} $*"; }
ok()      { echo -e "${GREEN}[✓]${NC} $*"; }
warn()    { echo -e "${YELLOW}[!]${NC} $*"; }
fail()    { echo -e "${RED}[✗]${NC} $*"; exit 1; }
section() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  𒁾  $*${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Tools dir is workspace/scripts/tools/ — repo root is 3 levels up
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
WS_ROOT="$REPO_ROOT/workspace/bahyway_v4"

# ── Standalone registry ────────────────────────────────────────────────
declare -A STANDALONE_APPS=(
    ["vaultEngine"]="standalone/vaultEngine"
    ["storeEngine"]="standalone/storeEngine"
    ["pollutionEngine"]="standalone/pollutionEngine"
    ["dubEngine"]="standalone/dubEngine"
    ["nuskuEngine"]="standalone/nuskuEngine"
)

# ── Parse args ──────────────────────────────────────────────────────────
ARG1="${1:-}"
ARG2="${2:-}"
ARG3="${3:-}"
MODE="debug"
DO_OPEN=false
DO_FIX=false
for arg in "$@"; do
    case "$arg" in
        release) MODE="release" ;;
        debug)   MODE="debug"   ;;
        open)    DO_OPEN=true   ;;
        --fix)   DO_FIX=true    ;;
    esac
done

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# HELP
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
show_help() {
    echo -e "${CYAN}𒁾  BahyWay Build Script v3 (pure-cargo)${NC}\n"
    echo -e "${YELLOW}WORKSPACE:${NC}"
    echo "  ./build_Final.sh v4 --workspace release"
    echo "  ./build_Final.sh v4 enkidb release"
    echo ""
    echo -e "${YELLOW}STANDALONE:${NC}"
    echo "  ./build_Final.sh --standalone vaultEngine release"
    echo "  ./build_Final.sh --standalone vaultEngine release open"
    echo ""
    echo -e "${YELLOW}ALL STANDALONES:${NC}"
    echo "  ./build_Final.sh --all-standalones release"
    echo ""
    echo -e "${YELLOW}RUN:${NC}"
    echo "  ./build_Final.sh --run vaultEngine"
    echo "  ./build_Final.sh --run vaultEngine release"
    echo ""
    echo -e "${YELLOW}REGISTERED APPS:${NC}"
    for name in "${!STANDALONE_APPS[@]}"; do
        local path="${STANDALONE_APPS[$name]}"
        if [[ -d "$WS_ROOT/$path" ]]; then
            echo -e "  ${GREEN}$name${NC}  →  $path  ${GREEN}(exists)${NC}"
        else
            echo -e "  ${DIM}$name  →  $path  (not created yet)${NC}"
        fi
    done
    echo ""
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# STANDALONE BUILD — pure cargo
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
build_standalone() {
    local APP_NAME="$1"
    local BUILD_MODE="$2"

    local APP_PATH="${STANDALONE_APPS[$APP_NAME]:-}"
    [[ -z "$APP_PATH" ]] && \
        fail "Unknown app: '$APP_NAME'. Add it to STANDALONE_APPS in build_Final.sh"

    local FULL_PATH="$WS_ROOT/$APP_PATH"
    [[ ! -d "$FULL_PATH" ]]              && fail "Directory not found: $FULL_PATH"
    [[ ! -f "$FULL_PATH/Cargo.toml" ]]   && fail "No Cargo.toml at: $FULL_PATH"

    local LOG_DIR="$FULL_PATH/logs"
    mkdir -p "$LOG_DIR"
    local TS; TS=$(date '+%Y%m%d_%H%M%S')
    local RAW_LOG="$LOG_DIR/build_${APP_NAME}_${TS}.log"
    local REPORT="$LOG_DIR/build_${APP_NAME}_${TS}.report"

    if $DO_FIX; then
        local FIX="$SCRIPT_DIR/Invoke-CodeFix.sh"
        [[ -f "$FIX" ]] && WORKSPACE_PATH="$WS_ROOT" bash "$FIX" --category font --category deprecated \
            || warn "Invoke-CodeFix.sh not found"
    fi

    section "Standalone: $APP_NAME  ($BUILD_MODE)"
    info "Path : $FULL_PATH"
    info "Log  : $RAW_LOG"

    local CARGO_ARGS=("build")
    [[ "$BUILD_MODE" == "release" ]] && CARGO_ARGS+=("--release")

    local T0; T0=$(date +%s)
    pushd "$FULL_PATH" > /dev/null
    set +e
    cargo "${CARGO_ARGS[@]}" 2>&1 | tee "$RAW_LOG"
    local EXIT_CODE=${PIPESTATUS[0]}
    set -e
    popd > /dev/null
    local DUR=$(( $(date +%s) - T0 ))

    local ERRORS;   ERRORS=$(grep -c '^error'    "$RAW_LOG" 2>/dev/null || echo 0)
    local WARNINGS; WARNINGS=$(grep -c '^warning' "$RAW_LOG" 2>/dev/null || echo 0)
    local CRATES;   CRATES=$(grep -c 'Compiling ' "$RAW_LOG" 2>/dev/null || echo 0)
    local STATUS;   [[ $EXIT_CODE -eq 0 ]] && STATUS="SUCCESS" || STATUS="FAILED"

    {
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    printf "║  𒁾  BahyWay Standalone Build %-45s ║\n" "— $APP_NAME"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    printf "║  %-18s : %-55s ║\n" "App"       "$APP_NAME"
    printf "║  %-18s : %-55s ║\n" "Mode"      "$BUILD_MODE"
    printf "║  %-18s : %-55s ║\n" "Path"      "$APP_PATH"
    printf "║  %-18s : %-55s ║\n" "Started"   "$(date '+%Y-%m-%d %H:%M:%S')"
    printf "║  %-18s : %-55s ║\n" "Duration"  "$(printf '%02d:%02d' $((DUR/60)) $((DUR%60)))s"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    printf "║  %-18s : %-55s ║\n" "Result"    "$STATUS"
    printf "║  %-18s : %-55s ║\n" "Errors"    "$ERRORS"
    printf "║  %-18s : %-55s ║\n" "Warnings"  "$WARNINGS"
    printf "║  %-18s : %-55s ║\n" "Crates"    "$CRATES compiled"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    if [[ $ERRORS -gt 0 ]]; then
        echo "║  ERROR LINES:"
        grep '^error' "$RAW_LOG" 2>/dev/null | head -15 | \
            while IFS= read -r line; do printf "║    %-70s ║\n" "${line:0:70}"; done
        echo "╠════════════════════════════════════════════════════════════════════════════╣"
    fi
    for candidate in \
        "$FULL_PATH/target/release/${APP_NAME}" \
        "$FULL_PATH/target/debug/${APP_NAME}"; do
        if [[ -f "$candidate" ]]; then
            local SZ; SZ=$(du -sh "$candidate" 2>/dev/null | cut -f1)
            printf "║  %-18s : %-55s ║\n" "Binary" "$candidate"
            printf "║  %-18s : %-55s ║\n" "Size"   "$SZ"
            break
        fi
    done
    printf "║  %-18s : %-55s ║\n" "Report" "$(basename "$REPORT")"
    echo "║  𒁾  BahyWay.Ecosystem v4 — DUB.SAR$(printf '%*s' 40 '')║"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    } | tee "$REPORT"

    echo ""
    if [[ $EXIT_CODE -eq 0 ]]; then
        ok "PASSED — $CRATES crates — $(printf '%02d:%02d' $((DUR/60)) $((DUR%60)))s"
        $DO_OPEN && command -v code &>/dev/null && code "$REPORT" || true
    else
        echo -e "${RED}[✗] FAILED — $ERRORS error(s) — see: $REPORT${NC}"
    fi

    return $EXIT_CODE
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ALL STANDALONES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
build_all_standalones() {
    local BUILD_MODE="$1"
    local PASS=0 FAIL=0 SKIP=0

    section "ALL STANDALONES ($BUILD_MODE)"

    for APP_NAME in vaultEngine storeEngine pollutionEngine dubEngine nuskuEngine; do
        local APP_PATH="${STANDALONE_APPS[$APP_NAME]}"
        local FULL_PATH="$WS_ROOT/$APP_PATH"

        if [[ ! -d "$FULL_PATH" ]] || [[ ! -f "$FULL_PATH/Cargo.toml" ]]; then
            warn "SKIP $APP_NAME — not created yet (run bahyway.sh --scaffold)"
            SKIP=$(( SKIP + 1 ))
            continue
        fi

        echo -e "\n${DIM}[$((PASS+FAIL+1))/5] $APP_NAME...${NC}"
        if build_standalone "$APP_NAME" "$BUILD_MODE"; then
            PASS=$(( PASS + 1 ))
        else
            FAIL=$(( FAIL + 1 ))
            warn "Continuing with remaining apps..."
        fi
    done

    section "SUMMARY"
    echo -e "  ${GREEN}PASSED  : $PASS${NC}"
    echo -e "  ${RED}FAILED  : $FAIL${NC}"
    echo -e "  ${DIM}SKIPPED : $SKIP (not created yet)${NC}\n"
    [[ $FAIL -eq 0 ]]
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RUN STANDALONE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
run_standalone() {
    local APP_NAME="$1"
    local BUILD_FIRST="${2:-}"

    local APP_PATH="${STANDALONE_APPS[$APP_NAME]:-}"
    [[ -z "$APP_PATH" ]] && fail "Unknown app: $APP_NAME"
    local FULL_PATH="$WS_ROOT/$APP_PATH"

    [[ "$BUILD_FIRST" == "release" || "$BUILD_FIRST" == "debug" ]] && \
        build_standalone "$APP_NAME" "$BUILD_FIRST"

    local BINARY=""
    for candidate in \
        "$FULL_PATH/target/release/${APP_NAME}" \
        "$FULL_PATH/target/debug/${APP_NAME}"; do
        [[ -f "$candidate" ]] && BINARY="$candidate" && break
    done

    [[ -z "$BINARY" ]] && \
        fail "Binary not found. Build first:\n  ./build_Final.sh --standalone $APP_NAME release"

    section "Running $APP_NAME"
    info "Binary: $BINARY"
    echo ""
    RUST_LOG=info "$BINARY"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# WORKSPACE BUILD — pure cargo (no PowerShell)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
build_workspace() {
    local VERSION="$1"
    local CRATE="$2"
    local BUILD_MODE="$3"

    [[ ! -f "$WS_ROOT/Cargo.toml" ]] && \
        fail "Workspace not found at $WS_ROOT — run bahyway.sh --scaffold first"

    local LOG_DIR="$WS_ROOT/logs"
    mkdir -p "$LOG_DIR"
    local TS; TS=$(date '+%Y%m%d_%H%M%S')
    local RAW_LOG="$LOG_DIR/build_ws_${VERSION}_${TS}.log"
    local REPORT="$LOG_DIR/build_ws_${VERSION}_${TS}.report"

    section "Workspace: ${CRATE:-all} ($BUILD_MODE)"
    info "Root : $WS_ROOT"
    info "Log  : $RAW_LOG"

    local CARGO_ARGS=("build")
    [[ "$BUILD_MODE" == "release" ]] && CARGO_ARGS+=("--release")
    if [[ "$CRATE" == "--workspace" || -z "$CRATE" ]]; then
        CARGO_ARGS+=("--workspace")
    else
        CARGO_ARGS+=("-p" "$CRATE")
    fi

    local T0; T0=$(date +%s)
    pushd "$WS_ROOT" > /dev/null
    set +e
    cargo "${CARGO_ARGS[@]}" 2>&1 | tee "$RAW_LOG"
    local EXIT_CODE=${PIPESTATUS[0]}
    set -e
    popd > /dev/null
    local DUR=$(( $(date +%s) - T0 ))

    local ERRORS;   ERRORS=$(grep -c '^error'    "$RAW_LOG" 2>/dev/null || echo 0)
    local WARNINGS; WARNINGS=$(grep -c '^warning' "$RAW_LOG" 2>/dev/null || echo 0)
    local CRATES;   CRATES=$(grep -c 'Compiling ' "$RAW_LOG" 2>/dev/null || echo 0)
    local STATUS;   [[ $EXIT_CODE -eq 0 ]] && STATUS="SUCCESS" || STATUS="FAILED"

    {
    echo "╔════════════════════════════════════════════════════════════════════════════╗"
    printf "║  𒁾  BahyWay Workspace Build %-46s ║\n" "— $VERSION"
    echo "╠════════════════════════════════════════════════════════════════════════════╣"
    printf "║  %-18s : %-55s ║\n" "Version"   "$VERSION"
    printf "║  %-18s : %-55s ║\n" "Crate"     "${CRATE:---workspace}"
    printf "║  %-18s : %-55s ║\n" "Mode"      "$BUILD_MODE"
    printf "║  %-18s : %-55s ║\n" "Duration"  "$(printf '%02d:%02d' $((DUR/60)) $((DUR%60)))s"
    printf "║  %-18s : %-55s ║\n" "Result"    "$STATUS"
    printf "║  %-18s : %-55s ║\n" "Errors"    "$ERRORS"
    printf "║  %-18s : %-55s ║\n" "Warnings"  "$WARNINGS"
    printf "║  %-18s : %-55s ║\n" "Crates"    "$CRATES compiled"
    printf "║  %-18s : %-55s ║\n" "Report"    "$(basename "$REPORT")"
    echo "╚════════════════════════════════════════════════════════════════════════════╝"
    } | tee "$REPORT"

    [[ $EXIT_CODE -eq 0 ]] && ok "Workspace build PASSED" || \
        echo -e "${RED}[✗] Workspace build FAILED — see: $REPORT${NC}"

    return $EXIT_CODE
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# DISPATCH
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" 2>/dev/null || true

case "$ARG1" in
    ""|"--help"|"-h")
        show_help ;;

    "--standalone")
        [[ -z "$ARG2" ]] && fail "Usage: ./build_Final.sh --standalone <app> [release]"
        build_standalone "$ARG2" "$MODE" ;;

    "--all-standalones")
        build_all_standalones "$MODE" ;;

    "--run")
        [[ -z "$ARG2" ]] && fail "Usage: ./build_Final.sh --run <app> [release]"
        run_standalone "$ARG2" "$ARG3" ;;

    v*)
        CRATE="${ARG2:---workspace}"
        build_workspace "$ARG1" "$CRATE" "$MODE" ;;

    *)
        echo -e "${RED}Unknown: $ARG1${NC}"
        show_help; exit 1 ;;
esac
