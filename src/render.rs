// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::engine::EngineState;
use crate::mode::{Mode, PomodoroPhase};
use std::io::{self, Write};
use std::time::Duration;

/// Renders the current timer state to stdout.
///
/// - `state`    — `Running`, `Paused`, or `Completed`; controls the status label.
/// - `progress` — `Some(0.0..=1.0)` for bounded modes; `None` for unbounded.
pub fn draw(
    mode: Mode,
    elapsed: Duration,
    remaining: Option<Duration>,
    state: EngineState,
    progress: Option<f32>,
) {
    // ── Title ─────────────────────────────────────────────────────────────────
    let title = match mode {
        Mode::TimerUp => "TIMER UP".to_string(),
        Mode::TimerDown { .. } => "TIMER DOWN".to_string(),
        Mode::Stopwatch => "STOPWATCH".to_string(),
        Mode::Pomodoro { phase, emoji, .. } => {
            let base = match phase {
                PomodoroPhase::Focus => "POMODORO FOCUS",
                PomodoroPhase::Break => "POMODORO BREAK",
            };
            let dyn_idx = emoji.wrapping_add((elapsed.as_secs() / 5) as u8);
            format!("{} {}", base, pomodoro_emoji(dyn_idx))
        }
    };

    // ── Primary time display ──────────────────────────────────────────────────
    let primary = match mode {
        Mode::TimerDown { .. } | Mode::Pomodoro { .. } => remaining.unwrap_or_default(),
        _ => elapsed,
    };
    let time_str = format_hms(primary);

    // ── Build line list ───────────────────────────────────────────────────────
    let mut lines: Vec<String> = vec!["ZENRITME".to_string(), title, String::new(), time_str];

    match mode {
        Mode::TimerDown { .. } => {
            lines.push(String::new());
            lines.push(format!("Elapsed: {}", format_hms(elapsed)));
        }
        Mode::Pomodoro { .. } => {
            lines.push(String::new());
            lines.push(format!("Session: {}", format_hms(elapsed)));
        }
        _ => {}
    }

    // ── Progress bar (bounded modes only) ────────────────────────────────────
    if let Some(p) = progress {
        lines.push(String::new());
        lines.push(progress_bar(p, 20));
    }

    // ── State label ───────────────────────────────────────────────────────────
    let state_label = match state {
        EngineState::Paused => Some("[ PAUSED ]"),
        EngineState::Completed => Some("[ DONE ]"),
        EngineState::Running => None,
    };
    if let Some(label) = state_label {
        lines.push(String::new());
        lines.push(label.to_string());
    }

    // ── Render into a centred box ─────────────────────────────────────────────
    let (cols, rows) = terminal_size();
    let max_inner = cols.saturating_sub(6).max(10);
    let (boxed, _, _) = boxed_centered(&lines, max_inner, cols, rows);

    let mut out = String::new();
    out.push_str("\x1b[2J\x1b[H");
    out.push_str(&boxed);

    let mut stdout = io::stdout();
    let _ = stdout.write_all(out.as_bytes());
    let _ = stdout.flush();
}

// ─── Progress bar ─────────────────────────────────────────────────────────────

/// Returns a fixed-width progress bar string.
///
/// `[████████░░░░░░░░░░░░]  42%`
///
/// `█` (U+2588) and `░` (U+2591) are each 1 column wide in most terminal fonts.
fn progress_bar(progress: f32, width: usize) -> String {
    let filled = ((progress * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!(
        "[{}{}] {:3.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        progress * 100.0,
    )
}

// ─── Pomodoro emoji ───────────────────────────────────────────────────────────

fn pomodoro_emoji(idx: u8) -> &'static str {
    const EMOJIS: [&str; 10] = ["🍅", "☕", "🌙", "⚡", "🧠", "🎧", "🌿", "📌", "🔥", "🕯️"];
    EMOJIS[(idx as usize) % EMOJIS.len()]
}

// ─── Display-width helpers ────────────────────────────────────────────────────

fn display_width(s: &str) -> usize {
    s.chars().map(char_display_width).sum()
}

fn char_display_width(ch: char) -> usize {
    let u = ch as u32;
    if u == 0 {
        return 0;
    }
    // Zero-width joiner and variation selectors
    if u == 0x200D || (0xFE00..=0xFE0F).contains(&u) {
        return 0;
    }
    // Combining marks
    if (0x0300..=0x036F).contains(&u)
        || (0x1AB0..=0x1AFF).contains(&u)
        || (0x1DC0..=0x1DFF).contains(&u)
        || (0x20D0..=0x20FF).contains(&u)
        || (0xFE20..=0xFE2F).contains(&u)
    {
        return 0;
    }
    // Wide characters (CJK, emoji blocks, …)
    if (0x1100..=0x115F).contains(&u)
        || u == 0x2329
        || u == 0x232A
        || (0x2E80..=0xA4CF).contains(&u)
        || (0xAC00..=0xD7A3).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0xFE10..=0xFE19).contains(&u)
        || (0xFE30..=0xFE6F).contains(&u)
        || (0xFF00..=0xFF60).contains(&u)
        || (0xFFE0..=0xFFE6).contains(&u)
        || (0x2600..=0x27BF).contains(&u)
        || (0x1F1E6..=0x1F1FF).contains(&u)
        || (0x1F300..=0x1FAFF).contains(&u)
    {
        return 2;
    }
    1
}

// ─── Terminal size ────────────────────────────────────────────────────────────

fn terminal_size() -> (usize, usize) {
    #[cfg(target_os = "linux")]
    {
        if let Some((c, r)) = terminal_size_linux_ioctl() {
            return (c.max(20), r.max(10));
        }
    }
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(80);
    let rows = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(24);
    (cols.max(20), rows.max(10))
}

#[cfg(target_os = "linux")]
fn terminal_size_linux_ioctl() -> Option<(usize, usize)> {
    use std::os::fd::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    extern "C" {
        fn ioctl(fd: i32, request: u64, ...) -> i32;
    }

    const TIOCGWINSZ: u64 = 0x5413;
    let fd = io::stdout().as_raw_fd();
    let mut ws = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let rc = unsafe { ioctl(fd, TIOCGWINSZ, &mut ws) };
    if rc == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some((ws.ws_col as usize, ws.ws_row as usize))
    } else {
        None
    }
}

// ─── Box drawing ──────────────────────────────────────────────────────────────

fn boxed_centered(
    lines: &[String],
    max_inner: usize,
    cols: usize,
    rows: usize,
) -> (String, usize, usize) {
    let wrapped = wrap_lines(lines, max_inner);
    let inner_width = wrapped
        .iter()
        .map(|l| display_width(l))
        .max()
        .unwrap_or(0)
        .max(1);

    let pad = 1usize;
    let inside = inner_width + pad * 2;
    let box_width = inside + 2;
    let box_height = wrapped.len() + 2;

    let left = cols.saturating_sub(box_width) / 2;
    let top = rows.saturating_sub(box_height) / 2;

    let mut out = String::new();
    for _ in 0..top {
        out.push('\n');
    }

    let margin = " ".repeat(left);

    out.push_str(&margin);
    out.push('┌');
    out.push_str(&"─".repeat(inside));
    out.push('┐');
    out.push('\n');

    for line in wrapped {
        let len = display_width(&line);
        let remaining = inner_width.saturating_sub(len);
        let left_extra = remaining / 2;
        let right_spaces = remaining - left_extra;
        out.push_str(&margin);
        out.push('│');
        out.push_str(&" ".repeat(pad));
        out.push_str(&" ".repeat(left_extra));
        out.push_str(&line);
        out.push_str(&" ".repeat(right_spaces));
        out.push_str(&" ".repeat(pad));
        out.push('│');
        out.push('\n');
    }

    out.push_str(&margin);
    out.push('└');
    out.push_str(&"─".repeat(inside));
    out.push('┘');
    out.push('\n');

    (out, box_width, box_height)
}

fn wrap_lines(lines: &[String], width: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut current_width = 0usize;
        for word in line.split_whitespace() {
            let word_len = display_width(word);
            let sep = if current.is_empty() { 0 } else { 1 };
            if current_width + sep + word_len <= width {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
                current_width += sep + word_len;
            } else {
                if !current.is_empty() {
                    out.push(current);
                }
                if word_len <= width {
                    current = word.to_string();
                    current_width = word_len;
                } else {
                    let mut chunk = String::new();
                    let mut chunk_width = 0usize;
                    for ch in word.chars() {
                        let ch_w = char_display_width(ch);
                        if chunk_width + ch_w > width && chunk_width > 0 {
                            out.push(chunk);
                            chunk = String::new();
                            chunk_width = 0;
                        }
                        chunk.push(ch);
                        chunk_width += ch_w;
                    }
                    current = chunk;
                    current_width = display_width(&current);
                }
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}

// ─── Time formatting ──────────────────────────────────────────────────────────

fn format_hms(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
