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
        phase: PomodoroPhase,
        focus: std::time::Duration,
        short_break: std::time::Duration,
        long_break: std::time::Duration,
        cycles: u32,
        current_cycle: u32,
        emoji: u8,
    },
}

impl Mode {
    /// v10: Returns the current Pomodoro phase, or a sentinel for non-Pomodoro modes.
    /// Used for dirty tracking — detects phase switches without matching the full enum.
    pub fn phase_marker(&self) -> u8 {
        match self {
            Mode::TimerUp => 0,
            Mode::TimerDown { .. } => 1,
            Mode::Stopwatch => 2,
            Mode::Pomodoro {
                phase,
                current_cycle,
                ..
            } => 10 + (*phase as u8) + (*current_cycle as u8 * 10),
        }
    }
}
