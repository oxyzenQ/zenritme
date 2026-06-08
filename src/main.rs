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

    loop {
        // ── Process keypresses ────────────────────────────────────────────────
        if let Some(rx) = rx.as_ref() {
            while let Ok(b) = rx.try_recv() {
                match b {
                    // Quit
                    b'q' | b'Q' | 3 => return,

                    // Pause / resume (no-op when Completed)
                    b'p' | b'P' => {
                        engine.toggle_pause();
                        if !muted {
                            sound::play_event(sound::SoundEvent::Pause);
                        }
                    }

                    // Reset current session
                    b'r' | b'R' => engine.reset(),

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

        let state = render::RenderState {
            mode: engine.mode(),
            elapsed: engine.elapsed(),
            remaining: engine.remaining(),
            total: match engine.mode() {
                mode::Mode::TimerDown { total } => Some(total),
                mode::Mode::Pomodoro {
                    phase,
                    focus,
                    short_break,
                    long_break,
                    ..
                } => Some(match phase {
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
        };

        render::draw(&state);

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
        std::thread::sleep(std::time::Duration::from_millis(80));
    }
}
