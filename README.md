# Wooting Hack

Rust CLI experiments for Wooting RGB keyboards.

This project uses the official [`WootingKb/wooting-rgb-sdk`](https://github.com/WootingKb/wooting-rgb-sdk) C ABI through runtime dynamic loading. The SDK source is tracked as a git submodule in `external/wooting-rgb-sdk` for reference and local builds.

## Safety

- Start with low brightness (`--brightness 16` or lower).
- Commands open a Wooting RGB session and call `wooting_rgb_close()` on normal exit, which restores the original keyboard lighting.
- `Ctrl-C` is handled during timed effects so the process exits the loop and resets the keyboard.

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

Paint a low-brightness row test pattern, then reset:

```sh
cargo run -- test --brightness 16 --seconds 3
```

Run a bounded rainbow animation, then reset:

```sh
cargo run -- rainbow --brightness 24 --seconds 10 --fps 30
```

Try the SDK direct single-key feature call:

```sh
cargo run -- direct --row 0 --column 0 --brightness 16 --seconds 3
```

`direct` is useful for SDK experimentation, but may not work on every device/transport. Use `test` and `rainbow` as the primary starter paths because they use array-frame updates.

## Development

```sh
make check
make test
```

Useful environment variable:

```sh
export WOOTING_RGB_SDK_PATH="$PWD/external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"
```

## Troubleshooting

- `failed to load Wooting RGB SDK`: build the SDK and pass `--sdk-path`, or set `WOOTING_RGB_SDK_PATH`.
- `no Wooting RGB keyboard found`: confirm the keyboard is connected and supported by the RGB SDK.
- No lighting change: try a low static test first, confirm SDK debug logs, and verify OS HID permissions.
- Close/reset warning after an effect: the SDK did not acknowledge `wooting_rgb_close()`. The CLI has closed the handle; if lighting is not restored, rerun `cargo run -- info`, try a short `test`, or unplug/replug the keyboard.

## Later

Analog input is separate from RGB control. Research `WootingKb/wooting-analog-sdk` before adding pressure-sensitive input or gamepad-style control.
