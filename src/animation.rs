// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Pure animation functions for terminal rendering.
//!
//! All functions are deterministic based on a frame counter and produce
//! terminal-safe output without any side effects.

/// Braille spinner cycle — calm, 10-frame rotation.
pub fn spinner(frame: u64) -> &'static str {
    const CHARS: [&str; 10] = [
        "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}",
        "\u{2827}", "\u{2807}", "\u{280F}",
    ];
    CHARS[(frame as usize) % CHARS.len()]
}

/// Calm orbit pattern: a ring of dots with one active braille dot that drifts.
///
/// The active position advances every 2 frames for a relaxed rotation speed.
pub fn orbit(frame: u64) -> String {
    const N: usize = 14;
    const ACTIVE: [&str; 14] = [
        "\u{2801}", "\u{2802}", "\u{2804}", "\u{2840}", "\u{2840}", "\u{2820}", "\u{2810}",
        "\u{2808}", "\u{280A}", "\u{2812}", "\u{2814}", "\u{2822}", "\u{2821}", "\u{2803}",
    ];
    let step = (frame / 2) as usize % N;
    let mut s = String::with_capacity(N * 3);
    for i in 0..N {
        if i > 0 {
            s.push(' ');
        }
        if i == step {
            s.push_str(ACTIVE[step]);
        } else {
            s.push('\u{00B7}'); // ·
        }
    }
    s
}

/// Fixed-width progress bar with fill and empty characters.
///
/// `[████████░░░░░░░░░░░░]  42%`
///
/// `█` (U+2588) and `░` (U+2591) are each 1 column wide in most terminals.
#[allow(dead_code)]
pub fn progress_bar(progress: f32, width: usize) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = ((clamped * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!(
        "[{}{}] {:3.0}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        clamped * 100.0,
    )
}

/// Completion burst: a pulsing braille fill that cycles through fill stages.
///
/// Used when the timer completes to give a subtle celebratory pulse.
pub fn completion_burst(frame: u64) -> &'static str {
    const STAGES: [&str; 8] = [
        "\u{00B7}", "\u{2801}", "\u{2809}", "\u{281B}", "\u{287F}", "\u{28FF}", "\u{287F}",
        "\u{281B}",
    ];
    STAGES[(frame / 3) as usize % STAGES.len()]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_cycles() {
        let a = spinner(0);
        let b = spinner(10);
        assert_eq!(a, b);
    }

    #[test]
    fn spinner_returns_braille() {
        for f in 0..20 {
            let s = spinner(f);
            assert!(!s.is_empty(), "spinner({}) should not be empty", f);
            assert!(s.len() <= 4, "spinner({}) should be braille-sized", f);
        }
    }

    #[test]
    fn orbit_has_dots() {
        for f in 0..28 {
            let o = orbit(f);
            assert!(
                o.contains('\u{00B7}'),
                "orbit({}) should have dots: {}",
                f,
                o
            );
        }
    }

    #[test]
    fn orbit_cycles() {
        let a = orbit(0);
        let b = orbit(28); // 28 / 2 = 14 = full cycle
        assert_eq!(a, b);
    }

    #[test]
    fn orbit_length_is_consistent() {
        let a = orbit(0);
        let b = orbit(7);
        assert_eq!(a.chars().count(), b.chars().count());
    }

    #[test]
    fn progress_bar_zero() {
        let bar = progress_bar(0.0, 10);
        assert!(bar.starts_with('['));
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn progress_bar_full() {
        let bar = progress_bar(1.0, 10);
        assert!(bar.contains("100%"));
        assert!(!bar.contains('\u{2591}'));
    }

    #[test]
    fn progress_bar_half() {
        let bar = progress_bar(0.5, 10);
        assert!(bar.contains(" 50%"));
    }

    #[test]
    fn progress_bar_clamped() {
        assert!(progress_bar(1.5, 10).contains("100%"));
        assert!(progress_bar(-0.5, 10).contains("  0%"));
    }

    #[test]
    fn progress_bar_width() {
        let bar = progress_bar(0.5, 20);
        assert!(bar.starts_with('['));
        assert!(bar.contains(']'));
    }

    #[test]
    fn completion_burst_cycles() {
        let a = completion_burst(0);
        let b = completion_burst(24); // 24 / 3 = 8 = full cycle
        assert_eq!(a, b);
    }

    #[test]
    fn completion_burst_all_non_empty() {
        for f in 0..24 {
            assert!(!completion_burst(f).is_empty());
        }
    }
}
