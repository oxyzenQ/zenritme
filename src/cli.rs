// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::mode::{Mode, PomodoroPhase};
use crate::render::ViewMode;
use crate::theme::Theme;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Command {
    Help,
    Run {
        mode: Mode,
        theme: Theme,
        view: ViewMode,
    },
    SoundTest,
}

/// Pomodoro-specific options extracted during the CLI pre-pass.
#[derive(Default)]
struct PomodoroOpts {
    focus: Option<std::time::Duration>,
    short_break: Option<std::time::Duration>,
    long_break: Option<std::time::Duration>,
    cycles: Option<u32>,
}

pub fn usage() -> String {
    format!(
        "zenritme v{ver}\n\n\
         Usage:\n\
         \x20 zenritme --timer-up\n\
         \x20 zenritme --timer-down <DURATION>\n\
         \x20 zenritme --stopwatch\n\
         \x20 zenritme --pomodoro [FOCUS BREAK]\n\
         \x20 zenritme --sound-test\n\
         \x20 zenritme --help\n\n\
         Options:\n\
         \x20 --theme <THEME>          void | ember | aura | forest | mono  (default: void)\n\
         \x20 --view <VIEW>            minimal | orbit | cinematic             (default: orbit)\n\
         \x20 --focus <DURATION>       focus session length                   (default: 25m)\n\
         \x20 --break <DURATION>       short break length                    (default: 5m)\n\
         \x20 --long-break <DURATION>  long break length                     (default: 15m)\n\
         \x20 --cycles <N>             focus sessions per round               (default: 4)\n\n\
         Duration format:\n\
         \x20 <number>s | <number>m | <number>h\n\
         \x20 Examples: 30s  10m  1h\n\n\
         Pomodoro examples:\n\
         \x20 zenritme --pomodoro\n\
         \x20 zenritme --pomodoro 3s 2s\n\
         \x20 zenritme --pomodoro --cycles 4\n\
         \x20 zenritme --pomodoro --focus 45m --break 10m --long-break 20m --cycles 3\n\n\
         Controls while running:\n\
         \x20 q / Esc   quit\n\
         \x20 p         pause / resume\n\
         \x20 r         reset current session",
        ver = env!("CARGO_PKG_VERSION")
    )
}

/// Parse all arguments. `--theme`, `--view`, and pomodoro-specific flags are
/// extracted in a pre-pass so they may appear before or after the mode flag.
pub fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let all: Vec<String> = args.collect();

    let mut theme = Theme::Void;
    let mut view = ViewMode::Orbit;
    let mut pomodoro_opts = PomodoroOpts::default();
    let mut mode_args: Vec<String> = Vec::new();
    let mut i = 0;

    while i < all.len() {
        match all[i].as_str() {
            "--theme" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --theme".to_string())?;
                theme = Theme::from_name(val)
                    .ok_or_else(|| format!("unknown theme: {}  (see --help)", val))?;
                i += 2;
            }
            "--view" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --view".to_string())?;
                view = ViewMode::from_name(val)
                    .ok_or_else(|| format!("unknown view: {}  (see --help)", val))?;
                i += 2;
            }
            "--focus" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --focus".to_string())?;
                let dur = parse_duration(val)?;
                if dur.is_zero() {
                    return Err("--focus duration must be > 0".to_string());
                }
                pomodoro_opts.focus = Some(dur);
                i += 2;
            }
            "--break" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --break".to_string())?;
                let dur = parse_duration(val)?;
                if dur.is_zero() {
                    return Err("--break duration must be > 0".to_string());
                }
                pomodoro_opts.short_break = Some(dur);
                i += 2;
            }
            "--long-break" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --long-break".to_string())?;
                let dur = parse_duration(val)?;
                if dur.is_zero() {
                    return Err("--long-break duration must be > 0".to_string());
                }
                pomodoro_opts.long_break = Some(dur);
                i += 2;
            }
            "--cycles" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --cycles".to_string())?;
                let n: u32 = val
                    .parse()
                    .map_err(|_| "--cycles must be a positive integer".to_string())?;
                if n == 0 {
                    return Err("--cycles must be > 0".to_string());
                }
                pomodoro_opts.cycles = Some(n);
                i += 2;
            }
            _ => {
                mode_args.push(all[i].clone());
                i += 1;
            }
        }
    }

    parse_mode(mode_args.into_iter(), theme, view, pomodoro_opts)
}

/// Parse the mode-specific arguments (after pre-pass extraction).
fn parse_mode<I>(
    mut args: I,
    theme: Theme,
    view: ViewMode,
    pomo: PomodoroOpts,
) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let Some(flag) = args.next() else {
        return Ok(Command::Help);
    };

    match flag.as_str() {
        "--help" | "-h" => {
            reject_extra(&mut args, "--help")?;
            Ok(Command::Help)
        }

        "--sound-test" => {
            reject_extra(&mut args, "--sound-test")?;
            Ok(Command::SoundTest)
        }

        "--timer-up" | "--timer-upward-minute" => {
            reject_extra(&mut args, "--timer-up")?;
            Ok(Command::Run {
                mode: Mode::TimerUp,
                theme,
                view,
            })
        }

        "--stopwatch" => {
            reject_extra(&mut args, "--stopwatch")?;
            Ok(Command::Run {
                mode: Mode::Stopwatch,
                theme,
                view,
            })
        }

        "--pomodoro" => {
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
                    reject_extra(&mut args, "--pomodoro")?;
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

            Ok(Command::Run {
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
            })
        }

        "--timer-down" | "--timer-back" => {
            let Some(dur_str) = args.next() else {
                return Err("missing DURATION after --timer-down".to_string());
            };
            let dur = parse_duration(&dur_str)?;
            if dur.is_zero() {
                return Err("duration must be > 0".to_string());
            }
            reject_extra(&mut args, "--timer-down")?;
            Ok(Command::Run {
                mode: Mode::TimerDown { total: dur },
                theme,
                view,
            })
        }

        other => Err(format!("unknown argument: {}", other)),
    }
}

/// Returns `Err` if the iterator still has items.
fn reject_extra<I>(args: &mut I, flag: &str) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    if args.next().is_some() {
        Err(format!("{} takes no extra arguments", flag))
    } else {
        Ok(())
    }
}

fn parse_duration(s: &str) -> Result<std::time::Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration is empty".to_string());
    }

    let lower = s.to_ascii_lowercase();
    let (num_str, unit) = match lower.chars().last() {
        Some('s') => (&lower[..lower.len() - 1], 's'),
        Some('m') => (&lower[..lower.len() - 1], 'm'),
        Some('h') => (&lower[..lower.len() - 1], 'h'),
        _ => return Err("duration must end with s, m, or h".to_string()),
    };

    let value: u64 = num_str
        .parse()
        .map_err(|_| "duration number is invalid".to_string())?;

    let secs = match unit {
        's' => value,
        'm' => value.saturating_mul(60),
        'h' => value.saturating_mul(3600),
        _ => return Err("duration unit is invalid".to_string()),
    };

    Ok(std::time::Duration::from_secs(secs))
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
    use super::*;

    /// Helper: build a `parse_args`-compatible iterator from a slice of string literals.
    fn args(v: &[&str]) -> impl Iterator<Item = String> {
        v.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    // ── Duration parsing (tested via --timer-down) ────────────────────────────

    #[test]
    fn parse_30s() {
        assert!(parse_args(args(&["--timer-down", "30s"])).is_ok());
    }

    #[test]
    fn parse_10m() {
        assert!(parse_args(args(&["--timer-down", "10m"])).is_ok());
    }

    #[test]
    fn parse_1h() {
        assert!(parse_args(args(&["--timer-down", "1h"])).is_ok());
    }

    #[test]
    fn parse_zero_duration_rejected() {
        assert!(parse_args(args(&["--timer-down", "0s"])).is_err());
        assert!(parse_args(args(&["--timer-down", "0m"])).is_err());
    }

    #[test]
    fn parse_missing_unit_rejected() {
        assert!(parse_args(args(&["--timer-down", "30"])).is_err());
    }

    #[test]
    fn parse_empty_duration_rejected() {
        assert!(parse_args(args(&["--timer-down", ""])).is_err());
    }

    #[test]
    fn parse_invalid_number_rejected() {
        assert!(parse_args(args(&["--timer-down", "abcs"])).is_err());
        assert!(parse_args(args(&["--timer-down", "-5m"])).is_err());
    }

    // ── Missing arguments ─────────────────────────────────────────────────────

    #[test]
    fn missing_duration_timer_down() {
        assert!(parse_args(args(&["--timer-down"])).is_err());
    }

    #[test]
    fn pomodoro_missing_break() {
        assert!(parse_args(args(&["--pomodoro", "25m"])).is_err());
    }

    // ── Extra args rejected ───────────────────────────────────────────────────

    #[test]
    fn extra_args_help() {
        assert!(parse_args(args(&["--help", "extra"])).is_err());
        assert!(parse_args(args(&["-h", "extra"])).is_err());
    }

    #[test]
    fn extra_args_stopwatch() {
        assert!(parse_args(args(&["--stopwatch", "extra"])).is_err());
    }

    #[test]
    fn extra_args_timer_up() {
        assert!(parse_args(args(&["--timer-up", "extra"])).is_err());
        assert!(parse_args(args(&["--timer-upward-minute", "extra"])).is_err());
    }

    #[test]
    fn extra_args_timer_down() {
        assert!(parse_args(args(&["--timer-down", "10m", "extra"])).is_err());
    }

    #[test]
    fn extra_args_sound_test() {
        assert!(parse_args(args(&["--sound-test", "extra"])).is_err());
    }

    #[test]
    fn extra_args_pomodoro() {
        assert!(parse_args(args(&["--pomodoro", "25m", "5m", "extra"])).is_err());
    }

    // ── Valid cases ───────────────────────────────────────────────────────────

    #[test]
    fn no_args_shows_help() {
        assert!(matches!(parse_args(args(&[])), Ok(Command::Help)));
    }

    #[test]
    fn pomodoro_no_args_ok() {
        assert!(parse_args(args(&["--pomodoro"])).is_ok());
    }

    #[test]
    fn pomodoro_custom_durations_ok() {
        assert!(parse_args(args(&["--pomodoro", "3s", "2s"])).is_ok());
    }

    #[test]
    fn timer_back_alias_ok() {
        assert!(parse_args(args(&["--timer-back", "5m"])).is_ok());
    }

    #[test]
    fn timer_upward_minute_alias_ok() {
        assert!(parse_args(args(&["--timer-upward-minute"])).is_ok());
    }

    // ── Theme and view defaults ──────────────────────────────────────────────

    #[test]
    fn theme_default_is_void() {
        let cmd = parse_args(args(&["--timer-up"])).unwrap();
        if let Command::Run { theme, .. } = cmd {
            assert_eq!(theme, Theme::Void);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn view_default_is_orbit() {
        let cmd = parse_args(args(&["--timer-up"])).unwrap();
        if let Command::Run { view, .. } = cmd {
            assert_eq!(view, ViewMode::Orbit);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn theme_flag_before_mode() {
        let cmd = parse_args(args(&["--theme", "ember", "--timer-up"])).unwrap();
        if let Command::Run { theme, .. } = cmd {
            assert_eq!(theme, Theme::Ember);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn theme_flag_after_mode() {
        let cmd = parse_args(args(&["--timer-up", "--theme", "aura"])).unwrap();
        if let Command::Run { theme, .. } = cmd {
            assert_eq!(theme, Theme::Aura);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn view_flag_parsed() {
        let cmd = parse_args(args(&["--view", "cinematic", "--timer-up"])).unwrap();
        if let Command::Run { view, .. } = cmd {
            assert_eq!(view, ViewMode::Cinematic);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn theme_and_view_combined() {
        let cmd = parse_args(args(&[
            "--theme",
            "forest",
            "--view",
            "minimal",
            "--timer-down",
            "5m",
        ]))
        .unwrap();
        if let Command::Run { theme, view, .. } = cmd {
            assert_eq!(theme, Theme::Forest);
            assert_eq!(view, ViewMode::Minimal);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn unknown_theme_rejected() {
        assert!(parse_args(args(&["--theme", "neon", "--timer-up"])).is_err());
    }

    #[test]
    fn unknown_view_rejected() {
        assert!(parse_args(args(&["--view", "holographic", "--timer-up"])).is_err());
    }

    #[test]
    fn theme_missing_value_rejected() {
        assert!(parse_args(args(&["--theme"])).is_err());
    }

    #[test]
    fn view_missing_value_rejected() {
        assert!(parse_args(args(&["--view"])).is_err());
    }

    #[test]
    fn theme_view_preserve_timer_down() {
        let cmd = parse_args(args(&[
            "--theme",
            "mono",
            "--view",
            "minimal",
            "--timer-down",
            "30s",
        ]))
        .unwrap();
        if let Command::Run { mode, theme, view } = cmd {
            assert!(matches!(mode, Mode::TimerDown { .. }));
            assert_eq!(theme, Theme::Mono);
            assert_eq!(view, ViewMode::Minimal);
        } else {
            panic!("expected Run");
        }
    }

    // ── Pomodoro Masterclass CLI ──────────────────────────────────────────────

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
        if let Command::Run { mode, theme, view } = cmd {
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

    // ── Positional / option conflict ────────────────────────────────────────

    /// Helper: unwrap the Err side of a parse result (avoids needing Debug on Command).
    fn expect_err(cli_args: &[&str]) -> String {
        match parse_args(args(cli_args)) {
            Ok(_) => panic!("expected Err, got Ok"),
            Err(e) => e,
        }
    }

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
}
