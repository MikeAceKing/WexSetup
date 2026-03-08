#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLE_ROOT="${SCRIPT_DIR}/../src-tauri/target/release/bundle"

find_latest() {
  local dir="$1"
  local pattern="$2"
  find "$dir" -type f -name "$pattern" 2>/dev/null | sort | tail -n 1
}

DEB_PATH="$(find_latest "${BUNDLE_ROOT}/deb" "*.deb")"
APPIMAGE_PATH="$(find_latest "${BUNDLE_ROOT}/appimage" "*.AppImage")"

if [[ -n "${DEB_PATH}" ]]; then
  echo "Installing Debian package: ${DEB_PATH}"
  sudo apt install -y "${DEB_PATH}"
  exit 0
fi

if [[ -n "${APPIMAGE_PATH}" ]]; then
  echo "No .deb found, enabling latest AppImage: ${APPIMAGE_PATH}"
  chmod +x "${APPIMAGE_PATH}"
  echo "Run it with:"
  echo "${APPIMAGE_PATH}"
  exit 0
fi

echo "No Ubuntu bundle found. Run 'npm run build:ubuntu' first." >&2
exit 1
