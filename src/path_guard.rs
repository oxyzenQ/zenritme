// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Path security guard — strict allowlist + denylist for user-provided paths.
//!
//! Zenritme must NEVER read sensitive system files (`/etc/shadow`, `~/.ssh/`,
//! `/proc/`, etc.).  All user-provided filesystem paths (currently: the
//! `ZENRITME_SOUND_*` env vars) are validated through this module before
//! being opened or passed to playback subprocesses.
//!
//! # Policy
//!
//! ## Allowed roots (recursive)
//!
//! 1. **Home directory** — `~` (recursive)
//! 2. **Current working directory** — `.` (recursive)
//! 3. **Project config dir** — `~/.config/zenritme/` (recursive)
//! 4. **Project data dir** — `~/.local/share/zenritme/` (recursive)
//!
//! ## Denied subpaths (always rejected, even inside an allowed root)
//!
//! - `~/.ssh/`, `~/.gnupg/`, `~/.aws/`, `~/.docker/`, `~/.kube/`
//! - `~/.config/systemd/`, `~/.local/share/keyrings/`
//! - `/etc/shadow`, `/etc/gshadow`, `/etc/shadow-`, `/etc/gshadow-`
//! - `/etc/ssh/`
//! - `/root/` (unless running as UID 0)
//! - `/proc/`, `/sys/`
//!
//! ## Symlink safety
//!
//! Paths are canonicalized (symlinks resolved) before the policy check, so a
//! symlink inside an allowed root that points to a denied location is rejected.
//! For non-existent paths (e.g. a sound file the user will create later), the
//! **parent directory** is canonicalized and the policy is applied to it.

use std::path::{Path, PathBuf};

// ─── Error type ────────────────────────────────────────────────────────────

/// Reasons a user-provided path may be rejected by the guard.
#[derive(Debug)]
pub enum PathError {
    /// The input was empty or whitespace-only.
    Empty,
    /// The path resolves to a denied/sensitive location (e.g. `~/.ssh/`).
    Denied(String),
    /// The path is outside all allowed roots.
    OutsideAllowed(String),
    /// The path or its parent could not be resolved on the filesystem.
    ResolutionFailed(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::Empty => write!(f, "empty path"),
            PathError::Denied(p) => {
                write!(f, "denied path (sensitive location): {}", p)
            }
            PathError::OutsideAllowed(p) => {
                write!(f, "path outside allowed roots: {}", p)
            }
            PathError::ResolutionFailed(msg) => {
                write!(f, "path resolution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for PathError {}

// ─── Path helpers ──────────────────────────────────────────────────────────

/// Returns the user's home directory from `$HOME`, if set and non-empty.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
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

// ─── Policy tables ─────────────────────────────────────────────────────────

/// Sensitive subdirectories under `$HOME` that must never be read.
const DENIED_HOME_SUBDIRS: &[&str] = &[
    ".ssh",
    ".gnupg",
    ".aws",
    ".docker",
    ".kube",
    ".config/systemd",
    ".local/share/keyrings",
];

/// Sensitive absolute paths that must never be read.
const DENIED_ABSOLUTE: &[&str] = &[
    "/etc/shadow",
    "/etc/gshadow",
    "/etc/shadow-",
    "/etc/gshadow-",
    "/etc/ssh",
    "/root",
    "/proc",
    "/sys",
];

/// Returns `true` if the canonical path falls inside any denied location.
fn is_denied(canonical: &Path) -> bool {
    // Absolute denials — apply unconditionally.
    for denied in DENIED_ABSOLUTE {
        let denied_path = Path::new(denied);
        // `/root` is only denied when running as non-root.
        if *denied == "/root" && unix_uid() == 0 {
            continue;
        }
        if canonical == denied_path || canonical.starts_with(denied_path) {
            return true;
        }
    }
    // Home-relative denials — only apply when HOME is set.
    if let Some(home) = home_dir() {
        for sub in DENIED_HOME_SUBDIRS {
            let denied_path = home.join(sub);
            if canonical == denied_path || canonical.starts_with(&denied_path) {
                return true;
            }
        }
    }
    false
}

/// Returns `true` if the canonical path falls inside any allowed root.
fn is_allowed(canonical: &Path) -> bool {
    // 1. Current working directory (recursive).
    if let Ok(cwd) = std::env::current_dir() {
        if canonical == cwd || canonical.starts_with(&cwd) {
            return true;
        }
    }
    // 2. Home directory (recursive) — covers ~/.config/zenritme/ and
    //    ~/.local/share/zenritme/ automatically since they are subdirs.
    if let Some(home) = home_dir() {
        if canonical == home || canonical.starts_with(&home) {
            return true;
        }
    }
    false
}

/// Returns the effective Unix UID (0 = root).  Returns a non-zero value on
/// non-Unix targets so the `/root` deny rule applies.
#[cfg(unix)]
fn unix_uid() -> u32 {
    // SAFETY: `getuid` is async-signal-safe and takes no arguments.
    unsafe { libc_getuid() }
}

#[cfg(unix)]
extern "C" {
    fn getuid() -> u32;
}

#[cfg(unix)]
unsafe fn libc_getuid() -> u32 {
    getuid()
}

#[cfg(not(unix))]
fn unix_uid() -> u32 {
    1000
}

// ─── Public entry point ────────────────────────────────────────────────────

/// Validate a user-provided filesystem path against the security policy.
///
/// Steps:
/// 1. Trim and reject empty input.
/// 2. Expand leading `~` to `$HOME`.
/// 3. **String-based policy check** on the expanded path — catches obvious
///    denials (`~/.ssh/`, `/etc/shadow`, `/proc/`) and out-of-allowlist paths
///    without touching the filesystem.
/// 4. **Canonical policy check** — if the file (or its parent) exists on disk,
///    canonicalize it (resolves symlinks) and re-apply the policy.  This
///    catches symlink-escape attacks where a symlink inside an allowed root
///    points to a denied location.
/// 5. If the file does not yet exist and the parent cannot be canonicalized
///    either, accept the string-validated expanded path (the user may create
///    the file later).
///
/// Returns the canonicalized path when possible, otherwise the expanded path.
pub fn validate_user_path(input: &str) -> Result<PathBuf, PathError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PathError::Empty);
    }

    let expanded = expand_tilde(trimmed);

    // Resolve relative paths against the current working directory so that
    // `./foo.wav` and `foo.wav` are treated as inside-cwd.
    let resolved = if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&expanded))
            .map_err(|e| PathError::ResolutionFailed(format!("cwd: {}", e)))?
    };

    // Step 1: string-based check on the resolved path (no FS access).
    if is_denied(&resolved) {
        return Err(PathError::Denied(resolved.display().to_string()));
    }
    if !is_allowed(&resolved) {
        return Err(PathError::OutsideAllowed(resolved.display().to_string()));
    }

    // Step 2: canonical check for symlink-escape detection.
    // If the file exists, canonicalize it directly.
    if let Ok(canonical) = std::fs::canonicalize(&resolved) {
        if is_denied(&canonical) {
            return Err(PathError::Denied(canonical.display().to_string()));
        }
        if !is_allowed(&canonical) {
            return Err(PathError::OutsideAllowed(canonical.display().to_string()));
        }
        return Ok(canonical);
    }

    // Step 3: file may not exist yet — canonicalize parent and re-join.
    if let Some(parent) = resolved.parent() {
        if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
            if is_denied(&canonical_parent) {
                return Err(PathError::Denied(canonical_parent.display().to_string()));
            }
            if !is_allowed(&canonical_parent) {
                return Err(PathError::OutsideAllowed(
                    canonical_parent.display().to_string(),
                ));
            }
            if let Some(filename) = resolved.file_name() {
                return Ok(canonical_parent.join(filename));
            }
        }
    }

    // Step 4: filesystem access failed but string checks passed — accept the
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
        // Either HOME or fallback "~" (when HOME unset in test env).
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

    // ── is_denied ─────────────────────────────────────────────────────────

    #[test]
    fn denied_absolute_etc_shadow() {
        assert!(is_denied(Path::new("/etc/shadow")));
        assert!(is_denied(Path::new("/etc/shadow-")));
        assert!(is_denied(Path::new("/etc/gshadow")));
    }

    #[test]
    fn denied_absolute_proc_sys() {
        assert!(is_denied(Path::new("/proc")));
        assert!(is_denied(Path::new("/proc/self")));
        assert!(is_denied(Path::new("/sys")));
        assert!(is_denied(Path::new("/sys/kernel")));
    }

    #[test]
    fn denied_absolute_etc_ssh() {
        assert!(is_denied(Path::new("/etc/ssh")));
        assert!(is_denied(Path::new("/etc/ssh/sshd_config")));
    }

    #[test]
    fn denied_home_ssh() {
        if let Some(home) = home_dir() {
            assert!(is_denied(&home.join(".ssh")));
            assert!(is_denied(&home.join(".ssh/id_rsa")));
            assert!(is_denied(&home.join(".ssh/config")));
        }
    }

    #[test]
    fn denied_home_gnupg() {
        if let Some(home) = home_dir() {
            assert!(is_denied(&home.join(".gnupg")));
            assert!(is_denied(&home.join(".gnupg/secring.gpg")));
        }
    }

    #[test]
    fn denied_home_aws_docker_kube() {
        if let Some(home) = home_dir() {
            assert!(is_denied(&home.join(".aws/credentials")));
            assert!(is_denied(&home.join(".docker/config.json")));
            assert!(is_denied(&home.join(".kube/config")));
        }
    }

    #[test]
    fn denied_home_systemd_keyrings() {
        if let Some(home) = home_dir() {
            assert!(is_denied(&home.join(".config/systemd/user")));
            assert!(is_denied(&home.join(".local/share/keyrings/login.keyring")));
        }
    }

    #[test]
    fn denied_root_only_when_non_root() {
        if unix_uid() != 0 {
            assert!(is_denied(Path::new("/root")));
            assert!(is_denied(Path::new("/root/.bashrc")));
        } else {
            // When running as root, /root is allowed.
            assert!(!is_denied(Path::new("/root")));
        }
    }

    #[test]
    fn not_denied_safe_paths() {
        if let Some(home) = home_dir() {
            assert!(!is_denied(&home.join("music/chime.wav")));
            assert!(!is_denied(&home.join(".config/zenritme/sounds.wav")));
            assert!(!is_denied(&home.join(".local/share/zenritme/x.wav")));
        }
        assert!(!is_denied(Path::new("/usr/bin/pw-play")));
        assert!(!is_denied(Path::new("/tmp/custom.wav")));
    }

    // ── is_allowed ────────────────────────────────────────────────────────

    #[test]
    fn allowed_inside_home() {
        if let Some(home) = home_dir() {
            assert!(is_allowed(&home));
            assert!(is_allowed(&home.join("music/chime.wav")));
            assert!(is_allowed(&home.join(".config/zenritme/x.wav")));
        }
    }

    #[test]
    fn allowed_inside_cwd() {
        if let Ok(cwd) = std::env::current_dir() {
            assert!(is_allowed(&cwd));
            assert!(is_allowed(&cwd.join("local.wav")));
            assert!(is_allowed(&cwd.join("subdir/local.wav")));
        }
    }

    #[test]
    fn not_allowed_outside_home_and_cwd() {
        // /usr is neither home nor cwd (in normal test environments).
        assert!(!is_allowed(Path::new("/usr/local/bin")));
        assert!(!is_allowed(Path::new("/var/log/syslog")));
        // /tmp is NOT in the allowlist (only home, cwd, and home subdirs are).
        assert!(!is_allowed(Path::new("/tmp")));
        assert!(!is_allowed(Path::new("/tmp/custom.wav")));
    }

    // ── validate_user_path — happy path ───────────────────────────────────

    #[test]
    fn validate_home_relative_path() {
        if let Some(home) = home_dir() {
            // Use a non-existent filename — parent (home) must canonicalize.
            let r = validate_user_path("~/zenritme-test-sound.wav");
            assert!(r.is_ok(), "expected Ok, got: {:?}", r);
            let p = r.unwrap();
            assert!(p.starts_with(&home));
            assert!(p.ends_with("zenritme-test-sound.wav"));
        }
    }

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

    // ── validate_user_path — rejections ───────────────────────────────────

    #[test]
    fn validate_rejects_empty() {
        assert!(matches!(validate_user_path(""), Err(PathError::Empty)));
        assert!(matches!(validate_user_path("   "), Err(PathError::Empty)));
    }

    #[test]
    fn validate_rejects_etc_shadow() {
        let r = validate_user_path("/etc/shadow");
        assert!(matches!(r, Err(PathError::Denied(_))), "got: {:?}", r);
    }

    #[test]
    fn validate_rejects_home_ssh() {
        let r = validate_user_path("~/.ssh/id_rsa");
        assert!(matches!(r, Err(PathError::Denied(_))), "got: {:?}", r);
    }

    #[test]
    fn validate_rejects_proc() {
        let r = validate_user_path("/proc/self/environ");
        // /proc is denied AND outside allowed roots — Denied takes precedence.
        assert!(r.is_err());
    }

    #[test]
    fn validate_rejects_tmp_outside_allowed() {
        let r = validate_user_path("/tmp/custom.wav");
        assert!(
            matches!(r, Err(PathError::OutsideAllowed(_))),
            "expected OutsideAllowed, got: {:?}",
            r
        );
    }

    #[test]
    fn validate_rejects_usr_path() {
        let r = validate_user_path("/usr/share/sounds/test.wav");
        assert!(
            matches!(r, Err(PathError::OutsideAllowed(_))),
            "got: {:?}",
            r
        );
    }

    // ── Policy table sanity ───────────────────────────────────────────────

    #[test]
    fn denied_tables_non_empty() {
        assert!(!DENIED_ABSOLUTE.is_empty());
        assert!(!DENIED_HOME_SUBDIRS.is_empty());
    }

    #[test]
    fn denied_absolute_all_start_with_slash() {
        for p in DENIED_ABSOLUTE {
            assert!(
                p.starts_with('/'),
                "denied absolute must start with /: {}",
                p
            );
        }
    }

    #[test]
    fn denied_home_subdirs_never_start_with_slash() {
        for p in DENIED_HOME_SUBDIRS {
            assert!(
                !p.starts_with('/'),
                "denied home subdir must be relative: {}",
                p
            );
            assert!(!p.is_empty());
        }
    }
}
