#!/bin/bash
# =============================================================================
# Zenritme — Install Script
# =============================================================================
# Installs a pre-built zenritme binary, sound assets, manpage, and shell
# completions to the system.
#
# Usage:
#   ./scripts/install.sh              # install to /usr/local
#   PREFIX=/usr ./scripts/install.sh  # install to /usr
#   DESTDIR=/tmp/pkg ./scripts/install.sh  # stage into /tmp/pkg
#
# Prerequisites:
#   - target/release/zenritme must exist (build with: cargo build --release --locked)
#   - write permission to the target directory (may need sudo)
#
# This script does NOT use curl, wget, or any network access.
# This script does NOT call sudo internally.
# =============================================================================

set -euo pipefail

# --- Configuration (overridable via environment) -----------------------------

readonly PROJECT_NAME="zenritme"
PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"
BINDIR="${DESTDIR}${PREFIX}/bin"
DATADIR="${DESTDIR}${PREFIX}/share/${PROJECT_NAME}"
MANDIR="${DESTDIR}${PREFIX}/share/man/man1"
BASHCOMPDIR="${DESTDIR}${PREFIX}/share/bash-completion/completions"
ZSHCOMPDIR="${DESTDIR}${PREFIX}/share/zsh/site-functions"
FISHCOMPDIR="${DESTDIR}${PREFIX}/share/fish/vendor_completions.d"

# --- Locate project root ----------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# --- Locate binary -----------------------------------------------------------

BINARY="${PROJECT_ROOT}/target/release/${PROJECT_NAME}"

if [ ! -f "${BINARY}" ]; then
    echo "Error: ${BINARY} not found." >&2
    echo "Build the release binary first:" >&2
    echo "  cargo build --release --locked" >&2
    exit 1
fi

# --- Install binary ----------------------------------------------------------

mkdir -p "${BINDIR}"
cp "${BINARY}" "${BINDIR}/${PROJECT_NAME}"
chmod 755 "${BINDIR}/${PROJECT_NAME}"
echo "Installed ${PROJECT_NAME} to ${BINDIR}/${PROJECT_NAME}"

# --- Install sound assets (optional, non-fatal) ------------------------------

SOUND_SRC="${PROJECT_ROOT}/assets/sounds"
if [ -d "${SOUND_SRC}" ]; then
    mkdir -p "${DATADIR}/sounds"
    for wav in "${SOUND_SRC}"/*.wav; do
        [ -f "$wav" ] || continue
        cp "$wav" "${DATADIR}/sounds/"
        echo "  Installed sound: $(basename "$wav")"
    done
    echo "Sound assets installed to ${DATADIR}/sounds/"
fi

# --- Install manpage (optional, non-fatal) -----------------------------------

MANPAGE_SRC="${PROJECT_ROOT}/man/zenritme.1"
if [ -f "${MANPAGE_SRC}" ]; then
    mkdir -p "${MANDIR}"
    cp "${MANPAGE_SRC}" "${MANDIR}/zenritme.1"
    chmod 644 "${MANDIR}/zenritme.1"
    echo "Manpage installed to ${MANDIR}/zenritme.1"
else
    echo "Manpage not found (skipped)"
fi

# --- Install shell completions (optional, non-fatal) -------------------------

# Bash
BASH_COMP_SRC="${PROJECT_ROOT}/completions/zenritme.bash"
if [ -f "${BASH_COMP_SRC}" ]; then
    mkdir -p "${BASHCOMPDIR}"
    cp "${BASH_COMP_SRC}" "${BASHCOMPDIR}/zenritme"
    chmod 644 "${BASHCOMPDIR}/zenritme"
    echo "Bash completion installed to ${BASHCOMPDIR}/zenritme"
fi

# Zsh
ZSH_COMP_SRC="${PROJECT_ROOT}/completions/zenritme.zsh"
if [ -f "${ZSH_COMP_SRC}" ]; then
    mkdir -p "${ZSHCOMPDIR}"
    cp "${ZSH_COMP_SRC}" "${ZSHCOMPDIR}/_zenritme"
    chmod 644 "${ZSHCOMPDIR}/_zenritme"
    echo "Zsh completion installed to ${ZSHCOMPDIR}/_zenritme"
fi

# Fish
FISH_COMP_SRC="${PROJECT_ROOT}/completions/zenritme.fish"
if [ -f "${FISH_COMP_SRC}" ]; then
    mkdir -p "${FISHCOMPDIR}"
    cp "${FISH_COMP_SRC}" "${FISHCOMPDIR}/zenritme.fish"
    chmod 644 "${FISHCOMPDIR}/zenritme.fish"
    echo "Fish completion installed to ${FISHCOMPDIR}/zenritme.fish"
fi

echo ""
echo "Installation complete."