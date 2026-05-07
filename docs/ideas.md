# Wooting Extension Candidates

This project is a host-side Wooting Extension Host: extensions read local state and render temporary RGB overlays while Wootility remains responsible for keyboard configuration and baseline profiles.

## Command Pulse

- Input: a local command such as `make check`, `cargo test`, or `npm test`.
- Output: running sweep, green success hold, red failure hold, orange timeout, purple interrupt.
- Status: first real extension target.
- Risks: child process cancellation, output policy, timeout defaults.

## Focus Cockpit

- Input: local timer state: focus, break, overtime, meeting-safe dim mode.
- Output: progress bar across function row or arrows; red pulse on overtime; gentle break sparkle.
- Status: strong second extension candidate.
- Risks: continuous lifecycle, reset behavior, avoiding distraction.

## Git Nebula

- Input: local git dirty state, branch state, Graphite stack state, PR/CI provider status.
- Output: small status zones for clean, dirty, ahead, failing, review, merge conflict.
- Status: build after extension state/polling model settles.
- Risks: provider API auth, polling frequency, avoiding noisy alerts.

## App Aura

- Input: frontmost macOS application or manual profile switch.
- Output: coding, terminal, meeting, gaming, late-night, and recording profiles.
- Status: later platform-specific track.
- Risks: macOS Accessibility/Automation permissions and portable fallback.

## Soundwave Desk Toy

- Input: microphone or system audio levels.
- Output: spectrum bars, bass pulses, ambient color wash.
- Status: later wow-factor track.
- Risks: audio permissions, CPU use, platform-specific audio capture.

## Analog Lava Lab

- Input: Wooting analog key pressure via future `wooting-analog-sdk_dist` backend.
- Output: pressure visualizer, typing heatmap, rapid-trigger trainer, analog game overlays.
- Status: later Wooting-specific track after analog backend research.
- Risks: SDK distribution, device permissions, HID-keycode-to-RGB-matrix mapping.

## Extension point

Extensions should feed small typed state snapshots into renderers. Rendering stays deterministic where possible: `state + tick + layout + palette + brightness -> Frame`. The runner owns polling, timing, cancellation, logging, keyboard lifecycle, and restore policy.
