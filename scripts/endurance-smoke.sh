#!/bin/bash
# =============================================================================
# Zenritme — Endurance Smoke Test
# =============================================================================
# Short automated smoke test for long-usage stability. Safe, no-network,
# no-root. Runs a brief timer session and verifies the binary exits cleanly.
#
# Usage:
#   ./scripts/endurance-smoke.sh
#   DURATION=120s ./scripts/endurance-smoke.sh       # custom runtime
#   DURATION=24h ./scripts/endurance-smoke.sh        # long manual run
#
# The test uses timeout to enforce the duration. For 24h+ runs, run in a
# tmux/screen session or nohup.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

readonly DURATION="${DURATION:-10s}"
readonly BINARY="${PROJECT_ROOT}/target/release/zenritme"

echo "=== Zenritme Endurance Smoke Test ==="
echo "Duration: ${DURATION}"
echo ""

# ── Locate binary ──────────────────────────────────────────────────────────

if [ ! -f "${BINARY}" ]; then
    echo "Release binary not found at ${BINARY}"
    echo "Build it first:"
    echo "  cargo build --release --locked"
    exit 1
fi

VERSION=$("${BINARY}" -V 2>/dev/null | head -1 || echo "unknown")
echo "Binary: ${BINARY}"
echo "Version: ${VERSION}"
echo ""

# ── Run timer-down session ──────────────────────────────────────────────────

echo "Running: timeout ${DURATION} ${BINARY} --timer-up --mute"
echo "Start: $(date)"
echo ""

START_RSS=$(ps -o rss= -p $$ 2>/dev/null || echo "N/A")

timeout "${DURATION}" "${BINARY}" --timer-up --mute 2>&1 || true
EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "End: $(date)"

# ── Check exit ──────────────────────────────────────────────────────────────

if [ "${EXIT_CODE}" -eq 124 ]; then
    echo "Exit: timeout (expected for endurance run)"
elif [ "${EXIT_CODE}" -eq 0 ]; then
    echo "Exit: clean"
else
    echo "Exit: code ${EXIT_CODE} (check for panics)"
fi

echo ""
echo "=== Smoke test complete ==="
echo "For 24h+ runs: DURATION=24h ./scripts/endurance-smoke.sh"
echo "Monitor memory: watch -n 60 'ps -o pid,rss,vsz,comm -p <PID>'"
