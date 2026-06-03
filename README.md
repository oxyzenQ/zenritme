# zenritme — a quiet terminal timer

Minimal terminal timer, stopwatch, and pomodoro.

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
zenritme --help
```

Duration format: `30s`, `10m`, `1h`

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
