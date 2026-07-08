// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

mod animation;
mod cli;
mod engine;
mod mode;
mod render;
mod sound;
mod terminal;
mod theme;
mod update;
mod version;

use mode::PomodoroPhase;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn main() {
    // Register temp-file cleanup so embedded sounds are removed on exit.
    // TempCleanupGuard covers normal return and panic unwind.
    // Signal termination (SIGINT, SIGKILL) may bypass Drop — see
    // docs/ENDURANCE.md "Signal termination caveat" for details.
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
        let input_changed = true;

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
        let need_redraw = input_changed
            || state_changed
            || phase_changed
            || second_changed
            || frame == 0
            || near_end;

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
        total: match engine.mode() {
            mode::Mode::TimerDown { total } => Some(total),
            mode::Mode::Pomodoro {
                focus,
                short_break,
                long_break,
                ..
            } => Some(match engine.pomo_phase() {
                PomodoroPhase::Focus => focus,
                PomodoroPhase::ShortBreak => short_break,
                PomodoroPhase::LongBreak => long_break,
            }),
            _ => None,
        },
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
