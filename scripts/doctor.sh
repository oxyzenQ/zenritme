#!/bin/bash
# =============================================================================
# Zenritme — Local Health Check
# =============================================================================
# Runs a lightweight set of checks to verify the binary is healthy.
# No network, no root, no long-running tests.
#
# Usage:
#   ./scripts/doctor.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

readonly BINARY="${PROJECT_ROOT}/target/release/zenritme"
FAILED=0

echo "=== Zenritme Doctor ==="
echo ""

# ── Binary exists ──────────────────────────────────────────────────────────

if [ -f "${BINARY}" ]; then
    echo "[OK] Release binary: ${BINARY}"
else
    echo "[MISS] Release binary not found. Run: cargo build --release --locked"
    FAILED=$((FAILED + 1))
fi

# ── Version output ───────────────────────────────────────────────────────────

if [ -f "${BINARY}" ]; then
    VERSION_OUT=$("${BINARY}" -V 2>&1) || {
        echo "[FAIL] -V exited non-zero"
        FAILED=$((FAILED + 1))
    }
    if echo "${VERSION_OUT}" | grep -q "Version: v"; then
        echo "[OK] -V: $(echo "${VERSION_OUT}" | head -1)"
    else
        echo "[FAIL] -V output missing version line"
        FAILED=$((FAILED + 1))
    fi
fi

# ── Help output ─────────────────────────────────────────────────────────────

if [ -f "${BINARY}" ]; then
    HELP_OUT=$("${BINARY}" --help 2>&1) || {
        echo "[FAIL] --help exited non-zero"
        FAILED=$((FAILED + 1))
    }
    if echo "${HELP_OUT}" | head -1 | grep -q "zenritme v"; then
        echo "[OK] --help: version in header"
    else
        echo "[FAIL] --help missing version header"
        FAILED=$((FAILED + 1))
    fi
fi

# ── Sound test (no panic check) ───────────────────────────────────────────────

if [ -f "${BINARY}" ]; then
    if "${BINARY}" --sound-test --mute 2>&1 | grep -q "Temp sound files cleaned up"; then
        echo "[OK] --sound-test: completed without panic"
    else
        echo "[FAIL] --sound-test did not complete cleanly"
        FAILED=$((FAILED + 1))
    fi
fi

# ── LOC guard ──────────────────────────────────────────────────────────────

if [ -f "${PROJECT_ROOT}/scripts/check-loc.sh" ]; then
    if bash "${PROJECT_ROOT}/scripts/check-loc.sh" >/dev/null 2>&1; then
        echo "[OK] LOC guard: all files under limit"
    else
        echo "[FAIL] LOC guard: one or more files exceed limit"
        FAILED=$((FAILED + 1))
    fi
fi

# ── Temp cleanup ─────────────────────────────────────────────────────────────

TEMP_DIR="/tmp/zenritme-sounds-$$(pgrep -f "${BINARY}" 2>/dev/null || echo "none")"
if [ -d "${TEMP_DIR}" ]; then
    echo "[WARN] Temp sound dir still exists: ${TEMP_DIR}"
else
    echo "[OK] Temp sound cleanup: no active temp dir"
fi

echo ""

if [ "${FAILED}" -eq 0 ]; then
    echo "All checks passed."
    exit 0
else
    echo "${FAILED} check(s) failed."
    exit 1
fi
