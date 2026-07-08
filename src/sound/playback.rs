// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Sound playback primitives.
//!
//! Provides the low-level playback functions used by the sound system:
//! - `pw-play` invocation (PipeWire / PulseAudio compat)
//! - Terminal bell (`\x07`) fallback
//! - Visual bell (screen flash via DEC private mode)

use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Tries to play a WAV file via `pw-play` (PipeWire / PulseAudio compat).
/// Returns `true` if the process spawned successfully.
/// In debug builds, logs the failure reason to stderr for diagnostics.
pub(crate) fn play_file_via_pw(path: &std::path::Path) -> bool {
    match Command::new("pw-play")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(_) => true,
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!(
                "[zenritme:debug] pw-play spawn failed for {}: {}",
                path.display(),
                e
            );
            let _ = e;
            false
        }
    }
}

/// Plays a terminal bell character (`\x07`).
pub(crate) fn terminal_bell() {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x07");
    let _ = stdout.flush();
}

/// Triggers a visual bell (brief screen flash via DEC private mode).
/// Spawns a background thread so the 60 ms sleep does not block the main loop.
pub(crate) fn visual_bell() {
    // Capture stdout handle in the spawned thread to avoid blocking.
    std::thread::spawn(|| {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(b"\x1b[?5h");
        let _ = stdout.flush();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let _ = stdout.write_all(b"\x1b[?5l");
        let _ = stdout.flush();
    });
}
