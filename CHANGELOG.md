# Changelog

All notable changes to zenritme.

## [v11.0.0] — 2026-07-09

### Code Quality

### Fixed — CI
- Fixed clippy `redundant_pattern_matching` in `src/sound/playback.rs` — replaced `match { Ok(_) => true, Err(_) => false }` with `.is_ok()`

### Removed — Test Bloat
- Removed 103 redundant tests (295 → 192) that tested stdlib behavior, trivial struct literals, duplicated coverage, or boolean tautologies
- Kept all tests exercising real project logic: engine state machine, path guard whitelist, CLI parsing branches, sound cooldown, version comparison, JSON extraction
- Reduced compile time and maintenance burden with zero coverage loss

### Removed — Secret Content
- Removed `docs/ROADMAP.md` — future development plans are confidential
- Removed roadmap link from README.md project docs section

### Removed — Dead Code
- Removed unused `ColorFields::label` field (never read anywhere in the codebase)
- Removed unused `RenderState::total` field (computed but never consumed by any view)
- Removed `#[allow(dead_code)]` from `ColorFields` and `RenderState` (all fields now used)
- Removed redundant `c.reset` in `colored_bar` format string (double reset — `colored()` already appends reset)

### Changed — Deduplication
- Extracted `pause_aware_elapsed()` helper in engine.rs, eliminating duplicated pause-now computation between `elapsed()` and `phase_elapsed()`
- Extracted `Mode::phase_duration(PomodoroPhase)` and `Mode::pomodoro_cycles()` methods, eliminating 4 repeated PomodoroPhase-to-duration match blocks across engine.rs, main.rs, and render/progress.rs
- Shared `args()` test helper between `cli/mod.rs` and `cli/pomodoro.rs` (was duplicated identically)
- Replaced local `plain_colors()` test helper in progress.rs with `ColorFields::plain()` from theme.rs
- Made `ColorFields::plain()` `pub(crate)` for test reuse

### Changed — Documentation
- Removed version output section from README.md (no hardcoded versions in docs)
- README now contains no roadmap or version number references

### Added — Signal Handling Hardening
- Added SIGINT handler alongside SIGTERM/SIGHUP for complete signal coverage
- Terminal now restores cleanly on `pkill`, external SIGINT, SIGHUP, and Ctrl+C
- Updated man page EXIT BEHAVIOR section to reflect the new signal coverage

### Added — 5 Tron Legacy Color Variants
- `tron-green`  — green circuit board aesthetic
- `tron-cyan`   — cyan glow
- `tron-orange` — orange flare
- `tron-red`    — red alert
- `tron-yellow` — gold accent
- Full palette implementations, CLI `--theme` matching, shell completions, man page, and tests

### Changed — CLI Polish
- Updated shell completions (bash, zsh, fish) with all 11 themes
- Updated man page `--theme` list with all 11 themes
- Updated README: theme table (11 themes), Options section

### Added — Tron Legacy View
- `--view tron` — perspective grid inspired by Tron Legacy (2010)
  - Clear horizon line separating dark sky from the grid floor
  - Vertical lines converge to center at the horizon
  - Horizontal lines span full width with perspective compression
  - Animated grid scroll toward the viewer
  - Light trail sweep — bright accent-colored band like a light cycle
  - Title, time, and progress float in the dark sky above the horizon
- Updated `--list-views`, shell completions (bash, zsh, fish), and man page

### Verified
- All tests PASS
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
