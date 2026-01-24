use crate::mode::{Mode, PomodoroPhase};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Command {
    Help,
    Run(Mode),
    SoundTest,
}

pub fn usage() -> String {
    format!(
        "zenritme v{}\n\nUsage:\n  zenritme --timer-up\n  zenritme --timer-down <DURATION>\n  zenritme --stopwatch\n  zenritme --pomodoro [FOCUS] [BREAK]\n  zenritme --sound-test\n  zenritme --help\n\nDuration format:\n  <number>s | <number>m | <number>h\nExamples:\n  30s\n  10m\n  1h\n\nPomodoro examples:\n  zenritme --pomodoro\n  zenritme --pomodoro 3s 2s",
        env!("CARGO_PKG_VERSION")
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
        "--help" | "-h" => Ok(Command::Help),
        "--sound-test" => Ok(Command::SoundTest),
        "--timer-up" | "--timer-upward-minute" => Ok(Command::Run(Mode::TimerUp)),
        "--stopwatch" => Ok(Command::Run(Mode::Stopwatch)),
        "--pomodoro" => {
            let default_focus = std::time::Duration::from_secs(25 * 60);
            let default_break = std::time::Duration::from_secs(5 * 60);

            let (focus, short_break) = match args.next() {
                None => (default_focus, default_break),
                Some(focus_str) => {
                    let Some(break_str) = args.next() else {
                        return Err("missing BREAK after --pomodoro FOCUS".to_string());
                    };
                    if args.next().is_some() {
                        return Err("too many arguments for --pomodoro".to_string());
                    }
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
            Ok(Command::Run(Mode::TimerDown { total: dur }))
        }
        other => Err(format!("unknown argument: {}", other)),
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
        'h' => value.saturating_mul(60 * 60),
        _ => return Err("duration unit is invalid".to_string()),
    };

    Ok(std::time::Duration::from_secs(secs))
}

fn pick_pomodoro_emoji() -> u8 {
    let emojis_len = 10u8;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as u8) % emojis_len
}
