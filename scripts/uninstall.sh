#!/bin/bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
# =============================================================================
# Zenritme — Uninstall Script
# =============================================================================
# Removes the zenritme binary, sound assets, manpage, and shell completions
# from the system.
#
# Usage:
#   ./scripts/uninstall.sh              # remove from ~/.local
#   PREFIX=/tmp/zenritme ./scripts/uninstall.sh  # remove from a custom prefix
#   DESTDIR=/tmp/pkg ./scripts/uninstall.sh  # remove from staged /tmp/pkg
#
# This script is safe to run if files do not exist.
# This script does NOT call sudo internally.
# =============================================================================

set -euo pipefail

# --- Configuration (overridable via environment) -----------------------------

readonly PROJECT_NAME="zenritme"
PREFIX="${PREFIX:-${HOME}/.local}"
DESTDIR="${DESTDIR:-}"
BINDIR="${DESTDIR}${PREFIX}/bin"
DATADIR="${DESTDIR}${PREFIX}/share/${PROJECT_NAME}"
MANDIR="${DESTDIR}${PREFIX}/share/man/man1"
BASHCOMPDIR="${DESTDIR}${PREFIX}/share/bash-completion/completions"
ZSHCOMPDIR="${DESTDIR}${PREFIX}/share/zsh/site-functions"
FISHCOMPDIR="${DESTDIR}${PREFIX}/share/fish/vendor_completions.d"

# --- Helper: remove file if it exists, print status -------------------------

remove_file() {
    local target="$1"
    local label="$2"
    if [ -f "${target}" ]; then
        rm -f "${target}"
        echo "Removed ${label}: ${target}"
    else
        echo "Not found (skipped): ${label}"
    fi
}

# --- Helper: remove directory if it exists -----------------------------------

remove_dir() {
    local target="$1"
    local label="$2"
    if [ -d "${target}" ]; then
        rm -rf "${target}"
        echo "Removed ${label}: ${target}"
    else
        echo "Not found (skipped): ${label}"
    fi
}

# --- Uninstall binary --------------------------------------------------------

remove_file "${BINDIR}/${PROJECT_NAME}" "binary"

# --- Uninstall sound assets --------------------------------------------------

remove_dir "${DATADIR}" "sound assets"

# --- Uninstall manpage -------------------------------------------------------

remove_file "${MANDIR}/zenritme.1" "manpage"

# --- Uninstall shell completions ---------------------------------------------

remove_file "${BASHCOMPDIR}/zenritme" "bash completion"
remove_file "${ZSHCOMPDIR}/_zenritme" "zsh completion"
remove_file "${FISHCOMPDIR}/zenritme.fish" "fish completion"

echo ""
echo "Uninstallation complete."
