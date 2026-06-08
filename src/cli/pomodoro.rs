// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

use crate::mode::{Mode, PomodoroPhase};
use crate::render::ViewMode;
use crate::theme::Theme;
use std::time::{SystemTime, UNIX_EPOCH};

use super::duration::parse_duration;

/// Try to extract a pomodoro-related flag from the argument list.
///
/// Returns `Ok(true)` if the flag was consumed by this function,
/// `Ok(false)` if it was not a pomodoro flag (caller should handle it).
pub(super) fn extract_flag(
    all: &[String],
    i: &mut usize,
    opts: &mut super::PomodoroOpts,
) -> Result<bool, String> {
    match all[*i].as_str() {
        "--focus" => {
            let val = all
                .get(*i + 1)
                .ok_or("missing value after --focus".to_string())?;
            let dur = parse_duration(val)?;
            if dur.is_zero() {
                return Err("--focus duration must be > 0".to_string());
            }
            opts.focus = Some(dur);
            *i += 2;
            Ok(true)
        }
        "--break" => {
            let val = all
                .get(*i + 1)
                .ok_or("missing value after --break".to_string())?;
            let dur = parse_duration(val)?;
            if dur.is_zero() {
                return Err("--break duration must be > 0".to_string());
            }
            opts.short_break = Some(dur);
            *i += 2;
            Ok(true)
        }
        "--long-break" => {
            let val = all
                .get(*i + 1)
                .ok_or("missing value after --long-break".to_string())?;
            let dur = parse_duration(val)?;
            if dur.is_zero() {
                return Err("--long-break duration must be > 0".to_string());
            }
            opts.long_break = Some(dur);
            *i += 2;
            Ok(true)
        }
        "--cycles" => {
            let val = all
                .get(*i + 1)
                .ok_or("missing value after --cycles".to_string())?;
            let n: u32 = val
                .parse()
                .map_err(|_| "--cycles must be a positive integer".to_string())?;
            if n == 0 {
                return Err("--cycles must be > 0".to_string());
            }
            opts.cycles = Some(n);
            *i += 2;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Resolve the `--pomodoro` mode arm: positional args, defaults, and conflict detection.
pub(super) fn resolve_mode<I>(
    mut args: I,
    pomo: super::PomodoroOpts,
    theme: Theme,
    view: ViewMode,
    mute: bool,
    profile: crate::sound::SoundProfile,
) -> Result<super::Command, String>
where
    I: Iterator<Item = String>,
{
    let default_focus = std::time::Duration::from_secs(25 * 60);
    let default_short_break = std::time::Duration::from_secs(5 * 60);
    let default_long_break = std::time::Duration::from_secs(15 * 60);
    let default_cycles: u32 = 4;

    // Check for positional FOCUS BREAK (legacy syntax).
    // If --focus or --break were already used via options, positional args
    // are ambiguous and must be rejected with a clear message.
    let (focus, short_break) = match args.next() {
        None => (
            pomo.focus.unwrap_or(default_focus),
            pomo.short_break.unwrap_or(default_short_break),
        ),
        Some(focus_str) => {
            if pomo.focus.is_some() || pomo.short_break.is_some() {
                return Err(format!(
                    "unexpected argument '{}' (positional FOCUS BREAK conflicts with --focus/--break options)",
                    focus_str
                ));
            }
            let Some(break_str) = args.next() else {
                return Err("missing BREAK after --pomodoro FOCUS".to_string());
            };
            super::reject_extra(&mut args, "--pomodoro")?;
            let focus = parse_duration(&focus_str)?;
            let short_break = parse_duration(&break_str)?;
            if focus.is_zero() || short_break.is_zero() {
                return Err("pomodoro durations must be > 0".to_string());
            }
            (focus, short_break)
        }
    };

    let long_break = pomo.long_break.unwrap_or(default_long_break);
    let cycles = pomo.cycles.unwrap_or(default_cycles);

    Ok(super::Command::Run {
        mode: Mode::Pomodoro {
            phase: PomodoroPhase::Focus,
            focus,
            short_break,
            long_break,
            cycles,
            current_cycle: 1,
            emoji: pick_pomodoro_emoji(),
        },
        theme,
        view,
        mute,
        profile,
    })
}

fn pick_pomodoro_emoji() -> u8 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as u8) % 10
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::{parse_args, Command};
    use crate::mode::Mode;
    use crate::render::ViewMode;
    use crate::theme::Theme;

    /// Helper: build a `parse_args`-compatible iterator from a slice of string literals.
    fn args(v: &[&str]) -> impl Iterator<Item = String> {
        v.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Helper: extract the Pomodoro fields from a parsed Command.
    fn pomodoro_fields(
        cmd: Command,
    ) -> (
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
        u32,
    ) {
        if let Command::Run {
            mode:
                Mode::Pomodoro {
                    focus,
                    short_break,
                    long_break,
                    cycles,
                    ..
                },
            ..
        } = cmd
        {
            return (focus, short_break, long_break, cycles);
        }
        panic!("expected Pomodoro Run command");
    }

    /// Helper: unwrap the Err side of a parse result (avoids needing Debug on Command).
    fn expect_err(cli_args: &[&str]) -> String {
        match parse_args(args(cli_args)) {
            Ok(_) => panic!("expected Err, got Ok"),
            Err(e) => e,
        }
    }

    // ── Pomodoro defaults and valid combos ─────────────────────────────────

    #[test]
    fn pomodoro_defaults() {
        let cmd = parse_args(args(&["--pomodoro"])).unwrap();
        let (focus, short_break, long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(25 * 60));
        assert_eq!(short_break, std::time::Duration::from_secs(5 * 60));
        assert_eq!(long_break, std::time::Duration::from_secs(15 * 60));
        assert_eq!(cycles, 4);
    }

    #[test]
    fn pomodoro_old_syntax_still_works() {
        let cmd = parse_args(args(&["--pomodoro", "3s", "2s"])).unwrap();
        let (focus, short_break, long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(3));
        assert_eq!(short_break, std::time::Duration::from_secs(2));
        // long_break and cycles should be defaults when using old syntax
        assert_eq!(long_break, std::time::Duration::from_secs(15 * 60));
        assert_eq!(cycles, 4);
    }

    #[test]
    fn pomodoro_focus_flag() {
        let cmd = parse_args(args(&["--pomodoro", "--focus", "45m"])).unwrap();
        let (focus, short_break, long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(45 * 60));
        assert_eq!(short_break, std::time::Duration::from_secs(5 * 60));
        assert_eq!(long_break, std::time::Duration::from_secs(15 * 60));
        assert_eq!(cycles, 4);
    }

    #[test]
    fn pomodoro_break_flag() {
        let cmd = parse_args(args(&["--pomodoro", "--break", "10m"])).unwrap();
        let (_focus, short_break, _long_break, _cycles) = pomodoro_fields(cmd);
        assert_eq!(short_break, std::time::Duration::from_secs(10 * 60));
    }

    #[test]
    fn pomodoro_long_break_flag() {
        let cmd = parse_args(args(&["--pomodoro", "--long-break", "20m"])).unwrap();
        let (_focus, _short_break, long_break, _cycles) = pomodoro_fields(cmd);
        assert_eq!(long_break, std::time::Duration::from_secs(20 * 60));
    }

    #[test]
    fn pomodoro_cycles_flag() {
        let cmd = parse_args(args(&["--pomodoro", "--cycles", "3"])).unwrap();
        let (_focus, _short_break, _long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(cycles, 3);
    }

    #[test]
    fn pomodoro_all_flags_combined() {
        let cmd = parse_args(args(&[
            "--pomodoro",
            "--focus",
            "45m",
            "--break",
            "10m",
            "--long-break",
            "20m",
            "--cycles",
            "3",
        ]))
        .unwrap();
        let (focus, short_break, long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(45 * 60));
        assert_eq!(short_break, std::time::Duration::from_secs(10 * 60));
        assert_eq!(long_break, std::time::Duration::from_secs(20 * 60));
        assert_eq!(cycles, 3);
    }

    #[test]
    fn pomodoro_flags_with_theme_and_view() {
        let cmd = parse_args(args(&[
            "--pomodoro",
            "--focus",
            "3s",
            "--break",
            "2s",
            "--long-break",
            "4s",
            "--cycles",
            "2",
            "--theme",
            "aura",
            "--view",
            "cinematic",
        ]))
        .unwrap();
        if let Command::Run {
            mode, theme, view, ..
        } = cmd
        {
            if let Mode::Pomodoro {
                focus,
                short_break,
                long_break,
                cycles,
                ..
            } = mode
            {
                assert_eq!(focus, std::time::Duration::from_secs(3));
                assert_eq!(short_break, std::time::Duration::from_secs(2));
                assert_eq!(long_break, std::time::Duration::from_secs(4));
                assert_eq!(cycles, 2);
            } else {
                panic!("expected Pomodoro");
            }
            assert_eq!(theme, Theme::Aura);
            assert_eq!(view, ViewMode::Cinematic);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn pomodoro_old_syntax_with_long_break_override() {
        let cmd = parse_args(args(&[
            "--pomodoro",
            "3s",
            "2s",
            "--long-break",
            "4s",
            "--cycles",
            "2",
        ]))
        .unwrap();
        let (focus, short_break, long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(3));
        assert_eq!(short_break, std::time::Duration::from_secs(2));
        assert_eq!(long_break, std::time::Duration::from_secs(4));
        assert_eq!(cycles, 2);
    }

    #[test]
    fn pomodoro_cycles_can_be_one() {
        let cmd = parse_args(args(&["--pomodoro", "--cycles", "1"])).unwrap();
        let (_focus, _short_break, _long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(cycles, 1);
    }

    #[test]
    fn pomodoro_flags_before_mode() {
        let cmd = parse_args(args(&["--focus", "3s", "--cycles", "2", "--pomodoro"])).unwrap();
        let (focus, _short_break, _long_break, cycles) = pomodoro_fields(cmd);
        assert_eq!(focus, std::time::Duration::from_secs(3));
        assert_eq!(cycles, 2);
    }

    // ── Pomodoro error cases ──────────────────────────────────────────────────

    #[test]
    fn pomodoro_missing_focus_value() {
        assert!(parse_args(args(&["--pomodoro", "--focus"])).is_err());
    }

    #[test]
    fn pomodoro_missing_break_value() {
        assert!(parse_args(args(&["--pomodoro", "--break"])).is_err());
    }

    #[test]
    fn pomodoro_missing_long_break_value() {
        assert!(parse_args(args(&["--pomodoro", "--long-break"])).is_err());
    }

    #[test]
    fn pomodoro_missing_cycles_value() {
        assert!(parse_args(args(&["--pomodoro", "--cycles"])).is_err());
    }

    #[test]
    fn pomodoro_zero_cycles_rejected() {
        assert!(parse_args(args(&["--pomodoro", "--cycles", "0"])).is_err());
    }

    #[test]
    fn pomodoro_invalid_cycles_rejected() {
        assert!(parse_args(args(&["--pomodoro", "--cycles", "abc"])).is_err());
    }

    #[test]
    fn pomodoro_zero_focus_rejected() {
        assert!(parse_args(args(&["--pomodoro", "--focus", "0s"])).is_err());
    }

    #[test]
    fn pomodoro_zero_break_rejected() {
        assert!(parse_args(args(&["--pomodoro", "--break", "0s"])).is_err());
    }

    #[test]
    fn pomodoro_zero_long_break_rejected() {
        assert!(parse_args(args(&["--pomodoro", "--long-break", "0s"])).is_err());
    }

    // ── Positional / option conflict ────────────────────────────────────────

    #[test]
    fn pomodoro_options_with_extra_positional_rejected() {
        let err = expect_err(&["--pomodoro", "--focus", "3s", "--break", "2s", "extra"]);
        assert!(
            err.contains("unexpected"),
            "expected 'unexpected' in error, got: {}",
            err
        );
    }

    #[test]
    fn pomodoro_only_focus_option_with_extra_positional_rejected() {
        let err = expect_err(&["--pomodoro", "--focus", "3s", "extra"]);
        assert!(
            err.contains("unexpected"),
            "expected 'unexpected' in error, got: {}",
            err
        );
    }

    #[test]
    fn pomodoro_old_syntax_still_yields_missing_break() {
        // When no options are used, a lone positional arg still gives the
        // original "missing BREAK" message.
        let err = expect_err(&["--pomodoro", "25m"]);
        assert_eq!(err, "missing BREAK after --pomodoro FOCUS");
    }

    // ── extract_flag unit tests ──────────────────────────────────────────────

    #[test]
    fn extract_flag_focus_consumed() {
        let all: Vec<String> = ["--focus", "30s", "--pomodoro"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut opts = super::super::PomodoroOpts::default();
        let mut i = 0;
        let consumed = super::extract_flag(&all, &mut i, &mut opts).unwrap();
        assert!(consumed);
        assert_eq!(i, 2); // advanced past --focus and 30s
        assert_eq!(opts.focus, Some(std::time::Duration::from_secs(30)));
        assert!(opts.short_break.is_none());
    }

    #[test]
    fn extract_flag_unknown_not_consumed() {
        let all: Vec<String> = ["--timer-up", "--focus", "30s"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut opts = super::super::PomodoroOpts::default();
        let mut i = 0;
        let consumed = super::extract_flag(&all, &mut i, &mut opts).unwrap();
        assert!(!consumed);
        assert_eq!(i, 0); // not advanced
    }

    #[test]
    fn extract_flag_cycles_consumed() {
        let all: Vec<String> = ["--cycles", "6"].iter().map(|s| s.to_string()).collect();
        let mut opts = super::super::PomodoroOpts::default();
        let mut i = 0;
        let consumed = super::extract_flag(&all, &mut i, &mut opts).unwrap();
        assert!(consumed);
        assert_eq!(i, 2);
        assert_eq!(opts.cycles, Some(6));
    }
}
