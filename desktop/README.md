# Wexio Desktop

Minimal Tauri wrapper that launches the hosted Wexio app in a native desktop window.

## Prerequisites

- Node.js 20+
- Rust toolchain
- A Windows C/C++ toolchain on Windows (MSVC Build Tools) or equivalent native compiler toolchain on macOS/Linux

## Commands

```bash
npm install
npm run dev
npm run build
```

## Output

Tauri bundles installers under:

```text
src-tauri/target/release/bundle/
```

Copy the produced files into:

```text
../installers/windows/WexioSetup.exe
../installers/macos/WexioDesktop.dmg
../installers/linux/WexioDesktop.AppImage
```
