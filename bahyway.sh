#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#  𒁾  BahyWay.Ecosystem v4.0 — Sovereign Environment Orchestrator
#  Single entry point: environment · workspace scaffold · build · verify
#
#  USAGE (Bash / Vagrant Fedora Box):
#    bash bahyway.sh --all --yes                  # full unattended run
#    bash bahyway.sh --env --scaffold --build      # pick phases
#    bash bahyway.sh --scaffold --build            # skip env (already done)
#    bash bahyway.sh --verify                      # check everything
#    bash bahyway.sh --build --release             # release build only
#
#  USAGE (Ansible):
#    - name: BahyWay full sovereign setup
#      shell: bash /home/vagrant/EnkiDB/bahyway.sh --all --yes
#      args:
#        chdir: /home/vagrant/EnkiDB
#
#  PHASES (executed in this order):
#    --env        Fedora system packages + Rust toolchain (bahywaylab.sh)
#    --gpu        Vulkan/OpenGL/GLSL-SPIR-V stack (setup-gpu-stack.sh)
#    --scaffold   Create Cargo workspace + all crate skeletons
#    --fix        Declarative code-fix sweep (Invoke-CodeFix.sh)
#    --build      cargo check + build --workspace
#    --test       cargo test --workspace
#    --verify     Sanity-check all tools and workspace structure
#    --all        All phases in order
#
#  OPTIONS:
#    --yes / -y      No interactive prompts (Ansible-safe)
#    --dry-run       Preview only, no changes
#    --release       Build in release mode (default: debug)
#    --profile P     ENV profile: rust|bevy|os|all  (→ bahywaylab.sh)
#    --skip-gpu      Skip GPU phase even in --all
#    --skip-nvidia   Skip NVIDIA driver install
#    --version VER   Workspace version label (default: v4)
#
#  Log: /tmp/bahyway-master.log
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
set -uo pipefail

# ── Paths ────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS_DIR="$SCRIPT_DIR/workspace/scripts/tools"
WS_DIR="$SCRIPT_DIR/workspace/bahyway_v4"
LOG="/tmp/bahyway-master.log"
: > "$LOG"

# ── Colours ──────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
    BOLD=$'\033[1m'; DIM=$'\033[2m'
    GOLD=$'\033[1;33m'; CYAN=$'\033[0;36m'; GREEN=$'\033[0;32m'
    RED=$'\033[0;31m'; YELLOW=$'\033[1;33m'; MAGENTA=$'\033[0;35m'
    NC=$'\033[0m'
else
    BOLD=""; DIM=""; GOLD=""; CYAN=""; GREEN=""
    RED=""; YELLOW=""; MAGENTA=""; NC=""
fi

# ── Phase flags ───────────────────────────────────────────────────────────
DO_ENV=0; DO_GPU=0; DO_SCAFFOLD=0; DO_FIX=0
DO_BUILD=0; DO_TEST=0; DO_VERIFY=0

# ── Options ───────────────────────────────────────────────────────────────
ASSUME_YES=0
DRY_RUN=0
BUILD_RELEASE=0
ENV_PROFILE="rust"
SKIP_GPU=0
SKIP_NVIDIA=0
WS_VERSION="v4"

# ── Counters ──────────────────────────────────────────────────────────────
PHASES_RUN=0; PHASES_PASSED=0; PHASES_FAILED=0
declare -A PHASE_STATUS=()

# ── Helpers ───────────────────────────────────────────────────────────────
banner() {
cat <<'EOF'

   𒁾  ──────────────────────────────────────────────────────────────  𒁾
          B A H Y W A Y . E C O S Y S T E M   v 4 . 0
        Pure Rust · EnkiDB · EnkiDW · HeptaScript · AAOL · WAY
   𒁾  ──────────────────────────────────────────────────────────────  𒁾
EOF
}

section()   { echo "" | tee -a "$LOG"; echo "${BOLD}${MAGENTA}━━━ $* ━━━${NC}" | tee -a "$LOG"; }
say()       { echo "${BOLD}${CYAN}==>${NC} $*" | tee -a "$LOG"; }
ok()        { echo "${GREEN}  ✓${NC} $*" | tee -a "$LOG"; }
warn()      { echo "${YELLOW}  ⚠${NC} $*" | tee -a "$LOG"; }
fail_msg()  { echo "${RED}  ✗${NC} $*" | tee -a "$LOG"; }
info()      { echo "${DIM}    $*${NC}" | tee -a "$LOG"; }

phase_start() {
    section "PHASE: $1"
    PHASES_RUN=$(( PHASES_RUN + 1 ))
}
phase_pass() {
    ok "PHASE PASSED: $1"
    PHASES_PASSED=$(( PHASES_PASSED + 1 ))
    PHASE_STATUS["$1"]="PASS"
}
phase_fail() {
    fail_msg "PHASE FAILED: $1"
    PHASES_FAILED=$(( PHASES_FAILED + 1 ))
    PHASE_STATUS["$1"]="FAIL"
}

run_tool() {
    local script="$1"; shift
    local args=("$@")
    if [[ ! -f "$TOOLS_DIR/$script" ]]; then
        warn "$script not found at $TOOLS_DIR/$script"
        return 1
    fi
    (( ASSUME_YES )) && args+=("--yes")
    (( DRY_RUN ))    && args+=("--dry-run")
    bash "$TOOLS_DIR/$script" "${args[@]}"
}

# ── Argument parsing ──────────────────────────────────────────────────────
apply_all() {
    DO_ENV=1; DO_GPU=1; DO_SCAFFOLD=1; DO_FIX=1
    DO_BUILD=1; DO_TEST=1; DO_VERIFY=1
    (( SKIP_GPU )) && DO_GPU=0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)          apply_all ;;
        --env)          DO_ENV=1 ;;
        --gpu)          DO_GPU=1 ;;
        --scaffold)     DO_SCAFFOLD=1 ;;
        --fix)          DO_FIX=1 ;;
        --build)        DO_BUILD=1 ;;
        --test)         DO_TEST=1 ;;
        --verify)       DO_VERIFY=1 ;;
        --yes|-y)       ASSUME_YES=1 ;;
        --dry-run)      DRY_RUN=1 ;;
        --release)      BUILD_RELEASE=1 ;;
        --skip-gpu)     SKIP_GPU=1; DO_GPU=0 ;;
        --skip-nvidia)  SKIP_NVIDIA=1 ;;
        --profile)      ENV_PROFILE="${2:-rust}"; shift ;;
        --version)      WS_VERSION="${2:-v4}"; shift ;;
        --help|-h)      sed -n '2,45p' "$0"; exit 0 ;;
        *) echo "${RED}Unknown option: $1${NC}"; exit 1 ;;
    esac
    shift
done

if (( DO_ENV+DO_GPU+DO_SCAFFOLD+DO_FIX+DO_BUILD+DO_TEST+DO_VERIFY == 0 )); then
    sed -n '2,45p' "$0"; exit 0
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PRE-FLIGHT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
banner
say "Log      : $LOG"
say "Repo     : $SCRIPT_DIR"
say "Workspace: $WS_DIR"
say "Tools    : $TOOLS_DIR"
(( DRY_RUN ))    && warn "DRY RUN MODE — no changes will be made"
(( ASSUME_YES )) && info "Non-interactive mode (--yes)"

if (( ! DRY_RUN )) && command -v dnf &>/dev/null; then
    sudo -v 2>/dev/null || warn "sudo not available — some steps may fail"
    ( while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done ) 2>/dev/null &
    SUDO_PID=$!
    trap 'kill $SUDO_PID 2>/dev/null; true' EXIT
fi

[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" 2>/dev/null || true

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 1: ENVIRONMENT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if (( DO_ENV )); then
    phase_start "ENVIRONMENT"
    if run_tool "bahywaylab.sh" --profile "$ENV_PROFILE"; then
        phase_pass "ENVIRONMENT"
    else
        phase_fail "ENVIRONMENT"
        warn "Environment setup had issues — continuing"
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 2: GPU STACK
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if (( DO_GPU )); then
    phase_start "GPU STACK"
    gpu_args=()
    (( SKIP_NVIDIA )) && gpu_args+=("--skip-nvidia")
    if run_tool "setup-gpu-stack.sh" "${gpu_args[@]+"${gpu_args[@]}"}"; then
        phase_pass "GPU STACK"
    else
        phase_fail "GPU STACK"
        warn "GPU stack had issues — continuing"
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 3: WORKSPACE SCAFFOLD
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
scaffold_workspace() {
    say "Scaffolding BahyWay v4.0 Cargo workspace → $WS_DIR"

    if [[ -f "$WS_DIR/Cargo.toml" ]] && (( ! DRY_RUN )); then
        ok "Workspace already exists — skipping"
        return 0
    fi

    (( DRY_RUN )) && { say "[dry-run] Would create workspace at $WS_DIR"; return 0; }

    mkdir -p "$WS_DIR"/{crates,bin,tests/{e2e,chaos,fixtures},logs}

    # ── Workspace Cargo.toml — 35 crates, 10 layers ──────────────────────
    cat > "$WS_DIR/Cargo.toml" <<'WSTOML'
[workspace]
resolver = "2"
members = [
    # Layer 0: Foundation
    "crates/bahyway-core",
    "crates/bahyway-crc",

    # Layer 1: KAKI Identity
    "crates/enkidb-kaki",
    "crates/enkidb-vector-id",

    # Layer 2: Storage Substrate
    "crates/enkidb-block",
    "crates/enkidb-journal",
    "crates/enkidb-storage",
    "crates/enkidb-snapshot",
    "crates/enkidb-recovery",

    # Layer 3: Native Indexes (7 modules in one crate)
    "crates/enkidb-indexes",

    # Layer 4: EnkiDB Engine
    "crates/enkidb-engine",
    "crates/enkidb-query",

    # Layer 5: Operational Engines
    "crates/story-engine",
    "crates/score-engine",
    "crates/alert-engine",
    "crates/snapshot-job",

    # Layer 6: Cross-Tribe / IDU
    "crates/idu-prober",
    "crates/idu-batching",

    # Layer 7: Templates / Governance
    "crates/template-engine",
    "crates/template-library",
    "crates/diagnosis-templates",
    "crates/damadmbok-dictionary",

    # Layer 8: Pipeline / Stations
    "crates/adad-gate",
    "crates/musaru-security",
    "crates/compare-tribe-schema",
    "crates/vgca-validation",
    "crates/data-structure-station",
    "crates/data-cleansing-station",
    "crates/data-steward-station",
    "crates/permanent-storage",

    # Layer 9: Languages
    "crates/aaol",
    "crates/heptascript",

    # Layer 10: Runtime / OS
    "crates/eridu-runtime",
    "crates/eridu-scheduler",
    "crates/eridu-supervisor",

    # Layer 11: UI / IDE
    "crates/dubsar-ide",
    "crates/dubsar-visualizer",

    # Binaries
    "bin/bahyway-server",
    "bin/bahyway-cli",
    "bin/najaf-ingest",
]

[workspace.package]
version    = "4.0.0"
edition    = "2021"
authors    = ["Bahaa Fadam — BahyWay Sovereign Ecosystem"]
license    = "Proprietary"
repository = "https://github.com/bahyway/enkidb"

[workspace.dependencies]
# Serialization
serde              = { version = "1",    features = ["derive"] }
serde_json         = "1"
# Async runtime
tokio              = { version = "1",    features = ["full"] }
# Byte buffers
bytes              = "1"
# Lexer
logos              = "0.14"
# Error handling
thiserror          = "1"
# Tracing
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
# Math / random
rand               = "0.9"
# UUID for KAKI identity
uuid               = { version = "1",   features = ["v4", "serde"] }
# Memory-mapped I/O for storage
memmap2            = "0.9"
WSTOML
    ok "Cargo.toml (workspace root, 38 members)"

    # ── Helper: library crate with explicit deps list ─────────────────────
    mk_lib() {
        local name="$1" desc="$2"
        shift 2
        local deps=("$@")   # remaining args = extra [dependencies] lines
        local dir="$WS_DIR/crates/$name"
        mkdir -p "$dir/src"
        {
            cat <<CTOML
[package]
name              = "$name"
version.workspace = true
edition.workspace = true
authors.workspace = true
description       = "$desc"

[dependencies]
thiserror = { workspace = true }
tracing   = { workspace = true }
CTOML
            for dep in "${deps[@]+"${deps[@]}"}"; do echo "$dep"; done
        } > "$dir/Cargo.toml"
        cat > "$dir/src/lib.rs" <<CLIB
//! $name — $desc
#![allow(dead_code)]

pub mod prelude {}
CLIB
        ok "  $name"
    }

    # ── Helper: binary crate ──────────────────────────────────────────────
    mk_bin_crate() {
        local name="$1" desc="$2"
        shift 2
        local deps=("$@")
        local dir="$WS_DIR/bin/$name"
        mkdir -p "$dir/src"
        {
            cat <<BTOML
[package]
name              = "$name"
version.workspace = true
edition.workspace = true
authors.workspace = true
description       = "$desc"

[[bin]]
name = "$name"
path = "src/main.rs"

[dependencies]
tracing            = { workspace = true }
tracing-subscriber = { workspace = true }
tokio              = { workspace = true }
BTOML
            for dep in "${deps[@]+"${deps[@]}"}"; do echo "$dep"; done
        } > "$dir/Cargo.toml"
        cat > "$dir/src/main.rs" <<BMAIN
//! $name — $desc
#[tokio::main]
async fn main() {
    println!("𒁾 $name — BahyWay.Ecosystem v4.0");
}
BMAIN
        ok "  $name"
    }

    # ════════════════════════════════════════════════════════════════════
    # LAYER 0: FOUNDATION
    # ════════════════════════════════════════════════════════════════════
    say "Layer 0 — Foundation"
    mk_lib "bahyway-core" "Core types, errors, traits used across all crates" \
        'serde  = { workspace = true }' \
        'bytes  = { workspace = true }'

    mk_lib "bahyway-crc"  "Native CRC-16/CCITT implementation (no external deps)" \
        'bahyway-core = { path = "../bahyway-core" }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 1: KAKI IDENTITY
    # ════════════════════════════════════════════════════════════════════
    say "Layer 1 — KAKI Identity"
    mk_lib "enkidb-kaki" "16-byte KAKI Identity, Event, CrossTribe types" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'bahyway-crc   = { path = "../bahyway-crc" }' \
        'serde         = { workspace = true }' \
        'uuid          = { workspace = true }'

    mk_lib "enkidb-vector-id" "Vector IDs for Default.Jobs and Default.Indexes" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 2: STORAGE SUBSTRATE
    # ════════════════════════════════════════════════════════════════════
    say "Layer 2 — Storage Substrate"
    mk_lib "enkidb-block" "64MB block-aligned cold storage format" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'bytes         = { workspace = true }' \
        'memmap2       = { workspace = true }'

    mk_lib "enkidb-storage" "Native storage primitives: mmap, fsync barriers" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-block  = { path = "../enkidb-block" }' \
        'memmap2       = { workspace = true }'

    mk_lib "enkidb-journal" "Append-only Journal — sovereignty event log" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'enkidb-block  = { path = "../enkidb-block" }' \
        'enkidb-storage = { path = "../enkidb-storage" }'

    mk_lib "enkidb-snapshot" "Snapshot mechanism for point-in-time recovery" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'enkidb-journal = { path = "../enkidb-journal" }'

    mk_lib "enkidb-recovery" "Crash Recovery and Abrupt-Termination handler" \
        'bahyway-core    = { path = "../bahyway-core" }' \
        'enkidb-journal  = { path = "../enkidb-journal" }' \
        'enkidb-snapshot = { path = "../enkidb-snapshot" }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 3: NATIVE INDEXES (7 modules, one crate)
    # ════════════════════════════════════════════════════════════════════
    say "Layer 3 — Native Indexes"
    local idx_dir="$WS_DIR/crates/enkidb-indexes"
    mkdir -p "$idx_dir/src"/{identity,sovereignty,typerole,temporal,colorid,eav,snapshot_idx}
    cat > "$idx_dir/Cargo.toml" <<'IDXTOML'
[package]
name              = "enkidb-indexes"
version.workspace = true
edition.workspace = true
authors.workspace = true
description       = "All seven native EnkiDB indexes (one crate, 7 modules)"

[dependencies]
thiserror    = { workspace = true }
tracing      = { workspace = true }
bytes        = { workspace = true }
bahyway-core = { path = "../bahyway-core" }
enkidb-kaki  = { path = "../enkidb-kaki" }
enkidb-storage = { path = "../enkidb-storage" }
IDXTOML
    cat > "$idx_dir/src/lib.rs" <<'IDXLIB'
//! enkidb-indexes — all seven native EnkiDB indexes
//!
//! Index 1: identity      — hash on uuid_hash (O(1) point lookup)
//! Index 2: sovereignty   — sorted array per tribe (range scan)
//! Index 3: typerole      — 3×3 KAKI taxonomy bitmap
//! Index 4: temporal      — native LSM-tree (time range)
//! Index 5: colorid       — native R-tree variant (quality range)
//! Index 6: eav           — inverted index with native bitmaps
//! Index 7: snapshot      — sparse B-tree (snapshot pointers)

pub mod identity;
pub mod sovereignty;
pub mod typerole;
pub mod temporal;
pub mod colorid;
pub mod eav;
pub mod snapshot_idx;

pub mod prelude {}
IDXLIB
    for module in identity sovereignty typerole temporal colorid eav snapshot_idx; do
        cat > "$idx_dir/src/$module/mod.rs" <<MODRS
//! $module index module
MODRS
    done
    ok "  enkidb-indexes (7 modules)"

    # ════════════════════════════════════════════════════════════════════
    # LAYER 4: ENKIDB ENGINE
    # ════════════════════════════════════════════════════════════════════
    say "Layer 4 — EnkiDB Engine"
    mk_lib "enkidb-engine" "Main EnkiDB orchestration engine" \
        'bahyway-core     = { path = "../bahyway-core" }' \
        'enkidb-kaki      = { path = "../enkidb-kaki" }' \
        'enkidb-storage   = { path = "../enkidb-storage" }' \
        'enkidb-journal   = { path = "../enkidb-journal" }' \
        'enkidb-snapshot  = { path = "../enkidb-snapshot" }' \
        'enkidb-recovery  = { path = "../enkidb-recovery" }' \
        'enkidb-indexes   = { path = "../enkidb-indexes" }' \
        'tokio            = { workspace = true }'

    mk_lib "enkidb-query" "Query execution against indexes and journal" \
        'bahyway-core   = { path = "../bahyway-core" }' \
        'enkidb-kaki    = { path = "../enkidb-kaki" }' \
        'enkidb-engine  = { path = "../enkidb-engine" }' \
        'enkidb-indexes = { path = "../enkidb-indexes" }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 5: OPERATIONAL ENGINES
    # ════════════════════════════════════════════════════════════════════
    say "Layer 5 — Operational Engines"
    mk_lib "story-engine"  "StoryEngine — CQRS read-side projection" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-query  = { path = "../enkidb-query" }' \
        'serde         = { workspace = true }'

    mk_lib "score-engine"  "Quality scoring engine (ColorID computation)" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }'

    mk_lib "alert-engine"  "ColorID drift monitoring and alerting [NEW v4.0]" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'score-engine  = { path = "../score-engine" }'

    mk_lib "snapshot-job"  "Snapshot_Job scheduler" \
        'bahyway-core    = { path = "../bahyway-core" }' \
        'enkidb-snapshot = { path = "../enkidb-snapshot" }' \
        'enkidb-engine   = { path = "../enkidb-engine" }' \
        'tokio           = { workspace = true }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 6: CROSS-TRIBE / IDU
    # ════════════════════════════════════════════════════════════════════
    say "Layer 6 — Cross-Tribe / IDU"
    mk_lib "idu-prober"   "Shamash IDU Probe — cross-tribe identity resolution" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'enkidb-query  = { path = "../enkidb-query" }'

    mk_lib "idu-batching" "IDU Probe Batching — bulk cross-tribe ops [NEW v4.0]" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'idu-prober    = { path = "../idu-prober" }' \
        'tokio         = { workspace = true }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 7: TEMPLATES / GOVERNANCE
    # ════════════════════════════════════════════════════════════════════
    say "Layer 7 — Templates / Governance"
    mk_lib "template-engine"      "Template Engine — Default + Stakeholder templates" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'serde         = { workspace = true }' \
        'serde_json    = { workspace = true }'

    mk_lib "template-library"     "tribe.templates — template library as a Tribe" \
        'bahyway-core    = { path = "../bahyway-core" }' \
        'template-engine = { path = "../template-engine" }'

    mk_lib "diagnosis-templates"  "ColorID Diagnosis Templates [NEW v4.0]" \
        'bahyway-core    = { path = "../bahyway-core" }' \
        'template-engine = { path = "../template-engine" }' \
        'alert-engine    = { path = "../alert-engine" }'

    mk_lib "damadmbok-dictionary" "DAMA-DMBOK Dictionary Layer — data governance vocab" \
        'bahyway-core    = { path = "../bahyway-core" }' \
        'template-engine = { path = "../template-engine" }' \
        'serde           = { workspace = true }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 8: PIPELINE / STATIONS
    # ════════════════════════════════════════════════════════════════════
    say "Layer 8 — Pipeline / Stations"
    for station in \
        "adad-gate:Adad Gate — ingestion entry point" \
        "musaru-security:Musarû Security check station" \
        "compare-tribe-schema:Compare_Tribe_Schema validation station" \
        "vgca-validation:VGCA validation beam station" \
        "data-structure-station:Data Structure Station" \
        "data-cleansing-station:Data Cleansing Station" \
        "data-steward-station:Data Steward Station" \
        "permanent-storage:PERMANENT Golden Records storage"
    do
        local sname="${station%%:*}"
        local sdesc="${station#*:}"
        mk_lib "$sname" "$sdesc" \
            'bahyway-core    = { path = "../bahyway-core" }' \
            'enkidb-kaki     = { path = "../enkidb-kaki" }' \
            'enkidb-engine   = { path = "../enkidb-engine" }' \
            'template-engine = { path = "../template-engine" }' \
            'score-engine    = { path = "../score-engine" }'
    done

    # ════════════════════════════════════════════════════════════════════
    # LAYER 9: LANGUAGES
    # ════════════════════════════════════════════════════════════════════
    say "Layer 9 — Languages"
    mk_lib "aaol"        "AAOL — Akkadian Actor Orchestration Language (.akk)" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'logos         = { workspace = true }' \
        'serde         = { workspace = true }'

    mk_lib "heptascript" "HeptaScript — 7-Dimensional Query Language (.hepta)" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-query  = { path = "../enkidb-query" }' \
        'logos         = { workspace = true }' \
        'serde         = { workspace = true }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 10: RUNTIME / OS
    # ════════════════════════════════════════════════════════════════════
    say "Layer 10 — Runtime / OS"
    mk_lib "eridu-runtime"    "UrOS runtime daemon" \
        'bahyway-core   = { path = "../bahyway-core" }' \
        'enkidb-engine  = { path = "../enkidb-engine" }' \
        'tokio          = { workspace = true }'

    mk_lib "eridu-scheduler"  "UrOS job scheduling" \
        'bahyway-core   = { path = "../bahyway-core" }' \
        'eridu-runtime  = { path = "../eridu-runtime" }' \
        'tokio          = { workspace = true }'

    mk_lib "eridu-supervisor" "UrOS system supervisor / process management" \
        'bahyway-core     = { path = "../bahyway-core" }' \
        'eridu-runtime    = { path = "../eridu-runtime" }' \
        'eridu-scheduler  = { path = "../eridu-scheduler" }' \
        'tokio            = { workspace = true }'

    # ════════════════════════════════════════════════════════════════════
    # LAYER 11: UI / IDE
    # ════════════════════════════════════════════════════════════════════
    say "Layer 11 — UI / IDE"
    mk_lib "dubsar-ide"        "DubSar IDE main (VSCodium integration)" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'heptascript   = { path = "../heptascript" }' \
        'aaol          = { path = "../aaol" }'

    mk_lib "dubsar-visualizer" "ColorID visualization engine [NEW v4.0]" \
        'bahyway-core  = { path = "../bahyway-core" }' \
        'enkidb-kaki   = { path = "../enkidb-kaki" }' \
        'score-engine  = { path = "../score-engine" }'

    # ════════════════════════════════════════════════════════════════════
    # BINARIES
    # ════════════════════════════════════════════════════════════════════
    say "Bin crates"
    mk_bin_crate "bahyway-server" "Main BahyWay daemon" \
        'bahyway-core     = { path = "../../crates/bahyway-core" }' \
        'enkidb-engine    = { path = "../../crates/enkidb-engine" }' \
        'eridu-runtime    = { path = "../../crates/eridu-runtime" }' \
        'eridu-supervisor = { path = "../../crates/eridu-supervisor" }'

    mk_bin_crate "bahyway-cli"    "CLI: HeptaScript REPL + admin ops" \
        'bahyway-core  = { path = "../../crates/bahyway-core" }' \
        'heptascript   = { path = "../../crates/heptascript" }' \
        'aaol          = { path = "../../crates/aaol" }' \
        'enkidb-query  = { path = "../../crates/enkidb-query" }'

    mk_bin_crate "najaf-ingest"   "Najaf cemetery data ingestion binary" \
        'bahyway-core = { path = "../../crates/bahyway-core" }' \
        'adad-gate    = { path = "../../crates/adad-gate" }' \
        'enkidb-engine = { path = "../../crates/enkidb-engine" }'

    # ── .gitignore ────────────────────────────────────────────────────────
    cat > "$WS_DIR/.gitignore" <<'GI'
/target/
**/*.rs.bk
*.swp
.DS_Store
logs/*.log
logs/*.report
GI

    ok "Workspace scaffold complete"
    local count
    count=$(find "$WS_DIR" -name "Cargo.toml" | grep -vc "^$WS_DIR/Cargo.toml$")
    info "$count crates created under $WS_DIR"
    info "Layers: Foundation(2) → KAKI(2) → Storage(5) → Indexes(1×7mod)"
    info "        → Engine(2) → Ops(4) → IDU(2) → Templates(4)"
    info "        → Pipeline(8) → Languages(2) → Runtime(3) → UI(2) → Bin(3)"
}

if (( DO_SCAFFOLD )); then
    phase_start "WORKSPACE SCAFFOLD"
    if scaffold_workspace; then
        phase_pass "WORKSPACE SCAFFOLD"
    else
        phase_fail "WORKSPACE SCAFFOLD"
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 4: CODE FIX
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if (( DO_FIX )); then
    phase_start "CODE FIX"
    if WORKSPACE_PATH="$WS_DIR" run_tool "Invoke-CodeFix.sh" --category all; then
        phase_pass "CODE FIX"
    else
        phase_fail "CODE FIX"
        warn "Code fix had issues — continuing"
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 5: BUILD
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
run_cargo() {
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" 2>/dev/null || true
    if [[ ! -f "$WS_DIR/Cargo.toml" ]]; then
        fail_msg "Workspace not found — run --scaffold first"; return 1
    fi
    if (( DRY_RUN )); then
        say "[dry-run] Would run: cargo $* in $WS_DIR"; return 0
    fi
    pushd "$WS_DIR" > /dev/null
    cargo "$@" 2>&1 | tee -a "$LOG"
    local rc=${PIPESTATUS[0]}
    popd > /dev/null
    return $rc
}

if (( DO_BUILD )); then
    phase_start "BUILD"
    build_ok=1
    say "cargo check --workspace"
    run_cargo check --workspace || build_ok=0

    if (( build_ok )); then
        say "cargo build --workspace"
        if (( BUILD_RELEASE )); then
            run_cargo build --workspace --release || build_ok=0
        else
            run_cargo build --workspace || build_ok=0
        fi
    fi

    (( build_ok )) && phase_pass "BUILD" || phase_fail "BUILD"
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 6: TEST
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if (( DO_TEST )); then
    phase_start "TEST"
    if run_cargo test --workspace; then
        phase_pass "TEST"
    else
        phase_fail "TEST"
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 7: VERIFY
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
verify_tool() {
    local label="$1"; shift
    "$@" &>/dev/null 2>&1 && ok "$label" || warn "$label — not found"
}

if (( DO_VERIFY )); then
    phase_start "VERIFY"
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" 2>/dev/null || true

    say "System tools:"
    verify_tool "gcc"     gcc --version
    verify_tool "cmake"   cmake --version
    verify_tool "git"     git --version
    verify_tool "curl"    curl --version

    say "Rust toolchain:"
    verify_tool "rustc"          rustc --version
    verify_tool "cargo"          cargo --version
    verify_tool "rust-analyzer"  rust-analyzer --version
    verify_tool "bacon"          bacon --version
    verify_tool "sccache"        sccache --version

    say "Workspace:"
    if [[ -f "$WS_DIR/Cargo.toml" ]]; then
        ok "Cargo.toml present"
        crate_count=$(find "$WS_DIR" -name "Cargo.toml" | grep -vc "^$WS_DIR/Cargo.toml$" || true)
        ok "$crate_count crates in workspace"
    else
        warn "Not scaffolded — run --scaffold"
    fi

    say "BahyWay tool scripts:"
    for t in bahywaylab.sh setup-gpu-stack.sh build_Final.sh dev.sh compare_versions.sh Invoke-CodeFix.sh; do
        [[ -f "$TOOLS_DIR/$t" ]] && ok "$t" || warn "$t — missing from $TOOLS_DIR"
    done

    phase_pass "VERIFY"
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SUMMARY
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
section "SUMMARY"

{
echo "╔═══════════════════════════════════════════════════════════════════╗"
printf "║  𒁾  BahyWay.Ecosystem v4.0  %-36s║\n" "— Run Complete"
echo "╠═══════════════════════════════════════════════════════════════════╣"
printf "║  %-20s : %-44s║\n" "Phases run"     "$PHASES_RUN"
printf "║  %-20s : %-44s║\n" "Passed"         "$PHASES_PASSED"
printf "║  %-20s : %-44s║\n" "Failed"         "$PHASES_FAILED"
printf "║  %-20s : %-44s║\n" "Workspace"      "${WS_DIR/$HOME/\~}"
printf "║  %-20s : %-44s║\n" "Log"            "$LOG"
printf "║  %-20s : %-44s║\n" "Finished"       "$(date '+%Y-%m-%d %H:%M:%S')"
echo "╠═══════════════════════════════════════════════════════════════════╣"
for phase in "${!PHASE_STATUS[@]}"; do
    st="${PHASE_STATUS[$phase]}"
    if [[ "$st" == "PASS" ]]; then
        printf "║  ${GREEN}✓${NC} %-20s %-43s║\n" "$phase" ""
    else
        printf "║  ${RED}✗${NC} %-20s %-34s${RED}FAILED${NC}     ║\n" "$phase" ""
    fi
done
echo "╠═══════════════════════════════════════════════════════════════════╣"
echo "║  NEXT STEPS:                                                      ║"
echo "║    cd workspace/bahyway_v4 && cargo build --workspace             ║"
echo "║    bash workspace/scripts/tools/dev.sh v4 crates                  ║"
echo "║    bash workspace/scripts/tools/compare_versions.sh v3 v4         ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
} | tee -a "$LOG"

(( PHASES_FAILED == 0 )) && exit 0 || exit 1
