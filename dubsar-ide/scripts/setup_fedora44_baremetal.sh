#!/usr/bin/env bash
# ============================================================
# DubSar IDE Setup — Fedora 44 Workstation (Bare Metal)
# GTX 1650 (4GB VRAM) — Vulkan backend enabled
# ============================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
section(){ echo -e "\n${CYAN}══ $* ══${NC}"; }

# ── 1. System packages ──────────────────────────────────────────────────────
section "System packages (Fedora 44)"
sudo dnf install -y \
  git curl wget zsh \
  gcc gcc-c++ cmake make ninja-build pkg-config \
  openssl-devel \
  vulkan-loader vulkan-headers glslang spirv-tools \
  vulkan-tools vulkan-validation-layers \
  mesa-libGL mesa-vulkan-drivers \
  xorg-x11-server-Xwayland wayland-devel libxkbcommon-devel \
  alsa-lib-devel pipewire-devel \
  zeromq zeromq-devel \
  python3 python3-pip python3-devel \
  nodejs npm \
  vscodium || sudo dnf install -y codium

# NVIDIA driver (GTX 1650) — enable RPM Fusion first
if ! lsmod | grep -q nvidia; then
  warn "NVIDIA driver not loaded. Installing via RPM Fusion..."
  sudo dnf install -y \
    "https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm" \
    "https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm" \
    || warn "RPM Fusion install failed — add manually if needed"
  sudo dnf install -y akmod-nvidia xorg-x11-drv-nvidia-cuda || warn "NVIDIA driver install failed"
  info "NVIDIA driver installed. Reboot required before GPU path works."
fi

# ── 2. Rust ──────────────────────────────────────────────────────────────────
section "Rust toolchain"
if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
fi
source "$HOME/.cargo/env"
rustup update stable
info "$(rustc --version)"

# ── 3. Python / Jupyter ─────────────────────────────────────────────────────
section "Python & Jupyter"
pip3 install --user jupyter jupyterlab ipykernel ipywidgets pyzmq numpy matplotlib pillow

# ── 4. Nix (optional but recommended for reproducibility) ───────────────────
section "Nix (optional)"
if ! command -v nix &>/dev/null; then
  read -rp "Install Nix for reproducible dev environment? [y/N] " ans
  if [[ "$ans" =~ ^[Yy] ]]; then
    # Ensure SELinux is permissive first
    sudo setenforce 0 2>/dev/null || true
    curl -L https://nixos.org/nix/install | sh -s -- --daemon
    . /etc/profile.d/nix.sh
  fi
fi

# ── 5. Build dubsar-kernel with Vulkan backend ───────────────────────────────
section "Build dubsar-kernel (Vulkan backend)"
REPO_DIR="$HOME/EnkiDB"   # adjust to your actual clone location
DUBSAR_DIR="$REPO_DIR/dubsar-ide"

cd "$DUBSAR_DIR"

# On bare metal: remove the WGPU_BACKEND=gl override
unset WGPU_BACKEND

info "Building with vulkan-backend feature..."
cargo build --release --features vulkan-backend 2>&1 | tail -30

KERNEL_BIN="$DUBSAR_DIR/target/release/dubsar-kernel"
info "Build OK: $KERNEL_BIN"

# ── 6. Install kernel spec ───────────────────────────────────────────────────
section "Jupyter kernel spec"
"$KERNEL_BIN" --install

# ── 7. Check Vulkan ─────────────────────────────────────────────────────────
section "Vulkan check"
if command -v vulkaninfo &>/dev/null; then
  vulkaninfo --summary 2>/dev/null | head -20
else
  warn "vulkaninfo not found — install vulkan-tools"
fi

# ── 8. VSCodium extension (dev install) ─────────────────────────────────────
section "VSCodium extension"
EXTENSION_DIR="$DUBSAR_DIR/dubsar-extension"
cd "$EXTENSION_DIR"

if command -v npm &>/dev/null; then
  npm install
  npm run compile
  # Package and install
  if command -v vsce &>/dev/null; then
    vsce package --no-dependencies
    codium --install-extension dubsar-ide-*.vsix 2>/dev/null || \
      vscodium --install-extension dubsar-ide-*.vsix 2>/dev/null || \
      warn "Install VSIX manually: codium --install-extension dubsar-ide-0.1.0.vsix"
  else
    warn "vsce not found. Install: npm install -g @vscode/vsce"
    warn "Then run: cd $EXTENSION_DIR && vsce package && codium --install-extension *.vsix"
  fi
fi

# ── 9. VSCodium settings ────────────────────────────────────────────────────
section "VSCodium settings"
CODIUM_SETTINGS="$HOME/.config/VSCodium/User/settings.json"
mkdir -p "$(dirname "$CODIUM_SETTINGS")"

cat > "$CODIUM_SETTINGS" << EOF
{
  "dubsar.kernelPath": "${KERNEL_BIN}",
  "dubsar.backend": "vulkan",
  "dubsar.particleCount": 1000000,
  "dubsar.maxVisible": 500000,
  "jupyter.askForKernelRestart": false,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.fontFamily": "'JetBrains Mono', 'Fira Code', monospace",
  "editor.fontSize": 14,
  "workbench.colorTheme": "Default Dark+"
}
EOF

# ── 10. Demo ─────────────────────────────────────────────────────────────────
section "Demo run"
"$KERNEL_BIN" --demo

info ""
info "╔══════════════════════════════════════════════════════════════╗"
info "║  DubSar IDE setup complete on Fedora 44 bare-metal!         ║"
info "╠══════════════════════════════════════════════════════════════╣"
info "║  GPU: GTX 1650 (4GB) — supports up to ~140M particles       ║"
info "║  For 1B particles: multi-GPU or AMD MI100/NVIDIA A100 needed ║"
info "╠══════════════════════════════════════════════════════════════╣"
info "║  1. Open VSCodium: codium $REPO_DIR                          ║"
info "║  2. Open: dubsar-ide/notebooks/storage_fragmentation.dubsar  ║"
info "║  3. Select kernel: DubSar (Rust + Vulkan)                    ║"
info "║  4. Shift+F5 to open 3D Viewport                             ║"
info "╚══════════════════════════════════════════════════════════════╝"
