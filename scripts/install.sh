#!/bin/bash
# =============================================================================
# Zenritme — Install Script
# =============================================================================
# Installs a pre-built zenritme binary to the system.
#
# Usage:
#   ./scripts/install.sh              # install to /usr/local/bin
#   PREFIX=/usr ./scripts/install.sh  # install to /usr/bin
#   DESTDIR=/tmp/pkg ./scripts/install.sh  # stage into /tmp/pkg
#
# Prerequisites:
#   - target/release/zenritme must exist (build with: cargo build --release --locked)
#   - write permission to the target directory (may need sudo)
#
# This script does NOT use curl, wget, or any network access.
# =============================================================================

set -euo pipefail

# --- Configuration (overridable via environment) -----------------------------

readonly PROJECT_NAME="zenritme"
PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"
BINDIR="${DESTDIR}${PREFIX}/bin"
DATADIR="${DESTDIR}${PREFIX}/share/${PROJECT_NAME}"

# --- Locate binary -----------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

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
