# Zenritme Future Roadmap

> **Status:** v10.0.0 released. This document is for future development.
> **Last updated:** 2026-07-01
> **Maintainer:** rezky_nightky (oxyzenQ)

---

## Locked Principles

| Principle | Detail |
|-----------|--------|
| **Zero dependencies** | Pure std — no external crates |
| **Linux first** | amd64 Linux binaries (gnu + musl) |
| **Calm UX** | Never stressful, never rushed |
| **CPU efficient** | Adaptive tick rate, dirty tracking |
| **Under 1000 LOC per file** | LOC guard enforced |

---

## Completed

| Version | Focus | Highlights |
|---------|-------|-----------|
| v10.0.0 | Peak Performance | Adaptive tick (6x CPU reduction), dirty render, musl binary |

---

## Future Phases

### Phase 1: v10.1.0 — Polish & Features

| Feature | Complexity |
|---------|-----------|
| Session logging (JSONL to ~/.local/share/zenritme/) | Low |
| Statistics dashboard (total focus time today/week) | Medium |
| Custom sound file override via env vars | Low |
| Colorblind mode (shape-based phase indicators) | Low |
| Config file (~/.config/zenritme/config.toml) | Medium |

### Phase 2: v10.2.0 — Integration

| Feature | Complexity |
|---------|-----------|
| Desktop notifications (notify-send on phase complete) | Low |
| Prometheus metrics export (--metrics) | Low |
| Shell integration (tmux status bar, waybar module) | Medium |
| i3/sway integration (workspace marker) | Low |

### Phase 3: v11.0.0 — Ecosystem

| Feature | Complexity |
|---------|-----------|
| Plugin system (Lua scripts for custom sounds/themes) | High |
| Community theme marketplace | Medium |
| Web dashboard (local HTTP server for stats) | Medium |
| Cross-platform (macOS, FreeBSD) | Medium |

---

## Explicitly Rejected

| Feature | Why |
|---------|-----|
| ~~GUI (Qt/GTK)~~ | Terminal tool, TUI is the UX |
| ~~Background daemon~~ | Interactive tool, not a service |
| ~~Cloud sync~~ | Local tool, not cloud |
| ~~External dependencies~~ | Zero deps is a feature |
