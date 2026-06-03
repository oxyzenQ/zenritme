// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

mod cli;
mod engine;
mod mode;
mod render;
mod sound;
mod terminal;

use mode::{Mode, PomodoroPhase};
use std::sync::mpsc;

fn main() {
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
        cli::Command::SoundTest => {
            sound::sound_test();
        }
        cli::Command::Run(mode) => {
            run(mode);
        }
    }
}

fn run(mode: Mode) {
    let (_term, rx) = terminal::spawn_input();
    let mut engine = engine::Engine::new(mode);

    loop {
        // ── Process keypresses ────────────────────────────────────────────────
        if let Some(rx) = rx.as_ref() {
            while let Ok(b) = rx.try_recv() {
                match b {
                    // Quit
                    b'q' | b'Q' | 3 => return,

                    // Pause / resume (no-op when Completed)
                    b'p' | b'P' => engine.toggle_pause(),

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

        let elapsed = engine.elapsed();
        let remaining = engine.remaining();
        let state = engine.state();
        let progress = compute_progress(&engine);

        render::draw(engine.mode(), elapsed, remaining, state, progress);

        // ── Handle events ─────────────────────────────────────────────────────
        if let Some(ev) = engine.take_event() {
            match ev {
                engine::EngineEvent::Completed => {
                    sound::beep(3);
                    // Stay in the loop — display [ DONE ] until user quits.
                }
                engine::EngineEvent::PhaseSwitched => {
                    sound::beep(1);
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Returns `Some(0.0..=1.0)` for bounded modes, `None` for unbounded.
fn compute_progress(engine: &engine::Engine) -> Option<f32> {
    match engine.mode() {
        Mode::TimerDown { total } => {
            if total.is_zero() {
                None
            } else {
                Some((engine.elapsed().as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0))
            }
        }
        Mode::Pomodoro {
            phase,
            focus,
            short_break,
            ..
        } => {
            let phase_total = match phase {
                PomodoroPhase::Focus => focus,
                PomodoroPhase::Break => short_break,
            };
            let remaining = engine.remaining()?;
            if phase_total.is_zero() {
                None
            } else {
                let used = phase_total.saturating_sub(remaining);
                Some((used.as_secs_f32() / phase_total.as_secs_f32()).clamp(0.0, 1.0))
            }
        }
        _ => None,
    }
}
