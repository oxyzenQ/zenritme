// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

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
