#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
#  DubSar IDE Setup — EriduOSv4Dev (Fedora / Hyper-V)
#  Run once inside the VM: bash /vagrant/.vscode/dubsar-setup.sh
# ═══════════════════════════════════════════════════════════════════
set -euo pipefail

echo "▶ Installing DubSar IDE dependencies..."

# ── System packages ───────────────────────────────────────────────
sudo dnf install -y \
  vulkan-loader vulkan-tools mesa-vulkan-drivers \
  mesa-libGL mesa-dri-drivers \
  fontconfig \
  jetbrains-mono-fonts fira-code-fonts \
  git curl wget unzip \
  zsh util-linux-user

# ── Vulkan (Path B — LavaPipe software renderer) ──────────────────
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json
echo "export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json" >> ~/.zshrc
echo "export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json" >> ~/.bashrc

# Verify Vulkan works
vulkaninfo --summary 2>/dev/null | grep "GPU" || echo "⚠ Vulkan: LavaPipe software renderer active"

# ── Rust (if not already installed) ──────────────────────────────
if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
fi
source "$HOME/.cargo/env"
rustup component add rust-analyzer rust-src clippy rustfmt

# ── Lean 4 (via elan) ────────────────────────────────────────────
if ! command -v elan &>/dev/null; then
  curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh -s -- -y
fi
source "$HOME/.elan/env"
elan install leanprover/lean4:stable

# ── Python tools ─────────────────────────────────────────────────
sudo dnf install -y python3-pip python3-virtualenv
pip3 install --user black pylint flask

# ── VSCodium extensions (Open VSX) ───────────────────────────────
CODIUM=codium
if ! command -v codium &>/dev/null; then
  echo "⚠ codium not found — trying code-oss"
  CODIUM=code-oss
fi

echo "▶ Installing VSCodium extensions..."
$CODIUM --install-extension rust-lang.rust-analyzer      --force
$CODIUM --install-extension serayuzgur.crates             --force
$CODIUM --install-extension tamasfe.even-better-toml     --force
$CODIUM --install-extension vadimcn.vscode-lldb           --force
$CODIUM --install-extension leanprover.lean4              --force
$CODIUM --install-extension ms-python.python              --force
$CODIUM --install-extension cweijan.vscode-postgresql-client2 --force
$CODIUM --install-extension mtxr.sqltools                 --force
$CODIUM --install-extension mtxr.sqltools-driver-pg       --force
$CODIUM --install-extension raczzalan.webgl-glsl-editor   --force
$CODIUM --install-extension eamodio.gitlens               --force
$CODIUM --install-extension mhutchie.git-graph            --force
$CODIUM --install-extension hediet.vscode-drawio          --force
$CODIUM --install-extension bierner.markdown-mermaid      --force
$CODIUM --install-extension PKief.material-icon-theme     --force
$CODIUM --install-extension usernamehw.errorlens          --force
$CODIUM --install-extension aaron-bond.better-comments    --force
$CODIUM --install-extension oderwat.indent-rainbow        --force
$CODIUM --install-extension redhat.vscode-yaml            --force

# ── Register Akkadian syntax grammar ─────────────────────────────
GRAMMAR_DIR="$HOME/.config/VSCodium/User/globalStorage/bahyway.akkadian"
mkdir -p "$GRAMMAR_DIR"
cp /vagrant/.vscode/akkadian-syntax/akkadian.tmLanguage.json "$GRAMMAR_DIR/"

# ── Install EnkiDB theme ──────────────────────────────────────────
THEME_DIR="$HOME/.config/VSCodium/User/globalStorage/bahyway.enkidb-theme"
mkdir -p "$THEME_DIR"
cp /vagrant/.vscode/enkidb-theme.json "$THEME_DIR/"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo " DubSar IDE setup complete."
echo " Launch: codium /vagrant --enable-gpu --no-sandbox"
echo "═══════════════════════════════════════════════════════════"
