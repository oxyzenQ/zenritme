#!/usr/bin/env python3
"""Zenritme Procedural Sound Generator

Generates calm, minimal WAV notification sounds using only the Python
standard library.  No external dependencies, no downloaded samples, no
copyrighted assets.

Output:  assets/sounds/{start,pause,phase,complete}.wav
Usage:   python3 scripts/generate-sounds.py
"""

import math
import struct
import wave
from pathlib import Path

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SAMPLE_RATE = 22050          # Half of CD quality — plenty for notifications
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "assets" / "sounds"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _envelope(n, attack=0.05, release=0.10):
    """Soft attack / sustain / release envelope (list of float 0-1)."""
    env = []
    for i in range(n):
        t = i / max(n - 1, 1)
        if t < attack:
            env.append(t / attack)
        elif t > 1.0 - release:
            env.append((1.0 - t) / release)
        else:
            env.append(1.0)
    return env


def _sine(freq, duration, volume=0.3):
    """Return ``duration`` seconds of a sine wave at *freq* Hz."""
    n = int(SAMPLE_RATE * duration)
    return [volume * math.sin(2.0 * math.pi * freq * i / SAMPLE_RATE)
            for i in range(n)]


def _write_wav(name, samples):
    """Write a mono 16-bit PCM WAV from a list of float samples in [-1, 1]."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUTPUT_DIR / name

    pcm = b""
    for s in samples:
        s = max(-1.0, min(1.0, s))
        pcm += struct.pack("<h", int(s * 32767))

    with wave.open(str(path), "w") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SAMPLE_RATE)
        wf.writeframes(pcm)

    secs = len(samples) / SAMPLE_RATE
    print(f"  {name:<20} {len(samples):>6} samples  {secs:.3f}s")


# ---------------------------------------------------------------------------
# Sound definitions  (all original sine-wave compositions)
# ---------------------------------------------------------------------------


def _gen_start():
    """Gentle two-tone ascending chime: C5 -> E5,  ~250 ms."""
    dur = 0.25
    n = int(SAMPLE_RATE * dur)
    env = _envelope(n, attack=0.08, release=0.15)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        freq = 523.25 if t < dur / 2 else 659.25       # C5 / E5
        samples.append(0.25 * math.sin(2.0 * math.pi * freq * t) * env[i])
    _write_wav("start.wav", samples)


def _gen_pause():
    """Short soft blip: A4, ~120 ms."""
    dur = 0.12
    n = int(SAMPLE_RATE * dur)
    env = _envelope(n, attack=0.10, release=0.30)
    samples = [0.20 * math.sin(2.0 * math.pi * 440.0 * i / SAMPLE_RATE) * env[i]
               for i in range(n)]
    _write_wav("pause.wav", samples)


def _gen_phase():
    """Two-tone descending chime: E5 -> C5, ~500 ms."""
    dur = 0.50
    n = int(SAMPLE_RATE * dur)
    env = _envelope(n, attack=0.06, release=0.20)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        freq = 659.25 if t < dur * 0.4 else 523.25      # E5 / C5
        samples.append(0.20 * math.sin(2.0 * math.pi * freq * t) * env[i])
    _write_wav("phase.wav", samples)


def _gen_complete():
    """Three-note ascending melody: C5 -> E5 -> G5, ~900 ms."""
    notes = [
        (523.25, 0.00, 0.25),    # C5  250 ms
        (659.25, 0.25, 0.25),    # E5  250 ms
        (783.99, 0.50, 0.40),    # G5  400 ms
    ]
    total = 0.90
    n = int(SAMPLE_RATE * total)
    global_env = _envelope(n, attack=0.03, release=0.15)
    samples = [0.0] * n

    for freq, offset, note_dur in notes:
        note_n = int(SAMPLE_RATE * note_dur)
        note_env = _envelope(note_n, attack=0.08, release=0.20)
        start_idx = int(offset * SAMPLE_RATE)
        for i in range(note_n):
            idx = start_idx + i
            if idx < n:
                t = i / SAMPLE_RATE
                samples[idx] += (0.25 * math.sin(2.0 * math.pi * freq * t)
                                 * note_env[i])

    for i in range(n):
        samples[i] *= global_env[i]

    _write_wav("complete.wav", samples)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("Zenritme Procedural Sound Generator")
    print("=" * 40)
    print(f"  Sample rate : {SAMPLE_RATE} Hz")
    print(f"  Output dir  : {OUTPUT_DIR}")
    print()

    _gen_start()
    _gen_pause()
    _gen_phase()
    _gen_complete()

    print()
    print("Done — all sounds are original procedural WAV files.")
