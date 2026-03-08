# Wexio Desktop

Bridge-enabled Tauri desktop runtime for Wexio. It opens the hosted Wexio workspace in a native window and exposes a localhost bridge on port `47821` so Wexio can launch approved local apps, open files and URLs, and read basic machine info when the desktop app is installed.

## Features

- native Wexio desktop window for `https://ui.wexio.be`
- localhost bridge endpoints:
  - `GET /health`
  - `GET /system/info`
  - `POST /launch/app`
  - `POST /open/url`
  - `POST /open/file`
  - `POST /command/execute`
  - `GET /ws`
- request origin validation for `*.wexio.*`, `localhost`, and `127.0.0.1`
- fixed allowlists for desktop app launch and system command execution

## Prerequisites

- Node.js 20+
- Rust toolchain
- Tauri build prerequisites for your OS
- A Windows C/C++ toolchain on Windows (MSVC Build Tools) or equivalent native compiler toolchain on macOS/Linux

## Local development

```bash
npm install
npm run dev
```

## Build targets

- Windows: `npm run build:windows`
- macOS: `npm run build:macos`
- Ubuntu: `npm run build:ubuntu`

The default cross-platform build remains:

```bash
npm run build
```

## Install from terminal

### Windows

```powershell
npm run build:windows
npm run install:windows
```

This prefers the newest NSIS installer and falls back to MSI.

### macOS

```bash
npm run build:macos
npm run install:macos
```

If a `.app` bundle exists it is copied into `/Applications`. If only a DMG exists, the script opens it.

### Ubuntu

```bash
npm run build:ubuntu
npm run install:ubuntu
```

The installer prefers the generated `.deb`. If only an AppImage exists, it makes it executable and prints the launch command.

## Output

Tauri bundles installers under:

```text
src-tauri/target/release/bundle/
```

GitHub Actions publishes the final files into:

```text
../installers/windows/WexioSetup.exe
../installers/macos/WexioDesktop.dmg
../installers/linux/WexioDesktop.AppImage
```
