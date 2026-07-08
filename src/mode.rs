// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PomodoroPhase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    TimerUp,
    TimerDown {
        total: std::time::Duration,
    },
    Stopwatch,
    Pomodoro {
        focus: std::time::Duration,
        short_break: std::time::Duration,
        long_break: std::time::Duration,
        cycles: u32,
    },
}

impl Mode {
    /// Returns a mode-kind discriminator for dirty tracking.
    /// Non-Pomodoro modes use small sentinel values; Pomodoro uses 10+.
    /// The Engine tracks phase/cycle changes separately.
    pub fn kind_marker(&self) -> u8 {
        match self {
            Mode::TimerUp => 0,
            Mode::TimerDown { .. } => 1,
            Mode::Stopwatch => 2,
            Mode::Pomodoro { .. } => 10,
        }
    }

    /// Returns the duration for a given Pomodoro phase.
    /// Only valid when `self` is `Mode::Pomodoro`.
    pub fn phase_duration(&self, phase: PomodoroPhase) -> std::time::Duration {
        match self {
            Mode::Pomodoro {
                focus,
                short_break,
                long_break,
                ..
            } => match phase {
                PomodoroPhase::Focus => *focus,
                PomodoroPhase::ShortBreak => *short_break,
                PomodoroPhase::LongBreak => *long_break,
            },
            _ => std::time::Duration::ZERO,
        }
    }

    /// Returns the cycle count for Pomodoro mode.
    /// Returns 0 for non-Pomodoro modes.
    pub fn pomodoro_cycles(&self) -> u32 {
        match self {
            Mode::Pomodoro { cycles, .. } => *cycles,
            _ => 0,
        }
    }
}
