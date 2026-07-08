// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

/// Supported visual themes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Void,
    Ember,
    Aura,
    Forest,
    Tron,
    TronGreen,
    TronCyan,
    TronOrange,
    TronRed,
    TronYellow,
    Mono,
}

impl Theme {
    /// Parse a theme name (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "void" => Some(Self::Void),
            "ember" => Some(Self::Ember),
            "aura" => Some(Self::Aura),
            "forest" => Some(Self::Forest),
            "tron" => Some(Self::Tron),
            "tron-green" => Some(Self::TronGreen),
            "tron-cyan" => Some(Self::TronCyan),
            "tron-orange" => Some(Self::TronOrange),
            "tron-red" => Some(Self::TronRed),
            "tron-yellow" => Some(Self::TronYellow),
            "mono" => Some(Self::Mono),
            _ => None,
        }
    }

    /// Resolve to the theme's raw palette (ignores NO_COLOR).
    pub fn palette(&self) -> ColorFields {
        match self {
            Self::Void => ColorFields::void(),
            Self::Ember => ColorFields::ember(),
            Self::Aura => ColorFields::aura(),
            Self::Forest => ColorFields::forest(),
            Self::Tron => ColorFields::tron(),
            Self::TronGreen => ColorFields::tron_green(),
            Self::TronCyan => ColorFields::tron_cyan(),
            Self::TronOrange => ColorFields::tron_orange(),
            Self::TronRed => ColorFields::tron_red(),
            Self::TronYellow => ColorFields::tron_yellow(),
            Self::Mono => ColorFields::plain(),
        }
    }

    /// Resolve to a concrete color palette, respecting NO_COLOR.
    ///
    /// `NO_COLOR` is checked once per call (the env var is read-only at
    /// runtime in practice). The caller in `main()` stores the result,
    /// so this runs at most once per session.
    pub fn colors(&self) -> ColorFields {
        if no_color_active() {
            return ColorFields::plain();
        }
        self.palette()
    }
}

/// Check the [NO_COLOR](https://no-color.org/) environment variable.
/// Returns `true` if the variable is set (to any value, including empty).
fn no_color_active() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// ANSI color codes for each UI element.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ColorFields {
    pub title: &'static str,
    pub time: &'static str,
    pub progress_fill: &'static str,
    pub progress_empty: &'static str,
    pub label: &'static str,
    pub dim: &'static str,
    pub border: &'static str,
    pub accent: &'static str,
    pub spinner: &'static str,
    pub reset: &'static str,
}

pub const RESET: &str = "\x1b[0m";

impl ColorFields {
    pub(crate) fn plain() -> Self {
        Self {
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

    fn void() -> Self {
        Self {
            title: "\x1b[1;38;5;7m",
            time: "\x1b[1;38;5;15m",
            progress_fill: "\x1b[38;5;7m",
            progress_empty: "\x1b[38;5;240m",
            label: "\x1b[38;5;240m",
            dim: "\x1b[38;5;240m",
            border: "\x1b[38;5;240m",
            accent: "\x1b[38;5;15m",
            spinner: "\x1b[38;5;6m",
            reset: RESET,
        }
    }

    fn ember() -> Self {
        Self {
            title: "\x1b[1;38;5;208m",
            time: "\x1b[1;38;5;220m",
            progress_fill: "\x1b[38;5;208m",
            progress_empty: "\x1b[38;5;238m",
            label: "\x1b[38;5;238m",
            dim: "\x1b[38;5;238m",
            border: "\x1b[38;5;238m",
            accent: "\x1b[38;5;202m",
            spinner: "\x1b[38;5;214m",
            reset: RESET,
        }
    }

    fn aura() -> Self {
        Self {
            title: "\x1b[1;38;5;111m",
            time: "\x1b[1;38;5;159m",
            progress_fill: "\x1b[38;5;111m",
            progress_empty: "\x1b[38;5;238m",
            label: "\x1b[38;5;238m",
            dim: "\x1b[38;5;238m",
            border: "\x1b[38;5;238m",
            accent: "\x1b[38;5;147m",
            spinner: "\x1b[38;5;159m",
            reset: RESET,
        }
    }

    fn forest() -> Self {
        Self {
            title: "\x1b[1;38;5;71m",
            time: "\x1b[1;38;5;114m",
            progress_fill: "\x1b[38;5;71m",
            progress_empty: "\x1b[38;5;238m",
            label: "\x1b[38;5;238m",
            dim: "\x1b[38;5;238m",
            border: "\x1b[38;5;238m",
            accent: "\x1b[38;5;107m",
            spinner: "\x1b[38;5;114m",
            reset: RESET,
        }
    }

    fn tron() -> Self {
        Self {
            title: "\x1b[1;38;5;171m",
            time: "\x1b[1;38;5;213m",
            progress_fill: "\x1b[38;5;165m",
            progress_empty: "\x1b[38;5;53m",
            label: "\x1b[38;5;93m",
            dim: "\x1b[38;5;53m",
            border: "\x1b[38;5;93m",
            accent: "\x1b[38;5;219m",
            spinner: "\x1b[38;5;213m",
            reset: RESET,
        }
    }

    fn tron_green() -> Self {
        Self {
            title: "\x1b[1;38;5;82m",
            time: "\x1b[1;38;5;119m",
            progress_fill: "\x1b[38;5;46m",
            progress_empty: "\x1b[38;5;22m",
            label: "\x1b[38;5;28m",
            dim: "\x1b[38;5;22m",
            border: "\x1b[38;5;28m",
            accent: "\x1b[38;5;155m",
            spinner: "\x1b[38;5;119m",
            reset: RESET,
        }
    }

    fn tron_cyan() -> Self {
        Self {
            title: "\x1b[1;38;5;81m",
            time: "\x1b[1;38;5;123m",
            progress_fill: "\x1b[38;5;51m",
            progress_empty: "\x1b[38;5;17m",
            label: "\x1b[38;5;25m",
            dim: "\x1b[38;5;17m",
            border: "\x1b[38;5;25m",
            accent: "\x1b[38;5;159m",
            spinner: "\x1b[38;5;123m",
            reset: RESET,
        }
    }

    fn tron_orange() -> Self {
        Self {
            title: "\x1b[1;38;5;214m",
            time: "\x1b[1;38;5;222m",
            progress_fill: "\x1b[38;5;208m",
            progress_empty: "\x1b[38;5;52m",
            label: "\x1b[38;5;88m",
            dim: "\x1b[38;5;52m",
            border: "\x1b[38;5;88m",
            accent: "\x1b[38;5;220m",
            spinner: "\x1b[38;5;222m",
            reset: RESET,
        }
    }

    fn tron_red() -> Self {
        Self {
            title: "\x1b[1;38;5;197m",
            time: "\x1b[1;38;5;211m",
            progress_fill: "\x1b[38;5;196m",
            progress_empty: "\x1b[38;5;52m",
            label: "\x1b[38;5;88m",
            dim: "\x1b[38;5;52m",
            border: "\x1b[38;5;88m",
            accent: "\x1b[38;5;203m",
            spinner: "\x1b[38;5;211m",
            reset: RESET,
        }
    }

    fn tron_yellow() -> Self {
        Self {
            title: "\x1b[1;38;5;226m",
            time: "\x1b[1;38;5;228m",
            progress_fill: "\x1b[38;5;220m",
            progress_empty: "\x1b[38;5;58m",
            label: "\x1b[38;5;94m",
            dim: "\x1b[38;5;58m",
            border: "\x1b[38;5;94m",
            accent: "\x1b[38;5;229m",
            spinner: "\x1b[38;5;228m",
            reset: RESET,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_themes() {
        assert_eq!(Theme::from_name("void"), Some(Theme::Void));
        assert_eq!(Theme::from_name("EMBER"), Some(Theme::Ember));
        assert_eq!(Theme::from_name("Aura"), Some(Theme::Aura));
        assert_eq!(Theme::from_name("forest"), Some(Theme::Forest));
        assert_eq!(Theme::from_name("tron"), Some(Theme::Tron));
        assert_eq!(Theme::from_name("TRON-GREEN"), Some(Theme::TronGreen));
        assert_eq!(Theme::from_name("tron-cyan"), Some(Theme::TronCyan));
        assert_eq!(Theme::from_name("tron-orange"), Some(Theme::TronOrange));
        assert_eq!(Theme::from_name("tron-red"), Some(Theme::TronRed));
        assert_eq!(Theme::from_name("tron-yellow"), Some(Theme::TronYellow));
        assert_eq!(Theme::from_name("mono"), Some(Theme::Mono));
    }

    #[test]
    fn parse_unknown_theme() {
        assert_eq!(Theme::from_name("neon"), None);
        assert_eq!(Theme::from_name(""), None);
    }

    #[test]
    fn tron_theme_has_codes() {
        let c = Theme::Tron.palette();
        assert!(!c.title.is_empty());
        assert!(!c.time.is_empty());
        assert!(!c.progress_fill.is_empty());
        assert!(!c.accent.is_empty());
        assert!(!c.reset.is_empty());
    }

    #[test]
    fn void_theme_has_codes() {
        let c = Theme::Void.palette();
        assert!(!c.title.is_empty());
        assert!(!c.time.is_empty());
        assert!(!c.reset.is_empty());
    }

    #[test]
    fn mono_theme_is_plain() {
        let c = Theme::Mono.colors();
        assert!(c.title.is_empty());
        assert!(c.reset.is_empty());
    }

    #[test]
    fn plain_has_empty_reset() {
        let c = ColorFields::plain();
        assert!(c.reset.is_empty());
    }

    #[test]
    fn all_themes_produce_non_empty_border_when_colored() {
        for theme in [
            Theme::Void,
            Theme::Ember,
            Theme::Aura,
            Theme::Forest,
            Theme::Tron,
            Theme::TronGreen,
            Theme::TronCyan,
            Theme::TronOrange,
            Theme::TronRed,
            Theme::TronYellow,
        ] {
            let c = theme.palette();
            assert!(
                !c.border.is_empty(),
                "{:?} border should not be empty",
                theme
            );
            assert!(!c.reset.is_empty(), "{:?} reset should not be empty", theme);
        }
    }
}
