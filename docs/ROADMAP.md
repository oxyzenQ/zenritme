# Zenritme Future Roadmap

> **Status:** v11.0.0 released. This document is for future development.
> **Last updated:** 2026-07-09
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
| **IP protected** | Trademark + copyright owned by rezky_nightky (oxyzenQ) |

---

## Completed

| Version | Focus | Highlights |
|---------|-------|-----------|
| v10.0.0 | Peak Performance | Adaptive tick (6x CPU reduction), dirty render, musl binary |
| v11.0.0 | Tron Legacy + IP Hardening | Tron Legacy grid view, 6 Tron color themes, sound system v3, full IP/trademark protection, whitelist-only path security, 295 tests |

---

## Future Phases

### Phase 1: v11.1.0 — Polish & Quality

| Feature | Complexity |
|---------|-----------|
| Session logging (JSONL to ~/.config/zenritme/) | Low |
| Statistics dashboard (total focus time today/week) | Medium |
| Colorblind mode (shape-based phase indicators) | Low |
| Config file (~/.config/zenritme/config.toml) | Medium |

### Phase 2: v11.2.0 — Integration

| Feature | Complexity |
|---------|-----------|
| Desktop notifications (notify-send on phase complete) | Low |
| Shell integration (tmux status bar, waybar module) | Medium |
| i3/sway integration (workspace marker) | Low |

### Phase 3: v12.0.0 — Ecosystem

| Feature | Complexity |
|---------|-----------|
| Cross-platform (macOS, FreeBSD) | Medium |
| Configurable keybindings | Medium |

---

## Explicitly Rejected

| Feature | Why |
|---------|-----|
| ~~GUI (Qt/GTK)~~ | Terminal tool, TUI is the UX |
| ~~Background daemon~~ | Interactive tool, not a service |
| ~~Cloud sync~~ | Local tool, not cloud |
| ~~External dependencies~~ | Zero deps is a feature |
| ~~Plugin system~~ | Bloat risk, contradicts zero-deps principle |
| ~~Web dashboard~~ | Local terminal tool, not a web app |
