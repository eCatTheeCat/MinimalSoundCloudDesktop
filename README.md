# MinimalSoundCloudDesktop

Lightweight Tauri wrapper for SoundCloud with interruption-free playback and Last.fm scrobbling support.
**Note: Currently nothing is working right now. I have no experience with Tauri or Rust in general, so basically all of this is AI code.**

## Getting started

Prereqs (Windows-focused):
- Node.js LTS (install from nodejs.org)
- Rust (MSVC toolchain)
  - Install Rust: `https://rustup.rs/` → pick *default* (MSVC)
  - Verify target: `rustup show` should list `stable-x86_64-pc-windows-msvc` (if not, run `rustup toolchain install stable-x86_64-pc-windows-msvc` and `rustup default stable-x86_64-pc-windows-msvc`)
- Visual Studio Build Tools (C++ workload)
  - Download “Build Tools for Visual Studio 2022”
  - In the installer, select **Desktop development with C++** (this installs `link.exe`, Windows SDK, MSVC)
  - After install, restart your terminal so `link.exe` is on PATH
- Windows 10/11 SDK (comes with the C++ workload above)

Linux (best-effort): system toolchain with a working linker (e.g., `build-essential`, `pkg-config`, WebKitGTK per Tauri docs).

Install deps:
```bash
npm install
```

Run the desktop shell in dev mode:
```bash
npm run tauri:dev
```

Build a release bundle:
```bash
npm run tauri:build
```

## Project layout
- `src/` – React UI shell (ribbon, settings modal, SoundCloud wrapper view will live here)
- `src-tauri/` – Tauri backend (window, tray, build config)
