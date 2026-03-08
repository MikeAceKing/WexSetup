#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLE_ROOT="${SCRIPT_DIR}/../src-tauri/target/release/bundle"

find_latest() {
  local dir="$1"
  local pattern="$2"
  find "$dir" -type f -name "$pattern" 2>/dev/null | sort | tail -n 1
}

APP_PATH="$(find_latest "${BUNDLE_ROOT}/macos" "*.app")"
DMG_PATH="$(find_latest "${BUNDLE_ROOT}/dmg" "*.dmg")"

if [[ -n "${APP_PATH}" ]]; then
  echo "Installing app bundle to /Applications: ${APP_PATH}"
  rm -rf "/Applications/Wexio Desktop.app"
  cp -R "${APP_PATH}" "/Applications/Wexio Desktop.app"
  echo "Installed at /Applications/Wexio Desktop.app"
  exit 0
fi

if [[ -n "${DMG_PATH}" ]]; then
  echo "Opening DMG for manual drag-install: ${DMG_PATH}"
  open "${DMG_PATH}"
  exit 0
fi

echo "No macOS bundle found. Run 'npm run build:macos' first." >&2
exit 1
