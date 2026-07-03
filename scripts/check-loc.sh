#!/bin/bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
# =============================================================================
# Zenritme — LOC (Lines-of-Code) Guard
# =============================================================================
# Scans all core code files and fails if any file has >= 1000 lines.
# Core code: *.rs *.c *.h *.cpp *.hpp *.css *.js *.ts *.tsx
# Excludes: target/ directory and any generated artifacts.
#
# Usage:
#   ./scripts/check-loc.sh          # exit 0 if all pass, 1 on failure
# =============================================================================

set -euo pipefail

readonly LIMIT=1000
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly NC='\033[0m'

# Project root: parent directory of the scripts/ folder.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

failed=0

echo "=== Zenritme LOC Guard (limit: ${LIMIT} lines) ==="
echo ""

# Find all core code files, excluding target/
# Using null-delimited find + mapfile for safe filenames.
while IFS= read -r -d '' file; do
    lines=$(wc -l < "$file")
    if [ "$lines" -ge "$LIMIT" ]; then
        echo -e "${RED}FAIL${NC}: ${file}  ${lines} lines (>= ${LIMIT})"
        failed=1
    fi
done < <( \
    find "${PROJECT_ROOT}" \
        \( -name 'target' -prune \) -o \
        \( \
            -name '*.rs' -o \
            -name '*.c'  -o \
            -name '*.h'  -o \
            -name '*.cpp' -o \
            -name '*.hpp' -o \
            -name '*.css' -o \
            -name '*.js'  -o \
            -name '*.ts'  -o \
            -name '*.tsx' \
        \) -print0 \
)

# Always print a sorted report (by line count descending)
echo ""
echo "All core code files (sorted by largest):"
echo ""

while IFS= read -r -d '' file; do
    lines=$(wc -l < "$file")
    rel="${file#"${PROJECT_ROOT}"/}"
    printf "  %5d  %s\n" "$lines" "$rel"
done < <( \
    find "${PROJECT_ROOT}" \
        \( -name 'target' -prune \) -o \
        \( \
            -name '*.rs' -o \
            -name '*.c'  -o \
            -name '*.h'  -o \
            -name '*.cpp' -o \
            -name '*.hpp' -o \
            -name '*.css' -o \
            -name '*.js'  -o \
            -name '*.ts'  -o \
            -name '*.tsx' \
        \) -print0 \
) | sort -rn -k1

echo ""

if [ "$failed" -eq 0 ]; then
    echo -e "${GREEN}All files pass the ${LIMIT}-line LOC guard.${NC}"
    exit 0
else
    echo -e "${RED}One or more files exceed the ${LIMIT}-line limit. See docs/RULES.md.${NC}"
    exit 1
fi
