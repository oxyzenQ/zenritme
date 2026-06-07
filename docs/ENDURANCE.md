# Zenritme Endurance Testing

This document describes the intended long-running stability test for Zenritme.
Endurance testing is **optional and manual** — it is not required during fast
development iterations or for normal pull requests.

## Quick smoke test

A short automated smoke test is available in `scripts/endurance-smoke.sh`.
It runs a brief timer session and verifies the binary exits cleanly. No network,
no root, no long default runtime.

```sh
# Default 10-second smoke test (requires release binary)
./scripts/endurance-smoke.sh

# Custom duration
DURATION=60s ./scripts/endurance-smoke.sh
DURATION=1h ./scripts/endurance-smoke.sh
```

The script uses `--mute` to avoid audio during testing and `--timer-up` to run
continuously until the timeout expires. It checks that the process exits without
panics.

## Extended manual runs

For longer validation, run Zenritme in timer-up mode and monitor resource usage
with `/usr/bin/time`:

```sh
# 24-hour timer-up endurance run with resource measurement
/usr/bin/time -v timeout 24h zenritme --timer-up
```

For shorter validation runs (e.g., 1 hour or 8 hours), adjust the `timeout`
value accordingly:

```sh
# Quick 1-hour sanity check
/usr/bin/time -v timeout 1h zenritme --timer-up

# Full workday test
/usr/bin/time -v timeout 8h zenritme --timer-up
```

For 24h+ runs, use `tmux`, `screen`, or `nohup` to keep the session alive:

```sh
nohup /usr/bin/time -v timeout 24h zenritme --timer-up > endurance.log 2>&1 &
```

## What to inspect

After the test completes (or is interrupted), check the following:

### Panics and crashes

- The process should exit cleanly with code **0** when terminated by `timeout`
  (SIGTERM) or by pressing `q` / `Esc`.
- Any panic message in stderr indicates a bug. Check the backtrace with
  `RUST_BACKTRACE=1`.

### Memory growth

- `/usr/bin/time -v` prints **Maximum resident set size** (`Maximum RSS`) in
  kilobytes. For Zenritme this should remain essentially constant throughout
  the run — a steadily growing RSS indicates a memory leak.
- On Linux you can also sample live memory with:
  ```sh
  watch -n 60 'ps -o pid,rss,vsz,comm -p <PID>'
  ```

### CPU behavior

- While the timer is running, CPU usage should be near zero (the main loop
  sleeps 80 ms between ticks).
- Brief CPU spikes are acceptable during phase switches (Pomodoro) or startup.
- Sustained high CPU usage indicates a tight loop or a missing sleep.

### Terminal restore

After the process exits (normally or via signal), verify that:

1. The terminal cursor is visible.
2. Typed characters are echoed normally.
3. No escape-sequence garbage is displayed.
4. `stty` settings are restored (run `stty -a` and compare to a fresh terminal).

### Temp sound cleanup

- No `zenritme-sounds-*` directories should remain under `/tmp/` after the
  process exits normally. The RAII cleanup guard removes the PID-specific temp
  directory on all exit paths (normal return, panic unwind, and signal).

### Log output

- If you redirect stderr to a file (`2>endurance.log`), check for any warnings,
  error messages, or unexpected output over time.

## Ctrl+C caveat

Zenritme's terminal cleanup relies on Rust's `Drop` semantics for `TerminalGuard`
and `TempCleanupGuard`. On a normal quit (`q`/`Esc`), these guards fire reliably.
On Ctrl+C (SIGINT), Rust's default signal handling unwinds the stack and drops
all locals, so cleanup still fires. However, a SIGKILL (`kill -9`) bypasses all
cleanup — the terminal may be left in raw mode. This is a limitation of
signal-level termination and not a bug in Zenritme.

## Interpreting results

| Observation | Status | Action |
|---|---|---|
| Clean exit, stable RSS, terminal restored | Pass | No action needed |
| Clean exit but RSS grew significantly | Investigate | Profile allocations, check for unbounded growth |
| Panic on exit or signal | Fail | File an issue with backtrace |
| Terminal not restored on signal | Fail | Investigate signal handling and TerminalGuard::drop |
| Temp sound files remain after exit | Warn | Check TempCleanupGuard and RAII behavior |
