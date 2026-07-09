// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

use crate::mode::Mode;
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
        Mode::Pomodoro { .. } => {
            let phase_total = engine.mode().phase_duration(engine.pomo_phase());
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
        "[{}{}] {:3.0}%",
        colored(&"\u{2588}".repeat(filled), c.progress_fill, c.reset),
        colored(&"\u{2591}".repeat(empty), c.progress_empty, c.reset),
        clamped * 100.0,
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ColorFields;

    #[test]
    fn colored_bar_zero() {
        let c = ColorFields::plain();
        let bar = colored_bar(0.0, 10, &c);
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn colored_bar_half() {
        let c = ColorFields::plain();
        let bar = colored_bar(0.5, 10, &c);
        assert!(bar.contains(" 50%"));
    }

    #[test]
    fn colored_bar_full() {
        let c = ColorFields::plain();
        let bar = colored_bar(1.0, 10, &c);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn colored_bar_clamps_above_one() {
        let c = ColorFields::plain();
        let bar = colored_bar(1.5, 10, &c);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn colored_bar_clamps_below_zero() {
        let c = ColorFields::plain();
        let bar = colored_bar(-0.5, 10, &c);
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn colored_bar_width_one() {
        let c = ColorFields::plain();
        let bar = colored_bar(0.75, 1, &c);
        // At width 1, 0.75 rounds to 1 filled block
        assert!(bar.starts_with('['));
        assert!(bar.contains("%"));
    }

    #[test]
    fn colored_bar_empty_width() {
        let c = ColorFields::plain();
        let bar = colored_bar(0.5, 0, &c);
        // Width 0: progress is still shown but no fill/empty blocks
        assert!(bar.contains(" 50%"));
        assert!(!bar.contains('\u{2588}'));
        assert!(!bar.contains('\u{2591}'));
    }}
