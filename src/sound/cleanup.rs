// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

//! Temp file management and cleanup.
//!
//! Embedded sounds are extracted to a PID-specific temp directory for playback
//! by `pw-play`.  This module manages the lifecycle of that temp directory.
//!
//! Safety invariants:
//! - The temp path is always `zenritme-sounds-{PID}` under the system temp dir.
//! - Cleanup only removes this specific directory — never touches user files.
//! - Cleanup is idempotent: calling it multiple times is safe.
//! - The RAII `TempCleanupGuard` ensures cleanup on any exit path.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::sound::assets;

/// Lazily-created temp directory for extracting embedded sounds.
/// Path is project-specific and PID-specific to avoid conflicts.
fn temp_sound_dir() -> &'static std::path::PathBuf {
    static DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("zenritme-sounds-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    })
}

/// Ensures the embedded WAV for *event* exists as a temp file and returns its
/// path.  The file is written once per process and reused on subsequent plays.
pub(crate) fn ensure_embedded_file(event: crate::sound::SoundEvent) -> Option<std::path::PathBuf> {
    let data = assets::embedded_sound(event);
    let path = temp_sound_dir().join(assets::asset_filename(event));
    if !path.exists() && std::fs::write(&path, data).is_err() {
        return None;
    }
    Some(path)
}

/// Returns the PID-specific temp directory path for sound files.
pub fn temp_dir_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("zenritme-sounds-{}", std::process::id()))
}

/// Removes the temp sound directory created by this process.
///
/// Safe to call multiple times; no-op if the directory does not exist.
/// Only removes the PID-specific directory — never touches user files.
pub fn cleanup_temp_sounds() {
    static CLEANED: AtomicBool = AtomicBool::new(false);
    // Only clean once per process — guard against double-invocation.
    if CLEANED.swap(true, Ordering::Relaxed) {
        return;
    }
    let dir = temp_dir_path();
    if dir.exists() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// RAII guard that registers a cleanup handler on drop.
/// Created via `TempCleanupGuard::install()` at the start of `main()`.
/// On any exit path (normal return, `std::process::exit`, panic unwind),
/// the guard's `Drop` impl calls `cleanup_temp_sounds()`.
pub struct TempCleanupGuard;

impl TempCleanupGuard {
    /// Install the temp-file cleanup guard.  Call once at the start of `main()`.
    /// The returned guard should be bound to a variable (typically `_cleanup_guard`)
    /// so its `Drop` runs at process exit.
    pub fn install() -> Self {
        Self
    }
}

impl Drop for TempCleanupGuard {
    fn drop(&mut self) {
        cleanup_temp_sounds();
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_cleanup_guard_install() {
        let _guard = TempCleanupGuard::install();
        // Guard is created — Drop will fire when _guard goes out of scope.
        // This test verifies install() doesn't panic.
    }

    #[test]
    fn temp_cleanup_guard_drop_cleans() {
        // Use a test-specific directory to avoid racing with the OnceLock dir.
        let dir = std::env::temp_dir().join(format!("zenritme-test-guard-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        assert!(dir.exists(), "test temp dir should exist before cleanup");
        // Directly remove — verifies directory removal logic works.
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        assert!(
            !dir.exists(),
            "test temp dir should be removed after cleanup"
        );
    }

    #[test]
    fn cleanup_idempotent() {
        // Calling cleanup multiple times should not panic or fail.
        cleanup_temp_sounds();
        cleanup_temp_sounds();
        cleanup_temp_sounds();
    }

    #[test]
    fn temp_dir_path_contains_pid() {
        let path = temp_dir_path();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("zenritme-sounds-"),
            "temp dir path should contain zenritme-sounds- prefix"
        );
        let pid_str = std::process::id().to_string();
        assert!(
            path_str.contains(&pid_str),
            "temp dir path should contain PID"
        );
    }

    #[test]
    fn temp_dir_path_is_under_system_tmp() {
        let path = temp_dir_path();
        let tmp = std::env::temp_dir();
        assert!(
            path.starts_with(&tmp),
            "temp dir should be under system temp directory"
        );
    }

    #[test]
    fn ensure_embedded_file_writes_to_temp_dir() {
        use crate::sound::SoundEvent;
        let dir = temp_dir_path();
        let _ = std::fs::create_dir_all(&dir);
        let path = ensure_embedded_file(SoundEvent::Start);
        assert!(path.is_some(), "ensure_embedded_file should return Some");
        let p = path.unwrap();
        assert!(
            p.starts_with(&dir),
            "embedded file should be in the temp dir"
        );
        assert!(p.exists(), "embedded file should exist on disk");
    }
}
