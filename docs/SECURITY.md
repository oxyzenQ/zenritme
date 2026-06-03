# Zenritme Security and Supply-Chain Policy

This document describes the project's security posture, dependency policy,
and stability-related concerns.

## Zero external dependency policy

Zenritme currently has **zero external Rust dependencies** (see `Cargo.toml`).
All functionality is implemented using only the Rust standard library and
platform APIs invoked through `std::process::Command`.

This is an intentional design choice:

- **Smallest possible attack surface** — no third-party code enters the binary.
- **Predictable builds** — the compiler, standard library, and system tools
  are the only inputs to the build.
- **Audit simplicity** — every line of code in the project is written and
  reviewed by the project maintainers.

## Supply-chain minimization

New external dependencies are **discouraged** and require explicit justification
before being added. Any proposed dependency must satisfy the following criteria:

1. **No standard library alternative** — the functionality cannot be reasonably
   implemented with `std` alone.
2. **Minimal transitive graph** — the crate itself must have few or no
   dependencies.
3. **Active maintenance** — the crate must be actively maintained and compatible
   with the project's MSRV (minimum supported Rust version).
4. **Permissive or GPL-compatible license** — the crate's license must be
   compatible with `GPL-3.0-only`.

When a dependency is accepted, it should be pinned to an exact version in
`Cargo.toml` and audited periodically with `cargo audit`.

## Terminal safety

Zenritme takes over the terminal during a session. The following measures
ensure terminal safety:

- **TerminalGuard** captures the exact terminal state via `stty -g` before
  making changes and restores it precisely on drop, even across panics
  (thanks to Rust's `Drop` semantics).
- **Alternate screen buffer** is used when available, so the original terminal
  content is preserved and restored on exit.
- **Raw mode** is entered for single-keypress input and fully reversed on
  exit.
- A `stty sane` fallback is available if the exact state restore fails.

## Long-running stability

As a timer application, Zenritme may run for hours. Stability concerns include:

- **No unbounded allocations** — all internal data structures have fixed or
  bounded size.
- **No unbounded threads** — only a single reader thread is spawned for
  input.
- **No network access** — Zenritme never connects to the network.
- **No file I/O during runtime** — Zenritme does not read or write files
  while running (configuration is passed via CLI arguments and environment
  variables).

See [docs/ENDURANCE.md](ENDURANCE.md) for the recommended long-running test
procedure.

## Reporting vulnerabilities

If you discover a security issue, please open a GitHub issue with the
`security` label or contact the maintainer directly.
