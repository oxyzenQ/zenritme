// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

use crate::animation;
use crate::engine::EngineState;
use crate::mode::{Mode, PomodoroPhase};
use crate::theme::ColorFields;
use std::io::{self, Write};
use std::time::Duration;

// ─── Public types ────────────────────────────────────────────────────────────

/// View mode — controls how the timer is displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Minimal,
    Orbit,
    Cinematic,
}

impl ViewMode {
    /// Parse a view mode name (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "orbit" => Some(Self::Orbit),
            "cinematic" => Some(Self::Cinematic),
            _ => None,
        }
    }
}

/// Snapshot of all render-relevant state, produced each frame by the main loop.
#[allow(dead_code)]
pub struct RenderState<'a> {
    pub mode: Mode,
    pub elapsed: Duration,
    pub remaining: Option<Duration>,
    pub total: Option<Duration>,
    pub progress: Option<f32>,
    pub state: EngineState,
    pub frame: u64,
    pub colors: &'a ColorFields,
    pub view: ViewMode,
}

// ─── Public entry point ────────────────────────────────────────────────────

/// Render the current frame to stdout.
pub fn draw(state: &RenderState) {
    let output = match state.view {
        ViewMode::Minimal => draw_minimal(state),
        ViewMode::Orbit => draw_orbit(state),
        ViewMode::Cinematic => draw_cinematic(state),
    };
    let mut stdout = io::stdout();
    let _ = stdout.write_all(output.as_bytes());
    let _ = stdout.flush();
}

/// Compute `Some(0.0..=1.0)` for bounded modes, `None` for unbounded.
pub fn compute_progress(engine: &crate::engine::Engine) -> Option<f32> {
    match engine.mode() {
        Mode::TimerDown { total } => {
            if total.is_zero() {
                None
            } else {
                Some((engine.elapsed().as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0))
            }
        }
        Mode::Pomodoro {
            phase,
            focus,
            short_break,
            ..
        } => {
            let phase_total = match phase {
                PomodoroPhase::Focus => focus,
                PomodoroPhase::Break => short_break,
            };
            let remaining = engine.remaining()?;
            if phase_total.is_zero() {
                None
            } else {
                let used = phase_total.saturating_sub(remaining);
                Some((used.as_secs_f32() / phase_total.as_secs_f32()).clamp(0.0, 1.0))
            }
        }
        _ => None,
    }
}

// ─── View: Minimal ──────────────────────────────────────────────────────────

fn draw_minimal(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let mut lines: Vec<String> = vec![
        colored(&build_title(state), c.title, r),
        String::new(),
        colored(&build_time(state), c.time, r),
        String::new(),
    ];

    // Progress bar
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, 20, c));
        lines.push(String::new());
    }

    // Mode-specific info
    push_mode_info(&mut lines, state);

    // State label
    push_state_label(&mut lines, state, c, r);

    layout_box(&lines, c)
}

// ─── View: Orbit ─────────────────────────────────────────────────────────────

fn draw_orbit(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let mut lines: Vec<String> = Vec::new();

    // Orbit decoration
    lines.push(colored(&animation::orbit(state.frame), c.spinner, r));
    lines.push(String::new());

    // Title with spinner
    let sp = animation::spinner(state.frame);
    let title_text = format!("{}  {}  {}", sp, build_title(state), sp);
    lines.push(colored(&title_text, c.title, r));
    lines.push(String::new());

    // Time
    lines.push(colored(&build_time(state), c.time, r));
    lines.push(String::new());

    // Progress bar
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, 20, c));
    }

    // Mode-specific info
    push_mode_info(&mut lines, state);

    // State label
    push_state_label(&mut lines, state, c, r);

    layout_box(&lines, c)
}

// ─── View: Cinematic ────────────────────────────────────────────────────────

fn draw_cinematic(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let mut lines: Vec<String> = vec![
        colored(&animation::orbit(state.frame), c.spinner, r),
        String::new(),
        String::new(),
        colored(&build_title(state), c.title, r),
        String::new(),
        String::new(),
        colored(&build_time(state), c.time, r),
        String::new(),
    ];

    // Progress bar (wider)
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, 24, c));
        lines.push(String::new());
    }

    // Mode-specific info
    push_mode_info(&mut lines, state);

    lines.push(String::new());

    // Bottom orbit
    lines.push(colored(&animation::orbit(state.frame), c.spinner, r));
    lines.push(String::new());

    // State label
    push_state_label(&mut lines, state, c, r);

    layout_box(&lines, c)
}

// ─── Shared builders ──────────────────────────────────────────────────────────

fn build_title(state: &RenderState) -> String {
    match state.mode {
        Mode::TimerUp => "TIMER UP".to_string(),
        Mode::TimerDown { .. } => "TIMER DOWN".to_string(),
        Mode::Stopwatch => "STOPWATCH".to_string(),
        Mode::Pomodoro { phase, emoji, .. } => {
            let base = match phase {
                PomodoroPhase::Focus => "POMODORO FOCUS",
                PomodoroPhase::Break => "POMODORO BREAK",
            };
            let dyn_idx = emoji.wrapping_add((state.elapsed.as_secs() / 5) as u8);
            format!("{} {}", base, pomodoro_emoji(dyn_idx))
        }
    }
}

fn build_time(state: &RenderState) -> String {
    let primary = match state.mode {
        Mode::TimerDown { .. } | Mode::Pomodoro { .. } => state.remaining.unwrap_or_default(),
        _ => state.elapsed,
    };
    format_hms(primary)
}

fn push_mode_info(lines: &mut Vec<String>, state: &RenderState) {
    match state.mode {
        Mode::TimerDown { .. } => {
            lines.push(format!("Elapsed: {}", format_hms(state.elapsed)));
        }
        Mode::Pomodoro { .. } => {
            lines.push(format!("Session: {}", format_hms(state.elapsed)));
        }
        _ => {}
    }
}

fn push_state_label(lines: &mut Vec<String>, state: &RenderState, c: &ColorFields, r: &str) {
    match state.state {
        EngineState::Paused => {
            lines.push(String::new());
            lines.push(colored("[ PAUSED ]", c.accent, r));
        }
        EngineState::Completed => {
            lines.push(String::new());
            let burst = animation::completion_burst(state.frame);
            lines.push(colored(&format!("[ DONE ] {}", burst), c.accent, r));
        }
        EngineState::Running => {}
    }
}

fn colored_bar(progress: f32, width: usize, c: &ColorFields) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = ((clamped * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!(
        "[{}{}{}] {:3.0}%",
        colored(&"\u{2588}".repeat(filled), c.progress_fill, c.reset),
        colored(&"\u{2591}".repeat(empty), c.progress_empty, c.reset),
        c.reset,
        clamped * 100.0,
    )
}

fn colored(text: &str, code: &str, reset: &str) -> String {
    if code.is_empty() {
        text.to_string()
    } else {
        format!("{}{}{}", code, text, reset)
    }
}

// ─── Layout ─────────────────────────────────────────────────────────────────

fn layout_box(lines: &[String], c: &ColorFields) -> String {
    let (cols, rows) = terminal_size();
    let max_inner = cols.saturating_sub(6).max(10);
    let (boxed, _, _) = boxed_centered(lines, max_inner, cols, rows, c);
    format!("\x1b[2J\x1b[H{}", boxed)
}

// ─── Box drawing ─────────────────────────────────────────────────────────────

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

// ─── Display-width helpers ──────────────────────────────────────────────────

/// Compute the terminal display width of a string, ignoring ANSI escape sequences.
fn display_width(s: &str) -> usize {
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
fn strip_ansi(s: &str) -> String {
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

// ─── Terminal size ──────────────────────────────────────────────────────────

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

// ─── Time formatting ──────────────────────────────────────────────────────

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

// ─── Pomodoro emoji ────────────────────────────────────────────────────────

fn pomodoro_emoji(idx: u8) -> &'static str {
    const EMOJIS: [&str; 10] = [
        "\u{1F345}", // 🍅
        "\u{2615}",  // ☕
        "\u{1F319}", // 🌙
        "\u{26A1}",  // ⚡
        "\u{1F9E0}", // 🧠
        "\u{1F3A7}", // 🎧
        "\u{1F33F}", // 🌿
        "\u{1F4CC}", // 📌
        "\u{1F525}", // 🔥
        "\u{1F56F}", // 🕯️
    ];
    EMOJIS[(idx as usize) % EMOJIS.len()]
}
