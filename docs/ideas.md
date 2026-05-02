# Wooting Hack Ideas

## Pomodoro / focus timer

- Input: local timer state: focus, break, overtime.
- Output: progress bar across function row or arrows; red pulse on overtime.
- Risks: needs good layout zones and a way to run continuously without annoying reset behavior.

## Build/test notifier

- Input: shell command exit status, e.g. `cargo test` or `make check`.
- Output: keyboard turns green on success, red on failure, then restores or returns to idle profile.
- Risks: command output/logging, long-running commands, cancellation.

## Git/CI beacon

- Input: local git dirty state, branch state, PR/CI provider status.
- Output: small status zone: clean, dirty, ahead, failing, merging.
- Risks: provider API auth, polling frequency, avoiding distraction.

## App-context profiles

- Input: frontmost macOS application or manual profile switch.
- Output: coding, meeting, gaming, late-night, and recording profiles.
- Risks: macOS permissions and keeping app detection optional.

## Audio visualizer

- Input: microphone or system audio levels.
- Output: spectrum bars, bass pulses, ambient color wash.
- Risks: macOS audio permissions, high CPU if sampled too aggressively.

## Analog SDK track

- Input: Wooting analog key pressure via the separate Wooting Analog SDK.
- Output: pressure visualizer, typing heatmap, analog game overlays.
- Risks: separate SDK architecture, device permissions, and merging analog input with RGB output cleanly.

## Extension point sketch

Future integrations should feed a small event/state object into the runner. Effects should stay pure where possible: `state + tick + layout + palette -> Frame`. The runner owns polling, timing, cancellation, logging, and keyboard lifecycle.
