# TODO

1. [ ] Scrobbling engine: track detection, threshold logic, now playing + scrobble; respect toggles.

- [ ] Data sources (per platform):
- Primary (Windows/WebView2): MediaSession API (`navigator.mediaSession.metadata`, position/state events). Low CPU, accurate.
- Fallback (Linux/WebKitGTK): DOM selectors + `audio.currentTime` polling (only active if MediaSession is missing or incomplete).

- [ ] What we collect from the page:
- Track id (derive from URL or DOM link), title, artist, album (if present), duration.
- Playback state: play/pause, position, start time, seeks.
- Flags: ad/promoted (once ad handling lands, we’ll filter before scrobbling).

- [ ] Client-side logic in the injected JS:
- A tiny “player observer” module that:
  - Subscribes to MediaSession change events; falls back to a 1s poll of DOM/audio when MediaSession isn’t available.
  - Emits structured events to Rust via `invoke('report_playback', { ... })`:
    - `track_start` (with metadata, duration, started_at)
    - `progress` (position, paused/playing, seek info)
    - `track_end` or `track_change`
- Threshold tracking in JS: accumulate “listened time” only while playing; ignore forward seeks. When threshold crossed, send threshold_reached.

- [ ] Rust side state machine:
- Keep current track state (id/hash, started_at, duration, listened_ms, scrobbled flag).
- On `track_start`: reset state, send `track.updateNowPlaying` if enabled and session exists.
- On `progress`: add listened_ms when playing; ignore forward seeks; if threshold met and not scrobbled, queue scrobble.
- On `track_end/track_change`: finalize scrobble if threshold already met; otherwise drop.
- Dedup: hash (track id or title+artist+duration) + timestamp window to avoid duplicates.

- [ ] Last.fm calls (Rust):
- `track.updateNowPlaying` when playback starts (if toggle on, not ad/promoted).
- `track.scrobble` when threshold met; include timestamp (track start UTC).
- If network fails, enqueue and retry later; keep a small disk queue in store.

- [ ] Settings respected:
- Threshold slider (1–100%), default 50%.
- Enable scrobbling toggle; enable “Now Playing” toggle; enable notifications.
- Pull settings from store on startup and push to JS so the UI reflects saved values.

- [ ] Notifications:
- On successful scrobble send a toast (if toggle on). No “now playing” toasts.

- [ ] Dev/test hooks:
- Log the incoming playback events and decision points (threshold reached, scrobble queued/sent, failures).
- Add a lightweight “test scrobble” command (optional) to verify session without playing audio.

- [ ] Failure handling:
- Missing session key → skip scrobble, log once.
- Network error → queue and retry with backoff.
- Bad metadata (missing artist/title/duration) → skip scrobble, still allow Now Playing if data is sufficient.

- [ ] Integration order to minimize churn:
- Add Rust commands + state/queue + Last.fm API calls.
- Wire JS observer to send playback events; start with MediaSession path.
- Add DOM fallback polling.
- Wire settings load/save so UI toggles control behavior.
- Add notifications.
- Add retry queue.
---
2. [ ] Ad handling: playback-aware audio ad skip and promoted-track filtering; wire settings toggles.
---
3. [ ] Notifications: native toast on successful scrobble; toggle-controlled.
---
4. [ ] Settings persistence: load/save all toggles and slider; reflect on launch.
---
5. [ ] Dark mode: injected CSS for SoundCloud + chrome; toggle (optionally system-aware).
---
6. [ ] Tray/menu controls: minimize to tray, tray icon menu (open, play/pause, next/prev, quit).
---
7. [ ] Media keys: play/pause/next/prev forwarding (MediaSession primary, DOM fallback).
---
8. [ ] Overlay polish: ensure ribbon/modal don’t interfere with the site; keep version display correct in release.
