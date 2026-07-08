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
    use std::path::PathBuf;

    /// All tests in this module mutate process-global env vars via
    /// `std::env::set_var` / `remove_var`, which are not thread-safe.
    /// Rust runs tests in parallel by default, so we serialize them
    /// with a mutex to prevent env-var leakage between tests.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Returns a whitelisted config dir for test sound file paths.
    ///
    /// Uses `path_guard::user_config_dir()` (e.g. `~/.config/zenritme/`)
    /// so test paths pass the whitelist policy.  The directory is created
    /// on demand so canonicalize() succeeds.
    fn test_config_dir() -> PathBuf {
        let cfg = path_guard::user_config_dir().unwrap_or_else(|| {
            // Fallback: use cwd (also whitelisted).
            std::env::current_dir().expect("cwd must be available")
        });
        let _ = std::fs::create_dir_all(&cfg);
        cfg
    }

    /// Returns the canonical string form of a path, falling back to the
    /// input itself if canonicalization fails (e.g. file does not exist).
    fn canonical_or(p: PathBuf) -> String {
        std::fs::canonicalize(&p)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    }

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
        let cfg = test_config_dir();
        let input = cfg.join("custom.wav");
        std::env::set_var("ZENRITME_SOUND_START", input.to_string_lossy().as_ref());
        // resolve_sound_file returns the canonical path; parent (config dir)
        // exists, so canonical form == cfg + "/custom.wav".
        let expected = canonical_or(input.clone());
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
        let cfg = test_config_dir();
        let input = cfg.join("global.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", input.to_string_lossy().as_ref());
        let expected = canonical_or(input.clone());
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
        let cfg = test_config_dir();
        let start_input = cfg.join("test-start.wav");
        let global_input = cfg.join("test-global.wav");
        std::env::set_var(
            "ZENRITME_SOUND_START",
            start_input.to_string_lossy().as_ref(),
        );
        std::env::set_var(
            "ZENRITME_SOUND_FILE",
            global_input.to_string_lossy().as_ref(),
        );
        let expected = canonical_or(start_input);
        assert_eq!(resolve_sound_file(SoundEvent::Start), Some(expected));
        std::env::remove_var("ZENRITME_SOUND_START");
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_global_fallback_when_no_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_PAUSE);
        let cfg = test_config_dir();
        let input = cfg.join("test-global.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", input.to_string_lossy().as_ref());
        let expected = canonical_or(input);
        assert_eq!(resolve_sound_file(SoundEvent::Pause), Some(expected));
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_other_events_unaffected_by_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var("ZENRITME_SOUND_FILE");
        let cfg = test_config_dir();
        std::env::set_var(
            "ZENRITME_SOUND_START",
            cfg.join("test-start.wav").to_string_lossy().as_ref(),
        );
        // Phase should not be affected by the START override.
        assert_eq!(resolve_sound_file(SoundEvent::Phase), None);
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    // ── Path-guard integration: rejected overrides fall back to built-in ──

    #[test]
    fn resolve_rejects_etc_shadow_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZENRITME_SOUND_START", "/etc/shadow");
        // /etc/shadow is outside whitelist → resolve returns None → built-in used.
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
    fn resolve_rejects_proc_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZENRITME_SOUND_START", "/proc/self/environ");
        assert_eq!(resolve_sound_file(SoundEvent::Start), None);
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    #[test]
    fn resolve_rejects_home_documents_falls_back_to_builtin() {
        let _lock = ENV_LOCK.lock().unwrap();
        // ~/Documents/ is NOT in the whitelist → rejected → fallback to built-in.
        std::env::set_var("ZENRITME_SOUND_FILE", "~/Documents/secret.wav");
        assert_eq!(resolve_sound_file(SoundEvent::Complete), None);
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_accepts_temp_dir_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        // /tmp IS in the whitelist (system temp dir) → accepted.
        let tmp = std::env::temp_dir().join("zenritme-accept-test.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", tmp.to_string_lossy().as_ref());
        let r = resolve_sound_file(SoundEvent::Start);
        assert!(r.is_some(), "temp dir path should be accepted by whitelist");
        std::env::remove_var("ZENRITME_SOUND_FILE");
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
