// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::theme::ColorFields;
use std::io;

use super::colored;

// ─── Terminal size ──────────────────────────────────────────────────────────

pub(crate) fn terminal_size() -> (usize, usize) {
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

// ─── Display-width helpers ──────────────────────────────────────────────────

/// Compute the terminal display width of a string, ignoring ANSI escape sequences.
pub(crate) fn display_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        // Skip CSI sequences: ESC [ ... m
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
            continue;
        }
        width += char_display_width(ch);
    }
    width
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

/// Strip ANSI CSI escape sequences from a string.
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

// ─── Layout ─────────────────────────────────────────────────────────────────

pub(crate) fn layout_box(lines: &[String], c: &ColorFields) -> String {
    let (cols, rows) = terminal_size();
    let max_inner = cols.saturating_sub(6).max(10);
    let (boxed, _, _) = boxed_centered(lines, max_inner, cols, rows, c);
    format!("\x1b[2J\x1b[H{}", boxed)
}

fn boxed_centered(
    lines: &[String],
    max_inner: usize,
    cols: usize,
    rows: usize,
    c: &ColorFields,
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

    // Top border
    out.push_str(&margin);
    out.push_str(&colored("\u{250C}", c.border, c.reset));
    out.push_str(&colored(&"\u{2500}".repeat(inside), c.border, c.reset));
    out.push_str(&colored("\u{2510}", c.border, c.reset));
    out.push('\n');

    // Content lines
    for line in &wrapped {
        let len = display_width(line);
        let remaining = inner_width.saturating_sub(len);
        let left_extra = remaining / 2;
        let right_spaces = remaining - left_extra;
        out.push_str(&margin);
        out.push_str(&colored("\u{2502}", c.border, c.reset));
        out.push_str(&" ".repeat(pad));
        out.push_str(&" ".repeat(left_extra));
        out.push_str(line);
        out.push_str(&" ".repeat(right_spaces));
        out.push_str(&" ".repeat(pad));
        out.push_str(&colored("\u{2502}", c.border, c.reset));
        out.push('\n');
    }

    // Bottom border
    out.push_str(&margin);
    out.push_str(&colored("\u{2514}", c.border, c.reset));
    out.push_str(&colored(&"\u{2500}".repeat(inside), c.border, c.reset));
    out.push_str(&colored("\u{2518}", c.border, c.reset));
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
                    // Strip ANSI codes and split plain text character by character.
                    let plain = strip_ansi(word);
                    let mut chunk = String::new();
                    let mut chunk_width = 0usize;
                    for ch in plain.chars() {
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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_width_plain_text() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("hello world"), 11);
    }

    #[test]
    fn display_width_empty() {
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn display_width_ignores_ansi() {
        let colored_text = "\x1b[32mgreen\x1b[0m";
        assert_eq!(display_width(colored_text), 5);
    }

    #[test]
    fn display_width_wide_char() {
        // 'A' = 1, '中' = 2
        assert_eq!(display_width("A中"), 3);
    }

    #[test]
    fn display_width_emoji() {
        // Each emoji is 2 columns wide in terminal
        assert_eq!(display_width("AB"), 2);
    }

    #[test]
    fn strip_ansi_removes_csi() {
        let input = "\x1b[32mgreen\x1b[0m text";
        assert_eq!(strip_ansi(input), "green text");
    }

    #[test]
    fn strip_ansi_plain_unchanged() {
        assert_eq!(strip_ansi("no codes here"), "no codes here");
    }

    #[test]
    fn terminal_size_has_minimum() {
        let (c, r) = terminal_size();
        assert!(c >= 20);
        assert!(r >= 10);
    }

    #[test]
    fn wrap_lines_basic() {
        let lines = vec!["hello world this is long".to_string()];
        let wrapped = wrap_lines(&lines, 10);
        // Should have split into multiple lines
        assert!(wrapped.len() >= 2);
        for w in &wrapped {
            assert!(display_width(w) <= 10);
        }
    }

    #[test]
    fn wrap_lines_empty_input() {
        let lines: Vec<String> = vec![];
        let wrapped = wrap_lines(&lines, 40);
        assert!(wrapped.is_empty());
    }

    #[test]
    fn wrap_lines_preserves_blank_lines() {
        let lines = vec![String::new(), "hi".to_string(), String::new()];
        let wrapped = wrap_lines(&lines, 40);
        assert_eq!(wrapped.len(), 3);
        assert!(wrapped[0].is_empty());
        assert_eq!(wrapped[1], "hi");
        assert!(wrapped[2].is_empty());
    }
}
