# TODO

- [ ] Scrobbling engine: track detection, threshold logic, ~~now playing~~, scrobble; respect toggles.

  - [x] Data sources (per platform):
    - [ ] ~~Primary (Windows/WebView2): MediaSession API (`navigator.mediaSession.metadata`, position/state events). Low CPU, accurate.~~ (doesnt seem to work?)
    - [x] Fallback (Linux/WebKitGTK): DOM selectors + `audio.currentTime` polling (only active if MediaSession is missing or incomplete). (untested on linux)

  - [x] What we collect from the page:
    - [x] Track id (derive from URL or DOM link), title, artist, album (if present), duration.
    - [x] Playback state: play/pause, position, start time, seeks.
    - [ ] Flags: ad/promoted (once ad handling lands, we’ll filter before scrobbling).

  - [x] Client-side logic in the injected JS:
    - [x] A tiny “player observer” module that:
      - [x] Subscribes to MediaSession change events; falls back to a 1s poll of DOM/audio when MediaSession isn’t available.
      - [x] Emits structured events to Rust via `invoke('report_playback', { ... })`:
        - [x] `track_start` (with metadata, duration, started_at)
        - [x] `progress` (position, paused/playing, seek info)
        - [x] `track_end` or `track_change`
      - [x] Threshold tracking in JS: accumulate “listened time” only while playing; ignore forward seeks. When threshold crossed, send threshold_reached.

  - [ ] Rust side state machine:
    - [x] Keep current track state (id/hash, started_at, duration, listened_ms, scrobbled flag).
    - [ ] ~~On `track_start`: reset state, send `track.updateNowPlaying` if enabled and session exists.~~
    - [x] On `progress`: add listened_ms when playing; ignore forward seeks; if threshold met and not scrobbled, queue scrobble.
    - [x] On `track_end/track_change`: finalize scrobble if threshold already met; otherwise drop.
    - [ ] Dedup: hash (track id or title+artist+duration) + timestamp window to avoid duplicates.

  - [ ] Last.fm calls (Rust):
    - [ ] ~~`track.updateNowPlaying` when playback starts (if toggle on, not ad/promoted).~~
    - [x] `track.scrobble` when threshold met; include timestamp (track start UTC).
    - [ ] If network fails, enqueue and retry later; keep a small disk queue in store. 

  - [x] Settings respected:
    - [x] Threshold slider (1–100%), default 50%.
    - [x] Enable scrobbling toggle; ~~enable “Now Playing” toggle;~~ enable notifications.
    - [x] Pull settings from store on startup and push to JS so the UI reflects saved values.

  - [x] Notifications:
    - [x] On successful scrobble send a toast (if toggle on). No “now playing” toasts.

  - [x] Dev/test hooks:
    - [x] Log the incoming playback events and decision points (threshold reached, scrobble queued/sent, failures).
    - [ ] ~~Add a lightweight “test scrobble” command (optional) to verify session without playing audio.~~

  - [ ] Failure handling:
    - [x] Missing session key → skip scrobble, log once.
    - [ ] Network error → queue and retry with backoff.
    - [x] Bad metadata (missing artist/title/duration) → skip scrobble, ~~still allow Now Playing if data is sufficient~~.

---
- [ ] Ad handling: playback-aware audio ad skip and promoted-track filtering; wire settings toggles.
---
- [ ] Notifications: native toast on successful scrobble; toggle-controlled.
---
- [ ] Settings persistence: load/save all toggles and slider; reflect on launch.
---
- [ ] Dark mode: injected CSS for SoundCloud + chrome; toggle (optionally system-aware).
---
- [ ] Tray/menu controls: minimize to tray, tray icon menu (open, play/pause, next/prev, quit).
---
- [ ] Media keys: play/pause/next/prev forwarding (MediaSession primary, DOM fallback).
---
- [ ] Discord RPC: let the app talk to discord rpc to show on your profile what your actively listening to.
---
- [ ] Overlay polish: ensure ribbon/modal don’t interfere with the site; keep version display correct in release.
