// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::mode::{Mode, PomodoroPhase};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Command {
    Help,
    Run(Mode),
    SoundTest,
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
         Duration format:\n\
         \x20 <number>s | <number>m | <number>h\n\
         Examples:\n\
         \x20 30s\n\
         \x20 10m\n\
         \x20 1h\n\n\
         Pomodoro examples:\n\
         \x20 zenritme --pomodoro\n\
         \x20 zenritme --pomodoro 3s 2s\n\n\
         Controls while running:\n\
         \x20 q / Esc   quit\n\
         \x20 p         pause / resume\n\
         \x20 r         reset current session",
        ver = env!("CARGO_PKG_VERSION")
    )
}

pub fn parse_args<I>(mut args: I) -> Result<Command, String>
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
            Ok(Command::Run(Mode::TimerUp))
        }

        "--stopwatch" => {
            reject_extra(&mut args, "--stopwatch")?;
            Ok(Command::Run(Mode::Stopwatch))
        }

        "--pomodoro" => {
            let default_focus = std::time::Duration::from_secs(25 * 60);
            let default_break = std::time::Duration::from_secs(5 * 60);

            let (focus, short_break) = match args.next() {
                None => (default_focus, default_break),
                Some(focus_str) => {
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

            Ok(Command::Run(Mode::Pomodoro {
                phase: PomodoroPhase::Focus,
                focus,
                short_break,
                emoji: pick_pomodoro_emoji(),
            }))
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
            Ok(Command::Run(Mode::TimerDown { total: dur }))
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
}
