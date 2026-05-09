#!/usr/bin/env bash
set -euo pipefail

apply=false
for arg in "$@"; do
	case "$arg" in
	--apply) apply=true ;;
	*)
		echo "unknown argument: $arg" >&2
		exit 2
		;;
	esac
done

binary_dst="$HOME/.local/bin/wooting-signals"
legacy_extension_binary_dst="$HOME/.local/bin/wooting-extension"
legacy_hack_binary_dst="$HOME/.local/bin/wooting-hack"
plist_path="$HOME/Library/LaunchAgents/io.github.jfmyers9.wooting-signals.plist"
old_primary_plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-signals.plist"
legacy_extension_plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-extension.plist"
legacy_hack_plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-hack.plist"

run() {
	echo "+ $*"
	if [[ "$apply" == true ]]; then
		"$@" || true
	fi
}

echo "wooting-signals macOS uninstall ($([[ "$apply" == true ]] && echo apply || echo dry-run))"
run launchctl bootout "gui/$UID" "$plist_path"
run launchctl bootout "gui/$UID" "$old_primary_plist_path"
run launchctl bootout "gui/$UID" "$legacy_extension_plist_path"
run launchctl bootout "gui/$UID" "$legacy_hack_plist_path"
run rm -f "$plist_path" "$old_primary_plist_path" "$legacy_extension_plist_path" "$legacy_hack_plist_path" "$binary_dst" "$legacy_extension_binary_dst" "$legacy_hack_binary_dst"

echo "config and logs are left in place:"
echo "  $HOME/Library/Application Support/wooting-signals"
echo "  $HOME/Library/Application Support/wooting-extension"
echo "  $HOME/Library/Application Support/wooting-hack"
echo "  $HOME/Library/Logs/wooting-signals.log"
echo "  $HOME/Library/Logs/wooting-extension.log"
echo "  $HOME/Library/Logs/wooting-hack.log"
