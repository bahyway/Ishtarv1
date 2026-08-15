#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
#  EriduuOS v4.0 — Master Stack Installer
#  Run inside the Vagrant Fedora Box:
#    bash /vagrant/workspace/eriduu_stack/install.sh
#
#  What this installs:
#  TIER 1 — Native (dnf/cargo/pip): System tools, dev languages
#  TIER 2 — Docker/Podman: Web services (run with docker-compose)
#  TIER 3 — Optional: Heavy tools (Fooocus, Whisper, Cal.com)
# ═══════════════════════════════════════════════════════════════════
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; GOLD='\033[1;33m'; NC='\033[0m'

banner() { echo -e "\n${GOLD}══ $1 ══${NC}"; }
ok()     { echo -e "${GREEN}✓${NC} $1"; }
warn()   { echo -e "${YELLOW}⚠${NC}  $1"; }
info()   { echo -e "${CYAN}▶${NC} $1"; }

# ── Detect if running as vagrant user ────────────────────────────
if [ "$EUID" -eq 0 ]; then
  warn "Running as root. Some steps will use sudo where needed."
fi

banner "EriduuOS v4.0 Stack Installer"
echo "Host: $(hostname) | Fedora $(rpm -E %fedora) | RAM: $(free -h | awk '/^Mem:/{print $2}')"

# ═══════════════════════════════════════════════════════════════════
# TIER 1 — SYSTEM PACKAGES (dnf)
# ═══════════════════════════════════════════════════════════════════
banner "TIER 1 — System Packages"

info "Updating system..."
sudo dnf update -y -q

# ── RPM Fusion (needed for ffmpeg) ───────────────────────────────
info "Adding RPM Fusion repository (required for ffmpeg)..."
sudo dnf install -y -q \
  "https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm" \
  "https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm" \
  2>/dev/null || warn "RPM Fusion already installed or failed — continuing"

info "Installing base tools..."
# Note: dust and tokei installed via cargo below (not in Fedora repos)
# Note: eza may be 'eza' or need cargo on Fedora 39
sudo dnf install -y -q \
  git curl wget unzip tar \
  python3 python3-pip python3-virtualenv \
  podman podman-compose \
  cockpit \
  zsh neovim \
  ripgrep fd-find bat \
  btop \
  gcc gcc-c++ cmake ninja-build pkg-config \
  openssl-devel \
  zstd

# ffmpeg: Fedora ships ffmpeg-free; swap it for full RPM Fusion ffmpeg
info "Installing ffmpeg (swapping ffmpeg-free → ffmpeg)..."
sudo dnf swap -y ffmpeg-free ffmpeg --allowerasing 2>/dev/null && ok "ffmpeg" || \
  warn "ffmpeg swap failed — ffmpeg-free remains (sufficient for most uses)"

# ── Cockpit (system management web UI) ───────────────────────────
info "Enabling Cockpit on port 9090..."
sudo systemctl enable --now cockpit.socket
ok "Cockpit: http://$(hostname -I | awk '{print $1}'):9090"

# ── Podman socket (Docker compatibility) ─────────────────────────
info "Enabling Podman socket for docker-compose compatibility..."
systemctl --user enable --now podman.socket 2>/dev/null || true
sudo systemctl enable --now podman.socket 2>/dev/null || true

# Symlink docker → podman if docker not present
if ! command -v docker &>/dev/null; then
  sudo ln -sf /usr/bin/podman /usr/local/bin/docker 2>/dev/null || true
fi

ok "Podman ready (docker-compose compatible)"

# ═══════════════════════════════════════════════════════════════════
# TIER 1B — RUST TOOLS (cargo)
# ═══════════════════════════════════════════════════════════════════
banner "TIER 1B — Rust Dev Tools"

# Ensure Rust is available
if ! command -v cargo &>/dev/null; then
  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  source "$HOME/.cargo/env"
fi

source "$HOME/.cargo/env" 2>/dev/null || true

info "Installing Rust language tools for BahyWay languages..."

# dust — disk usage analyzer (not in Fedora repos)
cargo install du-dust 2>/dev/null && ok "dust (du-dust)" || warn "dust skipped"

# tokei — code statistics (not in Fedora repos)
cargo install tokei 2>/dev/null && ok "tokei" || warn "tokei skipped"

# tree-sitter — syntax highlighting / parser toolkit for HeptaScript
cargo install --locked tree-sitter-cli 2>/dev/null && ok "tree-sitter-cli" || warn "tree-sitter-cli failed"

# LALRPOP — LR parser generator (HeptaScript/AAOL grammar)
cargo install lalrpop 2>/dev/null && ok "lalrpop" || warn "lalrpop failed"

# gitui — terminal git UI (needs libxcb-devel on Fedora)
sudo dnf install -y -q libxcb-devel 2>/dev/null || true
cargo install gitui 2>/dev/null && ok "gitui" || warn "gitui failed"

# Helix editor — try dnf first, then cargo
sudo dnf install -y -q helix 2>/dev/null && ok "helix (dnf)" || \
  cargo install helix-term --locked 2>/dev/null && ok "helix (cargo)" || \
  warn "helix skipped"

ok "Rust tools installed"

# ═══════════════════════════════════════════════════════════════════
# TIER 1C — PYTHON TOOLS
# ═══════════════════════════════════════════════════════════════════
banner "TIER 1C — Python Tools"

info "Installing Python tools..."
pip3 install --user --quiet \
  duckdb \
  faster-whisper \
  jupyterlab \
  ipywidgets \
  plotly \
  polars \
  pyarrow \
  black pylint

ok "DuckDB (EnkiDW analytical engine)"
ok "faster-whisper (speech recognition)"
ok "JupyterLab (DubSar IDE notebook backend)"
ok "DuckDB + Polars + PyArrow (EnkiDW data stack)"

# evcxr_jupyter: Rust Jupyter kernel — must be installed via cargo, not pip
info "Installing evcxr_jupyter (Rust Jupyter kernel via cargo)..."
cargo install evcxr_jupyter 2>/dev/null && \
  ~/.cargo/bin/evcxr_jupyter --install && \
  ok "evcxr_jupyter (Rust kernel registered with Jupyter)" || \
  warn "evcxr_jupyter failed — install manually: cargo install evcxr_jupyter"

# ═══════════════════════════════════════════════════════════════════
# TIER 2 — OLLAMA (Local LLM)
# ═══════════════════════════════════════════════════════════════════
banner "TIER 2 — Ollama (Local LLM)"

if ! command -v ollama &>/dev/null; then
  info "Installing Ollama..."
  curl -fsSL https://ollama.com/install.sh | sh
  ok "Ollama installed"
else
  ok "Ollama already installed: $(ollama --version)"
fi

info "Starting Ollama service..."
sudo systemctl enable --now ollama 2>/dev/null || \
  nohup ollama serve &>/tmp/ollama.log & disown

warn "To pull a model (pick based on your RAM):"
warn "  7B model  (needs 6GB RAM): ollama pull llama3.2"
warn "  3B model  (needs 3GB RAM): ollama pull llama3.2:3b  ← recommended for 8GB VM"
warn "  Code model:                ollama pull qwen2.5-coder:3b"

# ═══════════════════════════════════════════════════════════════════
# TIER 2 — DOCKER/PODMAN SERVICES
# ═══════════════════════════════════════════════════════════════════
banner "TIER 2 — Docker/Podman Services Setup"

STACK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$STACK_DIR"

# Copy .env if not exists
if [ ! -f .env ]; then
  cp .env.example .env
  warn ".env created from template — EDIT IT before starting services:"
  warn "  nano $STACK_DIR/.env"
fi

# Create config directory
mkdir -p config

# Create multi-database init script for PostgreSQL
cat > config/init-multiple-dbs.sh << 'DBINIT'
#!/bin/bash
set -e
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
  CREATE DATABASE gitea;
  CREATE DATABASE plausible;
  CREATE DATABASE calcom;
  CREATE DATABASE appflowy;
  CREATE DATABASE penpot;
  GRANT ALL PRIVILEGES ON DATABASE gitea TO $POSTGRES_USER;
  GRANT ALL PRIVILEGES ON DATABASE plausible TO $POSTGRES_USER;
  GRANT ALL PRIVILEGES ON DATABASE calcom TO $POSTGRES_USER;
  GRANT ALL PRIVILEGES ON DATABASE appflowy TO $POSTGRES_USER;
  GRANT ALL PRIVILEGES ON DATABASE penpot TO $POSTGRES_USER;
EOSQL
DBINIT
chmod +x config/init-multiple-dbs.sh

# Create minimal Traefik config
cat > config/traefik.yml << 'TRAEFIK'
api:
  dashboard: true
  insecure: true

entryPoints:
  web:
    address: ":80"
  websecure:
    address: ":443"

providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false

log:
  level: INFO
TRAEFIK

ok "Stack configuration ready"

# ═══════════════════════════════════════════════════════════════════
# TIER 3 — yt-dlp (already installed check)
# ═══════════════════════════════════════════════════════════════════
banner "TIER 3 — yt-dlp"

if command -v yt-dlp &>/dev/null; then
  ok "yt-dlp already installed: $(yt-dlp --version)"
  info "Updating yt-dlp..."
  sudo yt-dlp -U 2>/dev/null || pip3 install --user -U yt-dlp
else
  info "Installing yt-dlp..."
  sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp \
    -o /usr/local/bin/yt-dlp
  sudo chmod a+rx /usr/local/bin/yt-dlp
  ok "yt-dlp installed"
fi

# ═══════════════════════════════════════════════════════════════════
# TIER 3 — FOOOCUS (GPU WARNING)
# ═══════════════════════════════════════════════════════════════════
banner "TIER 3 — Fooocus (GPU Required)"

warn "Fooocus requires a real GPU with 4GB+ VRAM"
warn "Your current VM uses LavaPipe (software renderer)"
warn "Skipping Fooocus — install when you have a real GPU:"
warn "  git clone https://github.com/lllyasviel/Fooocus.git ~/fooocus"
warn "  cd ~/fooocus && pip install -r requirements_versions.txt"
warn "  python entry_with_update.py"

# ═══════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════
banner "Installation Complete"

HOST_IP=$(hostname -I | awk '{print $1}')

echo ""
echo -e "${GOLD}════════════════════════════════════════════════════${NC}"
echo -e "${GOLD} EriduuOS v4.0 Service Map${NC}"
echo -e "${GOLD}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${CYAN}NATIVE TOOLS (running now):${NC}"
echo -e "  Cockpit Dashboard   http://$HOST_IP:9090"
echo -e "  Ollama API          http://$HOST_IP:11434"
echo -e "  JupyterLab          jupyter lab --ip=0.0.0.0 (run manually)"
echo ""
echo -e "  ${CYAN}DOCKER SERVICES (start with podman-compose up -d):${NC}"
echo -e "  Open WebUI          http://$HOST_IP:3000   (Ollama chat)"
echo -e "  Gitea               http://$HOST_IP:3300   (self-hosted Git)"
echo -e "  n8n / AAOL bridge   http://$HOST_IP:4040   (workflow automation)"
echo -e "  Plausible           http://$HOST_IP:5050   (analytics)"
echo -e "  Vaultwarden         http://$HOST_IP:5678   (passwords)"
echo -e "  Woodpecker CI       http://$HOST_IP:8050   (CI/CD)"
echo -e "  Penpot              http://$HOST_IP:9001   (design tool)"
echo -e "  AppFlowy            http://$HOST_IP:9002   (Notion alt)"
echo -e "  Cal.com             http://$HOST_IP:9003   (scheduling)"
echo -e "  Netdata             http://$HOST_IP:19999  (monitoring)"
echo ""
echo -e "  ${CYAN}DEV TOOLS (BahyWay Languages):${NC}"
echo -e "  tree-sitter         $(tree-sitter --version 2>/dev/null || echo 'installed')"
echo -e "  DuckDB              $(python3 -c 'import duckdb; print(duckdb.__version__)' 2>/dev/null || echo 'installed')"
echo -e "  faster-whisper      installed (python3 -m faster_whisper)"
echo ""
echo -e "  ${YELLOW}NEXT STEP:${NC}"
echo -e "  1. Edit .env:          nano $STACK_DIR/.env"
echo -e "  2. Start core stack:   cd $STACK_DIR && podman-compose up -d postgres gitea netdata vaultwarden"
echo -e "  3. Start AI stack:     podman-compose up -d ollama openwebui"
echo -e "  4. Pull LLM model:     ollama pull llama3.2:3b"
echo ""
echo -e "${GOLD}════════════════════════════════════════════════════${NC}"
