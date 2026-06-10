# Zenritme Release Checklist

Use this checklist before tagging and publishing a new release.

## Pre-release

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo test` passes (all tests green)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `bash scripts/check-loc.sh` passes (all files under 1000 LOC)
- [ ] `bash scripts/doctor.sh` passes (binary health check)
- [ ] `bash scripts/endurance-smoke.sh` completes without panic
- [ ] No new external Rust dependencies in `Cargo.toml`
- [ ] `Cargo.toml` version matches the intended tag version
- [ ] `Cargo.lock` is committed and in sync

## Build

- [ ] `cargo build --release --locked` succeeds
- [ ] `./target/release/zenritme -V` shows the correct version
- [ ] `./target/release/zenritme --help` shows correct version in header

## Functional smoke

- [ ] `./target/release/zenritme --sound-test` runs without panic
- [ ] `./target/release/zenritme --check-update` completes (requires network)
- [ ] `./target/release/zenritme --timer-down 1s --mute` runs and exits clean

## Distribution files

- [ ] `man/zenritme.1` exists
- [ ] `completions/zenritme.bash` exists
- [ ] `completions/zenritme.zsh` exists
- [ ] `completions/zenritme.fish` exists
- [ ] `scripts/install.sh` passes `bash -n`
- [ ] `scripts/uninstall.sh` passes `bash -n`
- [ ] `scripts/doctor.sh` passes `bash -n`
- [ ] `scripts/endurance-smoke.sh` passes `bash -n`
- [ ] `scripts/generate-sounds.py` passes `python3 -m py_compile`
- [ ] Sound assets regenerated: `python3 scripts/generate-sounds.py`
- [ ] All four WAV files present: `start.wav pause.wav phase.wav complete.wav`

## Release archive contents

Verify the archive (produced by the release workflow or manual `tar`) contains:

- [ ] `target/release/zenritme` (or `zenritme` at archive root)
- [ ] `README.md`
- [ ] `LICENSE`
- [ ] `NOTICE.md`
- [ ] `RULES.md`
- [ ] `docs/SECURITY.md`
- [ ] `docs/ENDURANCE.md`
- [ ] `scripts/install.sh`
- [ ] `scripts/uninstall.sh`
- [ ] `scripts/generate-sounds.py`
- [ ] `scripts/endurance-smoke.sh`
- [ ] `scripts/doctor.sh`
- [ ] `assets/sounds/*.wav`
- [ ] `assets/sounds/README.md`
- [ ] `completions/zenritme.bash`
- [ ] `completions/zenritme.zsh`
- [ ] `completions/zenritme.fish`
- [ ] `man/zenritme.1`

## Tag and push

- [ ] Git tag created: `git tag v<VERSION>`
- [ ] Tag pushed: `git push origin v<VERSION>`
- [ ] GitHub Actions release workflow runs successfully
- [ ] Release artifact published with correct prerelease flag
- [ ] SHA256SUMS generated and included in release

## Post-release

- [ ] Install test from tarball on a clean machine
- [ ] `man zenritme` displays the manpage after install
- [ ] Shell completions activate in at least one shell
- [ ] `./scripts/doctor.sh` passes on installed binary