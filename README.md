# Wooting Signals

Turn your Wooting keyboard into a live status display.

Wooting Signals reads local and remote signals — commands, GitHub, timers, APIs, and manual profiles — then paints temporary RGB overlays on your keyboard. Use it for build feedback, CI status, focus sessions, alerts, and ambient workstation cues.

## What it does

- **Command Pulse**: wraps a command and shows running / success / failure lighting.
- **GitHub / CI Beacon**: maps Actions and PR status to keyboard zones.
- **Focus Cockpit**: shows focus, break, paused, and overtime states.
- **Market Pulse** and **Sports / Racing Alerts**: poll provider APIs and render alert states.
- **App Aura**: manually switch workstation profiles such as terminal, meeting, game, or late night.
- **Soundwave**: opt-in desk-toy prototype driven by manual audio levels.
- **Static effects**: run comet, rainbow, matrix, breath, and row test scenes.

Wooting Signals is **not** a Wootility replacement. Configure firmware, key maps, actuation, rapid trigger, onboard profiles, and baseline lighting in Wootility; run Wooting Signals when you want host-driven overlays.

## Quick start

### 1. Build the RGB SDK

From the repository root, build the official [`WootingKb/wooting-rgb-sdk`](https://github.com/WootingKb/wooting-rgb-sdk). Wooting Signals loads this library at runtime.

macOS:

```sh
brew install automake pkg-config hidapi libusb
git submodule update --init --recursive
(cd external/wooting-rgb-sdk/mac && make)
```

Linux:

```sh
# Install your distro packages for pkg-config, gcc, make, and hidapi/hidapi-hidraw headers.
git submodule update --init --recursive
(cd external/wooting-rgb-sdk/linux && make)
```

Linux may also need udev rules or permissions that allow access to the keyboard HID interface.

If the SDK library is not found automatically, pass `--sdk-path` or set:

```sh
export WOOTING_RGB_SDK_PATH="$PWD/external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"
```

Use the matching `.so` path on Linux.

### 2. Check the keyboard

```sh
cargo run -- info
cargo run -- test --brightness 96 --seconds 3
```

### 3. Wrap a build or test command

```sh
cargo run -- signal run command-pulse --palette wooting -- make check
cargo run -- signal run command-pulse --timeout-seconds 120 -- cargo test
```

Command output is inherited by default, so your build/test logs remain visible. Add `--summary` for a short completion line or `--output quiet` when lighting feedback is enough.

## Common commands

```sh
# Device and layout
cargo run -- info
cargo run -- layout-info

# Effects
cargo run -- rainbow --brightness 128 --seconds 10 --fps 30
cargo run -- effect comet --palette cyberpunk --brightness 128 --seconds 10 --fps 30

# Direct signals
cargo run -- signal run static-effect --effect comet --palette cyberpunk --seconds 10
cargo run -- signal run focus-cockpit --focus-minutes 25 --break-minutes 5 --cycles 4 --dim
cargo run -- signal run github-ci --repo owner/repo --branch main
cargo run -- signal run app-aura --profile terminal --dim

# Offline previews: no keyboard SDK, network, or token required.
cargo run -- preview effect comet --ticks 3 --format ansi
cargo run -- preview effect comet --ticks 3 --format json
cargo run -- run --config examples/fixture-replay.toml --dry-run --preview --preview-format svg
```

## Run from a profile

Profiles are TOML files that let you save a signal, brightness, FPS, timing, and integration settings.

Validate without touching the keyboard:

```sh
cargo run -- run --config examples/wooting-signals.toml --dry-run
cargo run -- run --config examples/command-pulse.toml --dry-run
cargo run -- run --config examples/github-ci.toml --dry-run
cargo run -- run --config examples/focus-cockpit.toml --dry-run
```

Preview profile frames without touching the keyboard:

```sh
cargo run -- run --config examples/fixture-replay.toml --dry-run --preview
cargo run -- run --config examples/fixture-replay.toml --dry-run --preview --preview-format json
cargo run -- run --config examples/fixture-replay.toml --dry-run --preview --preview-format svg > preview.svg
```

Run a profile:

```sh
cargo run -- run --config examples/wooting-signals.toml
```

Minimal Command Pulse profile:

```toml
palette = "wooting"
brightness = 128
fps = 30
continuous = true

[signal]
kind = "command-pulse"
command = ["make", "check"]
output = "inherit"
summary = true
timeout_seconds = 600
success_hold_seconds = 3
failure_hold_seconds = 6
interrupted_hold_seconds = 2
```

See [`examples/`](examples/) for ready-to-edit profiles.

## Signal guide

### Command Pulse

Use this around local work that has a clear exit code: tests, builds, linters, deploy scripts, or long-running commands.

```sh
cargo run -- signal run command-pulse -- make check
cargo run -- signal run command-pulse --cwd "$PWD" --env RUST_LOG=info --summary -- cargo test
```

Lighting states:

- running: animated progress
- success: success hold
- failure: failure hold
- timeout: timeout alert
- interrupted: Ctrl-C/interrupted hold

### GitHub / CI Beacon

Poll GitHub and show Actions / PR state on keyboard zones.

```sh
export GITHUB_TOKEN=ghp_... # optional for public repos; recommended for private repos/rate limits
cargo run -- signal run github-ci --repo owner/repo --branch main
cargo run -- signal run github-ci --repo owner/repo --pull-request 123 --poll-seconds 60
```

Dry-run output prints token environment variable names only, never token values.

### Focus Cockpit

Render focus and break progress on the function row, with paused, dim, meeting-safe, and overtime modes.

```sh
cargo run -- signal run focus-cockpit --focus-minutes 25 --break-minutes 5 --cycles 4 --dim
cargo run -- signal run focus-cockpit --meeting-safe --dim
```

### Market Pulse and Sports / Racing Alerts

These poll provider-neutral JSON APIs. Keep tokens in environment variables, not config files. Dry-run output redacts query strings and token values.

```sh
cargo run -- run --config examples/market-pulse.toml --dry-run
cargo run -- run --config examples/sports-alerts.toml --dry-run
```

Expected market shape:

```json
{
  "market_open": true,
  "tickers": [
    {
      "symbol": "WOO",
      "price": 101.0,
      "previous_close": 100.0,
      "change_percent": 1.0
    }
  ]
}
```

Expected sports/racing shape:

```json
{
  "events": [
    {
      "id": "race-1",
      "favorite": "WOO",
      "status": "live",
      "score": 2,
      "opponent_score": 1,
      "previous_score": 1
    }
  ]
}
```

### App Aura and Soundwave

App Aura currently uses manual profiles and requires no macOS Accessibility permission:

```sh
cargo run -- signal run app-aura --profile terminal --dim
cargo run -- signal run app-aura --profile meeting --dim
```

Soundwave is disabled unless explicitly enabled and currently uses manual levels:

```sh
cargo run -- signal run soundwave --enabled --level 0.7 --bass 0.4
```

## macOS install helper

The installer is conservative: it can install the binary and write a LaunchAgent plist, but it does not load the agent unless you opt in.

Dry-run install:

```sh
scripts/install-macos.sh
```

Apply install:

```sh
scripts/install-macos.sh --apply
```

Installed paths:

| Item        | Path                                                              |
| ----------- | ----------------------------------------------------------------- |
| Binary      | `~/.local/bin/wooting-signals`                                    |
| Config      | `~/Library/Application Support/wooting-signals/config.toml`       |
| Log         | `~/Library/Logs/wooting-signals.log`                              |
| LaunchAgent | `~/Library/LaunchAgents/io.github.jfmyers9.wooting-signals.plist` |

After reviewing the config, opt into LaunchAgent mode manually:

```sh
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/io.github.jfmyers9.wooting-signals.plist
launchctl kickstart gui/$UID/io.github.jfmyers9.wooting-signals
launchctl print gui/$UID/io.github.jfmyers9.wooting-signals
```

Uninstall binary and LaunchAgent plist:

```sh
scripts/uninstall-macos.sh --apply
```

## Safety and coexistence

- Start with moderate brightness, for example `--brightness 96`.
- Wooting Signals opens an RGB session and attempts to reset/close it on normal exit and Ctrl-C.
- If Wootility or Wootility Background Service writes RGB at the same time, lighting is effectively last-writer-wins.
- Long-running profiles should use conservative brightness and polling intervals.

## Troubleshooting

- `failed to load Wooting RGB SDK`: build the SDK and pass `--sdk-path`, or set `WOOTING_RGB_SDK_PATH`.
- `no Wooting RGB keyboard found`: confirm the keyboard is connected and supported by the RGB SDK.
- No lighting change: try `cargo run -- info`, then a short `test`, and verify OS HID permissions.
- Close/reset warning after an effect: the SDK did not acknowledge `wooting_rgb_close()`. If lighting is not restored, rerun `cargo run -- info`, try a short `test`, or unplug/replug the keyboard.

## Development

```sh
make check
make test
make run-info
make run-effect
make config-dry-run
```

Profile v2 support is executable when a config uses typed `[[sources]]`, `[[rules]]`, and `[scenes]` without an overriding `[signal]`. Dry-run output prints `runtime: profile-v2` or `runtime: single-signal`.

```sh
cargo run -- run --config examples/profile-v2.toml --dry-run
cargo run -- run --config examples/profile-v2.toml --dry-run --preview --preview-format svg > profile-preview.svg
```

Showcase profiles are preview-safe and use fixture/replay sources:

```sh
cargo run -- run --config examples/visual-build-lane.toml --dry-run --preview
cargo run -- run --config examples/visual-ci-stack.toml --dry-run --preview
cargo run -- run --config examples/visual-focus-sprint.toml --dry-run --preview
cargo run -- run --config examples/visual-market-heatline.toml --dry-run --preview
cargo run -- run --config examples/visual-sports-burst.toml --dry-run --preview
cargo run -- run --config examples/visual-meeting-safe.toml --dry-run --preview
cargo run -- run --config examples/visual-app-aura.toml --dry-run --preview
```

Compatibility aliases `wooting-extension` and `wooting-hack` are retained during migration, but `wooting-signals` is the primary binary name.
