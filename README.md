# MinimalSoundCloudDesktop

Lightweight Tauri wrapper for SoundCloud with ad-free playback and Last.fm scrobbling.
**Note: Currently nothing is working right now. I have no experience with Tauri or Rust in general, so a lot of this is AI code.**

## Getting started

Prereqs:
- Node.js (LTS)
- Rust toolchain + MSVC build tools (Windows) or equivalent linker setup on Linux

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
