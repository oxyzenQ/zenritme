// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Path security guard — strict whitelist for user-provided paths.
//!
//! Zenritme must NEVER read sensitive system files (`/etc/shadow`, `~/.ssh/`,
//! `/proc/`, etc.).  Rather than maintain a blacklist of dangerous paths
//! (whack-a-mole, grows forever, easy to miss new vectors), we use a
//! **whitelist-only** approach: if a path is not inside one of the explicit
//! allowed roots, it is denied.  No exceptions, no denylist to maintain.
//!
//! # Allowed roots (cross-platform)
//!
//! | Platform | Allowed roots |
//! |----------|---------------|
//! | Linux / macOS | `~/.config/zenritme/`, `.`, `/etc/zenritme/`, system temp dir |
//! | Windows | `%APPDATA%\zenritme\`, `.`, `%TEMP%\` |
//!
//! Everything else is denied: `~/.ssh/`, `~/.aws/`, `/etc/shadow`,
//! `~/Documents/`, `/usr/share/sounds/`, `/home/user/file.txt` — all rejected
//! by default because they are not in the whitelist.
//!
//! # Symlink safety
//!
//! Paths are canonicalized (symlinks resolved) before the whitelist check, so
//! a symlink inside an allowed root that points outside is rejected.  For
//! non-existent paths (e.g. a sound file the user will create later), the
//! **parent directory** is canonicalized and the check is applied to it.

use std::path::{Path, PathBuf};

// ─── Error type ────────────────────────────────────────────────────────────

/// Reasons a user-provided path may be rejected by the guard.
#[derive(Debug)]
pub enum PathError {
    /// The input was empty or whitespace-only.
    Empty,
    /// The path is outside all whitelisted roots.
    OutsideWhitelist(String),
    /// The path or its parent could not be resolved on the filesystem.
    ResolutionFailed(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::Empty => write!(f, "empty path"),
            PathError::OutsideWhitelist(p) => {
                write!(f, "path outside whitelist: {}", p)
            }
            PathError::ResolutionFailed(msg) => {
                write!(f, "path resolution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for PathError {}

// ─── Path helpers ──────────────────────────────────────────────────────────

/// Returns the user's home directory from `$HOME` (Unix) or `%USERPROFILE%`
/// (Windows), if set and non-empty.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

/// Expand a leading `~` or `~/` to the user's home directory.
///
/// - `~`         → `$HOME`
/// - `~/foo/bar` → `$HOME/foo/bar`
/// - everything else is returned unchanged.
pub fn expand_tilde(input: &str) -> PathBuf {
    if input == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(input)
}

// ─── Whitelist ─────────────────────────────────────────────────────────────

/// Returns `true` if the canonical path falls inside any whitelisted root.
///
/// Whitelist (cross-platform):
/// - **Per-user config dir**: `~/.config/zenritme/` on Unix,
///   `%APPDATA%\zenritme\` on Windows.
/// - **System config dir** (Unix only): `/etc/zenritme/`.
/// - **Current working directory**: `.` (relative paths resolved here).
/// - **System temp dir**: `$TMPDIR` / `/tmp` on Unix, `%TEMP%` on Windows.
fn is_whitelisted(canonical: &Path) -> bool {
    for root in allowed_roots() {
        if canonical == root || canonical.starts_with(&root) {
            return true;
        }
    }
    false
}

/// Compute the list of allowed root directories for the current platform.
///
/// Roots that do not exist on this machine are still returned (the
/// `starts_with` check will simply never match a non-existent path), so the
/// whitelist is stable across environments.
fn allowed_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // 1. Per-user config dir.
    if let Some(cfg) = user_config_dir() {
        roots.push(cfg);
    }
    // 2. System-wide config dir (Unix only — Windows has no equivalent).
    if cfg!(unix) {
        roots.push(PathBuf::from("/etc/zenritme"));
    }
    // 3. Current working directory.
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    // 4. System temp dir.
    roots.push(std::env::temp_dir());

    roots
}

/// Returns the per-user config directory for zenritme, cross-platform.
///
/// - Unix: `$HOME/.config/zenritme` (respects `XDG_CONFIG_HOME`)
/// - Windows: `%APPDATA%\zenritme`
pub fn user_config_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        // Respect XDG_CONFIG_HOME if set, otherwise fall back to ~/.config.
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".config")))?;
        Some(base.join("zenritme"))
    }
    #[cfg(windows)]
    {
        let base = std::env::var_os("APPDATA").map(PathBuf::from)?;
        Some(base.join("zenritme"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

// ─── Public entry point ────────────────────────────────────────────────────

/// Validate a user-provided filesystem path against the whitelist policy.
///
/// Steps:
/// 1. Trim and reject empty input.
/// 2. Expand leading `~` to `$HOME`.
/// 3. Resolve relative paths against the current working directory.
/// 4. **String-based whitelist check** on the resolved path — rejects paths
///    outside all allowed roots without touching the filesystem.  This
///    catches `~/.ssh/`, `/etc/shadow`, `/usr/...`, etc. even when the file
///    does not exist.
/// 5. **Canonical check for symlink escape** — if the file (or its parent)
///    exists on disk, canonicalize it (resolves symlinks) and re-apply the
///    whitelist.  This catches symlinks inside an allowed root that point
///    outside.  Non-existent files skip this step and use the resolved path.
///
/// Returns the canonicalized path when possible, otherwise the resolved path.
pub fn validate_user_path(input: &str) -> Result<PathBuf, PathError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PathError::Empty);
    }

    let expanded = expand_tilde(trimmed);

    // Resolve relative paths against the current working directory.
    let resolved = if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&expanded))
            .map_err(|e| PathError::ResolutionFailed(format!("cwd: {}", e)))?
    };

    // Step 1: string-based whitelist check (no FS access).
    if !is_whitelisted(&resolved) {
        return Err(PathError::OutsideWhitelist(resolved.display().to_string()));
    }

    // Step 2: canonical check for symlink-escape detection.
    // If the file exists, canonicalize it directly.
    if let Ok(canonical) = std::fs::canonicalize(&resolved) {
        if !is_whitelisted(&canonical) {
            return Err(PathError::OutsideWhitelist(canonical.display().to_string()));
        }
        return Ok(canonical);
    }

    // Step 3: file may not exist yet — canonicalize parent and re-join.
    if let Some(parent) = resolved.parent() {
        if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
            if !is_whitelisted(&canonical_parent) {
                return Err(PathError::OutsideWhitelist(
                    canonical_parent.display().to_string(),
                ));
            }
            if let Some(filename) = resolved.file_name() {
                return Ok(canonical_parent.join(filename));
            }
        }
    }

    // Step 4: filesystem access failed but string check passed — accept the
    // resolved path.  This allows paths to files the user will create later.
    Ok(resolved)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── expand_tilde ──────────────────────────────────────────────────────

    #[test]
    fn expand_tilde_alone() {
        let expanded = expand_tilde("~");
        if let Some(home) = home_dir() {
            assert_eq!(expanded, home);
        } else {
            assert_eq!(expanded, PathBuf::from("~"));
        }
    }

    #[test]
    fn expand_tilde_with_subpath() {
        let expanded = expand_tilde("~/music/chime.wav");
        if let Some(home) = home_dir() {
            assert_eq!(expanded, home.join("music/chime.wav"));
        }
    }

    #[test]
    fn expand_tilde_no_tilde_prefix() {
        assert_eq!(expand_tilde("/etc/passwd"), PathBuf::from("/etc/passwd"));
        assert_eq!(expand_tilde("./local.wav"), PathBuf::from("./local.wav"));
        assert_eq!(expand_tilde("local.wav"), PathBuf::from("local.wav"));
    }

    #[test]
    fn expand_tilde_not_midpath() {
        // `~` is only expanded at the very start.
        assert_eq!(expand_tilde("/home/~weird"), PathBuf::from("/home/~weird"));
    }

    // ── is_whitelisted — explicit denials (whitelist-only) ────────────────

    #[test]
    fn denied_home_ssh() {
        if let Some(home) = home_dir() {
            assert!(!is_whitelisted(&home.join(".ssh")));
            assert!(!is_whitelisted(&home.join(".ssh/id_rsa")));
        }
    }

    #[test]
    fn denied_etc_shadow() {
        assert!(!is_whitelisted(Path::new("/etc/shadow")));
        assert!(!is_whitelisted(Path::new("/etc/gshadow")));
    }

    #[test]
    fn denied_proc_sys() {
        assert!(!is_whitelisted(Path::new("/proc")));
        assert!(!is_whitelisted(Path::new("/proc/self/environ")));
        assert!(!is_whitelisted(Path::new("/sys")));
    }

    #[test]
    fn denied_home_documents() {
        if let Some(home) = home_dir() {
            assert!(!is_whitelisted(&home.join("Documents/secret.txt")));
            assert!(!is_whitelisted(&home.join(".bashrc")));
            assert!(!is_whitelisted(&home.join(".aws/credentials")));
        }
    }

    #[test]
    fn denied_usr_var_tmp_outside_whitelist() {
        // /usr and /var are NOT whitelisted.
        assert!(!is_whitelisted(Path::new("/usr/share/sounds/test.wav")));
        assert!(!is_whitelisted(Path::new("/var/log/syslog")));
        // /tmp IS whitelisted on Unix (system temp dir).
        #[cfg(unix)]
        {
            // temp_dir() on Linux is usually /tmp.
            let tmp = std::env::temp_dir();
            assert!(is_whitelisted(&tmp));
            assert!(is_whitelisted(&tmp.join("custom.wav")));
        }
    }

    // ── is_whitelisted — explicit allowlist ───────────────────────────────

    #[test]
    fn allowed_user_config_dir() {
        if let Some(cfg) = user_config_dir() {
            assert!(is_whitelisted(&cfg));
            assert!(is_whitelisted(&cfg.join("sounds/start.wav")));
        }
    }

    #[test]
    fn allowed_etc_zenritme() {
        // /etc/zenritme is whitelisted on Unix (system-wide config).
        if cfg!(unix) {
            assert!(is_whitelisted(Path::new("/etc/zenritme")));
            assert!(is_whitelisted(Path::new("/etc/zenritme/sounds.wav")));
        }
    }

    #[test]
    fn allowed_cwd_recursive() {
        if let Ok(cwd) = std::env::current_dir() {
            assert!(is_whitelisted(&cwd));
            assert!(is_whitelisted(&cwd.join("local.wav")));
            assert!(is_whitelisted(&cwd.join("subdir/local.wav")));
        }
    }

    #[test]
    fn allowed_temp_dir_recursive() {
        let tmp = std::env::temp_dir();
        assert!(is_whitelisted(&tmp));
        assert!(is_whitelisted(&tmp.join("custom.wav")));
    }

    // ── allowed_roots ─────────────────────────────────────────────────────

    #[test]
    fn allowed_roots_non_empty() {
        let roots = allowed_roots();
        assert!(!roots.is_empty(), "whitelist must have at least one root");
    }

    #[test]
    fn allowed_roots_includes_cwd() {
        let cwd = std::env::current_dir().expect("cwd must be available");
        let roots = allowed_roots();
        assert!(
            roots.iter().any(|r| r == &cwd),
            "whitelist must include cwd"
        );
    }

    #[test]
    fn allowed_roots_includes_temp() {
        let tmp = std::env::temp_dir();
        let roots = allowed_roots();
        assert!(
            roots.iter().any(|r| r == &tmp),
            "whitelist must include system temp dir"
        );
    }

    // ── user_config_dir ───────────────────────────────────────────────────

    #[test]
    fn user_config_dir_ends_with_zenritme() {
        if let Some(cfg) = user_config_dir() {
            assert!(
                cfg.ends_with("zenritme"),
                "user config dir must end with 'zenritme': {}",
                cfg.display()
            );
        }
    }

    // ── validate_user_path — happy path ───────────────────────────────────

    #[test]
    fn validate_cwd_relative_path() {
        if let Ok(cwd) = std::env::current_dir() {
            let r = validate_user_path("./local-sound.wav");
            assert!(r.is_ok(), "expected Ok, got: {:?}", r);
            let p = r.unwrap();
            assert!(p.starts_with(&cwd));
            assert!(p.ends_with("local-sound.wav"));
        }
    }

    #[test]
    fn validate_temp_path() {
        let tmp = std::env::temp_dir();
        let input = tmp.join("zenritme-test.wav");
        let r = validate_user_path(input.to_string_lossy().as_ref());
        assert!(r.is_ok(), "expected Ok for temp path, got: {:?}", r);
    }

    // ── validate_user_path — rejections (whitelist-only) ──────────────────

    #[test]
    fn validate_rejects_empty() {
        assert!(matches!(validate_user_path(""), Err(PathError::Empty)));
        assert!(matches!(validate_user_path("   "), Err(PathError::Empty)));
    }

    #[test]
    fn validate_rejects_etc_shadow() {
        let r = validate_user_path("/etc/shadow");
        assert!(
            matches!(r, Err(PathError::OutsideWhitelist(_))),
            "expected OutsideWhitelist, got: {:?}",
            r
        );
    }

    #[test]
    fn validate_rejects_home_ssh() {
        let r = validate_user_path("~/.ssh/id_rsa");
        assert!(
            matches!(r, Err(PathError::OutsideWhitelist(_))),
            "expected OutsideWhitelist, got: {:?}",
            r
        );
    }

    #[test]
    fn validate_rejects_proc() {
        let r = validate_user_path("/proc/self/environ");
        assert!(r.is_err(), "/proc must be rejected");
    }

    #[test]
    fn validate_rejects_home_documents() {
        if let Some(home) = home_dir() {
            let input = home.join("Documents/secret.txt");
            let r = validate_user_path(input.to_string_lossy().as_ref());
            assert!(
                matches!(r, Err(PathError::OutsideWhitelist(_))),
                "expected OutsideWhitelist, got: {:?}",
                r
            );
        }
    }

    #[test]
    fn validate_rejects_usr_path() {
        let r = validate_user_path("/usr/share/sounds/test.wav");
        assert!(
            matches!(r, Err(PathError::OutsideWhitelist(_))),
            "got: {:?}",
            r
        );
    }

    #[test]
    fn validate_rejects_home_bashrc() {
        let r = validate_user_path("~/.bashrc");
        assert!(
            matches!(r, Err(PathError::OutsideWhitelist(_))),
            "got: {:?}",
            r
        );
    }

    // ── Cross-platform sanity ─────────────────────────────────────────────

    #[test]
    fn whitelist_is_cross_platform_aware() {
        let roots = allowed_roots();
        // Every root must be absolute (no relative roots in the whitelist).
        for r in &roots {
            assert!(
                r.is_absolute(),
                "whitelist root must be absolute: {}",
                r.display()
            );
        }
    }
}
