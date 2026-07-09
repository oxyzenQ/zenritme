// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

mod animation;
mod cli;
mod engine;
mod mode;
mod path_guard;
mod render;
mod sound;
mod terminal;
mod theme;
mod update;
mod version;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ── Signal handling (SIGTERM / SIGHUP / SIGINT) ────────────────────────
// `pkill zenritme` sends SIGTERM.  External `kill -INT <pid>` sends SIGINT.
// Without handlers the OS terminates the process immediately — no
// destructors run, terminal is left dirty.

#[cfg(unix)]
static TERMINATE_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
const SIGTERM: i32 = 15;

#[cfg(unix)]
const SIGHUP: i32 = 1;

#[cfg(unix)]
const SIGINT: i32 = 2;

#[cfg(unix)]
extern "C" {
    /// POSIX signal() — installs a simple signal handler.
    fn signal(signum: i32, handler: extern "C" fn(i32)) -> *const ();
}

#[cfg(unix)]
extern "C" fn handle_terminate_signal(_: i32) {
    TERMINATE_REQUESTED.store(true, Ordering::Relaxed);
}

/// Install signal handlers for graceful termination on SIGTERM, SIGHUP,
/// and SIGINT (external kill, hangup, or interrupt).  No-op on non-Unix.
///
/// SIGKILL (`kill -9`) cannot be caught — this is an OS limitation.
///
/// The handlers only set an AtomicBool (async-signal-safe).  The main loop
/// polls it, so there is a worst-case latency of one sleep cycle (max 1 s).
#[cfg(unix)]
fn install_terminate_handler() {
    unsafe {
        signal(SIGTERM, handle_terminate_signal);
        signal(SIGHUP, handle_terminate_signal);
        signal(SIGINT, handle_terminate_signal);
    }
}

#[cfg(not(unix))]
fn install_terminate_handler() {}

fn main() {
    // Register temp-file cleanup so embedded sounds are removed on exit.
    // TempCleanupGuard covers normal return and panic unwind.
    // SIGKILL may still bypass Drop — see docs/ENDURANCE.md for details.
    let _cleanup_guard = sound::TempCleanupGuard::install();

    let cmd = match cli::parse_args(std::env::args().skip(1)) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", err);
            eprintln!("\n{}", cli::usage());
            std::process::exit(2);
        }
    };

    match cmd {
        cli::Command::Help => {
            println!("{}", cli::usage());
        }
        cli::Command::Version => {
            println!("{}", version::version_report());
        }
        cli::Command::ListThemes => {
            println!("{}", cli::list_themes());
        }
        cli::Command::ListViews => {
            println!("{}", cli::list_views());
        }
        cli::Command::CheckUpdate => match update::check_update(env!("CARGO_PKG_VERSION")) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("update check failed: {e}");
                std::process::exit(1);
            }
        },
        cli::Command::SoundTest => {
            sound::sound_test();
        }
        cli::Command::Run {
            mode,
            theme,
            view,
            mute,
            profile,
        } => {
            let code = run(mode, theme, view, mute, profile);
            std::process::exit(code);
        }
    }
}

/// Exit control for the main loop.
/// `Break` exits the loop cleanly (allowing `TerminalGuard` to drop).
enum LoopAction {
    Continue,
    Break,
}

fn run(
    mode: mode::Mode,
    theme: theme::Theme,
    view: render::ViewMode,
    mute: bool,
    profile: sound::SoundProfile,
) -> i32 {
    // _term MUST stay alive for the entire duration of this function.
    // When `run` returns, _term is dropped → TerminalGuard::drop() restores
    // the terminal (leaves alt screen, shows cursor, restores stty).
    // This is why we NEVER call `std::process::exit` inside the loop.
    let (_term, rx) = terminal::spawn_input();
    install_terminate_handler();
    let mut engine = engine::Engine::new(mode);
    let colors = theme.colors();
    let muted = mute || profile.is_silent();

    // v10: Dirty tracking — only redraw when displayed content changes.
    let mut last_displayed_second: Option<u64> = None;
    let mut last_state = engine.state();
    let mut last_phase = engine.phase_marker();
    let mut frame: u64 = 0;
    let mut session_started = false;
    let mut completed_at: Option<Instant> = None;

    loop {
        let input_action = process_keypresses(rx.as_ref(), &mut engine, muted);
        if matches!(input_action, LoopAction::Break) {
            break;
        }

        // Check for external termination (SIGTERM / SIGHUP / SIGINT from
        // pkill, hangup, or external kill).
        #[cfg(unix)]
        if TERMINATE_REQUESTED.load(Ordering::Relaxed) {
            break;
        }

        // ── Advance engine ────────────────────────────────────────────────────
        engine.tick();

        // ── Start sound (once per session) ───────────────────────────────────
        if !session_started {
            session_started = true;
            if !muted {
                sound::play_event(sound::SoundEvent::Start);
            }
        }

        // ── Render (dirty-tracked) ────────────────────────────────────────────
        let current_second = engine
            .remaining()
            .or_else(|| Some(engine.elapsed()))
            .map(|d| d.as_secs());
        let state_changed = engine.state() != last_state;
        let phase_changed = engine.phase_marker() != last_phase;
        let second_changed = current_second != last_displayed_second;
        // Force redraw every tick when < 10 s so sub-second tenths animate.
        let near_end = engine.remaining().is_some_and(|r| r.as_secs() < 10)
            && engine.state() == engine::EngineState::Running;
        let need_redraw =
            state_changed || phase_changed || second_changed || frame == 0 || near_end;

        if need_redraw {
            let state = build_render_state(&engine, &colors, view, frame);
            render::draw(&state);
            last_displayed_second = current_second;
            last_state = engine.state();
            last_phase = engine.phase_marker();
        }

        // ── Handle events ─────────────────────────────────────────────────────
        handle_events(&mut engine, muted);

        frame += 1;

        // Track completion time for auto-exit and burst animation.
        if engine.state() == engine::EngineState::Completed {
            match completed_at {
                None => completed_at = Some(Instant::now()),
                Some(t) if t.elapsed() >= Duration::from_secs(5) => {
                    break;
                }
                Some(_) => {}
            }
        } else {
            completed_at = None;
        }

        // Completion burst: fast tick for 2 s so the animation is visible.
        let sleep = match engine.state() {
            engine::EngineState::Completed => {
                let since = completed_at.map(|t| t.elapsed()).unwrap_or(Duration::ZERO);
                if since < Duration::from_secs(2) {
                    Duration::from_millis(80)
                } else {
                    Duration::from_millis(1000)
                }
            }
            _ => sleep_duration(&engine),
        };
        std::thread::sleep(sleep);
    }
    0
}

/// Process pending keypresses. Returns the loop action (continue or break).
fn process_keypresses(
    rx: Option<&mpsc::Receiver<u8>>,
    engine: &mut engine::Engine,
    muted: bool,
) -> LoopAction {
    if let Some(rx) = rx {
        while let Ok(b) = rx.try_recv() {
            match b {
                b'q' | b'Q' | 3 => return LoopAction::Break,

                b'p' | b'P' => {
                    engine.toggle_pause();
                    if !muted {
                        sound::play_event(sound::SoundEvent::Pause);
                    }
                }

                b'r' | b'R' => {
                    engine.reset();
                }

                // ESC — debounce: wait briefly for ESC [ / ESC O sequences
                // to avoid mistaking the start of an arrow-key press for bare ESC.
                27 => {
                    match rx.recv_timeout(Duration::from_millis(30)) {
                        Ok(b'[') | Ok(b'O') => {
                            // Recognized escape sequence — drain remaining bytes.
                            while rx.try_recv().is_ok() {}
                        }
                        Ok(_) | Err(_) => {
                            // Bare ESC (timeout) or unknown sequence — quit.
                            return LoopAction::Break;
                        }
                    }
                }

                _ => {}
            }
        }
    }
    LoopAction::Continue
}

/// Build a `RenderState` snapshot from the current engine state.
fn build_render_state<'a>(
    engine: &engine::Engine,
    colors: &'a theme::ColorFields,
    view: render::ViewMode,
    frame: u64,
) -> render::RenderState<'a> {
    render::RenderState {
        mode: engine.mode(),
        elapsed: engine.elapsed(),
        remaining: engine.remaining(),
        progress: render::compute_progress(engine),
        state: engine.state(),
        frame,
        colors,
        view,
        engine_phase: engine.pomo_phase(),
        engine_cycle: engine.pomo_cycle(),
    }
}

/// Consume and play any pending engine events.
fn handle_events(engine: &mut engine::Engine, muted: bool) {
    if !muted {
        if let Some(ev) = engine.take_event() {
            match ev {
                engine::EngineEvent::Completed => {
                    sound::play_event(sound::SoundEvent::Complete);
                }
                engine::EngineEvent::PhaseSwitched => {
                    sound::play_event(sound::SoundEvent::Phase);
                }
            }
        }
    } else {
        // Still consume the event to prevent stale events
        let _ = engine.take_event();
    }
}

/// Compute the adaptive sleep duration based on engine state.
fn sleep_duration(engine: &engine::Engine) -> std::time::Duration {
    let ms = match engine.state() {
        engine::EngineState::Paused | engine::EngineState::Completed => 1000,
        engine::EngineState::Running => match engine.remaining() {
            Some(rem) if rem.as_secs() > 60 => 500,
            Some(rem) if rem.as_secs() >= 10 => 200,
            _ => 80,
        },
    };
    std::time::Duration::from_millis(ms)
}
