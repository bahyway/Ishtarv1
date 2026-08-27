#!/usr/bin/env bash
# ============================================================
# 05_pulse_enkidb.sh
# 𒁾 DECREE OF UNIFICATION — EnkiDB master → GitHub
# BahyWay.Ecosystem v4.0 / UrOS v4.0
#
# USAGE:
#   chmod +x 05_pulse_enkidb.sh
#   ./05_pulse_enkidb.sh "commit message"
#
# WHAT IT DOES:
#   1. Checks you are on the master branch
#   2. Runs cargo test --workspace (no broken code gets pushed)
#   3. Shows git status and asks for confirmation
#   4. Commits all changes with your message
#   5. Asks if this is a new release (tags + labels)
#   6. If yes: asks for version, tag message, and release notes
#   7. Creates annotated git tag with full release notes
#   8. Pushes master + tag to GitHub
# ============================================================

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
ok()      { echo -e "${GREEN}[✓]${NC}    $*"; }
err()     { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
ask()     { echo -e "${CYAN}[?]${NC}    $*"; }
section() {
  echo ""
  echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"
  echo -e "${CYAN}  $*${NC}"
  echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"
}

# ── 1. Guard: must be on master ───────────────────────────────────────────────
section "1 — Branch check"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "master" ]; then
  err "You must be on the 'master' branch.\n     Current branch: $CURRENT_BRANCH\n     Run: git checkout master"
fi
ok "On branch: master"

# ── 2. Commit message ─────────────────────────────────────────────────────────
section "2 — Commit message"
MESSAGE="${1:-}"
if [ -z "$MESSAGE" ]; then
  err "Please provide a commit message.\n     Usage: ./05_pulse_enkidb.sh \"your message here\""
fi
info "Commit message: \"$MESSAGE\""

# ── 3. Run tests ──────────────────────────────────────────────────────────────
section "3 — Sovereign Test Gate (cargo test --workspace)"
info "Running full workspace tests — no broken code enters the Golden Manifold..."
if ! cargo test --workspace --quiet 2>&1; then
  err "Tests FAILED. Fix all failures before pulsing.\n     The clay tablet must be correct before the seal is pressed."
fi
ok "All tests pass. The workspace is sovereign."

# ── 4. Git status preview ─────────────────────────────────────────────────────
section "4 — Staging Preview"
DIRTY=$(git status --short)
if [ -z "$DIRTY" ]; then
  warn "Working tree is clean — nothing to commit."
  warn "Continuing to push existing HEAD and optionally tag."
else
  info "Changes to be committed:"
  git status --short
  echo ""
  ask "Proceed with staging all changes? [y/N]"
  read -r CONFIRM
  if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    err "Aborted by user."
  fi
  git add .
  git commit -m "$MESSAGE"
  ok "Committed: $MESSAGE"
fi

# ── 5. Push master ────────────────────────────────────────────────────────────
section "5 — Transmitting to GitHub (master)"
info "Pushing master to origin..."
git push origin master
ok "master pushed to origin."

# ── 6. Release decision ───────────────────────────────────────────────────────
section "6 — Release Decision"
echo ""
ask "Is this push a NEW RELEASE? (creates annotated tag + release notes) [y/N]"
read -r IS_RELEASE

if [[ ! "$IS_RELEASE" =~ ^[Yy]$ ]]; then
  echo ""
  echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
  echo -e "${CYAN}  ✅ PULSE COMPLETE — No release tag created            ${NC}"
  echo -e "${CYAN}  master pushed to GitHub.                              ${NC}"
  echo -e "${CYAN}  Commit: $MESSAGE${NC}"
  echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
  echo ""
  exit 0
fi

# ── 7. Release tagging ────────────────────────────────────────────────────────
section "7 — Release Tagging"

# Show existing tags so user can pick the next version
info "Existing tags:"
git tag --sort=-version:refname | head -10
echo ""

# Auto-suggest next patch version
LATEST_TAG=$(git tag --sort=-version:refname | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "v4.0.0")
MAJOR=$(echo "$LATEST_TAG" | cut -d. -f1 | tr -d 'v')
MINOR=$(echo "$LATEST_TAG" | cut -d. -f2)
PATCH=$(echo "$LATEST_TAG" | cut -d. -f3)
NEXT_PATCH="v${MAJOR}.${MINOR}.$((PATCH + 1))"
NEXT_MINOR="v${MAJOR}.$((MINOR + 1)).0"
NEXT_MAJOR="v$((MAJOR + 1)).0.0"

echo -e "  Suggested versions:"
echo -e "  ${GREEN}[1]${NC} Patch release → ${BOLD}${NEXT_PATCH}${NC}  (bug fixes, small improvements)"
echo -e "  ${GREEN}[2]${NC} Minor release → ${BOLD}${NEXT_MINOR}${NC}  (new features, backward compatible)"
echo -e "  ${GREEN}[3]${NC} Major release → ${BOLD}${NEXT_MAJOR}${NC}  (breaking changes, new architecture)"
echo -e "  ${GREEN}[4]${NC} Custom version"
echo ""
ask "Choose version type [1/2/3/4]:"
read -r VERSION_CHOICE

case "$VERSION_CHOICE" in
  1) NEW_VERSION="$NEXT_PATCH" ;;
  2) NEW_VERSION="$NEXT_MINOR" ;;
  3) NEW_VERSION="$NEXT_MAJOR" ;;
  4)
    ask "Enter custom version (e.g. v4.2.0-alpha):"
    read -r NEW_VERSION
    ;;
  *) err "Invalid choice." ;;
esac

# Validate version format
if [[ ! "$NEW_VERSION" =~ ^v[0-9]+\.[0-9]+ ]]; then
  err "Invalid version format: $NEW_VERSION — must start with 'v' followed by semver (e.g. v4.0.3)"
fi

ok "New version: $NEW_VERSION"

# ── 8. Release notes ──────────────────────────────────────────────────────────
section "8 — Release Notes"

# Auto-generate changelog from commits since last tag
echo ""
info "Commits since $LATEST_TAG:"
git log "${LATEST_TAG}..HEAD" --oneline 2>/dev/null || git log --oneline -10
echo ""

ask "Enter release title (e.g. 'Quantum Freeze Protocol'):"
read -r RELEASE_TITLE

echo ""
info "Enter release notes (what changed in this release)."
info "Press ENTER twice when done:"
RELEASE_NOTES=""
while IFS= read -r line; do
  [ -z "$line" ] && break
  RELEASE_NOTES="${RELEASE_NOTES}${line}\n"
done

# Select release label/category
echo ""
echo -e "  Release labels:"
echo -e "  ${GREEN}[1]${NC} 🔧 fix      — Bug fixes and corrections"
echo -e "  ${GREEN}[2]${NC} ✨ feat     — New features"
echo -e "  ${GREEN}[3]${NC} 🏗️  infra    — Infrastructure and tooling"
echo -e "  ${GREEN}[4]${NC} 📐 algebra  — Mathematical foundation changes"
echo -e "  ${GREEN}[5]${NC} 🗄️  storage  — EnkiDB / EnkiDW / storage layer"
echo -e "  ${GREEN}[6]${NC} 🔐 security — Security, KAKI, cryptography"
echo -e "  ${GREEN}[7]${NC} 🌐 lang     — AkkadianAOL / HeptaScript / Way language"
echo -e "  ${GREEN}[8]${NC} 🎯 release  — Full release milestone"
echo ""
ask "Choose release label [1-8]:"
read -r LABEL_CHOICE

case "$LABEL_CHOICE" in
  1) LABEL="🔧 fix" ;;
  2) LABEL="✨ feat" ;;
  3) LABEL="🏗️  infra" ;;
  4) LABEL="📐 algebra" ;;
  5) LABEL="🗄️  storage" ;;
  6) LABEL="🔐 security" ;;
  7) LABEL="🌐 lang" ;;
  8) LABEL="🎯 release" ;;
  *) LABEL="🎯 release" ;;
esac

# ── 9. Confirm and tag ────────────────────────────────────────────────────────
section "9 — Tag Confirmation"

TAG_MESSAGE="${LABEL} BahyWay.Ecosystem ${NEW_VERSION} — ${RELEASE_TITLE}

${RELEASE_NOTES}
Commit: $(git rev-parse --short HEAD)
Branch: master
Author: $(git config user.name) <$(git config user.email)>
Date:   $(date -u '+%Y-%m-%d %H:%M UTC')

𒁾 The tablets are preserved. Build with Sovereignty.
— DUB.SAR Bahaa Fadam · BahyWay.Ecosystem · Netherlands"

echo ""
info "Tag to be created:"
echo -e "  Version : ${BOLD}${NEW_VERSION}${NC}"
echo -e "  Label   : ${LABEL}"
echo -e "  Title   : ${RELEASE_TITLE}"
echo -e "  Commit  : $(git rev-parse --short HEAD)"
echo ""
ask "Create tag ${NEW_VERSION} and push to GitHub? [y/N]"
read -r FINAL_CONFIRM

if [[ ! "$FINAL_CONFIRM" =~ ^[Yy]$ ]]; then
  warn "Tag creation aborted. master is already pushed."
  exit 0
fi

git tag -a "$NEW_VERSION" -m "$TAG_MESSAGE"
ok "Tag $NEW_VERSION created."

git push origin "$NEW_VERSION"
ok "Tag $NEW_VERSION pushed to GitHub."

# ── Final summary ─────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  𒁾 DECREE OF UNIFICATION COMPLETE                        ${NC}"
echo -e "${CYAN}  ${BOLD}${NEW_VERSION}${NC}${CYAN} — ${RELEASE_TITLE}              ${NC}"
echo -e "${CYAN}  Label  : ${LABEL}                                         ${NC}"
echo -e "${CYAN}  Commit : $(git rev-parse --short HEAD)                               ${NC}"
echo -e "${CYAN}  master → github.com:bahyway/EnkiDB                       ${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo ""
