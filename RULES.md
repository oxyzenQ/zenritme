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
It is wired into `./scripts/build.sh check-all` and runs in CI on every push to `main`.

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
- **Manpage and completions** — install/uninstall scripts should handle
  manpage and shell completion installation/removal as optional, non-fatal
  steps. Missing source files (manpage, completions) must not cause the
  script to fail.

## Static distribution files

Shell completion files under `completions/` and the manpage under `man/` are
static files. They must:

- Be committed directly to the repository (not generated at runtime).
- Contain no executable logic (plain text configuration only).
- Require no external tools or network access to install.
- Be included in the release archive by the release workflow.

## Network access policy

Network access in Zenritme is **opt-in and read-only by default**:

- **Normal timer modes** (timer-up, timer-down, stopwatch, pomodoro) must
  never access the network.
- **`--check-update`** is the only command that accesses the network. It must
  remain strictly read-only: it queries an API and prints a status report but
  never downloads, installs, or replaces binaries.
- **No `curl | sh` or remote execution** — downloaded content must never be
  passed to a shell or executed.
- **No `--install-update`** — automatic binary replacement is not permitted.
  Users must download and install updates manually.
- **Timeout required** — any network request must use a timeout (e.g.,
  `--max-time`) to prevent indefinite hanging.
- **No shell interpolation** — URLs and request parameters must be compile-time
  constants; user input must never be interpolated into commands.

## Scripts policy

Scripts under `scripts/` must follow these rules:

- **Strict shell mode** — all scripts use `set -euo pipefail`.
- **Syntax validation** — scripts must pass `bash -n`.
- **No network access** — utility and test scripts must not require network.
- **No root required** — scripts must work without root privileges.
- **No `sudo` inside scripts** — privilege escalation is the caller's
  responsibility.

## Generated assets policy

Assets that are generated programmatically (e.g., sound files, images) must
satisfy the following requirements:

- **Reproducible from scripts** — every generated asset must have a
  corresponding generator script under `scripts/` (e.g.,
  `scripts/generate-sounds.py`).
- **No external dependencies** — generator scripts must use only standard
  library modules (no `pip install`, no downloaded tools).
- **No downloaded content** — generated assets must be produced entirely from
  code (sine-wave math, algorithmic patterns, etc.). No external files, URLs,
  or network access during generation.
- **No copyrighted material** — generated assets must be original and free of
  third-party intellectual property.
- **Deterministic output** — re-running the generator with the same script
  must produce identical output (same checksums).
