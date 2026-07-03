#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 rezky_nightky (oxyzenQ)
#
# Uninstall zenritme: binary + sound assets + manpage + shell completions.
# Auto-detects and removes from all known locations:
#   binary:       /usr/local/bin/, ~/.local/bin/
#   sound assets: /usr/local/share/zenritme/, ~/.local/share/zenritme/
#   manpage:      /usr/local/share/man/man1/zenritme.1, ~/.local/share/man/man1/zenritme.1
#   completions:  /usr/local/share/bash-completion/completions/zenritme
#                 /usr/local/share/zsh/site-functions/_zenritme
#                 /usr/local/share/fish/vendor_completions.d/zenritme.fish
#                 (and ~/.local/share/... equivalents)
# Sudo is used ONLY for system paths. Run WITHOUT sudo.

set -uo pipefail

PROJECT_NAME="zenritme"

usage() {
    cat <<EOF
Usage: $0 [--system|--user|--all]

  (default)  Auto-detect: scan all known locations and remove every
             ${PROJECT_NAME} artifact found. Sudo only for system paths.
  --system   Remove only system paths under /usr/local/.
  --user     Remove only user paths under ~/.local/. No sudo.
  --all      Same as default.

EOF
}

MODE="--all"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --system) MODE="--system"; shift ;;
        --user)   MODE="--user";   shift ;;
        --all)    MODE="--all";    shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument: $1" >&2; usage; exit 2 ;;
    esac
done

removed=0

remove_at() {
    local target="$1"
    local need_sudo="$2"
    if [[ -e "${target}" ]]; then
        if [[ "${need_sudo}" == "yes" ]]; then
            sudo rm -rf "${target}"
        else
            rm -rf "${target}"
        fi
        echo "   removed: ${target}"
        removed=$((removed+1))
    fi
}

echo ">> Uninstalling ${PROJECT_NAME}"

SYSTEM_PREFIX="/usr/local"
USER_PREFIX="${HOME}/.local"

case "${MODE}" in
    --system)
        remove_at "${SYSTEM_PREFIX}/bin/${PROJECT_NAME}" yes
        remove_at "${SYSTEM_PREFIX}/share/${PROJECT_NAME}" yes
        remove_at "${SYSTEM_PREFIX}/share/man/man1/${PROJECT_NAME}.1" yes
        remove_at "${SYSTEM_PREFIX}/share/bash-completion/completions/${PROJECT_NAME}" yes
        remove_at "${SYSTEM_PREFIX}/share/zsh/site-functions/_${PROJECT_NAME}" yes
        remove_at "${SYSTEM_PREFIX}/share/fish/vendor_completions.d/${PROJECT_NAME}.fish" yes
        ;;
    --user)
        remove_at "${USER_PREFIX}/bin/${PROJECT_NAME}" no
        remove_at "${USER_PREFIX}/share/${PROJECT_NAME}" no
        remove_at "${USER_PREFIX}/share/man/man1/${PROJECT_NAME}.1" no
        remove_at "${USER_PREFIX}/share/bash-completion/completions/${PROJECT_NAME}" no
        remove_at "${USER_PREFIX}/share/zsh/site-functions/_${PROJECT_NAME}" no
        remove_at "${USER_PREFIX}/share/fish/vendor_completions.d/${PROJECT_NAME}.fish" no
        ;;
    --all)
        # Binary
        remove_at "${SYSTEM_PREFIX}/bin/${PROJECT_NAME}" yes
        remove_at "${USER_PREFIX}/bin/${PROJECT_NAME}" no
        # Sound assets
        remove_at "${SYSTEM_PREFIX}/share/${PROJECT_NAME}" yes
        remove_at "${USER_PREFIX}/share/${PROJECT_NAME}" no
        # Manpage
        remove_at "${SYSTEM_PREFIX}/share/man/man1/${PROJECT_NAME}.1" yes
        remove_at "${USER_PREFIX}/share/man/man1/${PROJECT_NAME}.1" no
        # Completions
        remove_at "${SYSTEM_PREFIX}/share/bash-completion/completions/${PROJECT_NAME}" yes
        remove_at "${USER_PREFIX}/share/bash-completion/completions/${PROJECT_NAME}" no
        remove_at "${SYSTEM_PREFIX}/share/zsh/site-functions/_${PROJECT_NAME}" yes
        remove_at "${USER_PREFIX}/share/zsh/site-functions/_${PROJECT_NAME}" no
        remove_at "${SYSTEM_PREFIX}/share/fish/vendor_completions.d/${PROJECT_NAME}.fish" yes
        remove_at "${USER_PREFIX}/share/fish/vendor_completions.d/${PROJECT_NAME}.fish" no
        ;;
esac

if [[ ${removed} -eq 0 ]]; then
    echo "   (nothing found to remove)"
    exit 0
fi

echo ">> Done. Removed ${removed} artifact(s)."
