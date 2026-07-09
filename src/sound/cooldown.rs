// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! No-spam cooldown system.
//!
//! Each sound event has an associated cooldown duration.  Rapid calls within
//! the cooldown window are silently suppressed to prevent audible spam.
//!
//! Cooldown durations (milliseconds per event type):
//! - Start:    0 ms   (one-shot, fires once per session)
//! - Pause:    500 ms (debounce on pause toggle)
//! - Phase:    1000 ms (debounce on phase switch)
//! - Complete: 2000 ms (prevents repeated completion sounds)

use std::sync::atomic::{AtomicU64, Ordering};

/// Cooldown durations per event (prevents sound spam on rapid toggling).
/// Each event tracks the last play time and skips playback if called again
/// within its cooldown window. Cooldown in milliseconds per event type.
pub fn cooldown_ms(event: crate::sound::SoundEvent) -> u64 {
    match event {
        crate::sound::SoundEvent::Start => 0,
        crate::sound::SoundEvent::Pause => 500,
        crate::sound::SoundEvent::Phase => 1_000,
        crate::sound::SoundEvent::Complete => 2_000,
    }
}

/// Global last-play timestamps for no-spam cooldown.
/// Uses wall-clock milliseconds (from SystemTime) so the 0-sentinel
/// for "never played" is never accidentally hit by a real timestamp.
static LAST_PLAY: [AtomicU64; 4] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

/// Returns wall-clock milliseconds since UNIX epoch.
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Returns `true` if the event should be played (outside cooldown window).
pub(crate) fn should_play(event: crate::sound::SoundEvent) -> bool {
    let idx = event as usize;
    let cooldown = cooldown_ms(event);
    if cooldown == 0 {
        return true;
    }
    let now = now_ms();
    let last = LAST_PLAY[idx].load(Ordering::Relaxed);
    // A last-play of 0 means "never played" — always allow.
    if last == 0 {
        LAST_PLAY[idx].store(now, Ordering::Relaxed);
        return true;
    }
    let elapsed = now.saturating_sub(last);
    if elapsed < cooldown {
        return false; // still in cooldown
    }
    LAST_PLAY[idx].store(now, Ordering::Relaxed);
    true
}

/// Reset the cooldown timer for a given event (used by `--sound-test`
/// to ensure each event plays during the demo sequence).
pub fn reset_cooldown(event: crate::sound::SoundEvent) {
    let idx = event as usize;
    LAST_PLAY[idx].store(0, Ordering::Relaxed);
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::assets::all_events;
    use crate::sound::SoundEvent;
    #[test]
    fn should_play_start_always() {
        reset_cooldown(SoundEvent::Start);
        assert!(should_play(SoundEvent::Start));
        assert!(should_play(SoundEvent::Start));
    }

    #[test]
    fn should_play_pause_debounces() {
        for e in all_events() {
            reset_cooldown(e);
        }
        assert!(should_play(SoundEvent::Pause), "first call should play");
        assert!(
            !should_play(SoundEvent::Pause),
            "immediate second call should be debounced"
        );
    }
    #[test]
    fn reset_cooldown_allows_replay() {
        for e in all_events() {
            reset_cooldown(e);
        }
        assert!(should_play(SoundEvent::Pause));
        assert!(!should_play(SoundEvent::Pause));
        reset_cooldown(SoundEvent::Pause);
        assert!(
            should_play(SoundEvent::Pause),
            "after reset, should play again"
        );
    }}
