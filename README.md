<p align="center">
  <img src="assets/zenritme-2026.png" alt="Zenritme logo" width="240">
</p>

<h1 align="center">Zenritme</h1>

<p align="center">
  <strong>A quiet terminal timer for focus, stopwatch, and Pomodoro sessions.</strong>
</p>

<p align="center">
  Minimal, fast, and keyboard-friendly timing from your terminal.
</p>

<p align="center">
  <a href="https://github.com/oxyzenQ/zenritme/actions/workflows/ci.yml">
    <img src="https://github.com/oxyzenQ/zenritme/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status">
  </a>
  <a href="https://github.com/oxyzenQ/zenritme/releases">
    <img src="https://img.shields.io/badge/release-GitHub-7C3AED?style=flat-square&labelColor=111827" alt="GitHub releases">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-GPL--3.0-6D28D9?style=flat-square&labelColor=111827" alt="GPL-3.0 license">
  </a>
  <img src="https://img.shields.io/badge/Rust-CLI-8B5CF6?style=flat-square&labelColor=111827" alt="Rust CLI">
  <img src="https://img.shields.io/badge/terminal-timer-A855F7?style=flat-square&labelColor=111827" alt="Terminal timer">
  <a href="https://ko-fi.com/rezky">
    <img src="https://img.shields.io/badge/Ko--fi-support-7C3AED?style=flat-square&logo=kofi&logoColor=white&labelColor=111827" alt="Support on Ko-fi">
  </a>
</p>


## Examples

```sh
zenritme --timer-down 25m --theme ember
zenritme --pomodoro --view cinematic --theme aura
zenritme --stopwatch --view minimal --theme mono
zenritme --timer-up --theme void --view orbit
```

## Usage

```
zenritme --timer-up
zenritme --timer-down <DURATION>
zenritme --stopwatch
zenritme --pomodoro [FOCUS BREAK]
zenritme --sound-test
zenritme --check-update
zenritme --help
zenritme -V, --version
```

Duration format: `30s`, `10m`, `1h`

### Themes

| Theme | Style |
|-------|-------|
| `void` | Minimal dark |
| `ember` | Warm red/orange |
| `aura` | Purple/magenta |
| `forest` | Green tones |
| `mono` | Monochrome/gray |

### Views

| View | Description |
|------|-------------|
| `minimal` | Compact single-line display |
| `orbit` | Circular progress indicator |
| `cinematic` | Full-width centered box layout |

### Version output

```sh
$ zenritme -V
Version: v2.0.0
Build: linux-x86_64 (1e84ccb)
Copyright: (c) 2026 Rezky_nightky
License: GPL-3.0-only
Source: https://github.com/oxyzenQ/zenritme
```

### Check for updates

`zenritme --check-update` queries the GitHub releases API and reports whether
a newer version is available. This command is **read-only** — it does not
download, install, or replace any binaries.

```sh
$ zenritme --check-update
Zenritme update check
Current: v2.0.0
Latest:  v2.0.0
Status:  up to date
Source:  https://github.com/oxyzenQ/zenritme/releases/latest
```

Requires `curl` on the system PATH. Does not require a GitHub token.

### Pomodoro Masterclass

```sh
zenritme --pomodoro                                    # defaults: 25m focus, 5m break, 15m long break, 4 cycles
zenritme --pomodoro 3s 2s                              # legacy positional syntax
zenritme --pomodoro --cycles 4                         # custom cycle count
zenritme --pomodoro --focus 45m --break 10m --long-break 20m --cycles 3
zenritme --pomodoro --focus 3s --break 2s --long-break 4s --cycles 2 --theme aura --view cinematic
```

| Option | Default | Description |
|--------|---------|-------------|
| `--focus` | `25m` | Focus session length |
| `--break` | `5m` | Short break length |
| `--long-break` | `15m` | Long break (after final focus cycle) |
| `--cycles` | `4` | Focus sessions per round |

Session flow: FOCUS 1/N → SHORT BREAK 1/N → … → FOCUS N/N → LONG BREAK → COMPLETE

### Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `p` | Pause / resume |
| `r` | Reset current session |

## Install

### From release tarball

```sh
tar xzf zenritme-vVERSION-x86_64-unknown-linux-gnu.tar.gz
cd zenritme-vVERSION-x86_64-unknown-linux-gnu
sudo ./install.sh
```

### From source

```sh
cargo build --release --locked
sudo ./scripts/install.sh
```

Custom prefix:

```sh
PREFIX=/usr ./scripts/install.sh
```

### Uninstall

```sh
sudo ./scripts/uninstall.sh
```

## Project docs

- [RULES.md](RULES.md) — project rules, LOC guard, splitting guidance
- [docs/ENDURANCE.md](docs/ENDURANCE.md) — long-running stability testing
- [docs/SECURITY.md](docs/SECURITY.md) — security policy and supply-chain minimization

## Principles

- **Zero external dependencies** — the project uses only the Rust standard library.
- **LOC guard** — all core code files must stay under 1000 lines. Enforced by `scripts/check-loc.sh`.

## License

Zenritme is licensed under the **GNU General Public License v3.0 only** (`GPL-3.0-only`).

You may use, study, modify, and redistribute Zenritme under the terms of the GPLv3. Modified versions that are distributed must preserve the same license terms and provide the corresponding source code.

Older versions that were previously released under the MIT License remain available under their original MIT terms.

The Zenritme name, logo, visual identity, README branding, screenshots, and release assets are reserved by the author unless explicitly stated otherwise.
