#!/usr/bin/env bash
set -euo pipefail

apply=false
bootstrap=false
for arg in "$@"; do
	case "$arg" in
	--apply) apply=true ;;
	--bootstrap) bootstrap=true ;;
	*)
		echo "unknown argument: $arg" >&2
		exit 2
		;;
	esac
done

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
binary_src="$repo/target/release/wooting-signals"
binary_dst="$HOME/.local/bin/wooting-signals"
legacy_extension_binary_dst="$HOME/.local/bin/wooting-extension"
legacy_hack_binary_dst="$HOME/.local/bin/wooting-hack"
config_dir="$HOME/Library/Application Support/wooting-signals"
legacy_extension_config_dir="$HOME/Library/Application Support/wooting-extension"
legacy_hack_config_dir="$HOME/Library/Application Support/wooting-hack"
config_dst="$config_dir/config.toml"
log_path="$HOME/Library/Logs/wooting-signals.log"
plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-signals.plist"
legacy_extension_plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-extension.plist"
legacy_hack_plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-hack.plist"
sdk_path="$repo/external/wooting-rgb-sdk/mac/libwooting-rgb-sdk.dylib"

cat <<INFO
wooting-signals macOS install
  mode: $([[ "$apply" == true ]] && echo apply || echo dry-run)
  binary: $binary_dst
  compatibility aliases: $legacy_extension_binary_dst, $legacy_hack_binary_dst
  config: $config_dst
  legacy configs left untouched: $legacy_extension_config_dir, $legacy_hack_config_dir
  log: $log_path
  plist: $plist_path
  legacy plists left untouched: $legacy_extension_plist_path, $legacy_hack_plist_path
  sdk: $sdk_path
  focus profile template: $repo/examples/focus-cockpit.toml
INFO

run() {
	echo "+ $*"
	if [[ "$apply" == true ]]; then
		"$@"
	fi
}

run cargo build --release
run mkdir -p "$HOME/.local/bin" "$config_dir" "$HOME/Library/Logs" "$HOME/Library/LaunchAgents"
run cp "$binary_src" "$binary_dst"
run ln -sfn "$binary_dst" "$legacy_extension_binary_dst"
run ln -sfn "$binary_dst" "$legacy_hack_binary_dst"

if [[ "$apply" == true && ! -f "$config_dst" ]]; then
	cp "$repo/examples/wooting-signals.toml" "$config_dst"
else
	echo "+ install default config if missing: $config_dst"
fi

plist=$(
	cat <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.jimmy.wooting-signals</string>
  <key>ProgramArguments</key>
  <array>
    <string>$binary_dst</string>
    <string>run</string>
    <string>--config</string>
    <string>$config_dst</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>WOOTING_RGB_SDK_PATH</key>
    <string>$sdk_path</string>
  </dict>
  <key>StandardOutPath</key>
  <string>$log_path</string>
  <key>StandardErrorPath</key>
  <string>$log_path</string>
  <key>RunAtLoad</key>
  <false/>
</dict>
</plist>
PLIST
)

if [[ "$apply" == true ]]; then
	printf '%s\n' "$plist" >"$plist_path"
else
	echo "+ write LaunchAgent plist"
fi

if [[ "$bootstrap" == true ]]; then
	if [[ "$apply" != true ]]; then
		echo "--bootstrap requires --apply" >&2
		exit 2
	fi
	run launchctl bootstrap "gui/$UID" "$plist_path"
else
	cat <<NEXT

LaunchAgent not loaded automatically.
To opt in after reviewing config:
  launchctl bootstrap gui/\$UID "$plist_path"
  launchctl kickstart gui/\$UID/com.jimmy.wooting-signals
  launchctl print gui/\$UID/com.jimmy.wooting-signals

For a long-running Focus Cockpit profile, first review and copy:
  cp "$repo/examples/focus-cockpit.toml" "$config_dst"
NEXT
fi
