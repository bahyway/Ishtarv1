#!/usr/bin/env bash
# ============================================================================
#  setup-gpu-stack.sh
#  Fedora 43 GPU + Graphics Development Stack Installer
#  ----------------------------------------------------------------------------
#  Installs Vulkan, OpenGL, GLSL/SPIR-V toolchain, GPU drivers, and Rust/Bevy
#  build dependencies. Designed to keep going when individual packages fail,
#  log everything, and auto-detect GPU vendor (NVIDIA / AMD / Intel).
#
#  Usage:
#    chmod +x setup-gpu-stack.sh
#    ./setup-gpu-stack.sh                    # interactive
#    ./setup-gpu-stack.sh --yes              # no prompts
#    ./setup-gpu-stack.sh --skip-nvidia      # skip NVIDIA driver even if detected
#
#  Log file: /tmp/setup-gpu-stack.log
# ============================================================================

set -u  # treat unset variables as errors (but NOT -e: we want to continue on failures)

# ---------- Colors -----------------------------------------------------------
if [[ -t 1 ]]; then
    BOLD=$'\033[1m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'
    RED=$'\033[31m'; BLUE=$'\033[34m'; RESET=$'\033[0m'
else
    BOLD=""; GREEN=""; YELLOW=""; RED=""; BLUE=""; RESET=""
fi

LOG="/tmp/setup-gpu-stack.log"
: > "$LOG"   # truncate log

ASSUME_YES=0
SKIP_NVIDIA=0
for arg in "$@"; do
    case "$arg" in
        --yes|-y)        ASSUME_YES=1 ;;
        --skip-nvidia)   SKIP_NVIDIA=1 ;;
        --help|-h)
            grep '^#' "$0" | head -20
            exit 0 ;;
    esac
done

# ---------- Helpers ----------------------------------------------------------
say()   { echo "${BOLD}${BLUE}==>${RESET} $*" | tee -a "$LOG"; }
ok()    { echo "${GREEN}  ✓${RESET} $*" | tee -a "$LOG"; }
warn()  { echo "${YELLOW}  ⚠${RESET} $*" | tee -a "$LOG"; }
fail()  { echo "${RED}  ✗${RESET} $*" | tee -a "$LOG"; }
hr()    { echo "----------------------------------------" | tee -a "$LOG"; }

# Track results
INSTALLED=()
FAILED=()

# Install one package; never aborts the script
install_one() {
    local pkg="$1"
    if rpm -q "$pkg" &>/dev/null; then
        ok "$pkg already installed"
        INSTALLED+=("$pkg")
        return 0
    fi
    if sudo dnf install -y "$pkg" >>"$LOG" 2>&1; then
        ok "installed $pkg"
        INSTALLED+=("$pkg")
        return 0
    else
        fail "could not install $pkg (see $LOG)"
        FAILED+=("$pkg")
        return 1
    fi
}

# Try a list of alternative package names; succeed if any one installs.
# Useful when Fedora renames things between versions.
install_any_of() {
    local label="$1"; shift
    for pkg in "$@"; do
        if rpm -q "$pkg" &>/dev/null; then
            ok "$label: $pkg already installed"
            INSTALLED+=("$pkg")
            return 0
        fi
    done
    for pkg in "$@"; do
        if sudo dnf install -y "$pkg" >>"$LOG" 2>&1; then
            ok "$label: installed $pkg"
            INSTALLED+=("$pkg")
            return 0
        fi
    done
    fail "$label: none of [$*] could be installed"
    FAILED+=("$label")
    return 1
}

# Install a group of packages, one at a time so a single failure doesn't abort
install_group() {
    local title="$1"; shift
    say "$title"
    for pkg in "$@"; do
        install_one "$pkg"
    done
}

confirm() {
    [[ $ASSUME_YES -eq 1 ]] && return 0
    local prompt="$1"
    read -r -p "${BOLD}${prompt} [y/N]:${RESET} " ans
    [[ "$ans" =~ ^[Yy]$ ]]
}

# ---------- Pre-flight -------------------------------------------------------
say "Pre-flight checks"

if [[ $EUID -eq 0 ]]; then
    warn "running as root; sudo calls will still work"
fi

if ! command -v dnf &>/dev/null; then
    fail "dnf not found — this script is for Fedora"
    exit 1
fi

FEDORA_VER=$(rpm -E %fedora 2>/dev/null || echo "unknown")
say "Fedora release: $FEDORA_VER"
if [[ "$FEDORA_VER" != "43" ]]; then
    warn "this script targets Fedora 43; you are on $FEDORA_VER (continuing anyway)"
fi

# Test sudo up-front so the rest of the run is non-interactive
sudo -v || { fail "sudo required"; exit 1; }
# Keep sudo alive
( while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done ) 2>/dev/null &
SUDO_KEEPALIVE=$!
trap 'kill $SUDO_KEEPALIVE 2>/dev/null' EXIT

# ---------- Refresh metadata -------------------------------------------------
say "Refreshing dnf metadata"
sudo dnf makecache --refresh >>"$LOG" 2>&1 && ok "metadata refreshed" || warn "metadata refresh had issues"

# ---------- GPU detection ----------------------------------------------------
hr
say "Detecting GPU"
GPU_INFO=$(lspci -nn | grep -Ei 'vga|3d|display' || true)
echo "$GPU_INFO" | tee -a "$LOG"

HAS_NVIDIA=0; HAS_AMD=0; HAS_INTEL=0
echo "$GPU_INFO" | grep -qi nvidia        && HAS_NVIDIA=1
echo "$GPU_INFO" | grep -qiE 'amd|radeon' && HAS_AMD=1
echo "$GPU_INFO" | grep -qi intel         && HAS_INTEL=1

(( HAS_NVIDIA )) && ok "NVIDIA GPU detected"
(( HAS_AMD ))    && ok "AMD GPU detected"
(( HAS_INTEL ))  && ok "Intel GPU detected"
(( HAS_NVIDIA + HAS_AMD + HAS_INTEL == 0 )) && warn "no recognized GPU detected"

# ---------- RPM Fusion (needed for NVIDIA, useful otherwise) -----------------
hr
if (( HAS_NVIDIA && ! SKIP_NVIDIA )); then
    say "Enabling RPM Fusion (required for NVIDIA driver)"
    sudo dnf install -y \
        "https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-${FEDORA_VER}.noarch.rpm" \
        "https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-${FEDORA_VER}.noarch.rpm" \
        >>"$LOG" 2>&1 \
        && ok "RPM Fusion enabled" \
        || warn "RPM Fusion install had issues (may already be enabled)"
fi

# ---------- Core build tools -------------------------------------------------
hr
install_group "Core build tools" \
    gcc gcc-c++ make cmake pkg-config git curl

# ---------- Mesa (OpenGL + Vulkan drivers for AMD/Intel) ---------------------
hr
install_group "Mesa drivers (OpenGL + Vulkan for AMD/Intel)" \
    mesa-dri-drivers \
    mesa-libGL \
    mesa-libGL-devel \
    mesa-libGLU \
    mesa-libGLU-devel \
    mesa-libEGL \
    mesa-libEGL-devel \
    mesa-libgbm \
    mesa-libgbm-devel \
    mesa-vulkan-drivers \
    glx-utils \
    mesa-demos

# ---------- Vulkan core ------------------------------------------------------
hr
install_group "Vulkan core" \
    vulkan-loader \
    vulkan-loader-devel \
    vulkan-tools \
    vulkan-headers \
    vulkan-validation-layers \
    vulkan-validation-layers-devel

# ---------- GLSL / SPIR-V toolchain ------------------------------------------
hr
say "GLSL / SPIR-V shader toolchain"
# Some of these have been renamed across Fedora versions; try alternatives.
install_any_of "glslang compiler"     glslang glslang-devel
install_any_of "shaderc (glslc)"      libshaderc libshaderc-devel shaderc
install_one    spirv-tools
install_one    spirv-headers

# ---------- NVIDIA driver (only if NVIDIA detected) --------------------------
hr
if (( HAS_NVIDIA && ! SKIP_NVIDIA )); then
    say "NVIDIA proprietary driver (kernel module + CUDA support)"
    if confirm "Install proprietary NVIDIA driver? (recommended for gaming/compute)"; then
        install_one akmod-nvidia
        install_one xorg-x11-drv-nvidia-cuda
        install_one nvidia-vaapi-driver
        warn "NVIDIA kernel module builds in the background after reboot — wait ~5 min before logging in"
    else
        warn "skipped NVIDIA driver"
    fi
elif (( SKIP_NVIDIA )); then
    say "Skipping NVIDIA driver (--skip-nvidia)"
fi

# ---------- Rust / Bevy build dependencies -----------------------------------
hr
install_group "Bevy / wgpu / Rust build dependencies" \
    libudev-devel \
    alsa-lib-devel \
    wayland-devel \
    wayland-protocols-devel \
    libxkbcommon-devel \
    libX11-devel \
    libXcursor-devel \
    libXi-devel \
    libXrandr-devel \
    libXinerama-devel \
    fontconfig-devel \
    freetype-devel \
    openssl-devel

# ---------- Optional but useful ---------------------------------------------
hr
install_group "Diagnostics & optional tools" \
    inxi \
    fastfetch \
    clinfo \
    ocl-icd \
    ocl-icd-devel \
    pciutils \
    mesa-vulkan-devel

# ---------- Verification -----------------------------------------------------
hr
say "Verification"

verify() {
    local label="$1"; shift
    if "$@" &>/dev/null; then
        ok "$label"
    else
        warn "$label — not working yet (driver/reboot may be needed)"
    fi
}

verify "vulkaninfo runs"       vulkaninfo --summary
verify "vkcube binary present" command -v vkcube
verify "glxinfo runs"          glxinfo -B
verify "glslangValidator"      command -v glslangValidator
verify "glslc"                 command -v glslc
verify "spirv-dis"             command -v spirv-dis

if (( HAS_NVIDIA && ! SKIP_NVIDIA )); then
    if command -v nvidia-smi &>/dev/null; then
        verify "nvidia-smi" nvidia-smi
    else
        warn "nvidia-smi not yet available — reboot required for NVIDIA driver"
    fi
fi

# ---------- Summary ----------------------------------------------------------
hr
say "Summary"
echo "${GREEN}Installed/verified:${RESET} ${#INSTALLED[@]} packages" | tee -a "$LOG"
if (( ${#FAILED[@]} > 0 )); then
    echo "${RED}Failed:${RESET} ${#FAILED[@]}" | tee -a "$LOG"
    for f in "${FAILED[@]}"; do echo "    - $f" | tee -a "$LOG"; done
    echo "" | tee -a "$LOG"
    echo "Check the log for details:  ${BOLD}$LOG${RESET}" | tee -a "$LOG"
    echo "Most failures are package-name renames; the script tried alternatives where it could." | tee -a "$LOG"
else
    ok "all packages installed cleanly"
fi

hr
say "Next steps"
cat <<EOF | tee -a "$LOG"
  1. If you installed the NVIDIA driver, REBOOT now:
         sudo reboot

  2. After reboot, verify the full stack:
         vulkaninfo --summary
         vkcube                # spinning Vulkan cube
         glxgears              # spinning OpenGL gears
         inxi -Gxxx            # GPU + driver summary
         nvidia-smi            # NVIDIA only

  3. Compile a test shader:
         echo '#version 450
         void main() { gl_Position = vec4(0); }' > test.vert
         glslc test.vert -o test.spv
         spirv-dis test.spv | head

  4. Full log written to:  $LOG
EOF

hr
ok "Done."
