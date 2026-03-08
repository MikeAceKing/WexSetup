# Wexio Desktop

Native Tauri desktop runtime for Wexio. It opens the hosted Wexio workspace in a native window, provides desktop window controls, and opens web applications inside dedicated Wexio webview windows.

## Features

- native Wexio desktop window for `https://ui.wexio.be`
- native window controls for minimize, maximize, close, and desktop/minimize behavior
- in-Wexio browser windows for websites opened from Wexio OS and WexSearch
- browser-style fallback for external links when the runtime cannot keep them inside Wexio

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

## Windows signing

Windows recognition and SmartScreen reputation depend on a real code-signing certificate. This repository is wired to use GitHub Actions secrets when available:

- `WINDOWS_CERTIFICATE`: base64-encoded `.pfx`
- `WINDOWS_CERTIFICATE_PASSWORD`: export password for the `.pfx`

When those secrets are present, the publish workflow imports the certificate, generates a Windows-specific Tauri signing config, and signs the installer with SHA-256 plus DigiCert timestamping. Without them, Windows builds are produced unsigned.

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
