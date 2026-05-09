# Wooting Signals

Data-driven RGB automation for Wooting keyboards. Wooting Signals watches external signals — shell commands, GitHub, markets, sports, calendars, games, APIs — and maps them to programmable lighting strategies on your keyboard.

Current state: the first signal is **Command Pulse**, which turns build/test commands into running/success/failure RGB feedback. The renderer and runner are structured so future signals can share the same RGB backend, layouts, palettes, and cleanup behavior.

The crate builds the `wooting-signals` binary. `wooting-extension` and `wooting-hack` are compatibility aliases during migration.

## Intended usage

Use this as a local RGB automation companion, not as a Wootility replacement:

- Wootility configures firmware, key maps, actuation, rapid trigger, onboard profiles, and baseline lighting.
- Wooting Signals temporarily paints host-side RGB overlays while it is running.
- Signal profiles connect sources to lighting strategies: command status now; GitHub, stocks, racing/sports, timers, and analog pressure later.
- Normal exit or Ctrl-C attempts to restore/reset lighting through the RGB SDK close path.

## Mental model

```text
external source -> signal state -> lighting strategy -> RGB frame -> Wooting keyboard
```

Examples:

- `make check` running -> cyan sweep
- tests passed -> green bloom
- GitHub Actions failed -> red alert zone
- stock up/down -> green/red market pulse
- favorite team scored -> team-color burst
- future analog pressure -> per-key heatmap

## Wootility coexistence

Wootility remains the source of truth for keyboard configuration and baseline profiles. Wooting Signals does not register inside Wootility and is not a Wootility plugin.

If Wootility or Wootility Background Service is actively writing RGB while Wooting Signals is running, live RGB control is effectively last-writer-wins. For best results, configure the keyboard in Wootility, then run Wooting Signals when you want data-driven overlays.

## SDK contracts

### RGB output

Current output uses the official [`WootingKb/wooting-rgb-sdk`](https://github.com/WootingKb/wooting-rgb-sdk) C ABI through runtime dynamic loading. The SDK source is tracked as a git submodule in `external/wooting-rgb-sdk` for reference and local builds.

Full-frame array updates are the primary rendering path. The direct single-key command remains available for SDK probing and simple notification experiments.

### Future analog input

Analog features should use `wooting-analog-sdk_dist` as an application dependency, following Wooting's distributable/system-SDK delegation model. This project is not an Analog SDK plugin unless it someday adds support for new hardware.

## Safety

- Start with moderate brightness (`--brightness 96`) and adjust per command.
- Commands open a Wooting RGB session and try `wooting_rgb_close()` on normal exit.
- Ctrl-C is handled during timed effects and signals so the process attempts cleanup.
- Long-running signal profiles should be opt-in and conservative.

## Prerequisites

### macOS

```sh
brew install automake pkg-config hidapi libusb
git submodule update --init --recursive
cd external/wooting-rgb-sdk/mac
make
```

If the CLI cannot find the library automatically:

```sh
cargo run -- --sdk-path external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib info
```

### Linux

Install your distro packages for `pkg-config`, `gcc`, `make`, and `hidapi`/`hidapi-hidraw` development headers. Then:

```sh
git submodule update --init --recursive
cd external/wooting-rgb-sdk/linux
make
```

Linux HID access may require udev rules or running with permissions that can open the keyboard HID interface.

## RGB/dev utilities

```sh
cargo run -- info
cargo run -- layout-info
cargo run -- test --brightness 96 --seconds 3
cargo run -- rainbow --brightness 128 --seconds 10 --fps 30
cargo run -- effect comet --palette cyberpunk --brightness 128 --seconds 10 --fps 30
cargo run -- direct --row 0 --column 0 --brightness 96 --seconds 3
```

## Signal commands

Run a static RGB scene through the signal runner:

```sh
cargo run -- signal run static-effect --effect comet --palette cyberpunk --seconds 10
```

Run Command Pulse around a build/test command:

```sh
cargo run -- signal run command-pulse --palette wooting -- make check
cargo run -- signal run command-pulse --timeout-seconds 120 -- cargo test
```

Compatibility: `extension run ...` remains accepted as an alias for now.

## Config runner

Validate without touching the keyboard:

```sh
cargo run -- run --config examples/wooting-signals.toml --dry-run
cargo run -- run --config examples/command-pulse.toml --dry-run
```

Run a profile:

```sh
cargo run -- run --config examples/wooting-signals.toml
```

Example static scene config:

```toml
effect = "comet"
palette = "cyberpunk"
brightness = 128
fps = 30
seconds = 10
continuous = false
warn_on_close_error = true

[signal]
kind = "static-effect"
effect = "comet"
```

Example Command Pulse config:

```toml
palette = "wooting"
brightness = 128
fps = 30
seconds = 0
continuous = true

[signal]
kind = "command-pulse"
command = ["make", "check"]
timeout_seconds = 600
success_hold_seconds = 3
failure_hold_seconds = 6
```

## Future profile direction

The current config selects one signal. The intended richer profile model is:

```toml
[[sources]]
id = "ci"
type = "github-actions"
repo = "owner/repo"

[[rules]]
when = "ci.status == 'failed'"
scene = "red-alert"

[scenes.red-alert]
effect = "pulse"
palette = "heat"
zones = ["function", "navigation"]
```

## Development

```sh
make check
make test
make run-info
make run-effect
make config-dry-run
```

Useful environment variable:

```sh
export WOOTING_RGB_SDK_PATH="$PWD/external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"
```

## macOS workstation workflow

The workstation mode is intentionally conservative: scripts install files and generate a LaunchAgent, but the agent is not loaded unless you opt in.

| Item                  | Path                                                          |
| --------------------- | ------------------------------------------------------------- |
| Binary                | `~/.local/bin/wooting-signals`                                |
| Compatibility aliases | `~/.local/bin/wooting-extension`, `~/.local/bin/wooting-hack` |
| Config                | `~/Library/Application Support/wooting-signals/config.toml`   |
| Log                   | `~/Library/Logs/wooting-signals.log`                          |
| LaunchAgent           | `~/Library/LaunchAgents/com.jimmy.wooting-signals.plist`      |
| SDK dylib             | repo `external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib`  |

Dry-run install:

```sh
scripts/install-macos.sh
```

Apply install:

```sh
scripts/install-macos.sh --apply
```

After reviewing config, opt into LaunchAgent mode manually:

```sh
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.jimmy.wooting-signals.plist
launchctl kickstart gui/$UID/com.jimmy.wooting-signals
launchctl print gui/$UID/com.jimmy.wooting-signals
```

Uninstall binary and LaunchAgent plist:

```sh
scripts/uninstall-macos.sh --apply
```

## Troubleshooting

- `failed to load Wooting RGB SDK`: build the SDK and pass `--sdk-path`, or set `WOOTING_RGB_SDK_PATH`.
- `no Wooting RGB keyboard found`: confirm the keyboard is connected and supported by the RGB SDK.
- No lighting change: try a static test first, confirm SDK debug logs, and verify OS HID permissions.
- Close/reset warning after an effect: the SDK did not acknowledge `wooting_rgb_close()`. The CLI has closed the handle; if lighting is not restored, rerun `cargo run -- info`, try a short `test`, or unplug/replug the keyboard.

## Signal candidates

See [`docs/ideas.md`](docs/ideas.md) for Command Pulse, GitHub/CI, market, sports/racing, Focus Cockpit, App Aura, Soundwave Desk Toy, and Analog Lava Lab.
