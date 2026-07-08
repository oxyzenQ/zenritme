// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

use crate::engine::EngineState;
use crate::mode::{Mode, PomodoroPhase};
use crate::theme::ColorFields;
use std::time::Duration;

use super::animation_wrap;
use super::colored;

// ─── Title builder ───────────────────────────────────────────────────────────

pub(crate) fn build_title(state: &super::RenderState) -> String {
    match state.mode {
        Mode::TimerUp => "TIMER UP".to_string(),
        Mode::TimerDown { .. } => "TIMER DOWN".to_string(),
        Mode::Stopwatch => "STOPWATCH".to_string(),
        Mode::Pomodoro { cycles, emoji, .. } => {
            let phase = state.engine_phase;
            let cycle = state.engine_cycle;
            let base = match state.state {
                EngineState::Completed => "COMPLETE".to_string(),
                _ => match phase {
                    PomodoroPhase::Focus => format!("FOCUS {}/{}", cycle, cycles),
                    PomodoroPhase::ShortBreak => {
                        format!("SHORT BREAK {}/{}", cycle, cycles)
                    }
                    PomodoroPhase::LongBreak => "LONG BREAK".to_string(),
                },
            };
            let dyn_idx = emoji.wrapping_add((state.elapsed.as_secs() / 5) as u8);
            format!("{} {}", base, pomodoro_emoji(dyn_idx))
        }
    }
}

// ─── Time formatting ─────────────────────────────────────────────────────────

pub(crate) fn format_hms(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

// ─── Time display builder ───────────────────────────────────────────────────

pub(crate) fn build_time(state: &super::RenderState) -> String {
    let primary = match state.mode {
        Mode::TimerDown { .. } | Mode::Pomodoro { .. } => state.remaining.unwrap_or_default(),
        _ => state.elapsed,
    };
    format_hms(primary)
}

// ─── Mode info line ──────────────────────────────────────────────────────────

pub(crate) fn push_mode_info(lines: &mut Vec<String>, state: &super::RenderState) {
    match state.mode {
        Mode::TimerDown { .. } => {
            lines.push(format!("Elapsed: {}", format_hms(state.elapsed)));
        }
        Mode::Pomodoro { .. } => {
            lines.push(format!("Session: {}", format_hms(state.elapsed)));
        }
        _ => {}
    }
}

// ─── State label (PAUSED / DONE) ────────────────────────────────────────────

pub(crate) fn push_state_label(
    lines: &mut Vec<String>,
    state: &super::RenderState,
    c: &ColorFields,
    r: &str,
) {
    match state.state {
        EngineState::Paused => {
            lines.push(String::new());
            lines.push(colored("[ PAUSED ]", c.accent, r));
        }
        EngineState::Completed => {
            lines.push(String::new());
            let burst = animation_wrap::completion_burst(state.frame);
            lines.push(colored(&format!("[ DONE ] {}", burst), c.accent, r));
        }
        EngineState::Running => {}
    }
}

// ─── Pomodoro emoji ──────────────────────────────────────────────────────────

pub(crate) fn pomodoro_emoji(idx: u8) -> &'static str {
    const EMOJIS: [&str; 10] = [
        "\u{1F345}", // 🍅
        "\u{2615}",  // ☕
        "\u{1F319}", // 🌙
        "\u{26A1}",  // ⚡
        "\u{1F9E0}", // 🧠
        "\u{1F3A7}", // 🎧
        "\u{1F33F}", // 🌿
        "\u{1F4CC}", // 📌
        "\u{1F525}", // 🔥
        "\u{1F56F}", // 🕯️
    ];
    EMOJIS[(idx as usize) % EMOJIS.len()]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_hms_seconds_only() {
        assert_eq!(format_hms(Duration::from_secs(45)), "00:45");
    }

    #[test]
    fn format_hms_minutes_and_seconds() {
        assert_eq!(format_hms(Duration::from_secs(135)), "02:15");
    }

    #[test]
    fn format_hms_with_hours() {
        assert_eq!(format_hms(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn format_hms_zero() {
        assert_eq!(format_hms(Duration::ZERO), "00:00");
    }

    #[test]
    fn pomodoro_emoji_cycles() {
        // Just verify it returns a non-empty string without panicking
        assert!(!pomodoro_emoji(0).is_empty());
        assert!(!pomodoro_emoji(5).is_empty());
        assert!(!pomodoro_emoji(255).is_empty());
    }

    #[test]
    fn pomodoro_emoji_wraps() {
        // Index 10 should wrap to index 0
        assert_eq!(pomodoro_emoji(0), pomodoro_emoji(10));
        assert_eq!(pomodoro_emoji(0), pomodoro_emoji(20));
    }

    #[test]
    fn format_hms_large_value() {
        let d = Duration::from_secs(86400 + 3661);
        assert_eq!(format_hms(d), "25:01:01");
    }
}
