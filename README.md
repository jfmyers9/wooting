# Wooting Hack

Rust CLI experiments for Wooting RGB keyboards, with a path toward a macOS workstation companion.

This project uses the official [`WootingKb/wooting-rgb-sdk`](https://github.com/WootingKb/wooting-rgb-sdk) C ABI through runtime dynamic loading. The SDK source is tracked as a git submodule in `external/wooting-rgb-sdk` for reference and local builds.

## Safety

- Start with moderate brightness (`--brightness 96`) and adjust per command.
- Commands open a Wooting RGB session and try `wooting_rgb_close()` on normal exit, which should restore the original keyboard lighting.
- `Ctrl-C` is handled during timed effects so the process exits the loop and attempts reset/close.

## Prerequisites

### macOS

```sh
brew install automake pkg-config hidapi libusb
```

Build the SDK dynamic library:

```sh
git submodule update --init --recursive
cd external/wooting-rgb-sdk/mac
make
```

If the CLI cannot find the library automatically, pass it explicitly:

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

If needed:

```sh
cargo run -- --sdk-path external/wooting-rgb-sdk/linux/libwooting-rgb-sdk.so info
```

Linux HID access may require udev rules or running with permissions that can open the keyboard HID interface.

## Commands

Print connected keyboard metadata:

```sh
cargo run -- info
```

Print inferred layout metadata:

```sh
cargo run -- layout-info
```

Paint a row test pattern, then reset:

```sh
cargo run -- test --brightness 96 --seconds 3
```

Run a bounded rainbow animation, then reset:

```sh
cargo run -- rainbow --brightness 128 --seconds 10 --fps 30
```

Run named effects with palettes:

```sh
cargo run -- effect comet --palette cyberpunk --brightness 128 --seconds 10 --fps 30
cargo run -- effect matrix --palette terminal --brightness 128 --seconds 10 --fps 30
cargo run -- effect breath --palette ocean --brightness 128 --seconds 10 --fps 30
```

Try the SDK direct single-key feature call:

```sh
cargo run -- direct --row 0 --column 0 --brightness 96 --seconds 3
```

`direct` is useful for SDK experimentation, but may not work on every device/transport. Use `test`, `rainbow`, and `effect` as the primary starter paths because they use array-frame updates.

## Config runner

Validate the example config without touching the keyboard:

```sh
cargo run -- run --config examples/wooting-hack.toml --dry-run
```

Run it:

```sh
cargo run -- run --config examples/wooting-hack.toml
```

Config keys:

```toml
effect = "comet"
palette = "cyberpunk"
brightness = 128
fps = 30
seconds = 10
continuous = false
warn_on_close_error = true
# sdk_path = "external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"
```

## Development

```sh
make check
make test
make run-info
make run-effect
```

Useful environment variable:

```sh
export WOOTING_RGB_SDK_PATH="$PWD/external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"
```

## macOS workstation workflow

The workstation mode is intentionally conservative: scripts install files and generate a LaunchAgent, but the agent is not loaded unless you opt in.

| Item | Path |
| --- | --- |
| Binary | `~/.local/bin/wooting-hack` |
| Config | `~/Library/Application Support/wooting-hack/config.toml` |
| Log | `~/Library/Logs/wooting-hack.log` |
| LaunchAgent | `~/Library/LaunchAgents/com.jimmy.wooting-hack.plist` |
| SDK dylib | repo `external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib` |

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
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.jimmy.wooting-hack.plist
launchctl kickstart gui/$UID/com.jimmy.wooting-hack
launchctl print gui/$UID/com.jimmy.wooting-hack
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

## Ideas

See [`docs/ideas.md`](docs/ideas.md) for future Pomodoro, build status, Git/CI, app-context, audio visualizer, and analog SDK tracks.
