// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

use crate::animation;

use super::labels;
use super::layout;
use super::progress::colored_bar;
use super::RenderState;

// ─── View: Minimal ──────────────────────────────────────────────────────────

pub(crate) fn draw_minimal(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

    let mut lines: Vec<String> = vec![
        super::colored(&labels::build_title(state), c.title, r),
        String::new(),
        super::colored(&labels::build_time(state), c.time, r),
        String::new(),
    ];

    // Progress bar
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, 20, c));
        lines.push(String::new());
    }

    // Mode-specific info
    labels::push_mode_info(&mut lines, state);

    // State label
    labels::push_state_label(&mut lines, state, c, r);

    layout::layout_box(&lines, c)
}

// ─── View: Orbit ──────────────────────────────────────────────────────────────

pub(crate) fn draw_orbit(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

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
        lines.push(colored_bar(p, 20, c));
    }

    // Mode-specific info
    labels::push_mode_info(&mut lines, state);

    // State label
    labels::push_state_label(&mut lines, state, c, r);

    layout::layout_box(&lines, c)
}

// ─── View: Cinematic ─────────────────────────────────────────────────────────

pub(crate) fn draw_cinematic(state: &RenderState) -> String {
    let c = state.colors;
    let r = c.reset;

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

    // Progress bar (wider)
    if let Some(p) = state.progress {
        lines.push(colored_bar(p, 24, c));
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

    layout::layout_box(&lines, c)
}
