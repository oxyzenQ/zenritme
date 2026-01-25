#[derive(Clone, Copy, Debug)]
pub enum PomodoroPhase {
    Focus,
    Break,
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
        emoji: u8,
    },
}
