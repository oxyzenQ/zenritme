// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

use crate::animation;

use super::colored;
use super::labels;
use super::layout;
use super::progress::colored_bar;
use super::RenderState;

// ─── View: Minimal ──────────────────────────────────────────────────────────

pub(crate) fn draw_minimal(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;
    let bar_w = (layout::terminal_size().0 / 3).clamp(10, 50);

    let mut lines: Vec<String> = vec![
        super::colored(&labels::build_title(state), c.title, r),
        String::new(),
        super::colored(&labels::build_time(state), c.time, r),
        String::new(),
    ];

    // Progress bar
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, bar_w, c));
        lines.push(String::new());
    }

    // Mode-specific info
    labels::push_mode_info(&mut lines, state);

    // State label
    labels::push_state_label(&mut lines, state, c, r);

    // Control hints
    labels::push_control_hints(&mut lines, state, c, r);

    layout::layout_box(&lines, c)
}

// ─── View: Orbit ──────────────────────────────────────────────────────────────

pub(crate) fn draw_orbit(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;
    let bar_w = (layout::terminal_size().0 / 3).clamp(10, 50);

    let mut lines: Vec<String> = Vec::new();

    // Orbit decoration
    lines.push(super::colored(&animation::orbit(state.frame), c.spinner, r));
    lines.push(String::new());

    // Title with spinner
    let sp = animation::spinner(state.frame);
    let title_text = format!("{}  {}  {}", sp, labels::build_title(state), sp);
    lines.push(super::colored(&title_text, c.title, r));
    lines.push(String::new());

    // Time
    lines.push(super::colored(&labels::build_time(state), c.time, r));
    lines.push(String::new());

    // Progress bar
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, bar_w, c));
    }

    // Mode-specific info
    labels::push_mode_info(&mut lines, state);

    // State label
    labels::push_state_label(&mut lines, state, c, r);

    // Control hints
    labels::push_control_hints(&mut lines, state, c, r);

    layout::layout_box(&lines, c)
}

// ─── View: Cinematic ─────────────────────────────────────────────────────────

pub(crate) fn draw_cinematic(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;
    let bar_w = (layout::terminal_size().0 / 3).clamp(10, 50);

    let mut lines: Vec<String> = vec![
        super::colored(&animation::orbit(state.frame), c.spinner, r),
        String::new(),
        String::new(),
        super::colored(&labels::build_title(state), c.title, r),
        String::new(),
        String::new(),
        super::colored(&labels::build_time(state), c.time, r),
        String::new(),
    ];

    // Progress bar (dynamic width)
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, bar_w, c));
        lines.push(String::new());
    }

    // Mode-specific info
    labels::push_mode_info(&mut lines, state);

    lines.push(String::new());

    // Bottom orbit
    lines.push(super::colored(&animation::orbit(state.frame), c.spinner, r));
    lines.push(String::new());

    // State label
    labels::push_state_label(&mut lines, state, c, r);

    // Control hints
    labels::push_control_hints(&mut lines, state, c, r);

    layout::layout_box(&lines, c)
}

// ─── View: Tron Legacy ──────────────────────────────────────────────────────
//
// Perspective grid inspired by the Tron Legacy (2010) film:
// - Horizontal lines with perspective compression toward a vanishing point
// - Vertical lines that are evenly spaced but also converge with perspective
// - A "light trail" scan-line that sweeps the grid like a light cycle
// - Glow effect on the horizon (brighter center line)
// - Dark, atmospheric — minimal UI floating above the grid

pub(crate) fn draw_tron(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let (cols, rows) = layout::terminal_size();
    let w = cols.saturating_sub(4);
    let grid_rows = rows.saturating_sub(10).max(8);

    // Vertical line count (3 to 8 vertical lines depending on width)
    let vline_count = (w / 14).clamp(3, 8);

    // Perspective grid: horizontal lines compressed toward horizon
    let h_line_count = 18;

    // Build the grid as raw lines (no layout_box — we handle our own frame)
    let mut out = String::from("\x1b[2J\x1b[H");
    let top_pad = 2;
    for _ in 0..top_pad {
        out.push('\n');
    }

    // ── Sky / title area (above the grid) ──────────────────────────────────
    let margin = " ".repeat(2);

    let title_text = labels::build_title(state);
    let title_w = layout::display_width(&title_text);
    let title_left = w.saturating_sub(title_w) / 2;
    out.push_str(&margin);
    out.push_str(&" ".repeat(title_left));
    out.push_str(&colored(&title_text, c.title, r));
    out.push('\n');
    out.push('\n');

    // ── Time display ───────────────────────────────────────────────────────
    let time_text = labels::build_time(state);
    let time_w = layout::display_width(&time_text);
    let time_left = w.saturating_sub(time_w) / 2;
    out.push_str(&margin);
    out.push_str(&" ".repeat(time_left));
    out.push_str(&colored(&time_text, c.time, r));
    out.push('\n');

    // ── Progress bar ───────────────────────────────────────────────────────
    if let Some(p) = state.progress {
        let bar_w = (w / 3).clamp(10, 50);
        let bar = colored_bar(p, bar_w, c);
        let bar_vis_w = layout::display_width(&bar);
        let bar_left = w.saturating_sub(bar_vis_w) / 2;
        out.push('\n');
        out.push_str(&margin);
        out.push_str(&" ".repeat(bar_left));
        out.push_str(&bar);
        out.push('\n');
    }

    out.push('\n');

    // ── Perspective grid ───────────────────────────────────────────────────

    // Scrolling offset — grid lines drift toward the viewer over time
    let scroll = (state.frame / 3) as f32;
    let scroll_f = scroll.fract();

    // Light trail position (scans across the grid repeatedly)
    let trail_cycle = 60.0;
    let trail_pos = ((state.frame as f32 / trail_cycle) % 1.0) * grid_rows as f32;

    for row_i in 0..grid_rows {
        // Normalized row: 0 = top (far/horizon), 1 = bottom (near/viewer)
        let t = (row_i as f32 + 1.0) / (grid_rows as f32 + 1.0);

        // Perspective factor: quadratic compression toward top
        // At t=0 (horizon) perspective = 0, at t=1 (near) perspective = 1
        let persp = t * t;

        // Scrolling horizontal line: space between lines grows with perspective
        let h_spacing = h_line_count as f32;
        let h_pos = (persp * h_spacing + scroll_f) % h_spacing;
        let near_h_line = h_pos.fract() < 0.10;

        // Light trail: a bright band that sweeps vertically through the grid
        let trail_dist = (row_i as f32 - trail_pos).abs();
        let is_trail = trail_dist < 1.0;

        // Vertical line x-positions for this row (converge toward center
        // near the horizon, spread out near the viewer)
        let vlines: Vec<usize> = (0..vline_count)
            .map(|vi| {
                let norm = if vline_count > 1 {
                    (vi as f32 / (vline_count as f32 - 1.0)) - 0.5
                } else {
                    0.0
                };
                let cx = w as f32 / 2.0;
                let spread = norm * w as f32 * 0.9 * persp;
                (cx + spread).round() as usize
            })
            .collect();

        // Build the grid line
        let mut line = String::with_capacity(w + 80);
        line.push_str(&margin);

        for col_i in 0..w {
            let is_vline = vlines.iter().any(|&vx| vx == col_i);

            // Center column — always drawn, slightly brighter (main axis)
            let center = w / 2;
            let is_center = col_i == center;

            if is_trail {
                // Trail row: glowing line with accent color
                if is_vline || is_center {
                    line.push_str(&colored("|", c.accent, r));
                } else if col_i % 2 == 0 {
                    line.push_str(&colored("-", c.progress_fill, r));
                } else {
                    line.push(' ');
                }
            } else if near_h_line || is_vline || is_center {
                line.push_str(&colored("|", c.border, r));
            } else {
                line.push(' ');
            }
        }

        out.push_str(&line);
        out.push('\n');
    }

    // ── Bottom UI: mode info, state label, controls ────────────────────────
    out.push('\n');

    let mut ui_lines: Vec<String> = Vec::new();
    labels::push_mode_info(&mut ui_lines, state);
    labels::push_state_label(&mut ui_lines, state, c, r);
    labels::push_control_hints(&mut ui_lines, state, c, r);

    for ui in &ui_lines {
        if ui.is_empty() {
            out.push('\n');
        } else {
            let ui_w = layout::display_width(ui);
            let ui_left = w.saturating_sub(ui_w) / 2;
            out.push_str(&margin);
            out.push_str(&" ".repeat(ui_left));
            out.push_str(ui);
            out.push('\n');
        }
    }

    out
}
