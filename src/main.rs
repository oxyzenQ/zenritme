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
            run(mode, theme, view, mute, profile);
        }
    }
}

fn run(
    mode: mode::Mode,
    theme: theme::Theme,
    view: render::ViewMode,
    mute: bool,
    profile: sound::SoundProfile,
) {
    let (_term, rx) = terminal::spawn_input();
    let mut engine = engine::Engine::new(mode);
    let colors = theme.colors();
    let mut frame: u64 = 0;
    let mut session_started = false;
    let muted = mute || profile.is_silent();

    // v10: Dirty tracking — only redraw when displayed content changes.
    let mut last_displayed_second: Option<u64> = None;
    let mut last_state = engine.state();
    let mut last_phase = engine.phase_marker();

    loop {
        // ── Process keypresses ────────────────────────────────────────────────
        let mut input_changed = false;
        if let Some(rx) = rx.as_ref() {
            while let Ok(b) = rx.try_recv() {
                match b {
                    // Quit
                    b'q' | b'Q' | 3 => return,

                    // Pause / resume (no-op when Completed)
                    b'p' | b'P' => {
                        engine.toggle_pause();
                        input_changed = true;
                        if !muted {
                            sound::play_event(sound::SoundEvent::Pause);
                        }
                    }

                    // Reset current session
                    b'r' | b'R' => {
                        engine.reset();
                        input_changed = true;
                    }

                    // ESC or ESC-sequence
                    27 => match rx.try_recv() {
                        // Arrow keys / function keys — consume the rest of the sequence
                        Ok(next) if next == b'[' || next == b'O' => while rx.try_recv().is_ok() {},
                        // Bare ESC or broken sequence → quit
                        Ok(_) | Err(mpsc::TryRecvError::Empty) => return,
                        Err(mpsc::TryRecvError::Disconnected) => return,
                    },

                    _ => {}
                }
            }
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

        // v10: Dirty tracking — check if we need to redraw.
        let current_second = engine
            .remaining()
            .or_else(|| Some(engine.elapsed()))
            .map(|d| d.as_secs());
        let state_changed = engine.state() != last_state;
        let phase_changed = engine.phase_marker() != last_phase;
        let second_changed = current_second != last_displayed_second;
        let need_redraw =
            input_changed || state_changed || phase_changed || second_changed || frame == 0;

        if need_redraw {
            let state = render::RenderState {
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
                progress: render::compute_progress(&engine),
                state: engine.state(),
                frame,
                colors: &colors,
                view,
                engine_phase: engine.pomo_phase(),
                engine_cycle: engine.pomo_cycle(),
            };

            render::draw(&state);
            last_displayed_second = current_second;
            last_state = engine.state();
            last_phase = engine.phase_marker();
        }

        // ── Handle events ─────────────────────────────────────────────────────
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

        frame += 1;

        // v10: Adaptive tick rate — reduce CPU based on context.
        // Paused/Completed: 1000ms (near-zero CPU)
        // Remaining > 60s: 500ms (no need for sub-second precision)
        // Remaining 10-60s: 200ms (slightly smoother)
        // Remaining < 10s: 80ms (smooth final countdown)
        // Stopwatch: 80ms (always smooth)
        let sleep_ms = match engine.state() {
            engine::EngineState::Paused | engine::EngineState::Completed => 1000,
            engine::EngineState::Running => match engine.remaining() {
                Some(rem) if rem.as_secs() > 60 => 500,
                Some(rem) if rem.as_secs() >= 10 => 200,
                _ => 80,
            },
        };
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    }
}
