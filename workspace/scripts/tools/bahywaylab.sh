#!/usr/bin/env bash
# ============================================================================
#   𒁾  bahyway-lab.sh  𒁾
#   ─────────────────────────────────────────────────────────────────────────
#   Fedora 43 Workstation  ─→  BahyWay Power Lab
#   Tailored for: Eridu OSv1.0, BahyWay.Ecosystem v3.5, AkkadianAOL,
#                 EnkiWay, DubWay, Bevy+Hanabi particle pipelines.
#
#   Author: scribe of DUB.SAR  𒁾   |   target: Fedora 43 Workstation
#   ─────────────────────────────────────────────────────────────────────────
#
#   USAGE
#     chmod +x bahyway-lab.sh
#     ./bahyway-lab.sh --all                       # full power lab
#     ./bahyway-lab.sh --profile rust              # just rust dev
#     ./bahyway-lab.sh --profile bevy              # rust + GPU + bevy deps
#     ./bahyway-lab.sh --profile os                # OS-building stack
#     ./bahyway-lab.sh --rust --gpu --shell        # pick modules
#     ./bahyway-lab.sh --all --no-desktop          # skip GNOME config
#     ./bahyway-lab.sh --list-modules              # show modules and exit
#     ./bahyway-lab.sh --dry-run --all             # preview only
#     ./bahyway-lab.sh --yes --all                 # no prompts
#
#   Log:   /tmp/bahyway-lab.log
# ============================================================================

set -u   # not -e: we WANT to continue past individual package failures

# ---------- Colors -----------------------------------------------------------
if [[ -t 1 ]]; then
    BOLD=$'\033[1m'; DIM=$'\033[2m'
    GREEN=$'\033[32m'; YELLOW=$'\033[33m'; RED=$'\033[31m'
    BLUE=$'\033[34m'; MAGENTA=$'\033[35m'; CYAN=$'\033[36m'
    RESET=$'\033[0m'
else
    BOLD=""; DIM=""; GREEN=""; YELLOW=""; RED=""
    BLUE=""; MAGENTA=""; CYAN=""; RESET=""
fi

LOG="/tmp/bahyway-lab.log"
: > "$LOG"

# ---------- Module flags (default: off; --all turns all on) ------------------
DO_CORE=0       # essentials, build tools
DO_REPOS=0      # RPM Fusion, Flathub
DO_RUST=0       # rustup + cargo extensions
DO_GPU=0        # Vulkan, OpenGL, GLSL, drivers
DO_BEVY=0       # Bevy/wgpu/winit system deps
DO_OS_DEV=0     # Eridu OS build: kernel headers, mock, lorax, mkosi, qemu
DO_CONTAINERS=0 # podman, libvirt, qemu
DO_CRYPTO=0     # libsodium, openssl-devel, gpg
DO_EDITORS=0    # neovim, helix, VS Code (flatpak)
DO_SHELL=0      # zsh, starship, tmux
DO_CLI=0        # ripgrep, fd, bat, eza, zoxide, fzf, just, hyperfine
DO_GIT=0        # git-lfs, gh, lazygit, delta
DO_PERF=0       # perf, flamegraph, valgrind, strace, ltrace
DO_NET=0        # nmap, iperf3, tcpdump, mtr
DO_MONITOR=0    # htop, btop, nvtop, iotop, glances
DO_DOCS=0       # pandoc, mdbook, asciidoctor
DO_FONTS=0      # JetBrains Mono Nerd Font + Noto Sans Cuneiform
DO_DESKTOP=0    # GNOME tweaks + sane defaults

ASSUME_YES=0
DRY_RUN=0

# ---------- CLI parsing ------------------------------------------------------
print_modules() {
    cat <<EOF
${BOLD}Available modules${RESET}
  --core         Essential build tools (gcc, cmake, pkg-config, git, curl)
  --repos        Enable RPM Fusion (free + nonfree) and Flathub
  --rust         Rust toolchain via rustup + cargo extensions
  --gpu          Vulkan, OpenGL, GLSL/SPIR-V, GPU drivers (NVIDIA/AMD/Intel)
  --bevy         Bevy 0.14 + wgpu + winit system dependencies
  --os-dev       Eridu OS build stack: kernel build deps, mock, lorax,
                 mkosi, livecd-tools, xorriso, dracut, grub2, syslinux
  --containers   podman, podman-compose, qemu/KVM, libvirt, virt-manager
  --crypto       libsodium, openssl-devel, gnupg, system crypto libs
  --editors      neovim, helix, VS Code (flatpak)
  --shell        zsh, starship, tmux
  --cli          ripgrep, fd, bat, eza, zoxide, fzf, just, hyperfine, jq, yq
  --git          git-lfs, gh, lazygit, delta
  --perf         perf, valgrind, strace, ltrace, flamegraph, RenderDoc
  --net          nmap, iperf3, tcpdump, mtr, dig, wireshark
  --monitor      htop, btop, nvtop, iotop, glances
  --docs         pandoc, mdbook, asciidoctor, graphviz
  --fonts        JetBrains Mono Nerd Font + Noto Sans Cuneiform (for Akkadian)
  --desktop      GNOME tweaks + dark theme + workspace defaults

${BOLD}Profiles${RESET}
  --profile rust     core + repos + rust + cli + git + editors + shell
  --profile bevy     core + repos + rust + gpu + bevy + cli + git
  --profile os       core + repos + os-dev + containers + perf + net
  --all              everything (recommended for a full power lab)
EOF
}

apply_profile() {
    case "$1" in
        rust)
            DO_CORE=1; DO_REPOS=1; DO_RUST=1
            DO_CLI=1; DO_GIT=1; DO_EDITORS=1; DO_SHELL=1
            ;;
        bevy)
            DO_CORE=1; DO_REPOS=1; DO_RUST=1
            DO_GPU=1; DO_BEVY=1
            DO_CLI=1; DO_GIT=1; DO_PERF=1
            ;;
        os)
            DO_CORE=1; DO_REPOS=1
            DO_OS_DEV=1; DO_CONTAINERS=1
            DO_PERF=1; DO_NET=1
            ;;
        *)
            echo "${RED}Unknown profile: $1${RESET}" >&2
            exit 1 ;;
    esac
}

apply_all() {
    DO_CORE=1; DO_REPOS=1; DO_RUST=1
    DO_GPU=1; DO_BEVY=1
    DO_OS_DEV=1; DO_CONTAINERS=1
    DO_CRYPTO=1; DO_EDITORS=1; DO_SHELL=1
    DO_CLI=1; DO_GIT=1; DO_PERF=1
    DO_NET=1; DO_MONITOR=1; DO_DOCS=1
    DO_FONTS=1; DO_DESKTOP=1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)            apply_all ;;
        --profile)        apply_profile "$2"; shift ;;
        --core)           DO_CORE=1 ;;
        --repos)          DO_REPOS=1 ;;
        --rust)           DO_RUST=1; DO_CORE=1 ;;
        --gpu)            DO_GPU=1; DO_REPOS=1; DO_CORE=1 ;;
        --bevy)           DO_BEVY=1; DO_CORE=1 ;;
        --os-dev)         DO_OS_DEV=1; DO_CORE=1 ;;
        --containers)     DO_CONTAINERS=1 ;;
        --crypto)         DO_CRYPTO=1 ;;
        --editors)        DO_EDITORS=1 ;;
        --shell)          DO_SHELL=1 ;;
        --cli)            DO_CLI=1 ;;
        --git)            DO_GIT=1 ;;
        --perf)           DO_PERF=1 ;;
        --net)            DO_NET=1 ;;
        --monitor)        DO_MONITOR=1 ;;
        --docs)           DO_DOCS=1 ;;
        --fonts)          DO_FONTS=1 ;;
        --desktop)        DO_DESKTOP=1 ;;
        --no-desktop)     DO_DESKTOP=0 ;;
        --no-rustup)      DO_RUST=0 ;;
        --yes|-y)         ASSUME_YES=1 ;;
        --dry-run)        DRY_RUN=1 ;;
        --list-modules)   print_modules; exit 0 ;;
        --help|-h)        sed -n '1,40p' "$0"; print_modules; exit 0 ;;
        *) echo "${RED}Unknown option: $1${RESET}"; exit 1 ;;
    esac
    shift
done

# Default: if nothing chosen, show help
if (( DO_CORE+DO_REPOS+DO_RUST+DO_GPU+DO_BEVY+DO_OS_DEV+DO_CONTAINERS+DO_CRYPTO \
      +DO_EDITORS+DO_SHELL+DO_CLI+DO_GIT+DO_PERF+DO_NET+DO_MONITOR+DO_DOCS \
      +DO_FONTS+DO_DESKTOP == 0 )); then
    sed -n '1,40p' "$0"
    print_modules
    exit 0
fi

# ---------- Helpers ----------------------------------------------------------
banner() {
cat <<'EOF'
                                                                            
   𒁾   ───────────────────────────────────────────────────────────  𒁾    
                  B A H Y W A Y    P O W E R    L A B                       
        Fedora 43 → Eridu OS · BahyWay v3.5 · AkkadianAOL · EnkiWay          
   𒁾   ───────────────────────────────────────────────────────────  𒁾    
EOF
}

section() {
    echo "" | tee -a "$LOG"
    echo "${BOLD}${MAGENTA}━━━ $* ━━━${RESET}" | tee -a "$LOG"
}
say()   { echo "${BOLD}${BLUE}==>${RESET} $*" | tee -a "$LOG"; }
ok()    { echo "${GREEN}  ✓${RESET} $*" | tee -a "$LOG"; }
warn()  { echo "${YELLOW}  ⚠${RESET} $*" | tee -a "$LOG"; }
fail()  { echo "${RED}  ✗${RESET} $*" | tee -a "$LOG"; }
hr()    { echo "${DIM}────────────────────────────────────────${RESET}" | tee -a "$LOG"; }
note()  { echo "${DIM}    $*${RESET}" | tee -a "$LOG"; }

INSTALLED=()
FAILED=()
SKIPPED=()

run() {
    if (( DRY_RUN )); then
        echo "${DIM}[dry-run]${RESET} $*" | tee -a "$LOG"
        return 0
    fi
    "$@"
}

install_one() {
    local pkg="$1"
    if rpm -q "$pkg" &>/dev/null; then
        ok "$pkg already installed"
        SKIPPED+=("$pkg"); return 0
    fi
    if (( DRY_RUN )); then
        echo "${DIM}[dry-run] would install $pkg${RESET}" | tee -a "$LOG"
        return 0
    fi
    if sudo dnf install -y "$pkg" >>"$LOG" 2>&1; then
        ok "installed $pkg"
        INSTALLED+=("$pkg"); return 0
    fi
    fail "could not install $pkg"
    FAILED+=("$pkg"); return 1
}

install_any_of() {
    local label="$1"; shift
    for pkg in "$@"; do
        if rpm -q "$pkg" &>/dev/null; then
            ok "$label: $pkg already installed"
            SKIPPED+=("$pkg"); return 0
        fi
    done
    if (( DRY_RUN )); then
        echo "${DIM}[dry-run] would try one of: $*${RESET}" | tee -a "$LOG"
        return 0
    fi
    for pkg in "$@"; do
        if sudo dnf install -y "$pkg" >>"$LOG" 2>&1; then
            ok "$label: installed $pkg"
            INSTALLED+=("$pkg"); return 0
        fi
    done
    fail "$label: none of [$*] installed"
    FAILED+=("$label"); return 1
}

install_group() {
    local title="$1"; shift
    say "$title"
    for pkg in "$@"; do install_one "$pkg"; done
}

confirm() {
    [[ $ASSUME_YES -eq 1 ]] && return 0
    local prompt="$1"
    read -r -p "${BOLD}${prompt} [y/N]:${RESET} " ans
    [[ "$ans" =~ ^[Yy]$ ]]
}

# ============================================================================
# PRE-FLIGHT
# ============================================================================
banner
say "Log file: $LOG"
(( DRY_RUN )) && warn "DRY RUN MODE — no changes will be made"

if ! command -v dnf &>/dev/null; then
    fail "dnf not found — this script targets Fedora"
    exit 1
fi

FEDORA_VER=$(rpm -E %fedora 2>/dev/null || echo "unknown")
say "Fedora release: $FEDORA_VER"
[[ "$FEDORA_VER" != "43" ]] && warn "tuned for Fedora 43; you are on $FEDORA_VER"

if (( ! DRY_RUN )); then
    sudo -v || { fail "sudo required"; exit 1; }
    ( while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done ) 2>/dev/null &
    SUDO_KEEPALIVE=$!
    trap 'kill $SUDO_KEEPALIVE 2>/dev/null' EXIT
fi

(( ! DRY_RUN )) && {
    say "Refreshing dnf metadata"
    sudo dnf makecache --refresh >>"$LOG" 2>&1 \
        && ok "metadata refreshed" \
        || warn "metadata refresh had issues"
}

# ============================================================================
# MODULE: REPOSITORIES (RPM Fusion + Flathub)
# ============================================================================
if (( DO_REPOS )); then
    section "REPOSITORIES — RPM Fusion + Flathub"

    say "Enabling RPM Fusion (free + nonfree)"
    if (( ! DRY_RUN )); then
        sudo dnf install -y \
            "https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-${FEDORA_VER}.noarch.rpm" \
            "https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-${FEDORA_VER}.noarch.rpm" \
            >>"$LOG" 2>&1 \
            && ok "RPM Fusion enabled" \
            || warn "RPM Fusion install had issues (probably already enabled)"
    fi

    say "Adding Flathub"
    install_one flatpak
    if (( ! DRY_RUN )); then
        flatpak remote-add --if-not-exists flathub \
            https://dl.flathub.org/repo/flathub.flatpakrepo >>"$LOG" 2>&1 \
            && ok "Flathub remote added" \
            || warn "Flathub add had issues"
    fi
fi

# ============================================================================
# MODULE: CORE — essential build tools
# ============================================================================
if (( DO_CORE )); then
    section "CORE — Build Toolchain"
    install_group "Compilers and build tools" \
        gcc gcc-c++ clang lld llvm \
        make cmake meson ninja-build \
        pkg-config autoconf automake libtool \
        git curl wget rsync \
        unzip zip xz tar \
        which file tree
    install_one mold   # very fast linker — Rust loves it
fi

# ============================================================================
# MODULE: RUST — rustup + cargo extensions
# ============================================================================
if (( DO_RUST )); then
    section "RUST — Toolchain & Cargo Extensions"

    if command -v rustc &>/dev/null && [[ -d "$HOME/.cargo" ]]; then
        ok "rustup already installed: $(rustc --version 2>/dev/null)"
    else
        say "Installing rustup (stable + rustfmt + clippy + rust-src + rust-analyzer)"
        if (( ! DRY_RUN )); then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
                | sh -s -- -y --default-toolchain stable \
                --component rustfmt clippy rust-src rust-analyzer llvm-tools-preview \
                >>"$LOG" 2>&1 \
                && ok "rustup installed" \
                || fail "rustup install failed"
        fi
    fi

    # Make cargo available in this session
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

    # Add rustup components (idempotent)
    if command -v rustup &>/dev/null && (( ! DRY_RUN )); then
        for c in rustfmt clippy rust-src rust-analyzer llvm-tools-preview; do
            rustup component add "$c" >>"$LOG" 2>&1 \
                && ok "rustup component: $c" \
                || warn "rustup component $c failed"
        done

        # Wasm target — for any wasm-pack work
        rustup target add wasm32-unknown-unknown >>"$LOG" 2>&1 \
            && ok "target wasm32-unknown-unknown" || true
    fi

    # cargo-binstall first → faster installs of all the others
    if command -v cargo &>/dev/null && (( ! DRY_RUN )); then
        if ! command -v cargo-binstall &>/dev/null; then
            say "Installing cargo-binstall (fast prebuilt binary installer)"
            curl -L --proto '=https' --tlsv1.2 -sSf \
                https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
                | bash >>"$LOG" 2>&1 \
                && ok "cargo-binstall installed" \
                || warn "cargo-binstall install failed"
        fi

        say "Installing cargo extensions (this can take a few minutes)"
        # Tools tuned for BahyWay workflow: 40+ crate workspace, particle pipelines,
        # Bevy hot-reload, AkkadianAOL compiler dev (logos/tower-lsp), audits.
        CARGO_TOOLS=(
            cargo-edit          # cargo add / rm / upgrade
            cargo-watch         # auto-rebuild on file change
            cargo-expand        # expand macros (essential for proc-macro work)
            cargo-nextest       # faster test runner
            cargo-audit         # CVE check (matters for SargonPassport / quppu-way)
            cargo-deny          # license + advisory + duplicate gate
            cargo-outdated      # find stale deps in 40-crate workspace
            cargo-update        # update installed cargo binaries
            cargo-machete       # find unused dependencies
            cargo-bloat         # binary size analysis
            cargo-flamegraph    # profiling for hot paths in particle pipeline
            cargo-modules       # visualize crate module tree
            bacon               # background compiler — best for Bevy iteration
            mdbook              # docs for BahyWay crates
            mdbook-mermaid      # diagrams in docs
            wasm-pack           # if you ever target wasm for visualizations
            sccache             # compile cache (huge speedup on workspace rebuilds)
            typos-cli           # spell check
            just                # command runner (better than make for Rust projects)
            tokei               # count lines across the 40-crate workspace
            hyperfine           # benchmarking
            ripgrep             # rg — also installed via dnf in --cli, harmless
        )
        for tool in "${CARGO_TOOLS[@]}"; do
            if cargo install --list 2>/dev/null | grep -q "^$tool "; then
                ok "$tool already installed"
            else
                if command -v cargo-binstall &>/dev/null; then
                    cargo binstall -y "$tool" >>"$LOG" 2>&1 \
                        && ok "installed $tool" \
                        || { cargo install "$tool" >>"$LOG" 2>&1 \
                                && ok "installed $tool (from source)" \
                                || warn "$tool install failed"; }
                else
                    cargo install "$tool" >>"$LOG" 2>&1 \
                        && ok "installed $tool" \
                        || warn "$tool install failed"
                fi
            fi
        done

        # Configure sccache as default Rust compiler wrapper
        if command -v sccache &>/dev/null; then
            mkdir -p "$HOME/.cargo"
            CARGO_CONF="$HOME/.cargo/config.toml"
            if ! grep -q 'rustc-wrapper' "$CARGO_CONF" 2>/dev/null; then
                cat >> "$CARGO_CONF" <<'EOF'

# sccache — added by bahyway-lab.sh
[build]
rustc-wrapper = "sccache"
EOF
                ok "sccache wired into ~/.cargo/config.toml"
            fi
        fi
    fi
fi

# ============================================================================
# MODULE: GPU — Vulkan, OpenGL, GLSL/SPIR-V, drivers
# ============================================================================
if (( DO_GPU )); then
    section "GPU — Vulkan + OpenGL + GLSL/SPIR-V"

    say "Detecting GPU"
    GPU_INFO=$(lspci -nn | grep -Ei 'vga|3d|display' || true)
    echo "$GPU_INFO" | tee -a "$LOG"
    HAS_NVIDIA=0; HAS_AMD=0; HAS_INTEL=0
    echo "$GPU_INFO" | grep -qi nvidia        && HAS_NVIDIA=1
    echo "$GPU_INFO" | grep -qiE 'amd|radeon' && HAS_AMD=1
    echo "$GPU_INFO" | grep -qi intel         && HAS_INTEL=1

    install_group "Mesa (OpenGL + Vulkan for AMD/Intel)" \
        mesa-dri-drivers mesa-libGL mesa-libGL-devel \
        mesa-libGLU mesa-libGLU-devel mesa-libEGL mesa-libEGL-devel \
        mesa-libgbm mesa-libgbm-devel mesa-vulkan-drivers \
        glx-utils mesa-demos

    install_group "Vulkan core" \
        vulkan-loader vulkan-loader-devel vulkan-tools \
        vulkan-headers vulkan-validation-layers vulkan-validation-layers-devel

    say "GLSL / SPIR-V toolchain"
    install_any_of "glslang"        glslang glslang-devel
    install_any_of "shaderc/glslc"  libshaderc libshaderc-devel shaderc
    install_one    spirv-tools
    install_one    spirv-headers

    if (( HAS_NVIDIA )); then
        say "NVIDIA proprietary driver detected"
        if confirm "Install NVIDIA driver (akmod-nvidia + CUDA)?"; then
            install_one akmod-nvidia
            install_one xorg-x11-drv-nvidia-cuda
            install_one nvidia-vaapi-driver
            warn "NVIDIA kernel module builds AFTER reboot — wait ~5 min before login"
        fi
    fi

    install_group "GPU diagnostics" inxi clinfo ocl-icd ocl-icd-devel
fi

# ============================================================================
# MODULE: BEVY — wgpu / winit system deps
# ============================================================================
if (( DO_BEVY )); then
    section "BEVY 0.14 — wgpu + winit system dependencies"
    install_group "Bevy build deps (audio, windowing, input, fonts)" \
        libudev-devel alsa-lib-devel \
        wayland-devel wayland-protocols-devel libxkbcommon-devel \
        libX11-devel libXcursor-devel libXi-devel libXrandr-devel libXinerama-devel \
        fontconfig-devel freetype-devel \
        openssl-devel \
        systemd-devel
fi

# ============================================================================
# MODULE: OS-DEV — Eridu OSv1.0 build stack
# ============================================================================
if (( DO_OS_DEV )); then
    section "ERIDU OS v1.0 — Build & Compose Stack"

    install_group "Kernel build dependencies" \
        kernel-devel kernel-headers \
        bison flex elfutils-libelf-devel \
        ncurses-devel openssl-devel \
        bc dwarves perl

    install_group "RPM packaging" \
        rpm-build rpm-devel rpmlint rpmdevtools \
        mock fedora-packager spectool

    install_group "ISO + Live media composition" \
        xorriso isomd5sum syslinux \
        grub2 grub2-tools grub2-tools-extra \
        dracut dracut-live \
        squashfs-tools cpio

    install_group "Modern Fedora compose tools" \
        lorax livecd-tools pungi mkosi \
        rpm-ostree

    install_group "Cross-compilation (ARM, RISC-V — for sovereign hardware targets)" \
        gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu \
        gcc-riscv64-linux-gnu binutils-riscv64-linux-gnu

    note "For Eridu OS:  start with mkosi for declarative builds, fall back to"
    note "lorax/livecd-tools for traditional Fedora-derivative live ISOs."
fi

# ============================================================================
# MODULE: CONTAINERS — podman + qemu/KVM + libvirt
# ============================================================================
if (( DO_CONTAINERS )); then
    section "CONTAINERS — podman + KVM + libvirt"

    install_group "Podman stack (Fedora's rootless container default)" \
        podman podman-compose buildah skopeo \
        toolbox distrobox

    install_group "QEMU + KVM + libvirt (for testing Eridu OS images)" \
        qemu-kvm qemu-img qemu-system-x86 \
        libvirt libvirt-client libvirt-daemon-config-network \
        virt-manager virt-install virt-viewer \
        bridge-utils

    if (( ! DRY_RUN )); then
        sudo systemctl enable --now libvirtd >>"$LOG" 2>&1 \
            && ok "libvirtd enabled" || warn "libvirtd enable failed"
        sudo usermod -aG libvirt "$USER" >>"$LOG" 2>&1 \
            && ok "added $USER to libvirt group (re-login required)" \
            || warn "libvirt group add failed"
    fi
fi

# ============================================================================
# MODULE: CRYPTO — system libs for AkkadiCipherEngine / SargonPassport
# (renamed from AkkadiCipherWay — GL-001 seals the "-Way" suffix as fully
# deprecated across v4.0, closing the prior exception for security files)
# ============================================================================
if (( DO_CRYPTO )); then
    section "CRYPTO — System libraries"
    install_group "Crypto dev libraries" \
        openssl-devel libsodium libsodium-devel \
        gnupg2 gnupg2-smime \
        nss-tools p11-kit \
        libgcrypt libgcrypt-devel
    note "ChaCha20-Poly1305, Argon2id, Ed25519 are all in the RustCrypto crates;"
    note "these system libs are mainly for FFI / interop validation."
fi

# ============================================================================
# MODULE: EDITORS — neovim, helix, VS Code
# ============================================================================
if (( DO_EDITORS )); then
    section "EDITORS"
    install_group "Terminal editors" neovim helix vim-enhanced
    if command -v flatpak &>/dev/null && (( ! DRY_RUN )); then
        say "Installing VS Code via Flatpak"
        flatpak install -y flathub com.visualstudio.code >>"$LOG" 2>&1 \
            && ok "VS Code installed (flatpak)" \
            || warn "VS Code flatpak install failed"
    fi
    note "Reminder: DubWay v3.5 (Bevy+egui) is your primary IDE — these are fallbacks."
fi

# ============================================================================
# MODULE: SHELL — zsh + starship + tmux
# ============================================================================
if (( DO_SHELL )); then
    section "SHELL — zsh + starship + tmux"
    install_group "Shell tools" zsh tmux
    # Starship from official installer (always latest)
    if ! command -v starship &>/dev/null && (( ! DRY_RUN )); then
        say "Installing starship prompt"
        curl -sS https://starship.rs/install.sh | sh -s -- -y >>"$LOG" 2>&1 \
            && ok "starship installed" \
            || warn "starship install failed"
    fi
fi

# ============================================================================
# MODULE: CLI — modern command-line replacements
# ============================================================================
if (( DO_CLI )); then
    section "CLI — modern productivity tools"
    install_group "Modern CLI replacements" \
        ripgrep fd-find bat eza zoxide fzf \
        jq yq \
        hyperfine tokei \
        the_silver_searcher \
        htop btop
    install_any_of "just runner" just rust-just
fi

# ============================================================================
# MODULE: GIT — git-lfs, gh, lazygit, delta
# ============================================================================
if (( DO_GIT )); then
    section "GIT — extended toolset"
    install_group "Git ecosystem" \
        git-lfs gh lazygit git-delta tig
    if (( ! DRY_RUN )); then
        git lfs install >>"$LOG" 2>&1 && ok "git-lfs initialized" || true
    fi
fi

# ============================================================================
# MODULE: PERF — profiling, debugging, GPU debug
# ============================================================================
if (( DO_PERF )); then
    section "PERFORMANCE — profiling & debugging"
    install_group "Profilers and debuggers" \
        perf valgrind strace ltrace \
        gdb lldb \
        bpftrace bcc-tools \
        sysprof
    install_any_of "RenderDoc (GPU debug)" renderdoc
    install_any_of "MangoHud (GPU overlay)" mangohud
fi

# ============================================================================
# MODULE: NET — networking diagnostics
# ============================================================================
if (( DO_NET )); then
    section "NETWORK — diagnostics"
    install_group "Network tools" \
        nmap iperf3 tcpdump mtr bind-utils \
        wireshark traceroute nc \
        net-tools iproute
fi

# ============================================================================
# MODULE: MONITOR — system monitoring
# ============================================================================
if (( DO_MONITOR )); then
    section "MONITORING"
    install_group "System monitors" \
        htop btop iotop glances powertop \
        lm_sensors smartmontools
    install_any_of "GPU monitor (nvtop)" nvtop
    if (( ! DRY_RUN )); then
        sudo sensors-detect --auto >>"$LOG" 2>&1 \
            && ok "lm_sensors configured" \
            || warn "sensors-detect skipped"
    fi
fi

# ============================================================================
# MODULE: DOCS — pandoc, mdbook, asciidoctor, graphviz
# ============================================================================
if (( DO_DOCS )); then
    section "DOCUMENTATION"
    install_group "Doc tools" \
        pandoc asciidoctor graphviz \
        plantuml texlive-scheme-basic
    note "mdbook is installed via cargo in --rust"
fi

# ============================================================================
# MODULE: FONTS — coding + cuneiform
# ============================================================================
if (( DO_FONTS )); then
    section "FONTS — coding + Akkadian/cuneiform support"
    install_group "Base font packages" \
        google-noto-fonts-common \
        google-noto-sans-fonts \
        google-noto-serif-fonts \
        google-noto-sans-mono-fonts \
        jetbrains-mono-fonts \
        fira-code-fonts
    # Noto Sans Cuneiform — for rendering 𒁾 and other cuneiform glyphs in DubWay
    install_any_of "Noto Sans Cuneiform" \
        google-noto-sans-cuneiform-fonts \
        google-noto-sans-cuneiform-vfonts

    # JetBrains Mono Nerd Font (icons in starship/eza)
    if (( ! DRY_RUN )); then
        FONT_DIR="$HOME/.local/share/fonts/JetBrainsMonoNerdFont"
        if [[ ! -d "$FONT_DIR" ]]; then
            say "Installing JetBrains Mono Nerd Font"
            mkdir -p "$FONT_DIR"
            curl -L -o /tmp/jbmono.zip \
                https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip \
                >>"$LOG" 2>&1 \
                && unzip -o /tmp/jbmono.zip -d "$FONT_DIR" >>"$LOG" 2>&1 \
                && fc-cache -f "$FONT_DIR" >>"$LOG" 2>&1 \
                && ok "JetBrains Mono Nerd Font installed" \
                || warn "Nerd Font install failed"
            rm -f /tmp/jbmono.zip
        else
            ok "JetBrains Mono Nerd Font already present"
        fi
    fi
fi

# ============================================================================
# MODULE: DESKTOP — GNOME tweaks + power-lab defaults
# ============================================================================
if (( DO_DESKTOP )); then
    section "DESKTOP — GNOME power-lab defaults"
    install_group "GNOME tweaks & extension manager" \
        gnome-tweaks gnome-extensions-app dconf-editor \
        gnome-shell-extension-appindicator

    if command -v gsettings &>/dev/null && (( ! DRY_RUN )); then
        say "Applying GNOME settings (dark theme, focus, workspaces)"

        # Dark theme
        gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark' 2>>"$LOG" \
            && ok "color-scheme: prefer-dark" || true
        gsettings set org.gnome.desktop.interface gtk-theme 'Adwaita-dark' 2>>"$LOG" || true

        # Monospace font (if Nerd Font installed)
        gsettings set org.gnome.desktop.interface monospace-font-name \
            'JetBrainsMono Nerd Font 11' 2>>"$LOG" \
            && ok "monospace font set" || true

        # Static workspaces — 6 of them (one per BAHYWAYOTAP pillar feels right)
        gsettings set org.gnome.mutter dynamic-workspaces false 2>>"$LOG" || true
        gsettings set org.gnome.desktop.wm.preferences num-workspaces 6 2>>"$LOG" \
            && ok "6 static workspaces (one per BAHYWAYOTAP pillar)" || true

        # Tap-to-click + natural scroll OFF (architects scroll the old way)
        gsettings set org.gnome.desktop.peripherals.touchpad tap-to-click true 2>>"$LOG" || true
        gsettings set org.gnome.desktop.peripherals.touchpad natural-scroll false 2>>"$LOG" || true

        # Battery percentage visible
        gsettings set org.gnome.desktop.interface show-battery-percentage true 2>>"$LOG" || true

        # Click-to-minimize on dock
        gsettings set org.gnome.shell.extensions.dash-to-dock click-action 'minimize' 2>>"$LOG" || true

        # Disable hot corners (interrupts deep-focus work)
        gsettings set org.gnome.desktop.interface enable-hot-corners false 2>>"$LOG" || true

        # Show seconds in clock
        gsettings set org.gnome.desktop.interface clock-show-seconds true 2>>"$LOG" || true
        gsettings set org.gnome.desktop.interface clock-show-weekday true 2>>"$LOG" || true

        ok "GNOME defaults applied"
    fi

    note "Recommended GNOME extensions to install manually via Extension Manager:"
    note "  • Pop Shell or Forge   — tiling window manager"
    note "  • Just Perfection      — fine-grained UI control"
    note "  • Vitals               — CPU/GPU/temp in top bar"
    note "  • Caffeine             — disable suspend during long compiles"
    note "  • AppIndicator Support — tray icons (already installed)"
fi

# ============================================================================
# VERIFICATION
# ============================================================================
section "VERIFICATION"

verify() {
    local label="$1"; shift
    if (( DRY_RUN )); then
        echo "${DIM}[dry-run] would verify: $label${RESET}" | tee -a "$LOG"
        return
    fi
    if "$@" &>/dev/null; then ok "$label"
    else warn "$label not working yet"; fi
}

(( DO_CORE ))    && { verify "gcc"     gcc --version
                      verify "cmake"   cmake --version
                      verify "mold"    command -v mold; }
(( DO_RUST ))    && { [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
                      verify "rustc"   rustc --version
                      verify "cargo"   cargo --version
                      verify "rust-analyzer" command -v rust-analyzer
                      verify "bacon"   command -v bacon
                      verify "sccache" command -v sccache; }
(( DO_GPU ))     && { verify "vulkaninfo"       vulkaninfo --summary
                      verify "glxinfo"          glxinfo -B
                      verify "glslangValidator" command -v glslangValidator
                      verify "glslc"            command -v glslc
                      verify "spirv-dis"        command -v spirv-dis; }
(( DO_OS_DEV ))  && { verify "mock"     command -v mock
                      verify "lorax"    command -v lorax
                      verify "mkosi"    command -v mkosi
                      verify "xorriso"  command -v xorriso
                      verify "qemu-kvm" command -v qemu-kvm; }
(( DO_CONTAINERS )) && { verify "podman"      command -v podman
                         verify "virt-manager" command -v virt-manager; }
(( DO_CLI ))     && { verify "rg" command -v rg
                      verify "fd" bash -c 'command -v fd || command -v fdfind'
                      verify "bat" bash -c 'command -v bat || command -v batcat'
                      verify "fzf" command -v fzf; }
(( DO_GIT ))     && { verify "git-lfs" git lfs version
                      verify "gh"      command -v gh; }

# ============================================================================
# SUMMARY
# ============================================================================
section "SUMMARY"
echo "${GREEN}Installed:${RESET} ${#INSTALLED[@]}    ${YELLOW}Already present:${RESET} ${#SKIPPED[@]}    ${RED}Failed:${RESET} ${#FAILED[@]}" | tee -a "$LOG"

if (( ${#FAILED[@]} > 0 )); then
    echo "" | tee -a "$LOG"
    echo "${RED}Failed packages:${RESET}" | tee -a "$LOG"
    for f in "${FAILED[@]}"; do echo "    - $f" | tee -a "$LOG"; done
    echo "Most failures = renamed packages between Fedora versions." | tee -a "$LOG"
    echo "Check the log:  ${BOLD}$LOG${RESET}" | tee -a "$LOG"
fi

# ============================================================================
# NEXT STEPS
# ============================================================================
section "NEXT STEPS"
cat <<EOF | tee -a "$LOG"
  1. ${BOLD}Reboot${RESET} if you installed the NVIDIA driver:
        sudo reboot

  2. ${BOLD}Re-login${RESET} (not just open a new terminal) so the libvirt
     group membership and shell config take effect.

  3. Source the Rust env in current shell:
        source ~/.cargo/env

  4. Smoke-test the full BahyWay stack:
        rustc --version && cargo --version
        vulkaninfo --summary
        bacon --version
        sccache --show-stats
        mkosi --version          # Eridu OS compose tool
        podman run --rm hello-world

  5. Set up your BahyWay workspace:
        mkdir -p ~/bahyway && cd ~/bahyway
        git init bahyway-ecosystem-v3.5

  6. Configure Cargo for your 40-crate workspace (one-time):
        cat ~/.cargo/config.toml   # check sccache + mold linker

  7. Recommended manual installs (not scripted):
        - JetBrains Toolbox (if you ever want CLion alongside DubWay)
        - GNOME extensions via the Extension Manager app
        - Slack / Element / Discord via Flathub for team comms

  8. Full log:  ${BOLD}$LOG${RESET}

   𒁾  May the tablets be inscribed cleanly.  𒁾
EOF
