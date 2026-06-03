// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::mode::{Mode, PomodoroPhase};
use crate::theme::ColorFields;

use super::colored;

// ─── Public: compute_progress ────────────────────────────────────────────────

/// Compute `Some(0.0..=1.0)` for bounded modes, `None` for unbounded.
pub(crate) fn compute_progress(engine: &crate::engine::Engine) -> Option<f32> {
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
            long_break,
            ..
        } => {
            let phase_total = match phase {
                PomodoroPhase::Focus => focus,
                PomodoroPhase::ShortBreak => short_break,
                PomodoroPhase::LongBreak => long_break,
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

// ─── Public: colored bar ─────────────────────────────────────────────────────

pub(crate) fn colored_bar(progress: f32, width: usize, c: &ColorFields) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = ((clamped * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!(
        "[{}{}{}] {:3.0}%",
        colored(&"\u{2588}".repeat(filled), c.progress_fill, c.reset),
        colored(&"\u{2591}".repeat(empty), c.progress_empty, c.reset),
        c.reset,
        clamped * 100.0,
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A no-color `ColorFields` for tests (all codes empty).
    fn plain_colors() -> ColorFields {
        ColorFields {
            title: "",
            time: "",
            progress_fill: "",
            progress_empty: "",
            label: "",
            dim: "",
            border: "",
            accent: "",
            spinner: "",
            reset: "",
        }
    }

    #[test]
    fn colored_bar_zero() {
        let c = plain_colors();
        let bar = colored_bar(0.0, 10, &c);
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn colored_bar_half() {
        let c = plain_colors();
        let bar = colored_bar(0.5, 10, &c);
        assert!(bar.contains(" 50%"));
    }

    #[test]
    fn colored_bar_full() {
        let c = plain_colors();
        let bar = colored_bar(1.0, 10, &c);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn colored_bar_clamps_above_one() {
        let c = plain_colors();
        let bar = colored_bar(1.5, 10, &c);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn colored_bar_clamps_below_zero() {
        let c = plain_colors();
        let bar = colored_bar(-0.5, 10, &c);
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn colored_bar_width_one() {
        let c = plain_colors();
        let bar = colored_bar(0.75, 1, &c);
        // At width 1, 0.75 rounds to 1 filled block
        assert!(bar.starts_with('['));
        assert!(bar.contains("%"));
    }

    #[test]
    fn colored_bar_empty_width() {
        let c = plain_colors();
        let bar = colored_bar(0.5, 0, &c);
        // Width 0: progress is still shown but no fill/empty blocks
        assert!(bar.contains(" 50%"));
        assert!(!bar.contains('\u{2588}'));
        assert!(!bar.contains('\u{2591}'));
    }

    #[test]
    fn colored_bar_narrow_width() {
        let c = plain_colors();
        let bar = colored_bar(0.3, 4, &c);
        // 0.3 * 4 = 1.2, rounds to 1 filled
        assert!(bar.contains(" 30%"));
    }
}
