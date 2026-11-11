# Continuous integration

[The workflow](../.github/workflows/ci.yml) runs on pushes to `main`, pull
requests and manual dispatch. It uses read-only repository permissions, cancels
superseded runs and selects exact Rust 1.97.1 with locked Cargo dependencies.

Normal all-target tests cover wet DSP, routing, persistence, controls and ABI contracts without opening JACK or ALSA. Development headers only are installed.

The workflow contains the exact reproducible commands. Historical/exhaustive
auditions and benchmarks remain opt-in. No physical audio, MIDI, DMX, playback,
service activation, media download or deployment is part of these checks.
Compilation and synthetic tests do not establish Raspberry Pi hardware acceptance.
Clippy with warnings denied and release builds are required by the workflow.

JACK 0.14 uses fallible port flags. Port assignment propagates a failed query
before pausing audio, changing connections or persisting settings; direction/type
validation remains mandatory. The dependency upgrade and signal-hook 0.4.6 are
covered by the normal device-free suite; live JACK acceptance stays separate.

## Failure and recovery notifications

[CI alerts](../.github/workflows/ci-alerts.yml) opens one GitHub issue when CI
fails on the current `main` commit, mentions `PaolaShultz`, and keeps repeated
failures in that thread. A passing run for the current commit posts **Recovered**,
renames and closes the issue. Ordinary successes and already-fixed historical
failures create no alerts. Cancelled or pending runs never count as recovery.

Email delivery follows GitHub's participating/@mention notification settings.
Native Actions emails remain a separate account preference; disabling those
avoids duplicate failure messages while retaining the incident thread.
The notifier uses a pinned shared action with `actions: read`, `contents: read`
and `issues: write`; it never runs the triggering code or accesses hardware.
See the [shared notification contract](https://github.com/gigpies/gigpies/blob/main/docs/development/CI_ALERTS.md)
for retry behavior, tests and manual reconciliation. Existing CI checks are unchanged.
