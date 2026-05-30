// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

//! Terminal state capture and restore.
//!
//! `TerminalGuard` captures the exact terminal state via `stty -g` on creation,
//! enters raw mode, hides the cursor, and optionally enters the alternate screen.
//! On `Drop`, the original state is restored precisely — `stty sane` is only a
//! last-resort fallback when the capture failed.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc;

/// Saves terminal state on creation and restores it on drop.
pub struct TerminalGuard {
    tty: Option<std::fs::File>,
    /// Opaque string from `stty -g`; empty means capture failed.
    saved: Option<String>,
    alt_screen: bool,
}

impl TerminalGuard {
    pub fn new() -> Self {
        let tty = std::fs::File::open("/dev/tty").ok();

        // ── 1. Capture current state ──────────────────────────────────────────
        let saved = capture_stty_state(tty.as_ref());

        // ── 2. Enter raw mode ─────────────────────────────────────────────────
        if let Some(t) = tty.as_ref().and_then(|f| f.try_clone().ok()) {
            let _ = Command::new("stty")
                .args(["-icanon", "-echo", "min", "1", "time", "0"])
                .stdin(Stdio::from(t))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        // ── 3. Alternate screen + hide cursor (skip for dumb terminals) ───────
        let alt_screen = std::env::var("TERM").map_or(true, |t| t != "dumb");
        if alt_screen {
            let mut stdout = std::io::stdout();
            // \x1b[?1049h  — enter alternate screen
            // \x1b[?25l    — hide cursor
            let _ = stdout.write_all(b"\x1b[?1049h\x1b[?25l");
            let _ = stdout.flush();
        }

        Self {
            tty,
            saved,
            alt_screen,
        }
    }
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // ── 1. Leave alternate screen + show cursor ───────────────────────────
        if self.alt_screen {
            let mut stdout = std::io::stdout();
            // \x1b[?1049l  — leave alternate screen
            // \x1b[?25h    — show cursor
            let _ = stdout.write_all(b"\x1b[?1049l\x1b[?25h");
            let _ = stdout.flush();
        }

        // ── 2. Restore exact saved state (primary path) ───────────────────────
        if let Some(ref saved) = self.saved {
            if !saved.is_empty() {
                if let Some(t) = self.tty.as_ref().and_then(|f| f.try_clone().ok()) {
                    let ok = Command::new("stty")
                        .arg(saved)
                        .stdin(Stdio::from(t))
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if ok {
                        return;
                    }
                }
            }
        }

        // ── 3. Fallback: stty sane ────────────────────────────────────────────
        if let Some(t) = self.tty.as_ref().and_then(|f| f.try_clone().ok()) {
            let _ = Command::new("stty")
                .arg("sane")
                .stdin(Stdio::from(t))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

/// Runs `stty -g` and returns the opaque state string, or `None` on failure.
fn capture_stty_state(tty: Option<&std::fs::File>) -> Option<String> {
    let t = tty?.try_clone().ok()?;
    let out = Command::new("stty")
        .arg("-g")
        .stdin(Stdio::from(t))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Sets up raw-mode input and spawns a reader thread.
///
/// Returns the `TerminalGuard` (keep it alive for the session lifetime) and
/// an optional receiver of single keypress bytes.
pub fn spawn_input() -> (TerminalGuard, Option<mpsc::Receiver<u8>>) {
    use std::io::Read;

    let guard = TerminalGuard::new();

    let Ok(mut tty) = std::fs::File::open("/dev/tty") else {
        return (guard, None);
    };

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = [0u8; 1];
        while let Ok(()) = tty.read_exact(&mut buf) {
            if tx.send(buf[0]).is_err() {
                break;
            }
        }
    });

    (guard, Some(rx))
}
