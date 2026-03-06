# WexioSetup

Desktop distribution repository for Wexio.

## Purpose

- host Windows, macOS, and Linux desktop installer artifacts
- publish download metadata for `download.wexio.be`
- separate desktop packaging work from the main Wexio application repo
- keep the Railway download domain independent from the main Wexio app

## Planned artifacts

- `WexioSetup.exe`
- `WexioDesktop.dmg`
- `WexioDesktop.AppImage`

## Structure

- `site/` static download surface
- `desktop/` Tauri desktop wrapper
- `installers/windows/WexioSetup.exe`
- `installers/macos/WexioDesktop.dmg`
- `installers/linux/WexioDesktop.AppImage`

## Build status

This repository now contains the Tauri scaffold and static download routes.
Native installers are not committed yet. The platform pages auto-check whether
the expected artifact file exists and only start the download when it is present.

## Cross-platform publishing

Use the GitHub Actions workflow `Publish Desktop Installers` to build native
installers on the correct operating systems and publish them back into this repo:

- Windows runner -> `installers/windows/WexioSetup.exe`
- macOS runner -> `installers/macos/WexioDesktop.dmg`
- Ubuntu runner -> `installers/linux/WexioDesktop.AppImage`

The workflow is manual by design. Run it from the Actions tab after desktop
changes are pushed to `main`.

## Railway

Deploy this repository through Docker.

Public routes:

- `https://download.wexio.be/windows/`
- `https://download.wexio.be/macos/`
- `https://download.wexio.be/linux/`

Direct artifact routes once binaries exist:

- `https://download.wexio.be/windows/WexioSetup.exe`
- `https://download.wexio.be/macos/WexioDesktop.dmg`
- `https://download.wexio.be/linux/WexioDesktop.AppImage`
