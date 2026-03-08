# WexioSetup

Desktop distribution repository for Wexio.

## Purpose

- build and publish Windows, macOS, and Linux desktop installers
- host the download surface for `download.wexio.be`
- keep desktop packaging separate from the main Wexio application repo
- ship the native Tauri runtime that hosts Wexio as a desktop workspace

## Included runtime

The desktop app under `desktop/` provides:

- the native Wexio desktop window for `https://ui.wexio.be`
- Tauri window controls for minimize, maximize, close, and desktop/minimize behavior
- web applications opened inside Wexio webview windows
- the desktop runtime needed for the in-Wexio web workspace experience

## Artifacts

- `installers/windows/WexioSetup.exe`
- `installers/macos/WexioDesktop.dmg`
- `installers/linux/WexioDesktop.AppImage`

## Structure

- `site/` static download surface
- `desktop/` Tauri desktop runtime
- `installers/windows/WexioSetup.exe`
- `installers/macos/WexioDesktop.dmg`
- `installers/linux/WexioDesktop.AppImage`

## Build and publish flow

Pushes to `main` that touch desktop, site, installers, or deployment files trigger the `Publish Desktop Installers` workflow. That workflow:

- builds on Windows, macOS, and Ubuntu runners
- collects native installer bundles
- commits updated installers back into this repo with `Publish desktop installers [skip ci]`

### Local build commands

From `desktop/`:

- `npm install`
- `npm run build:windows`
- `npm run build:macos`
- `npm run build:ubuntu`

### Local install commands

From `desktop/`:

- Windows: `npm run install:windows`
- macOS: `npm run install:macos`
- Ubuntu: `npm run install:ubuntu`

## Railway

Deploy this repository through Docker.

Public routes:

- `https://download.wexio.be/windows/`
- `https://download.wexio.be/macos/`
- `https://download.wexio.be/linux/`

Direct artifact routes:

- `https://download.wexio.be/windows/WexioSetup.exe`
- `https://download.wexio.be/macos/WexioDesktop.dmg`
- `https://download.wexio.be/linux/WexioDesktop.AppImage`
