use crate::mode::{Mode, PomodoroPhase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineEvent {
    Completed,
    PhaseSwitched,
}

pub struct Engine {
    mode: Mode,
    session_start: std::time::Instant,
    phase_start: std::time::Instant,
    done: bool,
    event: Option<EngineEvent>,
}

impl Engine {
    pub fn new(mode: Mode) -> Self {
        let now = std::time::Instant::now();
        Self {
            mode,
            session_start: now,
            phase_start: now,
            done: false,
            event: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn tick(&mut self) {
        if self.done {
            return;
        }

        match self.mode {
            Mode::TimerDown { total } => {
                if self.elapsed() >= total {
                    self.done = true;
                    if self.event.is_none() {
                        self.event = Some(EngineEvent::Completed);
                    }
                }
            }
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

                if self.phase_start.elapsed() >= phase_total {
                    let next_phase = match phase {
                        PomodoroPhase::Focus => PomodoroPhase::Break,
                        PomodoroPhase::Break => PomodoroPhase::Focus,
                    };
                    self.phase_start = std::time::Instant::now();
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

    pub fn elapsed(&self) -> std::time::Duration {
        self.session_start.elapsed()
    }

    pub fn remaining(&self) -> Option<std::time::Duration> {
        match self.mode {
            Mode::TimerDown { total } => {
                let elapsed = self.elapsed();
                Some(total.saturating_sub(elapsed))
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
                Some(phase_total.saturating_sub(self.phase_start.elapsed()))
            }
            _ => None,
        }
    }

    pub fn take_event(&mut self) -> Option<EngineEvent> {
        self.event.take()
    }
}
