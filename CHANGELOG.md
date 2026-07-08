# Changelog

All notable changes to zenritme.

## [v11.0.0] — 2026-07-09

### Code Quality

### Removed — Dead Code
- Removed unused `animation::progress_bar()` (replaced by `render/progress::colored_bar`)
- Removed unused `sound::beep()` legacy multi-beep function
- Removed unused `SoundProfile::as_str()` method
- Removed always-true `input_changed` variable from main render loop
- Removed 6 dead tests (progress_bar tests, as_str roundtrip)

### Changed — Deduplication
- Extracted `Mode::phase_duration(PomodoroPhase)` and `Mode::pomodoro_cycles()` methods, eliminating 4 repeated PomodoroPhase-to-duration match blocks across engine.rs, main.rs, and render/progress.rs
- Shared `args()` test helper between `cli/mod.rs` and `cli/pomodoro.rs` (was duplicated identically)
- Replaced local `plain_colors()` test helper in progress.rs with `ColorFields::plain()` from theme.rs
- Made `ColorFields::plain()` `pub(crate)` for test reuse

### Added — Signal Handling Hardening
- Added SIGINT handler alongside SIGTERM/SIGHUP for complete signal coverage
- Terminal now restores cleanly on `pkill`, external SIGINT, SIGHUP, and Ctrl+C
- Updated man page EXIT BEHAVIOR section to reflect the new signal coverage

### Changed — CLI Polish
- Updated shell completions (bash, zsh, fish) with all 11 themes (was missing 6 Tron variants)
- Updated man page `--theme` list with all 11 themes
- Updated README: theme table (11 themes), version examples (v11.0.0), Options section

### Verified
- 230 tests PASS
- clippy: 0 warnings
- Zero build warnings

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
