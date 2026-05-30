// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::mode::{Mode, PomodoroPhase};
use std::time::{Duration, Instant};

/// One-shot events emitted by the engine after each `tick()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineEvent {
    Completed,
    PhaseSwitched,
}

/// Lifecycle state of the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineState {
    Running,
    Paused,
    Completed,
}

pub struct Engine {
    mode: Mode,
    /// Wall-clock instant when the current session started.
    session_start: Instant,
    /// Wall-clock instant when the current phase started (Pomodoro only; mirrors
    /// `session_start` for other modes).
    phase_start: Instant,
    /// Total pause time accumulated across all pauses this session.
    session_paused: Duration,
    /// Total pause time accumulated within the current phase.
    phase_paused: Duration,
    /// Start of the currently-active pause; `None` when running.
    pause_start: Option<Instant>,
    state: EngineState,
    event: Option<EngineEvent>,
}

impl Engine {
    pub fn new(mode: Mode) -> Self {
        let now = Instant::now();
        Self {
            mode,
            session_start: now,
            phase_start: now,
            session_paused: Duration::ZERO,
            phase_paused: Duration::ZERO,
            pause_start: None,
            state: EngineState::Running,
            event: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn state(&self) -> EngineState {
        self.state
    }

    /// Toggles between `Running` and `Paused`. No-op when `Completed`.
    pub fn toggle_pause(&mut self) {
        match self.state {
            EngineState::Running => {
                self.state = EngineState::Paused;
                self.pause_start = Some(Instant::now());
            }
            EngineState::Paused => {
                if let Some(ps) = self.pause_start.take() {
                    let dur = ps.elapsed();
                    self.session_paused += dur;
                    self.phase_paused += dur;
                }
                self.state = EngineState::Running;
            }
            EngineState::Completed => {}
        }
    }

    /// Resets the session back to its initial state (clears all accumulators,
    /// sets state to `Running`, and rewinds Pomodoro to the Focus phase).
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.session_start = now;
        self.phase_start = now;
        self.session_paused = Duration::ZERO;
        self.phase_paused = Duration::ZERO;
        self.pause_start = None;
        self.state = EngineState::Running;
        self.event = None;

        // Rewind Pomodoro to Focus phase.
        if let Mode::Pomodoro {
            focus,
            short_break,
            emoji,
            ..
        } = self.mode
        {
            self.mode = Mode::Pomodoro {
                phase: PomodoroPhase::Focus,
                focus,
                short_break,
                emoji,
            };
        }
    }

    /// Advances the engine by one tick. No-op unless `Running`.
    pub fn tick(&mut self) {
        if self.state != EngineState::Running {
            return;
        }

        match self.mode {
            Mode::TimerDown { total } if self.elapsed() >= total => {
                self.state = EngineState::Completed;
                if self.event.is_none() {
                    self.event = Some(EngineEvent::Completed);
                }
            }
            Mode::TimerDown { .. } => {}
            Mode::Pomodoro {
                phase,
                focus,
                short_break,
                emoji,
            } => {
                let phase_total = match phase {
                    PomodoroPhase::Focus => focus,
                    PomodoroPhase::Break => short_break,
                };

                if self.phase_elapsed() >= phase_total {
                    let next_phase = match phase {
                        PomodoroPhase::Focus => PomodoroPhase::Break,
                        PomodoroPhase::Break => PomodoroPhase::Focus,
                    };
                    // Start fresh phase; reset phase-level pause accumulator.
                    self.phase_start = Instant::now();
                    self.phase_paused = Duration::ZERO;
                    self.mode = Mode::Pomodoro {
                        phase: next_phase,
                        focus,
                        short_break,
                        emoji,
                    };
                    if self.event.is_none() {
                        self.event = Some(EngineEvent::PhaseSwitched);
                    }
                }
            }
            _ => {}
        }
    }

    /// Pause-aware session elapsed time.
    pub fn elapsed(&self) -> Duration {
        let pause_now = self
            .pause_start
            .map(|ps| ps.elapsed())
            .unwrap_or(Duration::ZERO);
        self.session_start
            .elapsed()
            .saturating_sub(self.session_paused)
            .saturating_sub(pause_now)
    }

    /// Pause-aware elapsed time within the current phase.
    fn phase_elapsed(&self) -> Duration {
        let pause_now = self
            .pause_start
            .map(|ps| ps.elapsed())
            .unwrap_or(Duration::ZERO);
        self.phase_start
            .elapsed()
            .saturating_sub(self.phase_paused)
            .saturating_sub(pause_now)
    }

    /// Returns remaining time for bounded modes (`TimerDown`, `Pomodoro`).
    pub fn remaining(&self) -> Option<Duration> {
        match self.mode {
            Mode::TimerDown { total } => Some(total.saturating_sub(self.elapsed())),
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
                Some(phase_total.saturating_sub(self.phase_elapsed()))
            }
            _ => None,
        }
    }

    /// Takes the pending event, leaving `None` behind.
    pub fn take_event(&mut self) -> Option<EngineEvent> {
        self.event.take()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn timer_down(secs: u64) -> Engine {
        Engine::new(Mode::TimerDown {
            total: Duration::from_secs(secs),
        })
    }

    fn timer_down_ms(ms: u64) -> Engine {
        Engine::new(Mode::TimerDown {
            total: Duration::from_millis(ms),
        })
    }

    fn pomodoro_ms(focus_ms: u64, break_ms: u64) -> Engine {
        Engine::new(Mode::Pomodoro {
            phase: PomodoroPhase::Focus,
            focus: Duration::from_millis(focus_ms),
            short_break: Duration::from_millis(break_ms),
            emoji: 0,
        })
    }

    // ── Timer-down ────────────────────────────────────────────────────────────

    #[test]
    fn timer_down_completion() {
        let mut e = timer_down_ms(1);
        std::thread::sleep(Duration::from_millis(20));
        e.tick();
        assert_eq!(e.state(), EngineState::Completed);
        assert_eq!(e.take_event(), Some(EngineEvent::Completed));
    }

    #[test]
    fn timer_down_no_double_event() {
        let mut e = timer_down_ms(1);
        std::thread::sleep(Duration::from_millis(20));
        e.tick();
        let _ = e.take_event();
        e.tick(); // second tick must not re-fire
        assert_eq!(e.take_event(), None);
    }

    // ── Pomodoro ──────────────────────────────────────────────────────────────

    #[test]
    fn pomodoro_phase_switch() {
        let mut e = pomodoro_ms(1, 10_000);
        std::thread::sleep(Duration::from_millis(20));
        e.tick();
        assert_eq!(e.take_event(), Some(EngineEvent::PhaseSwitched));
        if let Mode::Pomodoro { phase, .. } = e.mode() {
            assert!(
                matches!(phase, PomodoroPhase::Break),
                "expected Break phase after focus expires"
            );
        } else {
            panic!("expected Pomodoro mode");
        }
    }

    // ── Pause / resume ────────────────────────────────────────────────────────

    #[test]
    fn pause_resume_does_not_advance_elapsed() {
        let mut e = timer_down(3600);
        // Pause immediately (any tiny gap before this is < 1 ms).
        e.toggle_pause();
        assert_eq!(e.state(), EngineState::Paused);

        // Sleep while paused — this time must NOT count towards elapsed.
        std::thread::sleep(Duration::from_millis(50));

        e.toggle_pause();
        assert_eq!(e.state(), EngineState::Running);

        // Elapsed should be only the tiny interval between new() and toggle_pause().
        assert!(
            e.elapsed() < Duration::from_millis(10),
            "elapsed was {:?}, expected < 10 ms (pause time must not advance elapsed)",
            e.elapsed()
        );
    }

    #[test]
    fn paused_engine_does_not_complete() {
        // Timer expires in 1 ms; pause before it can fire.
        let mut e = timer_down_ms(1);
        e.toggle_pause();
        std::thread::sleep(Duration::from_millis(20));
        e.tick(); // must be a no-op while paused
        assert_eq!(
            e.state(),
            EngineState::Paused,
            "paused engine must not complete"
        );
        assert_eq!(e.take_event(), None);
    }

    #[test]
    fn completed_pause_toggle_is_noop() {
        let mut e = timer_down_ms(1);
        std::thread::sleep(Duration::from_millis(20));
        e.tick();
        assert_eq!(e.state(), EngineState::Completed);
        e.toggle_pause(); // must not panic or change state
        assert_eq!(e.state(), EngineState::Completed);
    }

    // ── Reset ─────────────────────────────────────────────────────────────────

    #[test]
    fn reset_restores_running_state() {
        let mut e = timer_down(3600);
        std::thread::sleep(Duration::from_millis(20));
        e.toggle_pause();
        e.reset();
        assert_eq!(e.state(), EngineState::Running);
        assert!(
            e.elapsed() < Duration::from_millis(5),
            "elapsed after reset was {:?}",
            e.elapsed()
        );
    }

    #[test]
    fn reset_after_completion_allows_rerun() {
        let mut e = timer_down_ms(1);
        std::thread::sleep(Duration::from_millis(20));
        e.tick();
        assert_eq!(e.state(), EngineState::Completed);
        e.reset();
        assert_eq!(e.state(), EngineState::Running);
        // Should not immediately complete (just reset).
        e.tick();
        assert_ne!(e.state(), EngineState::Completed);
    }

    #[test]
    fn pomodoro_reset_goes_back_to_focus() {
        let mut e = pomodoro_ms(1, 10_000);
        std::thread::sleep(Duration::from_millis(20));
        e.tick(); // triggers PhaseSwitched → now in Break
        e.reset();
        if let Mode::Pomodoro { phase, .. } = e.mode() {
            assert!(
                matches!(phase, PomodoroPhase::Focus),
                "reset should rewind Pomodoro to Focus"
            );
        } else {
            panic!("expected Pomodoro mode after reset");
        }
    }
}
