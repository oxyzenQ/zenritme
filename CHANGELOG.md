# Changelog

All notable changes to zenritme.

## [v10.0.0] — 2026-07-01

### Peak Performance

### Added — Adaptive Tick Rate
- Main loop sleep duration now adapts based on context:
  - Paused/Completed: 1000ms (near-zero CPU)
  - Remaining > 60s: 500ms (no sub-second precision needed)
  - Remaining 10-60s: 200ms (slightly smoother)
  - Remaining < 10s: 80ms (smooth final countdown)
- 1-hour timer: 45,000 wake-ups → 7,200 wake-ups (6.25x reduction)

### Added — Dirty Render Tracking
- Only redraws screen when displayed content actually changes:
  - Elapsed seconds changed
  - Engine state changed (pause/resume/complete)
  - Pomodoro phase switched
  - User input detected
- 1-hour timer: 45,000 redraws → 3,600 redraws (12.5x reduction)

### Added — Static musl Binary
- release.yml now builds both gnu + musl binaries
- musl binary: zero dynamic dependencies (Alpine, embedded, any Linux)
- Both archives include full extras (scripts, docs, sounds, completions, man page)

### Verified
- 236 tests PASS
- clippy: 0 warnings
- Binary size: 619 KB (zero external dependencies)
- Sound playback: non-blocking (spawn-based)

## [v5.0.2] — Previous release

- Premium terminal UI with themed rendering
- 5 themes, 3 view modes
- Ritual sound architecture
- Zero external Rust dependencies
