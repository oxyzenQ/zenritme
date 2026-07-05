#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 rezky_nightky (oxyzenQ)
#
# Install zenritme: binary + sound assets + manpage + shell completions.
# Supports --system (system-wide) and --user (default, ~/.local).
# Run WITHOUT sudo: the script escalates via sudo ONLY for --system install steps.

set -euo pipefail

PROJECT_NAME="zenritme"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat <<EOF
Usage: $0 [--system|--user]

  --system   Install system-wide:
               binary       → /usr/bin/${PROJECT_NAME}
               sound assets → /usr/share/${PROJECT_NAME}/sounds/
               manpage      → /usr/share/man/man1/${PROJECT_NAME}.1
               completions  → /usr/share/bash-completion/completions/${PROJECT_NAME}
                              /usr/share/zsh/site-functions/_${PROJECT_NAME}
                              /usr/share/fish/vendor_completions.d/${PROJECT_NAME}.fish
             (script invokes sudo for the install steps)
  --user     Install to user-local (default, no sudo):
               binary       → ~/.local/bin/${PROJECT_NAME}
               sound assets → ~/.local/share/${PROJECT_NAME}/sounds/
               manpage      → ~/.local/share/man/man1/${PROJECT_NAME}.1
               completions  → ~/.local/share/bash-completion/completions/${PROJECT_NAME}
                              ~/.local/share/zsh/site-functions/_${PROJECT_NAME}
                              ~/.local/share/fish/vendor_completions.d/${PROJECT_NAME}.fish

The build step (cargo build --release --locked) ALWAYS runs as the current user.
EOF
}

MODE="--user"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --system) MODE="--system"; shift ;;
        --user)   MODE="--user";   shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument: $1" >&2; usage; exit 2 ;;
    esac
done

# Refuse to run as root — cargo build must run as the current user.
# If run with sudo, cargo build would create root-owned files in target/,
# breaking future `cargo clean` / `cargo build` for the normal user.
# The script uses sudo internally only for the install step in --system mode.
if [[ $EUID -eq 0 ]]; then
    echo "error: do not run this script with sudo." >&2
    echo "  cargo build would run as root, corrupting target/ ownership." >&2
    echo "  Run: $0 --system" >&2
    echo "  The script will use sudo internally only for the install step." >&2
    exit 1
fi

cd "${REPO_ROOT}"

if [[ ! -f Cargo.toml ]]; then
    echo "error: Cargo.toml not found." >&2
    exit 1
fi

echo ">> [1/5] Building ${PROJECT_NAME} (release, locked)"
cargo build --release --locked

BINARY="target/release/${PROJECT_NAME}"
if [[ ! -f "${BINARY}" ]]; then
    echo "error: build produced no binary at ${BINARY}" >&2
    exit 1
fi

# Set install paths + sudo prefix based on mode
if [[ "${MODE}" == "--system" ]]; then
    PREFIX="/usr"
    SUDO="sudo"
else
    PREFIX="${HOME}/.local"
    SUDO=""
fi

BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share/${PROJECT_NAME}"
MANDIR="${PREFIX}/share/man/man1"
BASHCOMPDIR="${PREFIX}/share/bash-completion/completions"
ZSHCOMPDIR="${PREFIX}/share/zsh/site-functions"
FISHCOMPDIR="${PREFIX}/share/fish/vendor_completions.d"

echo ">> [2/5] Installing binary (${MODE})"
${SUDO} mkdir -p "${BINDIR}"
${SUDO} install -Dm755 "${BINARY}" "${BINDIR}/${PROJECT_NAME}"
echo "   installed: ${BINDIR}/${PROJECT_NAME}"

echo ">> [3/5] Installing sound assets (${MODE})"
SOUND_SRC="${REPO_ROOT}/assets/sounds"
if [[ -d "${SOUND_SRC}" ]]; then
    ${SUDO} mkdir -p "${DATADIR}/sounds"
    for wav in "${SOUND_SRC}"/*.wav; do
        [[ -f "$wav" ]] || continue
        ${SUDO} install -Dm644 "$wav" "${DATADIR}/sounds/$(basename "$wav")"
    done
    echo "   installed: ${DATADIR}/sounds/"
else
    echo "   (no sound assets found, skipped)"
fi

echo ">> [4/5] Installing manpage (${MODE})"
MANPAGE_SRC="${REPO_ROOT}/man/${PROJECT_NAME}.1"
if [[ -f "${MANPAGE_SRC}" ]]; then
    ${SUDO} install -Dm644 "${MANPAGE_SRC}" "${MANDIR}/${PROJECT_NAME}.1"
    echo "   installed: ${MANDIR}/${PROJECT_NAME}.1"
else
    echo "   (no manpage found, skipped)"
fi

echo ">> [5/5] Installing shell completions (${MODE})"
BASH_COMP_SRC="${REPO_ROOT}/completions/${PROJECT_NAME}.bash"
ZSH_COMP_SRC="${REPO_ROOT}/completions/${PROJECT_NAME}.zsh"
FISH_COMP_SRC="${REPO_ROOT}/completions/${PROJECT_NAME}.fish"

if [[ -f "${BASH_COMP_SRC}" ]]; then
    ${SUDO} install -Dm644 "${BASH_COMP_SRC}" "${BASHCOMPDIR}/${PROJECT_NAME}"
    echo "   bash: ${BASHCOMPDIR}/${PROJECT_NAME}"
fi
if [[ -f "${ZSH_COMP_SRC}" ]]; then
    ${SUDO} install -Dm644 "${ZSH_COMP_SRC}" "${ZSHCOMPDIR}/_${PROJECT_NAME}"
    echo "   zsh:  ${ZSHCOMPDIR}/_${PROJECT_NAME}"
fi
if [[ -f "${FISH_COMP_SRC}" ]]; then
    ${SUDO} install -Dm644 "${FISH_COMP_SRC}" "${FISHCOMPDIR}/${PROJECT_NAME}.fish"
    echo "   fish: ${FISHCOMPDIR}/${PROJECT_NAME}.fish"
fi

echo
echo ">> Done."
echo
echo "Next steps:"
case "${MODE}" in
    --system) echo "  - Verify: ${PROJECT_NAME} --version" ;;
    --user)
        echo "  - Ensure ~/.local/bin is on your PATH"
        echo "  - Verify: ${PROJECT_NAME} --version"
        ;;
esac
echo "  - Uninstall: ./scripts/uninstall.sh"
