# Zenritme Project Rules

This document defines the structural and maintenance rules for the Zenritme codebase.
All contributors must follow these rules.

## Lines-of-Code (LOC) Guard

Every **core code file** in the repository must stay under **1000 LOC**.

### Core code files

The following file types are subject to the LOC guard:

- `*.rs`
- `*.c`
- `*.h`
- `*.cpp`
- `*.hpp`
- `*.css`
- `*.js`
- `*.ts`
- `*.tsx`

### Excluded from the LOC guard

The following are explicitly excluded:

- `*.md` — documentation and policy files
- `*.txt` — plain text files
- `LICENSE` — license text
- `NOTICE` / `NOTICE.md` — license notice
- Generated files (e.g., anything under `target/`)
- Build scripts and CI configuration (`.yml`, `.sh` at project root)

### `main.rs` constraint

`main.rs` is intended to serve as **bootstrap and wiring only** — it should
delegate all real logic to modules and stay ideally in the **100–300 LOC** range.
As of this writing it is well within that target.

### Splitting guidance

Files approaching **800–1000 LOC** should be proactively split by responsibility.
Prefer splitting a large module into smaller, focused modules rather than
refactoring the architecture. Suggested split domains include:

- **discovery** — argument parsing, configuration probing
- **validation** — input validation, constraint checks
- **execution** — core engine and timer logic
- **reporting** — progress output and status reporting
- **rendering** — terminal drawing and box-layout code
- **input** — keypress handling and terminal input
- **terminal** — terminal state management, raw mode, restore

### Enforcement

The script `scripts/check-loc.sh` enforces the LOC guard automatically.
It is wired into `./build.sh check-all` and runs in CI on every push to `main`.

Run it manually at any time:

```sh
./scripts/check-loc.sh
```

The script exits with **0** when all files pass and **1** (with a report)
when any file exceeds the limit.

## Install script safety

Shell scripts under `scripts/` that handle installation must follow these rules:

- **No network access** — scripts must never use `curl`, `wget`, or any
  network tool. Users download the tarball themselves and run the install
  script from the extracted archive.
- **No `curl | sh` pattern** — the project will never publish or recommend
  a one-liner that pipes remote content into a shell.
- **`set -euo pipefail`** — all shell scripts must use strict error handling.
- **PREFIX / DESTDIR** — install and uninstall scripts must respect
  `PREFIX` (default `/usr/local`) and `DESTDIR` for staging.
- **Syntax validation** — scripts must pass `bash -n` (syntax-only check).
- **No sudo inside scripts** — scripts must not call `sudo` internally.
  Users elevate privileges when invoking the script if needed.
