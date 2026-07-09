// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

mod duration;
mod pomodoro;
mod suggest;

use crate::mode::Mode;
use crate::render::ViewMode;
use crate::theme::Theme;
use std::sync::OnceLock;

#[derive(Debug)]
pub enum Command {
    Help,
    Version,
    CheckUpdate,
    ListThemes,
    ListViews,
    Run {
        mode: Mode,
        theme: Theme,
        view: ViewMode,
        mute: bool,
        profile: crate::sound::SoundProfile,
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

pub fn usage() -> &'static str {
    static USAGE: OnceLock<String> = OnceLock::new();
    USAGE.get_or_init(build_usage).as_str()
}

fn build_usage() -> String {
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
         \x20 zenritme --list-themes\n\
         \x20 zenritme --list-views\n\
         \x20 zenritme --help\n\
         \x20 zenritme -V, --version\n\n\
         Options:\n\
         \x20 --theme <THEME>          void | ember | aura | forest | tron | tron-green | tron-cyan | tron-orange | tron-red | tron-yellow | mono  (default: void)\n\
         \x20 --view <VIEW>            minimal | orbit | cinematic | tron       (default: orbit)\n\
         \x20 --sound-profile <P>    calm | silent                          (default: calm)\n\
         \x20 --mute                   suppress all notification sounds       (default: off)\n\
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
         \x20 r         reset current session\n\n\
         Sound environment variables:\n\
         \x20 ZENRITME_SOUND_START     override start sound file\n\
         \x20 ZENRITME_SOUND_PAUSE     override pause sound file\n\
         \x20 ZENRITME_SOUND_PHASE     override phase sound file\n\
         \x20 ZENRITME_SOUND_COMPLETE  override complete sound file\n\
         \x20 ZENRITME_SOUND_FILE      global fallback for all events\n\
         \x20 ZENRITME_VISUAL_BELL=1   enable visual bell (screen flash)\n\n\
         Sound file paths are validated against a strict whitelist policy\n\
         \x20 (only ~/.config/zenritme/, ., /etc/zenritme/, system temp).\n\
         \x20 Everything else is denied. See --help or docs/SECURITY.md.",
        ver = env!("CARGO_PKG_VERSION")
    )
}

/// Render the `--list-themes` output as a formatted table.
pub fn list_themes() -> String {
    let rows: &[(&str, &str)] = &[
        ("void", "Minimal dark"),
        ("ember", "Warm red/orange"),
        ("aura", "Purple/magenta"),
        ("forest", "Green tones"),
        ("tron", "Classic Tron blue/purple"),
        ("tron-green", "Tron Legacy green circuit"),
        ("tron-cyan", "Tron Legacy cyan glow"),
        ("tron-orange", "Tron Legacy orange flare"),
        ("tron-red", "Tron Legacy red alert"),
        ("tron-yellow", "Tron Legacy gold accent"),
        ("mono", "Monochrome/gray"),
    ];
    format_table("Available themes (--theme <THEME>):", rows)
}

/// Render the `--list-views` output as a formatted table.
pub fn list_views() -> String {
    let rows: &[(&str, &str)] = &[
        ("minimal", "Compact single-line display"),
        ("orbit", "Circular progress indicator (default)"),
        ("cinematic", "Full-width centered box layout"),
    ];
    format_table("Available views (--view <VIEW>):", rows)
}

/// Format a `(name, description)` table with aligned columns.
fn format_table(header: &str, rows: &[(&str, &str)]) -> String {
    let max_name = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");
    for (name, desc) in rows {
        out.push_str(&format!("  {:<width$}  {}\n", name, desc, width = max_name));
    }
    out
}

/// Build a "did you mean ...?" hint for an unknown theme/view value.
fn hint_unknown(label: &str, input: &str, candidates: &[&str]) -> String {
    match suggest::closest(input, candidates) {
        Some(better) => format!(
            "unknown {}: {}  (did you mean '{}'?  see --help)",
            label, input, better
        ),
        None => format!("unknown {}: {}  (see --help)", label, input),
    }
}

/// All valid theme names, for did-you-mean suggestions.
const THEME_NAMES: &[&str] = &[
    "void",
    "ember",
    "aura",
    "forest",
    "tron",
    "tron-green",
    "tron-cyan",
    "tron-orange",
    "tron-red",
    "tron-yellow",
    "mono",
];

/// All valid view names, for did-you-mean suggestions.
const VIEW_NAMES: &[&str] = &["minimal", "orbit", "cinematic"];

/// Parse all arguments. `--theme`, `--view`, and pomodoro-specific flags are
/// extracted in a pre-pass so they may appear before or after the mode flag.
pub fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let all: Vec<String> = args.collect();

    let mut theme = Theme::Void;
    let mut view = ViewMode::Orbit;
    let mut mute = false;
    let mut profile = crate::sound::SoundProfile::Calm;
    let mut pomodoro_opts = PomodoroOpts::default();
    let mut mode_args: Vec<String> = Vec::new();
    let mut i = 0;

    while i < all.len() {
        match all[i].as_str() {
            "--theme" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --theme".to_string())?;
                theme =
                    Theme::from_name(val).ok_or_else(|| hint_unknown("theme", val, THEME_NAMES))?;
                i += 2;
            }
            "--view" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --view".to_string())?;
                view = ViewMode::from_name(val)
                    .ok_or_else(|| hint_unknown("view", val, VIEW_NAMES))?;
                i += 2;
            }
            "--mute" => {
                mute = true;
                i += 1;
            }
            "--sound-profile" => {
                let val = all
                    .get(i + 1)
                    .ok_or("missing value after --sound-profile".to_string())?;
                profile = crate::sound::SoundProfile::from_name(val)
                    .ok_or_else(|| format!("unknown sound profile: {}  (see --help)", val))?;
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

    parse_mode(
        mode_args.into_iter(),
        theme,
        view,
        mute,
        profile,
        pomodoro_opts,
    )
}

/// Parse the mode-specific arguments (after pre-pass extraction).
fn parse_mode<I>(
    mut args: I,
    theme: Theme,
    view: ViewMode,
    mute: bool,
    profile: crate::sound::SoundProfile,
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

        "--list-themes" => {
            reject_extra(&mut args, "--list-themes")?;
            Ok(Command::ListThemes)
        }

        "--list-views" => {
            reject_extra(&mut args, "--list-views")?;
            Ok(Command::ListViews)
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
                mute,
                profile,
            })
        }

        "--stopwatch" => {
            reject_extra(&mut args, "--stopwatch")?;
            Ok(Command::Run {
                mode: Mode::Stopwatch,
                theme,
                view,
                mute,
                profile,
            })
        }

        "--pomodoro" => pomodoro::resolve_mode(args, pomo, theme, view, mute, profile),

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
                mute,
                profile,
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

// ─── Shared test helper ────────────────────────────────────────────────────

/// Build a `parse_args`-compatible iterator from a slice of string literals.
/// Shared across CLI submodules to avoid duplication.
#[cfg(test)]
pub(crate) fn args(v: &[&str]) -> impl Iterator<Item = String> {
    v.iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .into_iter()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn tron_view_parsed() {
        let cmd = parse_args(args(&["--view", "tron", "--timer-up"])).unwrap();
        if let Command::Run { view, .. } = cmd {
            assert_eq!(view, ViewMode::Tron);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn tron_view_case_insensitive() {
        let cmd = parse_args(args(&["--view", "TRON", "--timer-up"])).unwrap();
        if let Command::Run { view, .. } = cmd {
            assert_eq!(view, ViewMode::Tron);
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

    // ── --mute flag ──────────────────────────────────────────────────────────

    #[test]
    fn mute_flag_parsed() {
        let cmd = parse_args(args(&["--mute", "--timer-up"])).unwrap();
        if let Command::Run { mute, .. } = cmd {
            assert!(mute);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn mute_default_is_false() {
        let cmd = parse_args(args(&["--timer-up"])).unwrap();
        if let Command::Run { mute, .. } = cmd {
            assert!(!mute);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn mute_flag_after_mode() {
        let cmd = parse_args(args(&["--timer-up", "--mute"])).unwrap();
        if let Command::Run { mute, .. } = cmd {
            assert!(mute);
        } else {
            panic!("expected Run");
        }
    }

    // ── --sound-profile flag ─────────────────────────────────────────────────

    #[test]
    fn sound_profile_silent_parsed() {
        let cmd = parse_args(args(&["--sound-profile", "silent", "--timer-up"])).unwrap();
        if let Command::Run { profile, .. } = cmd {
            assert_eq!(profile, crate::sound::SoundProfile::Silent);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn sound_profile_calm_parsed() {
        let cmd = parse_args(args(&["--sound-profile", "calm", "--timer-up"])).unwrap();
        if let Command::Run { profile, .. } = cmd {
            assert_eq!(profile, crate::sound::SoundProfile::Calm);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn sound_profile_default_is_calm() {
        let cmd = parse_args(args(&["--timer-up"])).unwrap();
        if let Command::Run { profile, .. } = cmd {
            assert_eq!(profile, crate::sound::SoundProfile::Calm);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn sound_profile_unknown_rejected() {
        assert!(parse_args(args(&["--sound-profile", "loud", "--timer-up"])).is_err());
    }

    #[test]
    fn sound_profile_missing_value_rejected() {
        assert!(parse_args(args(&["--sound-profile"])).is_err());
    }

    #[test]
    fn sound_profile_case_insensitive() {
        let cmd = parse_args(args(&["--sound-profile", "Silent", "--timer-up"])).unwrap();
        if let Command::Run { profile, .. } = cmd {
            assert_eq!(profile, crate::sound::SoundProfile::Silent);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn mute_overrides_silent_profile() {
        // --mute + --sound-profile silent → both should be set; runtime resolves.
        let cmd = parse_args(args(&["--mute", "--sound-profile", "silent", "--timer-up"])).unwrap();
        if let Command::Run { mute, profile, .. } = cmd {
            assert!(mute, "--mute should be true");
            assert_eq!(profile, crate::sound::SoundProfile::Silent);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn mute_overrides_calm_profile() {
        let cmd = parse_args(args(&["--mute", "--sound-profile", "calm", "--timer-up"])).unwrap();
        if let Command::Run { mute, profile, .. } = cmd {
            assert!(mute, "--mute should be true");
            assert_eq!(profile, crate::sound::SoundProfile::Calm);
        } else {
            panic!("expected Run");
        }
    }

    #[test]
    fn sound_profile_with_pomodoro() {
        let cmd = parse_args(args(&["--sound-profile", "silent", "--pomodoro"])).unwrap();
        if let Command::Run { profile, .. } = cmd {
            assert_eq!(profile, crate::sound::SoundProfile::Silent);
        } else {
            panic!("expected Run");
        }
    }

    // ── --list-themes / --list-views ────────────────────────────────────────

    #[test]
    fn list_themes_parsed() {
        assert!(matches!(
            parse_args(args(&["--list-themes"])),
            Ok(Command::ListThemes)
        ));
    }

    #[test]
    fn list_views_parsed() {
        assert!(matches!(
            parse_args(args(&["--list-views"])),
            Ok(Command::ListViews)
        ));
    }

    #[test]
    fn list_themes_rejects_extra_args() {
        assert!(parse_args(args(&["--list-themes", "extra"])).is_err());
    }

    #[test]
    fn list_views_rejects_extra_args() {
        assert!(parse_args(args(&["--list-views", "extra"])).is_err());
    }

    #[test]
    fn list_themes_output_contains_all_themes() {
        let out = list_themes();
        for name in THEME_NAMES {
            assert!(out.contains(name), "list_themes missing: {}", name);
        }
    }

    #[test]
    fn list_views_output_contains_all_views() {
        let out = list_views();
        for name in VIEW_NAMES {
            assert!(out.contains(name), "list_views missing: {}", name);
        }
    }

    #[test]
    fn list_themes_output_has_header() {
        let out = list_themes();
        assert!(out.contains("Available themes"), "missing header: {}", out);
    }

    #[test]
    fn list_views_output_has_header() {
        let out = list_views();
        assert!(out.contains("Available views"), "missing header: {}", out);
    }

    // ── Did-you-mean suggestions ────────────────────────────────────────────

    #[test]
    fn unknown_theme_suggests_close_match() {
        let err = parse_args(args(&["--theme", "embar", "--timer-up"])).unwrap_err();
        assert!(
            err.contains("did you mean 'ember'"),
            "expected did-you-mean hint, got: {}",
            err
        );
    }

    #[test]
    fn unknown_view_suggests_close_match() {
        let err = parse_args(args(&["--view", "orbir", "--timer-up"])).unwrap_err();
        assert!(
            err.contains("did you mean 'orbit'"),
            "expected did-you-mean hint, got: {}",
            err
        );
    }

    #[test]
    fn unknown_theme_no_suggestion_when_too_far() {
        let err = parse_args(args(&["--theme", "zzzzzzz", "--timer-up"])).unwrap_err();
        assert!(
            !err.contains("did you mean"),
            "should not suggest for very different input, got: {}",
            err
        );
        assert!(err.contains("unknown theme"), "should still report unknown");
    }

    // ── usage() memoization ─────────────────────────────────────────────────

    #[test]
    fn usage_returns_static_str() {
        let s1: &'static str = usage();
        let s2: &'static str = usage();
        // Same pointer — memoized via OnceLock.
        assert!(std::ptr::eq(s1, s2), "usage() should be memoized");
    }

    #[test]
    fn usage_lists_list_themes_and_list_views() {
        let s = usage();
        assert!(
            s.contains("--list-themes"),
            "usage should mention --list-themes"
        );
        assert!(
            s.contains("--list-views"),
            "usage should mention --list-views"
        );
    }

    #[test]
    fn usage_mentions_security_policy() {
        let s = usage();
        assert!(
            s.contains("whitelist policy") || s.contains("whitelist"),
            "usage should mention whitelist policy"
        );
    }
}
