#!/bin/bash
# =============================================================================
# Zenritme — Uninstall Script
# =============================================================================
# Removes the zenritme binary from the system.
#
# Usage:
#   ./scripts/uninstall.sh              # remove from /usr/local/bin
#   PREFIX=/usr ./scripts/uninstall.sh  # remove from /usr/bin
#   DESTDIR=/tmp/pkg ./scripts/uninstall.sh  # remove from staged /tmp/pkg
#
# This script is safe to run if the file does not exist.
# =============================================================================

set -euo pipefail

# --- Configuration (overridable via environment) -----------------------------

readonly PROJECT_NAME="zenritme"
PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"
BINDIR="${DESTDIR}${PREFIX}/bin"
TARGET="${BINDIR}/${PROJECT_NAME}"

# --- Uninstall ---------------------------------------------------------------

if [ ! -f "${TARGET}" ]; then
    echo "${TARGET} does not exist. Nothing to uninstall."
    exit 0
fi

rm -f "${TARGET}"
echo "Removed ${TARGET}"
