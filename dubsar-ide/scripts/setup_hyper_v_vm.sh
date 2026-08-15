#!/usr/bin/env bash
# ============================================================
# DubSar IDE Setup — EriduOSv4Dev (Hyper-V VM, Fedora 39)
# Run this inside the VM after SSH login.
# ============================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# ── 1. System packages ──────────────────────────────────────────────────────
info "Installing system packages..."
sudo dnf install -y \
  git curl wget zsh \
  gcc gcc-c++ cmake make ninja-build pkg-config \
  openssl-devel \
  python3 python3-pip python3-devel \
  zeromq zeromq-devel \
  vulkan-loader vulkan-headers glslang \
  mesa-libGL mesa-vulkan-drivers \
  2>/dev/null || warn "Some packages may have failed — check output above"

# ── 2. Rust (if not already installed) ─────────────────────────────────────
if ! command -v rustup &>/dev/null; then
  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
else
  info "Rust already installed: $(rustc --version)"
fi
source "$HOME/.cargo/env"

# ── 3. Python / Jupyter ─────────────────────────────────────────────────────
info "Installing Jupyter..."
pip3 install --user jupyter jupyterlab ipykernel ipywidgets pyzmq numpy matplotlib pillow

# ── 4. Build the DubSar kernel ──────────────────────────────────────────────
REPO_DIR="$HOME/bahyway_v4"   # adjust if your repo is elsewhere
DUBSAR_DIR="$REPO_DIR/EnkiDB/dubsar-ide"

if [ ! -d "$DUBSAR_DIR" ]; then
  error "Could not find dubsar-ide at $DUBSAR_DIR"
  error "Clone the repo first: cd ~ && git clone git@github.com:bahyway/enkidb.git"
  exit 1
fi

info "Building dubsar-kernel (software backend)..."
cd "$DUBSAR_DIR"
RUSTFLAGS="" cargo build --release --features wgpu-backend 2>&1 | tail -20

KERNEL_BIN="$DUBSAR_DIR/target/release/dubsar-kernel"
if [ ! -f "$KERNEL_BIN" ]; then
  error "Build failed — see output above"
  exit 1
fi
info "Build OK: $KERNEL_BIN"

# ── 5. Install kernel spec for Jupyter ─────────────────────────────────────
info "Installing DubSar Jupyter kernel spec..."
"$KERNEL_BIN" --install

# ── 6. VSCodium settings ────────────────────────────────────────────────────
CODIUM_SETTINGS="$HOME/.config/VSCodium/User/settings.json"
mkdir -p "$(dirname "$CODIUM_SETTINGS")"

if [ ! -f "$CODIUM_SETTINGS" ]; then
  cat > "$CODIUM_SETTINGS" << 'EOF'
{
  "dubsar.kernelPath": "~/.cargo/bin/../../../bahyway_v4/EnkiDB/dubsar-ide/target/release/dubsar-kernel",
  "dubsar.backend": "software",
  "dubsar.particleCount": 100000,
  "jupyter.askForKernelRestart": false,
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.fontFamily": "'JetBrains Mono', 'Fira Code', monospace",
  "editor.fontSize": 14,
  "workbench.colorTheme": "Default Dark+"
}
EOF
  info "VSCodium settings written to $CODIUM_SETTINGS"
else
  warn "VSCodium settings already exist — not overwriting. Add dubsar.kernelPath manually."
fi

# ── 7. Quick smoke test ─────────────────────────────────────────────────────
info "Running demo (10 steps, saves PNGs to /tmp)..."
"$KERNEL_BIN" --demo

info ""
info "╔══════════════════════════════════════════════════╗"
info "║  DubSar IDE setup complete on EriduOSv4Dev!     ║"
info "╠══════════════════════════════════════════════════╣"
info "║  1. Open VSCodium: codium .                      ║"
info "║  2. Open notebook: dubsar-ide/notebooks/         ║"
info "║        storage_fragmentation.dubsar              ║"
info "║  3. Select kernel: DubSar (Rust + Vulkan)        ║"
info "║  4. Run cells top-to-bottom                      ║"
info "╚══════════════════════════════════════════════════╝"
