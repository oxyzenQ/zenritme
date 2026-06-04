// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

mod duration;
mod pomodoro;

use crate::mode::Mode;
use crate::render::ViewMode;
use crate::theme::Theme;

pub enum Command {
    Help,
    Version,
    CheckUpdate,
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
         \x20 zenritme --check-update\n\
         \x20 zenritme --check-updated\n\
         \x20 zenritme --help\n\
         \x20 zenritme -V, --version\n\n\
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
            _ => {
                if !pomodoro::extract_flag(&all, &mut i, &mut pomodoro_opts)? {
                    mode_args.push(all[i].clone());
                    i += 1;
                }
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

        "-V" | "--version" => {
            reject_extra(&mut args, "--version")?;
            Ok(Command::Version)
        }

        "--check-update" | "--check-updated" => {
            reject_extra(&mut args, "--check-update")?;
            Ok(Command::CheckUpdate)
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

        "--pomodoro" => pomodoro::resolve_mode(args, pomo, theme, view),

        "--timer-down" | "--timer-back" => {
            let Some(dur_str) = args.next() else {
                return Err("missing DURATION after --timer-down".to_string());
            };
            let dur = duration::parse_duration(&dur_str)?;
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
    fn extra_args_version() {
        assert!(parse_args(args(&["--version", "extra"])).is_err());
        assert!(parse_args(args(&["-V", "extra"])).is_err());
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
    fn extra_args_check_update() {
        assert!(parse_args(args(&["--check-update", "extra"])).is_err());
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
    fn version_flag_long() {
        assert!(matches!(
            parse_args(args(&["--version"])),
            Ok(Command::Version)
        ));
    }

    #[test]
    fn version_flag_short() {
        assert!(matches!(parse_args(args(&["-V"])), Ok(Command::Version)));
    }

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
        if let Command::Run {
            mode, theme, view, ..
        } = cmd
        {
            assert!(matches!(mode, Mode::TimerDown { .. }));
            assert_eq!(theme, Theme::Mono);
            assert_eq!(view, ViewMode::Minimal);
        } else {
            panic!("expected Run");
        }
    }

    // ── Unknown argument ─────────────────────────────────────────────────────

    #[test]
    fn unknown_argument_rejected() {
        assert!(parse_args(args(&["--nonexistent"])).is_err());
    }

    // ── check-update ────────────────────────────────────────────────────────

    #[test]
    fn check_update_parsed() {
        assert!(matches!(
            parse_args(args(&["--check-update"])),
            Ok(Command::CheckUpdate)
        ));
    }

    #[test]
    fn check_updated_alias_parsed() {
        assert!(matches!(
            parse_args(args(&["--check-updated"])),
            Ok(Command::CheckUpdate)
        ));
    }
}
