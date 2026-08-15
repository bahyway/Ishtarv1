#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#  𒁾  BahyWay.Ecosystem — dev.sh
#  Sovereign workspace dev runner
#  Usage: ./dev.sh <version> <command> [crate]
#  Example:
#    ./dev.sh v353 build
#    ./dev.sh v353 test kaki-corev353
#    ./dev.sh v353 check
#    ./dev.sh v353 clippy
#    ./dev.sh v353 doc
#    ./dev.sh v354 build nusku-wayv354
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

set -euo pipefail

VERSION="${1:-}"
CMD="${2:-help}"
CRATE="${3:-}"

# ── Color helpers ─────────────────────────────────────────────────
RED='\033[0;31m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
GREEN='\033[0;32m'; DIM='\033[2m'; NC='\033[0m'
info()  { echo -e "${CYAN}[𒁾 NuskuWay]${NC} $*"; }
ok()    { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[⚠]${NC} $*"; }
fail()  { echo -e "${RED}[✗]${NC} $*"; exit 1; }

# ── Resolve workspace root ────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR"

# Find the version workspace directory
resolve_workspace() {
    local ver="$1"
    # Try: repo-root/bahyway-codex-<ver>/
    # Try: repo-root/<ver>/
    # Try: repo-root/ (if Cargo.toml mentions the version)
    local candidates=(
        "$REPO_ROOT/bahyway-codex-$ver"
        "$REPO_ROOT/$ver"
        "$REPO_ROOT"
    )
    for c in "${candidates[@]}"; do
        if [[ -f "$c/Cargo.toml" ]]; then
            echo "$c"
            return 0
        fi
    done
    # Fall back to repo root
    echo "$REPO_ROOT"
}

if [[ -z "$VERSION" ]]; then
    echo -e "${YELLOW}Usage: ./dev.sh <version> <command> [crate]${NC}"
    echo ""
    echo "  Commands:"
    echo "    build   — cargo build --workspace"
    echo "    test    — cargo test [crate]"
    echo "    check   — cargo check --workspace"
    echo "    clippy  — cargo clippy --workspace"
    echo "    doc     — cargo doc --workspace --no-deps"
    echo "    fmt     — cargo fmt --all"
    echo "    clean   — cargo clean"
    echo "    crates  — list all crates in workspace"
    echo "    tree    — show dependency tree"
    echo ""
    exit 0
fi

WS=$(resolve_workspace "$VERSION")
info "Workspace: $WS"
info "Version:   $VERSION  |  Command: $CMD  |  Crate: ${CRATE:-all}"
echo ""

# Verify workspace exists
if [[ ! -f "$WS/Cargo.toml" ]]; then
    warn "No Cargo.toml found at: $WS"
    warn "Available workspaces in repo root:"
    find "$REPO_ROOT" -maxdepth 2 -name "Cargo.toml" | head -10
    echo ""
    warn "Try: ./dev.sh $VERSION build  (after creating the workspace)"
    exit 1
fi

cd "$WS"

# ── Commands ──────────────────────────────────────────────────────
case $CMD in
    build)
        info "Building workspace ($VERSION)..."
        if [[ -n "$CRATE" ]]; then
            cargo build -p "$CRATE" 2>&1 | grep -v "^$" || true
        else
            cargo build --workspace 2>&1 | grep -v "^$" || true
        fi
        ok "Build complete"
        ;;

    test)
        if [[ -n "$CRATE" ]]; then
            info "Testing crate: $CRATE"
            # First check if crate exists in workspace
            if ! cargo metadata --no-deps --format-version 1 2>/dev/null | \
               python3 -c "import sys,json; m=json.load(sys.stdin); \
               names=[p['name'] for p in m['packages']]; \
               print('\n'.join(names))" 2>/dev/null | grep -q "^${CRATE}$"; then
                warn "Crate '$CRATE' not found in workspace $WS"
                warn "Available crates:"
                cargo metadata --no-deps --format-version 1 2>/dev/null | \
                    python3 -c "import sys,json; m=json.load(sys.stdin); \
                    [print('  -', p['name']) for p in m['packages']]" 2>/dev/null || \
                    grep 'name = ' Cargo.toml | head -20
                exit 1
            fi
            cargo test -p "$CRATE" -- --nocapture 2>&1
        else
            info "Testing all workspace crates..."
            cargo test --workspace -- --nocapture 2>&1
        fi
        ok "Tests complete"
        ;;

    check)
        info "Checking workspace ($VERSION)..."
        cargo check --workspace 2>&1
        ok "Check complete"
        ;;

    clippy)
        info "𒁾 Clippy for $VERSION..."
        cargo clippy --workspace -- -D warnings 2>&1
        ok "Clippy complete"
        ;;

    doc)
        info "Generating docs ($VERSION)..."
        cargo doc --workspace --no-deps 2>&1
        ok "Docs generated: target/doc/"
        ;;

    fmt)
        info "Formatting ($VERSION)..."
        cargo fmt --all 2>&1
        ok "Format complete"
        ;;

    clean)
        info "Cleaning ($VERSION)..."
        cargo clean 2>&1
        ok "Clean complete"
        ;;

    crates)
        info "Crates in workspace $VERSION:"
        if command -v python3 &>/dev/null; then
            cargo metadata --no-deps --format-version 1 2>/dev/null | \
                python3 -c "
import sys, json
m = json.load(sys.stdin)
print('')
for p in sorted(m['packages'], key=lambda x: x['name']):
    print(f\"  {p['name']:40s}  v{p['version']}\")
print(f\"\\n  Total: {len(m['packages'])} crates\")
" 2>/dev/null || grep -E '^(name|version)\s*=' */Cargo.toml 2>/dev/null | head -40
        else
            grep -E '"[^"]+"\s*=\s*\{' Cargo.toml | head -30 || echo "  (install python3 for full listing)"
        fi
        ;;

    tree)
        info "Dependency tree ($VERSION)..."
        cargo tree --workspace 2>&1 | head -60
        ;;

    *)
        warn "Unknown command: $CMD"
        echo "Available: build | test | check | clippy | doc | fmt | clean | crates | tree"
        exit 1
        ;;
esac
