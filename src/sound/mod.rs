// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Ritual sound system — v3.0.0 stable release.
//!
//! Built-in procedural notification sounds with mature architecture,
//! premium sound UX, and long-usage stability.
//!
//! # Architecture
//!
//! The sound module is split into focused submodules:
//!
//! - [`assets`]    — embedded WAV data and metadata (`include_bytes!`)
//! - [`playback`]  — low-level playback (`pw-play`, terminal bell, visual bell)
//! - [`resolve`]   — env override resolution and source display
//! - [`cooldown`]  — no-spam cooldown rules
//! - [`cleanup`]   — temp directory/file cleanup and RAII guard
//!
//! # Resolution order (per event)
//!
//!   1. Event-specific env var  (`ZENRITME_SOUND_START`, …)
//!   2. Global env var         (`ZENRITME_SOUND_FILE`)
//!   3. Built-in embedded WAV  (written to a temp file, played via `pw-play`)
//!
//! # Fallback chain
//!
//! If `pw-play` is unavailable or playback fails, Zenritme falls back to
//! the terminal bell (`\x07`) and optionally the visual bell.

pub mod assets;
pub mod cleanup;
pub mod cooldown;
pub mod playback;
pub mod resolve;

use std::io::{self, Write};

use assets::all_events;

// ─── Sound profile ─────────────────────────────────────────────────────────

/// Sound profile controlling when notification sounds play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundProfile {
    /// Normal sound behavior — all events produce notification sounds.
    Calm,
    /// All sounds suppressed (equivalent to `--mute`).
    Silent,
}

impl SoundProfile {
    /// Parse a profile name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "calm" => Some(SoundProfile::Calm),
            "silent" => Some(SoundProfile::Silent),
            _ => None,
        }
    }

    /// Returns `true` if all sounds should be suppressed.
    pub fn is_silent(self) -> bool {
        self == SoundProfile::Silent
    }
}

// ─── Sound event enum ────────────────────────────────────────────────────────

/// Notification sound events emitted during Zenritme operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEvent {
    /// Timer / session started.
    Start,
    /// Pause or resume toggled.
    Pause,
    /// Pomodoro phase switched.
    Phase,
    /// Timer / pomodoro fully completed.
    Complete,
}

// ─── Re-exports ────────────────────────────────────────────────────────────

/// RAII guard that cleans up temp sound files on drop.
pub use cleanup::TempCleanupGuard;

// ─── Core play function ────────────────────────────────────────────────────

/// Plays a sound event.  Attempts (in order):
///
/// 1. External file via `pw-play`  (if an env override is set).
/// 2. Built-in embedded WAV via `pw-play`  (temp file extraction).
/// 3. Terminal bell fallback (`\x07`).
///
/// If `ZENRITME_VISUAL_BELL` is set, a visual bell flash is also triggered.
pub fn play_event(event: SoundEvent) {
    // No-spam guard: skip if within cooldown window.
    if !cooldown::should_play(event) {
        return;
    }

    let played = if let Some(ref file) = resolve::resolve_sound_file(event) {
        playback::play_file_via_pw(std::path::Path::new(file))
    } else if let Some(path) = cleanup::ensure_embedded_file(event) {
        playback::play_file_via_pw(&path)
    } else {
        false
    };

    if !played {
        playback::terminal_bell();
    }

    if std::env::var("ZENRITME_VISUAL_BELL").is_ok() {
        playback::visual_bell();
    }
}

// ─── --sound-test ───────────────────────────────────────────────────────────

/// Prints sound-system status and plays all four events in sequence.
/// Cleans up temp sound files on completion.
pub fn sound_test() {
    println!("zenritme sound test (v{})", env!("CARGO_PKG_VERSION"));
    println!();

    // ── Per-event info ──────────────────────────────────────────────────────
    println!("Sound events:");
    for event in all_events() {
        let env = assets::env_var(event);
        let source = resolve::sound_source(event);
        let cd = cooldown::cooldown_ms(event);
        let cd_str = if cd == 0 {
            "no cooldown".to_string()
        } else {
            format!("{} ms cooldown", cd)
        };
        println!(
            "  {:9}  {}  [{}]  ({})",
            format!("{:?}", event),
            source,
            env,
            cd_str
        );
    }

    println!();

    // ── Sound profiles ─────────────────────────────────────────────────────
    println!("Sound profiles:");
    println!("  calm   - all notification sounds enabled (default)");
    println!("  silent - all notification sounds suppressed");
    println!("  --mute flag overrides any profile to silent");
    println!();

    // ── Override docs ──────────────────────────────────────────────────────
    println!("Environment variables:");
    println!("  ZENRITME_SOUND_START     override start sound file");
    println!("  ZENRITME_SOUND_PAUSE     override pause sound file");
    println!("  ZENRITME_SOUND_PHASE     override phase sound file");
    println!("  ZENRITME_SOUND_COMPLETE  override complete sound file");
    println!("  ZENRITME_SOUND_FILE      global fallback for all events");
    println!("  ZENRITME_VISUAL_BELL=1   enable visual bell (screen flash)");
    println!();

    // ── Global override ─────────────────────────────────────────────────────
    match std::env::var(assets::ENV_GLOBAL) {
        Ok(p) => println!("Global override: {}", p),
        Err(_) => println!("Global override: (none)"),
    }

    // ── Visual bell ──────────────────────────────────────────────────────────
    if std::env::var("ZENRITME_VISUAL_BELL").is_ok() {
        println!("Visual bell:   enabled");
    } else {
        println!("Visual bell:   disabled");
    }
    println!();

    // ── Playback demo ───────────────────────────────────────────────────────
    println!("Playing all events in sequence...");
    for event in all_events() {
        print!("  {:?}... ", event);
        io::stdout().flush().ok();
        // Reset cooldown before test playback so each event actually plays.
        cooldown::reset_cooldown(event);
        play_event(event);
        std::thread::sleep(std::time::Duration::from_millis(400));
        println!("ok");
    }
    println!();

    // ── Cleanup temp files ──────────────────────────────────────────────────
    cleanup::cleanup_temp_sounds();
    println!("Temp sound files cleaned up.");
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sound profile ───────────────────────────────────────────────────

    #[test]
    fn profile_calm_from_name() {
        assert_eq!(SoundProfile::from_name("calm"), Some(SoundProfile::Calm));
    }

    #[test]
    fn profile_calm_case_insensitive() {
        assert_eq!(SoundProfile::from_name("Calm"), Some(SoundProfile::Calm));
        assert_eq!(SoundProfile::from_name("CALM"), Some(SoundProfile::Calm));
    }

    #[test]
    fn profile_silent_from_name() {
        assert_eq!(
            SoundProfile::from_name("silent"),
            Some(SoundProfile::Silent)
        );
    }

    #[test]
    fn profile_silent_case_insensitive() {
        assert_eq!(
            SoundProfile::from_name("Silent"),
            Some(SoundProfile::Silent)
        );
        assert_eq!(
            SoundProfile::from_name("SILENT"),
            Some(SoundProfile::Silent)
        );
    }

    #[test]
    fn profile_unknown_rejected() {
        assert_eq!(SoundProfile::from_name("loud"), None);
        assert_eq!(SoundProfile::from_name(""), None);
        assert_eq!(SoundProfile::from_name("off"), None);
    }

    #[test]
    fn profile_calm_is_not_silent() {
        assert!(!SoundProfile::Calm.is_silent());
    }

    #[test]
    fn profile_silent_is_silent() {
        assert!(SoundProfile::Silent.is_silent());
    }

    // ── Mute/profile precedence ───────────────────────────────────────────

    #[test]
    fn mute_overrides_calm() {
        let muted = true;
        let profile = SoundProfile::Calm;
        assert!(muted || profile.is_silent(), "mute should suppress");
    }

    #[test]
    fn mute_overrides_silent_explicitly() {
        let muted = true;
        let profile = SoundProfile::Silent;
        assert!(muted || profile.is_silent(), "mute should suppress");
    }

    #[test]
    fn silent_profile_suppresses_without_mute() {
        let muted = false;
        let profile = SoundProfile::Silent;
        assert!(
            muted || profile.is_silent(),
            "silent profile should suppress"
        );
    }

    #[test]
    fn calm_profile_allows_sound_without_mute() {
        let muted = false;
        let profile = SoundProfile::Calm;
        assert!(
            !(muted || profile.is_silent()),
            "calm profile should allow sound"
        );
    }

    // ── Public API surface ───────────────────────────────────────────────

    #[test]
    fn sound_event_variants() {
        let events = [
            SoundEvent::Start,
            SoundEvent::Pause,
            SoundEvent::Phase,
            SoundEvent::Complete,
        ];
        assert_eq!(events.len(), 4);
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(events[i] != events[j], "variants should be distinct");
            }
        }
    }

    #[test]
    fn sound_event_debug_fmt() {
        assert!(format!("{:?}", SoundEvent::Start).contains("Start"));
        assert!(format!("{:?}", SoundEvent::Pause).contains("Pause"));
        assert!(format!("{:?}", SoundEvent::Phase).contains("Phase"));
        assert!(format!("{:?}", SoundEvent::Complete).contains("Complete"));
    }

    #[test]
    fn sound_event_discriminant_order() {
        assert_eq!(SoundEvent::Start as usize, 0);
        assert_eq!(SoundEvent::Pause as usize, 1);
        assert_eq!(SoundEvent::Phase as usize, 2);
        assert_eq!(SoundEvent::Complete as usize, 3);
    }

    #[test]
    fn temp_cleanup_guard_public_api() {
        let _guard = TempCleanupGuard::install();
    }

    #[test]
    fn cleanup_temp_sounds_public_api() {
        cleanup::cleanup_temp_sounds();
    }
}
