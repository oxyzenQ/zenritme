// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Sound file resolution logic.
//!
//! Determines which sound source to use for each event:
//!   1. Event-specific env var  (`ZENRITME_SOUND_START`, …)
//!   2. Global env var         (`ZENRITME_SOUND_FILE`)
//!   3. Built-in embedded WAV  (written to a temp file, played via `pw-play`)
//!
//! # Security
//!
//! All env-var-provided paths are validated through [`crate::path_guard`]
//! before use.  Paths that resolve to sensitive locations (`~/.ssh/`,
//! `/etc/shadow`, `/proc/`, etc.) or fall outside the allowed roots are
//! rejected with a warning on stderr, and Zenritme falls back to the built-in
//! embedded sound.  See [`crate::path_guard`] for the full policy.

use crate::path_guard;
use crate::sound::assets;

/// Resolves the sound file path for an event.
///
/// Returns `Some(path)` when an external override exists and passes the
/// path-guard security check.  Returns `None` when:
///   - no env override is set, OR
///   - the env override is set but rejected by the path guard (falls back
///     to the built-in embedded sound).
pub fn resolve_sound_file(event: crate::sound::SoundEvent) -> Option<String> {
    // 1. Event-specific env override
    if let Ok(raw) = std::env::var(assets::env_var(event)) {
        return match path_guard::validate_user_path(&raw) {
            Ok(canonical) => Some(canonical.to_string_lossy().into_owned()),
            Err(e) => {
                eprintln!(
                    "zenritme: rejecting sound override {:?} for {:?}: {} (using built-in)",
                    raw, event, e
                );
                None
            }
        };
    }
    // 2. Global env override
    if let Ok(raw) = std::env::var(assets::ENV_GLOBAL) {
        return match path_guard::validate_user_path(&raw) {
            Ok(canonical) => Some(canonical.to_string_lossy().into_owned()),
            Err(e) => {
                eprintln!(
                    "zenritme: rejecting global sound override {:?}: {} (using built-in)",
                    raw, e
                );
                None
            }
        };
    }
    // 3. Built-in — no file path needed
    None
}

/// Returns a human-readable source description for an event's resolved sound.
pub fn sound_source(event: crate::sound::SoundEvent) -> String {
    match resolve_sound_file(event) {
        Some(ref path) => format!("override: {}", path),
        None => format!("built-in: {}", assets::asset_filename(event)),
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::SoundEvent;

    /// All tests in this module mutate process-global env vars via
    /// `std::env::set_var` / `remove_var`, which are not thread-safe.
    /// Rust runs tests in parallel by default, so we serialize them
    /// with a mutex to prevent env-var leakage between tests.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn sound_source_builtin_when_no_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_START);
        std::env::remove_var(assets::ENV_GLOBAL);
        assert_eq!(sound_source(SoundEvent::Start), "built-in: start.wav");
    }

    #[test]
    fn sound_source_override_when_event_env_set() {
        let _lock = ENV_LOCK.lock().unwrap();
        let home = std::env::var("HOME").expect("HOME must be set for tests");
        let input = format!("{home}/custom.wav");
        std::env::set_var("ZENRITME_SOUND_START", &input);
        // resolve_sound_file returns the canonical path; parent (home) exists,
        // so canonical form == home + "/custom.wav" (no symlink resolution
        // changes on a normal /home/<user> path).
        let expected = std::fs::canonicalize(&home)
            .unwrap_or_else(|_| std::path::PathBuf::from(&home))
            .join("custom.wav")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            sound_source(SoundEvent::Start),
            format!("override: {expected}")
        );
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    #[test]
    fn sound_source_override_when_global_env_set() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_PAUSE);
        let home = std::env::var("HOME").expect("HOME must be set for tests");
        let input = format!("{home}/global.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", &input);
        let expected = std::fs::canonicalize(&home)
            .unwrap_or_else(|_| std::path::PathBuf::from(&home))
            .join("global.wav")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            sound_source(SoundEvent::Pause),
            format!("override: {expected}")
        );
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_returns_none_when_no_env_set() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_START);
        std::env::remove_var(assets::ENV_GLOBAL);
        assert_eq!(resolve_sound_file(SoundEvent::Start), None);
    }

    #[test]
    fn resolve_event_env_over_global() {
        let _lock = ENV_LOCK.lock().unwrap();
        let home = std::env::var("HOME").expect("HOME must be set for tests");
        let canonical_home =
            std::fs::canonicalize(&home).unwrap_or_else(|_| std::path::PathBuf::from(&home));
        let start_input = format!("{home}/test-start.wav");
        let global_input = format!("{home}/test-global.wav");
        std::env::set_var("ZENRITME_SOUND_START", &start_input);
        std::env::set_var("ZENRITME_SOUND_FILE", &global_input);
        let expected = canonical_home
            .join("test-start.wav")
            .to_string_lossy()
            .into_owned();
        assert_eq!(resolve_sound_file(SoundEvent::Start), Some(expected));
        std::env::remove_var("ZENRITME_SOUND_START");
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_global_fallback_when_no_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_PAUSE);
        let home = std::env::var("HOME").expect("HOME must be set for tests");
        let input = format!("{home}/test-global.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", &input);
        let expected = std::fs::canonicalize(&home)
            .unwrap_or_else(|_| std::path::PathBuf::from(&home))
            .join("test-global.wav")
            .to_string_lossy()
            .into_owned();
        assert_eq!(resolve_sound_file(SoundEvent::Pause), Some(expected));
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_other_events_unaffected_by_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var("ZENRITME_SOUND_FILE");
        let home = std::env::var("HOME").expect("HOME must be set for tests");
        std::env::set_var("ZENRITME_SOUND_START", format!("{home}/test-start.wav"));
        // Phase should not be affected by the START override.
        assert_eq!(resolve_sound_file(SoundEvent::Phase), None);
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    // ── Path-guard integration: rejected overrides fall back to built-in ──

    #[test]
    fn resolve_rejects_etc_shadow_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZENRITME_SOUND_START", "/etc/shadow");
        // /etc/shadow is denied → resolve returns None → built-in used.
        assert_eq!(resolve_sound_file(SoundEvent::Start), None);
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    #[test]
    fn resolve_rejects_home_ssh_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZENRITME_SOUND_FILE", "~/.ssh/id_rsa");
        assert_eq!(resolve_sound_file(SoundEvent::Phase), None);
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_rejects_tmp_outside_allowed_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        // /tmp is not inside home or cwd → OutsideAllowed → fallback to built-in.
        std::env::set_var("ZENRITME_SOUND_FILE", "/tmp/custom.wav");
        assert_eq!(resolve_sound_file(SoundEvent::Complete), None);
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_rejects_proc_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZENRITME_SOUND_START", "/proc/self/environ");
        assert_eq!(resolve_sound_file(SoundEvent::Start), None);
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    #[test]
    fn sound_source_display_casing() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_START);
        std::env::remove_var(assets::ENV_GLOBAL);
        assert!(
            sound_source(SoundEvent::Complete).starts_with("built-in: "),
            "built-in source should have consistent prefix"
        );
    }
}
