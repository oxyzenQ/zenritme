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
}
