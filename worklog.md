---
Task ID: 1
Agent: main
Task: Prepare Zenritme v1.2.0 release readiness

Work Log:
- Verified version consistency: Cargo.toml = 1.2.0, Cargo.lock = 1.2.0, cli.rs uses env!("CARGO_PKG_VERSION"), README has no stale version
- Ran cargo fmt --all -- --check: PASS
- Ran cargo test: 58/58 tests pass
- Ran cargo clippy --all-targets --all-features -- -D warnings: PASS (zero warnings)
- Ran bash scripts/check-loc.sh: all 9 files under 1000 LOC (largest: render.rs 560 LOC)
- Ran ./build.sh check-all: ALL quality checks passed
- Verified README examples reference --theme and --view flags
- Verified RULES.md accuracy (LOC guard, main.rs constraint, split domains unchanged)
- Verified docs/SECURITY.md accuracy (zero deps, supply-chain, terminal safety)
- Verified docs/ENDURANCE.md still marked optional/manual

Stage Summary:
- All 5 validation gates pass with zero issues
- Version is consistent across Cargo.toml, Cargo.lock, and --help output
- Documentation is consistent and accurate
- No git tag created (not requested)
- Release is ready

---
Task ID: 2
Agent: main
Task: Prepare release notes for Zenritme v1.2.0

## Release Notes — Zenritme v1.2.0

### Premium Terminal UI

Zenritme v1.2.0 introduces a premium terminal UI with themes, view modes, and
rich animations — all built on zero external dependencies.

### New features

- **5 themes**: void (grayscale default), ember (warm orange), aura (cool blue),
  forest (green), mono (no color / plain)
- **3 view modes**: minimal (clean layout), orbit (spinner + orbit ring, default),
  cinematic (spacious with top and bottom orbit decorations)
- **NO_COLOR support**: setting the `NO_COLOR` environment variable forces plain
  output regardless of the selected theme, following the NO_COLOR community standard
- **Animations**: braille spinner (10-frame cycle), orbit ring pattern (14-dot),
  progress bar, and completion burst effect

### New CLI options

- `--theme <void|ember|aura|forest|mono>` (default: void)
- `--view <minimal|orbit|cinematic>` (default: orbit)
- Flags may appear before or after the mode flag

### Architecture changes

- New module: `src/animation.rs` — pure, deterministic animation functions with tests
- New module: `src/theme.rs` — theme definitions, ANSI constants, NO_COLOR support
- `src/render.rs` refactored around `RenderState` struct with three view dispatch paths
- `src/cli.rs` extended with theme/view parsing (pre-pass extraction)

### Preserved invariants

- Zero external dependencies (Cargo.toml [dependencies] remains empty)
- LOC guard: all 9 source files under 1000 LOC (largest: render.rs at 560 LOC)
- main.rs stays as bootstrap/wiring only (120 LOC)
- Timer accuracy via wall-clock `Instant`, not frame counting
- Calm frame rate (~12.5 FPS / 80ms loop interval)

### Tests

- Test count increased from 29 (v1.1.0) to 58
- New tests cover: animation cycles, theme parsing, NO_COLOR behavior,
  theme/view CLI flag combinations, unknown theme/view rejection

### No known runtime blockers

Stage Summary:
- Release notes document covers all v1.2.0 changes
- All invariants verified and preserved

---
Task ID: 3
Agent: main
Task: Implement Zenritme v1.3.0 — Pomodoro Masterclass

## Release Notes — Zenritme v1.3.0

### Pomodoro Masterclass

Zenritme v1.3.0 upgrades Pomodoro mode into a full focus ritual with
configurable sessions, long breaks, cycle tracking, and cycle-aware UI labels.

### New features

- **Long break phase**: after the final focus cycle, a dedicated long break
  (default 15m) replaces the short break, providing a proper rest interval
- **Cycle tracking**: the UI displays FOCUS 1/4, SHORT BREAK 1/4, LONG BREAK,
  and COMPLETE labels throughout the session
- **Configurable cycles**: `--cycles <N>` sets the number of focus sessions
  per round (default: 4)
- **Full CLI control**: `--focus`, `--break`, `--long-break`, `--cycles`
  flags may appear before or after `--pomodoro`
- **Backward compatible**: legacy `--pomodoro <FOCUS> <BREAK>` syntax still works

### Session flow

FOCUS 1/N → SHORT BREAK 1/N → FOCUS 2/N → SHORT BREAK 2/N → … →
FOCUS N/N → LONG BREAK → COMPLETE

### New CLI options

- `--focus <DURATION>` — focus session length (default: 25m)
- `--break <DURATION>` — short break length (default: 5m)
- `--long-break <DURATION>` — long break length (default: 15m)
- `--cycles <N>` — focus sessions per round (default: 4)

### Architecture changes

- `PomodoroPhase` enum: renamed `Break` to `ShortBreak`, added `LongBreak`
- `Mode::Pomodoro` variant: added `long_break`, `cycles`, `current_cycle` fields
- `engine.rs`: full Pomodoro state machine with cycle-aware transitions
- `cli.rs`: `PomodoroOpts` struct for pre-pass extraction of new flags
- `render.rs`: cycle-aware labels in `build_title` with COMPLETE state handling

### Preserved invariants

- Zero external dependencies
- LOC guard: all 9 source files under 1000 LOC
- main.rs stays as bootstrap/wiring only
- All v1.2.0 themes, views, and controls preserved
- Old `--pomodoro [FOCUS BREAK]` behavior preserved

### Tests

- Test count increased from 58 (v1.2.0) to 85
- New CLI tests: default config, all 4 flags, combined flags, flags with theme/view,
  old syntax compatibility, missing values, zero values, invalid values
- New engine tests: focus→short break, short break→next focus, final focus→long break,
  long break→completed, single cycle, pause during pomodoro, reset to focus 1/N,
  completed no double event

Stage Summary:
- 85 tests all pass
- All v1.2.0 invariants preserved
- Zero dependencies maintained
- No git tag created
