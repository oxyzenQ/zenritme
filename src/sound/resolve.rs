// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

//! Sound file resolution logic.
//!
//! Determines which sound source to use for each event:
//!   1. Event-specific env var  (`ZENRITME_SOUND_START`, …)
//!   2. Global env var         (`ZENRITME_SOUND_FILE`)
//!   3. Built-in embedded WAV  (written to a temp file, played via `pw-play`)

use crate::sound::assets;

/// Resolves the sound file path for an event.
///
/// Returns `Some(path)` when an external override exists (env var points to a
/// local file).  Returns `None` when the built-in embedded sound should be used.
pub fn resolve_sound_file(event: crate::sound::SoundEvent) -> Option<String> {
    // 1. Event-specific env override
    if let Ok(path) = std::env::var(assets::env_var(event)) {
        return Some(path);
    }
    // 2. Global env override
    if let Ok(path) = std::env::var(assets::ENV_GLOBAL) {
        return Some(path);
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
        std::env::set_var("ZENRITME_SOUND_START", "/tmp/custom.wav");
        assert_eq!(sound_source(SoundEvent::Start), "override: /tmp/custom.wav");
        std::env::remove_var("ZENRITME_SOUND_START");
    }

    #[test]
    fn sound_source_override_when_global_env_set() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_PAUSE);
        std::env::set_var("ZENRITME_SOUND_FILE", "/tmp/global.wav");
        assert_eq!(sound_source(SoundEvent::Pause), "override: /tmp/global.wav");
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
        std::env::set_var("ZENRITME_SOUND_START", "/tmp/test-start.wav");
        std::env::set_var("ZENRITME_SOUND_FILE", "/tmp/test-global.wav");
        assert_eq!(
            resolve_sound_file(SoundEvent::Start),
            Some("/tmp/test-start.wav".to_string())
        );
        std::env::remove_var("ZENRITME_SOUND_START");
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_global_fallback_when_no_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(assets::ENV_PAUSE);
        std::env::set_var("ZENRITME_SOUND_FILE", "/tmp/test-global.wav");
        assert_eq!(
            resolve_sound_file(SoundEvent::Pause),
            Some("/tmp/test-global.wav".to_string())
        );
        std::env::remove_var("ZENRITME_SOUND_FILE");
    }

    #[test]
    fn resolve_other_events_unaffected_by_event_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var("ZENRITME_SOUND_FILE");
        std::env::set_var("ZENRITME_SOUND_START", "/tmp/test-start.wav");
        // Phase should not be affected by the START override.
        assert_eq!(resolve_sound_file(SoundEvent::Phase), None);
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
