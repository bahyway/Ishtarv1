#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
#  DubSar IDE v4.0 — Sovereign Extension Installer
#  Symlinks the local bahyway.* extensions into VSCodium's
#  extensions directory so they load automatically.
#  Run once per machine: bash .vscode/install-extensions.sh
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXT_SRC="${REPO_ROOT}/.vscode/extensions"

# Detect VSCodium vs VSCode extensions directory
if   [ -d "$HOME/.config/VSCodium/User/extensions" ]; then
  EXT_DIR="$HOME/.config/VSCodium/User/extensions"
elif [ -d "$HOME/.vscode-oss/extensions" ]; then
  EXT_DIR="$HOME/.vscode-oss/extensions"
elif [ -d "$HOME/.vscode/extensions" ]; then
  EXT_DIR="$HOME/.vscode/extensions"
else
  echo "ERROR: Could not locate VSCodium/VSCode extensions directory."
  echo "       Create it manually and re-run this script."
  exit 1
fi

echo "Installing bahyway.* extensions → ${EXT_DIR}"

for ext_dir in "${EXT_SRC}"/bahyway-*; do
  ext_name="$(basename "$ext_dir")"
  target="${EXT_DIR}/${ext_name}"
  if [ -L "$target" ]; then
    echo "  ↺  ${ext_name} (already symlinked — updating)"
    rm "$target"
  elif [ -e "$target" ]; then
    echo "  ⚠  ${ext_name} exists as directory — skipping (remove manually)"
    continue
  fi
  ln -s "$ext_dir" "$target"
  echo "  ✓  ${ext_name} → ${target}"
done

echo ""
echo "Done. Reload VSCodium (Ctrl+Shift+P → 'Developer: Reload Window') to activate."
