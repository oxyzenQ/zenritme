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
// - Clear horizon line separating dark sky (above) from the grid floor (below)
// - Vertical lines (│) converge to the center at the horizon
// - Horizontal lines (─) span full width, spacing compressed by perspective
// - Grid scrolls toward the viewer (animated forward motion)
// - A "light trail" — bright accent-colored horizontal line — sweeps across
//   the grid like a light cycle on the Grid
// - Title and time float in the dark sky above the horizon

pub(crate) fn draw_tron(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let (cols, rows) = layout::terminal_size();
    let w = cols.saturating_sub(4);

    // Horizon position: ~30% from top — dark sky above, grid below
    let horizon = (rows as f32 * 0.30).round() as usize;
    let grid_rows = rows.saturating_sub(horizon + 4).max(4);

    // Vertical line count: adaptive to terminal width
    let vline_count = (w / 12).clamp(3, 12);

    // How many perspective-spaced horizontal lines
    let h_line_count: f32 = 16.0;

    // Scroll offset: grid lines drift toward the viewer
    let scroll_f = ((state.frame / 3) as f32).fract();

    // Light trail: bright band sweeping vertically through the grid
    let trail_cycle = 80.0;
    let trail_pos = ((state.frame as f32 / trail_cycle) % 1.0) * grid_rows as f32;

    // ── Build frame ────────────────────────────────────────────────────────
    let mut out = String::from("\x1b[2J\x1b[H");
    let margin = " ".repeat(2);

    // ── Sky (above horizon): title, time, progress ────────────────────────
    let sky_rows = horizon.saturating_sub(5).max(1);

    // Title — centered in the sky
    let title_text = labels::build_title(state);
    let title_w = layout::display_width(&title_text);
    let title_left = w.saturating_sub(title_w) / 2;
    for _ in 0..sky_rows.saturating_sub(2) {
        out.push('\n');
    }
    out.push_str(&margin);
    out.push_str(&" ".repeat(title_left));
    out.push_str(&colored(&title_text, c.title, r));
    out.push('\n');

    // Time — centered below title
    let time_text = labels::build_time(state);
    let time_w = layout::display_width(&time_text);
    let time_left = w.saturating_sub(time_w) / 2;
    out.push_str(&margin);
    out.push_str(&" ".repeat(time_left));
    out.push_str(&colored(&time_text, c.time, r));
    out.push('\n');

    // Progress bar — centered below time
    if let Some(p) = state.progress {
        let bar_w = (w / 3).clamp(10, 50);
        let bar = colored_bar(p, bar_w, c);
        let bar_vis_w = layout::display_width(&bar);
        let bar_left = w.saturating_sub(bar_vis_w) / 2;
        out.push_str(&margin);
        out.push_str(&" ".repeat(bar_left));
        out.push_str(&bar);
        out.push('\n');
    }

    // Pad to horizon
    let used_sky = 2 + if state.progress.is_some() { 1 } else { 0 };
    for _ in 0..horizon.saturating_sub(used_sky) {
        out.push('\n');
    }

    // ── Horizon line — bright, full-width ──────────────────────────────────
    out.push_str(&margin);
    let horiz_line = "\u{2500}".repeat(w);
    out.push_str(&colored(&horiz_line, c.accent, r));
    out.push('\n');

    // ── Grid (below horizon) ──────────────────────────────────────────────
    let h_dash = "\u{2500}"; // ─
    let v_bar = "\u{2502}";  // │

    for row_i in 0..grid_rows {
        // Normalized: 0 = just below horizon (far), 1 = bottom (near viewer)
        let t = (row_i as f32 + 1.0) / (grid_rows as f32 + 1.0);

        // Quadratic perspective: 0 at horizon, 1 at bottom
        let persp = t * t;

        // Scroll: horizontal lines move toward viewer over time
        let h_pos = (persp * h_line_count + scroll_f) % h_line_count;
        let is_h_line = h_pos.fract() < 0.12;

        // Light trail: a bright horizontal band sweeping through grid
        let trail_dist = (row_i as f32 - trail_pos).abs();
        let is_trail = trail_dist < 0.8;

        // Vertical line x-positions: converge to center at horizon
        let vlines: Vec<usize> = (0..vline_count)
            .map(|vi| {
                let norm = if vline_count > 1 {
                    (vi as f32 / (vline_count as f32 - 1.0)) - 0.5
                } else {
                    0.0
                };
                let cx = w as f32 / 2.0;
                let spread = norm * w as f32 * 0.92 * persp;
                (cx + spread).round() as usize
            })
            .collect();

        // Center column position
        let center = w / 2;

        let mut line = String::with_capacity(w * 4 + 80);
        line.push_str(&margin);

        if is_trail {
            // ── Trail row: bright accent-colored horizontal sweep ──────────
            // Full-width bright line, vertical intersections get │
            for col_i in 0..w {
                if vlines.contains(&col_i) || col_i == center {
                    line.push_str(&colored(v_bar, c.accent, r));
                } else {
                    line.push_str(&colored(h_dash, c.accent, r));
                }
            }
        } else if is_h_line {
            // ── Horizontal grid line ───────────────────────────────────────
            for col_i in 0..w {
                if vlines.contains(&col_i) || col_i == center {
                    line.push_str(&colored(v_bar, c.border, r));
                } else {
                    line.push_str(&colored(h_dash, c.border, r));
                }
            }
        } else {
            // ── Vertical-only row (spaces with │ at vline positions) ──────
            for col_i in 0..w {
                if vlines.contains(&col_i) || col_i == center {
                    line.push_str(&colored(v_bar, c.border, r));
                } else {
                    line.push(' ');
                }
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
