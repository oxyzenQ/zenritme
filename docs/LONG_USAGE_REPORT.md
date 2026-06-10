# Zenritme Long-Usage Report (v4.0.2)

This document summarizes the real-world long-usage validation of Zenritme v4.0.2.

## Scope

Zenritme v4.0.2 was tested in real daily usage across multiple sessions spanning
timer-up, timer-down, stopwatch, and Pomodoro modes. The goal was to confirm that
the application behaves stably under normal work conditions and that no
long-usage blockers exist.

## Results

### Timer / Pomodoro behavior

All timer modes (timer-up, timer-down, stopwatch, Pomodoro) behaved as expected
during real usage sessions. The 80 ms tick loop maintained accurate timekeeping
with no drift observed during multi-hour sessions. Pomodoro phase transitions
(focus -> short break -> long break) triggered correctly with the expected
sound events and visual state changes.

### Terminal recovery

On normal quit (pressing `q` or `Esc`), terminal state was restored reliably
across all tested terminals. The TerminalGuard RAII implementation correctly
saved and restored terminal settings via `stty -g`, and the alternate screen
buffer was released properly. No escape-sequence garbage or stuck cursor
situations were observed during normal exit paths.

### Temp sound cleanup

Temp sound directories created under `/tmp/zenritme-sounds-{PID}` were cleaned
up correctly on normal exit. The TempCleanupGuard RAII implementation removed
the PID-specific temp directory and its contents as expected. No orphaned temp
directories were found after normal quit sessions.

### No known long-usage blockers

No memory leaks, unbounded allocations, or growing RSS were observed during
extended runs. CPU usage remained near-zero during idle timer ticks, with only
brief spikes during phase transitions and startup. The application remained
responsive throughout all test sessions.

## Known caveats

### Signal termination

Signal-level termination (SIGINT via Ctrl+C, SIGKILL via `kill -9`) may bypass
Rust's `Drop` cleanup entirely. This is a fundamental limitation of POSIX signal
handling, not a bug in Zenritme. When a signal terminates the process:

- The terminal may be left in raw mode or alternate-screen state.
- Temp sound directories may remain under `/tmp/`.
- Recovery: press `Ctrl+J`, type `stty sane`, press Enter, or open a new
  terminal window.

This caveat is documented in `docs/ENDURANCE.md` and `docs/SECURITY.md`.

### 24h+ test

A full 24-hour endurance test is optional and manual. It is not required for
normal releases. The automated smoke test (`scripts/endurance-smoke.sh`) covers
a short default run (10 seconds) and can be extended with `DURATION=24h` for
manual validation.

## Conclusion

v4.0.2 is stable for real daily usage. No long-usage blockers were found.
Normal quit paths (q/Esc) provide reliable terminal recovery and temp cleanup.
Signal termination caveats remain documented and are a platform limitation.