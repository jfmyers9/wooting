#!/usr/bin/env bash
set -euo pipefail

apply=false
for arg in "$@"; do
  case "$arg" in
    --apply) apply=true ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

binary_dst="$HOME/.local/bin/wooting-hack"
plist_path="$HOME/Library/LaunchAgents/com.jimmy.wooting-hack.plist"

run() {
  echo "+ $*"
  if [[ "$apply" == true ]]; then
    "$@" || true
  fi
}

echo "wooting-hack macOS uninstall ($([[ "$apply" == true ]] && echo apply || echo dry-run))"
run launchctl bootout "gui/$UID" "$plist_path"
run rm -f "$plist_path" "$binary_dst"

echo "config and logs are left in place:"
echo "  $HOME/Library/Application Support/wooting-hack"
echo "  $HOME/Library/Logs/wooting-hack.log"
