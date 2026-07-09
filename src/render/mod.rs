// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

mod labels;
mod layout;
mod progress;
mod views;

// Re-export animation for labels module access.
pub(crate) use crate::animation as animation_wrap;

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
    Tron,
}

impl ViewMode {
    /// Parse a view mode name (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "orbit" => Some(Self::Orbit),
            "cinematic" => Some(Self::Cinematic),
            "tron" => Some(Self::Tron),
            _ => None,
        }
    }
}

/// Snapshot of all render-relevant state, produced each frame by the main loop.
pub struct RenderState<'a> {
    pub mode: crate::mode::Mode,
    pub elapsed: Duration,
    pub remaining: Option<Duration>,
    pub progress: Option<f32>,
    pub state: crate::engine::EngineState,
    pub frame: u64,
    pub colors: &'a ColorFields,
    pub view: ViewMode,
    /// Current Pomodoro phase from Engine (only meaningful for Pomodoro mode).
    pub engine_phase: crate::mode::PomodoroPhase,
    /// Current Pomodoro cycle from Engine (only meaningful for Pomodoro mode).
    pub engine_cycle: u32,
}

// ─── Shared utility ──────────────────────────────────────────────────────────

/// Wrap text in an ANSI color code if code is non-empty; NO_COLOR safe.
pub(crate) fn colored(text: &str, code: &str, reset: &str) -> String {
    if code.is_empty() {
        text.to_string()
    } else {
        format!("{}{}{}", code, text, reset)
    }
}

// ─── Public entry point ──────────────────────────────────────────────────────

/// Render the current frame to stdout.
pub fn draw(state: &RenderState) {
    let output = match state.view {
        ViewMode::Minimal => views::draw_minimal(state),
        ViewMode::Orbit => views::draw_orbit(state),
        ViewMode::Cinematic => views::draw_cinematic(state),
        ViewMode::Tron => views::draw_tron(state),
    };
    let mut stdout = io::stdout();
    let _ = stdout.write_all(output.as_bytes());
    let _ = stdout.flush();
}

/// Compute `Some(0.0..=1.0)` for bounded modes, `None` for unbounded.
pub fn compute_progress(engine: &crate::engine::Engine) -> Option<f32> {
    progress::compute_progress(engine)
}
