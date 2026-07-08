# Zenritme Built-in Sounds

This directory contains the default notification sounds bundled with Zenritme.

## Origin

All `.wav` files in this directory are **procedurally generated** by
`scripts/generate-sounds.py` using only the Python standard library
(`wave`, `math`, `struct`, `pathlib`).

- **No external samples** — every sound is a pure sine-wave composition.
- **No copyrighted assets** — no downloaded audio, AI sample packs, or
  third-party content.
- **No network required** — generation is fully offline.
- **Reproducible** — running the generator script reproduces identical
  output (deterministic sine-wave math).

## Sound events

| File        | Event     | Duration | Description                          |
|-------------|-----------|----------|--------------------------------------|
| `start.wav` | Start     | ~250 ms  | Gentle two-tone ascending chime       |
| `pause.wav` | Pause     | ~120 ms  | Short soft blip                      |
| `phase.wav` | Phase     | ~500 ms  | Two-tone descending chime            |
| `complete.wav` | Complete | ~900 ms | Three-note ascending melody          |

## Regeneration

```sh
python3 scripts/generate-sounds.py
```

## Environment overrides

Each sound event can be overridden individually via environment variables:

```
ZENRITME_SOUND_START     path to custom start sound
ZENRITME_SOUND_PAUSE     path to custom pause sound
ZENRITME_SOUND_PHASE     path to custom phase sound
ZENRITME_SOUND_COMPLETE  path to custom complete sound
ZENRITME_SOUND_FILE      global fallback for all events
```

See `zenritme --sound-test` for live status of all overrides.
