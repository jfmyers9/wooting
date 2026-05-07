# Wooting Extension Host

Host-side extension runner for Wooting keyboards. It runs local extensions such as RGB effects, build/test status, focus timers, and future analog visualizers, then renders temporary keyboard experiences through Wooting SDK backends.

The crate currently builds the `wooting-extension` binary. The older `wooting-hack` name is treated as a compatibility alias during migration.

## Intended usage

Use this as a local companion app, not as a Wootility replacement:

- inspect connected keyboard and inferred layout metadata
- run RGB dev utilities and demo effects
- run extension profiles from TOML
- run direct extensions such as Command Pulse for build/test feedback
- later, read analog key pressure through the Wooting Analog SDK distributable

## Wootility coexistence

Wootility remains the source of truth for firmware, key mappings, actuation, rapid trigger, onboard profiles, and baseline lighting.

This extension host owns host-side RGB only while it is running. On normal exit or Ctrl-C it calls the RGB SDK close/reset path so the keyboard can return to its original lighting. If Wootility or Wootility Background Service is actively writing RGB at the same time, live RGB control is effectively last-writer-wins; do not expect both apps to own lighting simultaneously.

## SDK contracts

### RGB output

Current output uses the official [`WootingKb/wooting-rgb-sdk`](https://github.com/WootingKb/wooting-rgb-sdk) C ABI through runtime dynamic loading. The SDK source is tracked as a git submodule in `external/wooting-rgb-sdk` for reference and local builds.

Full-frame array updates are the primary rendering path. The direct single-key command remains available for SDK probing and simple notification experiments.

### Future analog input

Analog features should use `wooting-analog-sdk_dist` as an application dependency, following Wooting's distributable/system-SDK delegation model. This project is not an Analog SDK plugin unless it someday adds support for new hardware.

## Safety

- Start with moderate brightness (`--brightness 96`) and adjust per command.
- Commands open a Wooting RGB session and try `wooting_rgb_close()` on normal exit.
- Ctrl-C is handled during timed effects and extensions so the process attempts cleanup.
- Long-running extension profiles should be opt-in and conservative.

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

## Extension commands

Run the static-effect extension path:

```sh
cargo run -- extension run static-effect --effect comet --palette cyberpunk --seconds 10
```

Run Command Pulse around a build/test command:

```sh
cargo run -- extension run command-pulse --palette wooting -- make check
cargo run -- extension run command-pulse --timeout-seconds 120 -- cargo test
```

Command Pulse renders a running animation, then a success/failure/timeout/interrupted hold before restoring through the RGB SDK close path.

## Config runner

Validate without touching the keyboard:

```sh
cargo run -- run --config examples/wooting-extension.toml --dry-run
cargo run -- run --config examples/command-pulse.toml --dry-run
```

Run a profile:

```sh
cargo run -- run --config examples/wooting-extension.toml
```

Example static-effect config:

```toml
effect = "comet"
palette = "cyberpunk"
brightness = 128
fps = 30
seconds = 10
continuous = false
warn_on_close_error = true

[extension]
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

[extension]
kind = "command-pulse"
command = ["make", "check"]
timeout_seconds = 600
success_hold_seconds = 3
failure_hold_seconds = 6
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

| Item                | Path                                                          |
| ------------------- | ------------------------------------------------------------- |
| Binary              | `~/.local/bin/wooting-extension`                              |
| Compatibility alias | `~/.local/bin/wooting-hack`                                   |
| Config              | `~/Library/Application Support/wooting-extension/config.toml` |
| Log                 | `~/Library/Logs/wooting-extension.log`                        |
| LaunchAgent         | `~/Library/LaunchAgents/com.jimmy.wooting-extension.plist`    |
| SDK dylib           | repo `external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib`  |

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
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.jimmy.wooting-extension.plist
launchctl kickstart gui/$UID/com.jimmy.wooting-extension
launchctl print gui/$UID/com.jimmy.wooting-extension
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

## Extension candidates

See [`docs/ideas.md`](docs/ideas.md) for Command Pulse, Focus Cockpit, Git Nebula, App Aura, Soundwave Desk Toy, and Analog Lava Lab.
