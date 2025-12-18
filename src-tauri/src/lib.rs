use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;
use serde::Deserialize;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::{Store, StoreExt};
use url::Url;

const STORE_PATH: &str = "lastfm.json";
const DEV_CALLBACK_URL: &str = "http://127.0.0.1:35729/callback";
const DEFAULT_THRESHOLD: f32 = 0.5;

#[derive(Debug, Deserialize)]
struct LocalLastfmConfig {
  api_key: String,
  api_secret: String,
  #[serde(default)]
  callback: Option<String>,
}

fn load_lastfm_config() -> Option<LocalLastfmConfig> {
  let mut candidates = Vec::new();

  if let Ok(p) = std::env::var("LASTFM_CONFIG") {
    candidates.push(std::path::PathBuf::from(p));
  }

  if cfg!(debug_assertions) {
    candidates.push(std::path::PathBuf::from("src-tauri/lastfm.local.json"));
  }

  candidates.push(std::path::PathBuf::from("lastfm.local.json"));

  if let Ok(mut exe) = std::env::current_exe() {
    exe.pop();
    candidates.push(exe.join("lastfm.local.json"));
  }

  for path in candidates {
    if path.exists() {
      if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<LocalLastfmConfig>(&text) {
          return Some(cfg);
        }
      }
    }
  }

  None
}

fn lastfm_key() -> Option<String> {
  std::env::var("LASTFM_API_KEY").ok().or_else(|| {
    load_lastfm_config()
      .map(|c| c.api_key)
      .or_else(|| option_env!("LASTFM_API_KEY").map(|s| s.to_string()))
  })
}

fn lastfm_secret() -> Option<String> {
  std::env::var("LASTFM_API_SECRET").ok().or_else(|| {
    load_lastfm_config()
      .map(|c| c.api_secret)
      .or_else(|| option_env!("LASTFM_API_SECRET").map(|s| s.to_string()))
  })
}

fn lastfm_callback() -> String {
  #[cfg(debug_assertions)]
  {
    load_lastfm_config()
      .and_then(|c| c.callback)
      .unwrap_or_else(|| DEV_CALLBACK_URL.to_string())
  }
  #[cfg(not(debug_assertions))]
  {
    std::env::var("LASTFM_CALLBACK").ok().or_else(|| {
      load_lastfm_config()
        .and_then(|c| c.callback)
        .or_else(|| option_env!("LASTFM_CALLBACK").map(|s| s.to_string()))
    }).unwrap_or_else(|| "mscd://lastfm-callback".to_string())
  }
}

fn build_overlay_script(lastfm_key: &str, lastfm_callback: &str, version: &str, playback_url: &str) -> String {
  let auth_url = format!(
    "https://www.last.fm/api/auth/?api_key={}&cb={}",
    lastfm_key, lastfm_callback
  );

  let template = r#"
    (() => {
      console.info('[MSCD] Overlay script loaded');
      if (window.__minimal_sc_overlay_installed) return;
      window.__minimal_sc_overlay_installed = true;

      const getInvoker = () => {
        try {
          const t = window.__TAURI__;
          return t?.invoke ?? t?.core?.invoke ?? null;
        } catch (e) {
          console.warn('[MSCD] __TAURI__ unavailable', e);
          return null;
        }
      };

      function inject() {
        try {
        console.info('[MSCD] Injecting overlay');
        const host = document.createElement('div');
        host.id = 'mscd-overlay-host';
        host.style.position = 'fixed';
        host.style.top = '0';
        host.style.left = '0';
        host.style.right = '0';
        host.style.padding = '8px 8px 0 8px';
        host.style.zIndex = '2147483647';
        host.style.pointerEvents = 'none';
        document.body.appendChild(host);

        const shadow = host.attachShadow({ mode: 'open' });

        const style = document.createElement('style');
        style.textContent = `
          :host { all: initial; }
          * { box-sizing: border-box; }
          .shell {
            width: 100%;
            pointer-events: auto;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            padding: 8px 12px;
            min-height: 36px;
            background: rgba(12, 14, 20, 0.96);
            border: 1px solid rgba(255,255,255,0.12);
            border-radius: 10px;
            box-shadow: 0 12px 26px rgba(0,0,0,0.25);
            backdrop-filter: blur(7px);
            color: #e9ecf5;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          }
          .brand { display: flex; align-items: center; gap: 10px; min-width: 0; }
          .title { font-weight: 700; white-space: nowrap; }
          .muted { color: #a7acb8; font-size: 12px; white-space: nowrap; }
          .actions { display: flex; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
          button {
            height: 30px;
            padding: 0 12px;
            border-radius: 9px;
            border: 1px solid rgba(255,255,255,0.16);
            background: #1b202b;
            color: #e9ecf5;
            font-weight: 600;
            cursor: pointer;
            transition: background 120ms ease, border-color 120ms ease, transform 120ms ease;
          }
          button:hover { background: #252c3b; border-color: rgba(255,255,255,0.22); }
          button:active { transform: translateY(1px); }

          .modal-backdrop {
            position: fixed;
            inset: 0;
            background: rgba(0,0,0,0.35);
            backdrop-filter: blur(5px);
            display: none;
            align-items: center;
            justify-content: center;
            padding: 16px;
            pointer-events: none;
          }
          .modal-backdrop.open { display: flex; pointer-events: auto; }
          .modal {
            width: min(520px, 96vw);
            background: #0f131c;
            border: 1px solid rgba(255,255,255,0.12);
            border-radius: 14px;
            padding: 18px;
            box-shadow: 0 30px 70px rgba(0,0,0,0.45);
            color: #e9ecf5;
            display: flex;
            flex-direction: column;
            gap: 14px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          }
          .row { display: flex; justify-content: space-between; gap: 8px; align-items: center; }
          .row + .row { margin-top: 4px; }
          .muted { color: #a7acb8; font-size: 13px; }
          .toggle { display: flex; align-items: center; gap: 8px; }
          .toggle input { accent-color: #3c57ff; }
          .slider-wrapper { display: inline-flex; align-items: center; gap: 8px; min-width: 170px; }
          .slider { width: 140px; }
          .section h3 { margin: 0; font-size: 15px; }
          .section {
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 10px;
            padding: 12px;
            background: rgba(255,255,255,0.03);
          }
          .modal header { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
          .close { height: 32px; padding: 0 10px; }
          .warning { color: #ffb95f; font-size: 12px; }
        `;
        shadow.appendChild(style);

        const offsetId = 'mscd-offset-style';
        if (!document.getElementById(offsetId)) {
          const s = document.createElement('style');
          s.id = offsetId;
          s.textContent = 'body { padding-top: 48px !important; }';
          document.head.appendChild(s);
        }

        let settingsOpen = false;
        let darkMode = true;

        const shell = document.createElement('div');
        shell.className = 'shell';

        const brand = document.createElement('div');
        brand.className = 'brand';
        const title = document.createElement('span');
        title.className = 'title';
        title.textContent = 'Minimal SC Desktop';
        const status = document.createElement('span');
        status.className = 'muted';
        status.textContent = 'v{version}';
        brand.append(title, status);

        const actions = document.createElement('div');
        actions.className = 'actions';

        const btnSettings = document.createElement('button');
        btnSettings.textContent = 'Settings';
        const btnDark = document.createElement('button');
        btnDark.textContent = 'Dark mode';
        const btnTray = document.createElement('button');
        btnTray.textContent = 'Minimize to tray';

        actions.append(btnSettings, btnDark, btnTray);
        shell.append(brand, actions);

        const backdrop = document.createElement('div');
        backdrop.className = 'modal-backdrop';

        const modal = document.createElement('div');
        modal.className = 'modal';

        const header = document.createElement('header');
        const h2 = document.createElement('h2');
        h2.textContent = 'Settings';
        const btnClose = document.createElement('button');
        btnClose.className = 'close';
        btnClose.textContent = 'Close';
        header.append(h2, btnClose);

        const makeToggleRow = (labelText, checked = true) => {
          const row = document.createElement('div');
          row.className = 'row';
          const label = document.createElement('span');
          label.textContent = labelText;
          const wrap = document.createElement('label');
          wrap.className = 'toggle';
          const input = document.createElement('input');
          input.type = 'checkbox';
          input.checked = !!checked;
          wrap.appendChild(input);
          row.append(label, wrap);
          return { row, input };
        };

        const makeSliderRow = () => {
          const row = document.createElement('div');
          row.className = 'row';
          const label = document.createElement('span');
          label.textContent = 'Scrobble threshold';

          const wrap = document.createElement('div');
          wrap.className = 'toggle slider-wrapper';

          const slider = document.createElement('input');
          slider.type = 'range';
          slider.min = '1';
          slider.max = '100';
          slider.value = '50';
          slider.className = 'slider';

          const val = document.createElement('span');
          val.className = 'muted';
          val.id = 'mscd-threshold-label';
          val.style.minWidth = '36px';
          val.style.display = 'inline-block';
          val.style.textAlign = 'right';
          val.textContent = '50%';

          wrap.append(slider, val);
          row.append(label, wrap);
          return { row, slider, val };
        };

        const makeLastfmRow = (authUrl, keyMissing, warnText) => {
          const row = document.createElement('div');
          row.className = 'row';

          const statusWrap = document.createElement('div');
          statusWrap.className = 'toggle';
          const statusLabel = document.createElement('span');
          statusLabel.textContent = 'Status: ';
          const statusValue = document.createElement('strong');
          statusValue.textContent = 'Not connected';
          statusWrap.append(statusLabel, statusValue);

          const toggleWrap = document.createElement('div');
          toggleWrap.className = 'toggle';
          toggleWrap.style.gap = '12px';

          const connectBtn = document.createElement('button');
          connectBtn.id = 'mscd-lastfm-connect';
          connectBtn.textContent = 'Connect in browser';
          if (keyMissing) connectBtn.disabled = true;

          const disconnectBtn = document.createElement('button');
          disconnectBtn.textContent = 'Disconnect';
          disconnectBtn.style.display = 'none';

          toggleWrap.append(connectBtn, disconnectBtn);
          row.append(statusWrap, toggleWrap);

          let warnNode = null;
          if (warnText) {
            warnNode = document.createElement('div');
            warnNode.className = 'warning';
            warnNode.textContent = warnText;
          }

          const authInfo = document.createElement('div');
          authInfo.className = 'muted';
          authInfo.textContent = `Auth URL: ${authUrl}`;

          const setStatus = (session) => {
            if (session && session.username) {
              statusValue.textContent = `Connected as ${session.username}`;
              connectBtn.disabled = true;
              disconnectBtn.disabled = false;
              disconnectBtn.style.display = '';
            } else {
              statusValue.textContent = 'Not connected';
              connectBtn.disabled = !!keyMissing;
              disconnectBtn.disabled = true;
              disconnectBtn.style.display = 'none';
            }
          };

          setStatus(null);

          return { row, connectBtn, disconnectBtn, warnNode, authInfo, setStatus };
        };

        const secPlayback = document.createElement('div');
        secPlayback.className = 'section';
        const s1Title = document.createElement('h3');
        s1Title.textContent = 'Playback & Ads';
        const adRow = makeToggleRow('Skip audio ads');
        const promoRow = makeToggleRow('Skip promoted tracks');
        secPlayback.append(s1Title, adRow.row, promoRow.row);

        const secScrobble = document.createElement('div');
        secScrobble.className = 'section';
        const s2Title = document.createElement('h3');
        s2Title.textContent = 'Scrobbling';
        const scrobbleToggle = makeToggleRow('Enable scrobbling');
        const thresholdRow = makeSliderRow();
        const nowPlayingRow = makeToggleRow('Send \"Now Playing\"');
        const notifyRow = makeToggleRow('Show scrobble notifications');
        secScrobble.append(s2Title, scrobbleToggle.row, thresholdRow.row, nowPlayingRow.row, notifyRow.row);

        const secLastfm = document.createElement('div');
        secLastfm.className = 'section';
        const s3Title = document.createElement('h3');
        s3Title.textContent = 'Last.fm';
        const keyMissing = '{key}' === 'REPLACE_ME';
        const authUrl = '{auth_url}';
        const warnText = keyMissing ? 'Set LASTFM_API_KEY & LASTFM_CALLBACK to enable auth.' : '';
        const lf = makeLastfmRow(authUrl, keyMissing, warnText);
        secLastfm.append(s3Title, lf.row);
        if (lf.warnNode) secLastfm.append(lf.warnNode);
        secLastfm.append(lf.authInfo);

        modal.append(header, secPlayback, secScrobble, secLastfm);
        backdrop.appendChild(modal);

        const setModalOpen = (open) => {
          settingsOpen = open;
          backdrop.classList.toggle('open', open);
        };

        btnSettings.onclick = () => setModalOpen(true);
        btnClose.onclick = () => setModalOpen(false);
        backdrop.onclick = (e) => {
          if (e.target === backdrop) setModalOpen(false);
        };

        btnDark.onclick = () => {
          darkMode = !darkMode;
          host.dataset.theme = darkMode ? 'dark' : 'light';
          btnDark.textContent = darkMode ? 'Dark mode' : 'Light mode';
        };
        btnTray.onclick = () => alert('Minimize to tray (placeholder)');

        const fallbackOpen = (url) => {
          const opened = window.open(url, '_blank', 'noopener,noreferrer');
          if (!opened) {
            console.warn('[MSCD] window.open blocked, trying anchor click');
            const anchor = document.createElement('a');
            anchor.href = url;
            anchor.target = '_blank';
            anchor.rel = 'noopener noreferrer';
            shadow.appendChild(anchor);
            anchor.click();
            shadow.removeChild(anchor);
          }
          if (!opened) {
            console.warn('[MSCD] Fallback to same-window navigation');
            window.location.href = url;
          }
        };

        const refreshLastfmStatus = async () => {
          const invoke = getInvoker();
          if (!invoke) return null;
          try {
            const session = await invoke('get_lastfm_status');
            lf.setStatus(session || null);
            return session || null;
          } catch (err) {
            console.warn('[MSCD] get_lastfm_status failed', err);
            return null;
          }
        };

        const pollForSession = (attempt = 0) => {
          if (attempt > 30) return;
          setTimeout(async () => {
            const session = await refreshLastfmStatus();
            if (!session) {
              pollForSession(attempt + 1);
            }
          }, 2000);
        };

        lf.connectBtn?.addEventListener('click', () => {
          if (keyMissing) {
            console.warn('[MSCD] Key missing; connect disabled');
            return;
          }

          try {
            const invoke = getInvoker();
            if (invoke) {
              console.info('[MSCD] Opening via open_external command', authUrl);
              Promise.resolve(invoke('open_external', { url: authUrl }))
                .then(() => {
                  console.info('[MSCD] open_external success');
                  pollForSession();
                })
                .catch((err) => {
                  console.warn('[MSCD] open_external failed; falling back', err);
                  fallbackOpen(authUrl);
                  pollForSession();
                });
              return;
            } else {
              console.warn('[MSCD] __TAURI__.invoke unavailable; falling back to window.open');
            }
          } catch (e) {
            console.warn('TAURI shell unavailable, falling back to window.open', e);
          }

          fallbackOpen(authUrl);
          pollForSession();
        });

        lf.disconnectBtn?.addEventListener('click', async () => {
          const invoke = getInvoker();
          if (!invoke) return;
          try {
            await invoke('disconnect_lastfm');
            lf.setStatus(null);
          } catch (err) {
            console.warn('[MSCD] disconnect_lastfm failed', err);
          }
        });

        const slider = thresholdRow.slider;
        const label = thresholdRow.val;
        slider?.addEventListener('input', () => {
          label.textContent = `${slider.value}%`;
        });

        shadow.append(shell, backdrop);

        refreshLastfmStatus();

        // --- Scrobble observer (MediaSession primary, DOM fallback) ---
        const startScrobbleObserver = () => {
          const endpoint = '{playback_url}';
          console.info('[MSCD] Scrobble observer starting, endpoint:', endpoint);
          if (!endpoint) {
            console.warn('[MSCD] playback endpoint missing, observer disabled');
            return;
          }
          let lastPayload = null;
          let logCount = 0;
          let lastLoggedTrack = null;

          const grabMeta = () => {
            const toSeconds = (text) => {
              if (!text) return 0;
              const match = text.match(/(\d{1,2}:)?\d{1,2}:\d{2}/g);
              const raw = match ? match[match.length - 1] : text.trim();
              const parts = raw.split(':').map((p) => parseInt(p, 10));
              if (!parts.length || parts.some((p) => Number.isNaN(p))) return 0;
              return parts.reduce((acc, part) => acc * 60 + part, 0);
            };

            const audio = document.querySelector('audio');
            if (!audio && logCount < 5) {
              console.info('[MSCD] No audio element found yet');
              logCount += 1;
            }
            const posMs = audio ? Math.floor((audio.currentTime || 0) * 1000) : 0;
            let paused = audio ? !!audio.paused : true;

            let title = null;
            let artist = null;
            let durationMs = audio ? Math.floor((audio.duration || 0) * 1000) : 0;

            if (navigator.mediaSession?.metadata) {
              title = navigator.mediaSession.metadata.title || title;
              artist = navigator.mediaSession.metadata.artist || artist;
              durationMs = navigator.mediaSession.metadata.duration
                ? Math.floor(navigator.mediaSession.metadata.duration * 1000)
                : durationMs;
            }

            if (!title || !artist) {
              const titleEl = document.querySelector('.playbackSoundBadge__titleLink span[aria-hidden="true"]')
                || document.querySelector('.playbackSoundBadge__titleLink')
                || document.querySelector('.playbackSoundBadge__title');
              const artistEl = document.querySelector('.playbackSoundBadge__lightLink');
              title = title || (titleEl ? titleEl.textContent?.trim() : null);
              artist = artist || (artistEl ? artistEl.textContent?.trim() : null);
            }

            if (title) {
              title = title.replace(/^Current track:\s*/i, '').trim();
            }

            if (navigator.mediaSession?.playbackState) {
              paused = navigator.mediaSession.playbackState !== 'playing';
            } else if (!audio) {
              const playBtn = document.querySelector('.playControls__play');
              const label = playBtn?.getAttribute('aria-label') || '';
              if (label) {
                paused = !/pause/i.test(label);
              } else if (playBtn?.classList.contains('playing')) {
                paused = false;
              }
            }

            const durationText =
              document.querySelector('.playbackTimeline__duration span[aria-hidden="true"]')?.textContent ||
              document.querySelector('.playbackTimeline__duration')?.textContent ||
              '';
            const passedText =
              document.querySelector('.playbackTimeline__timePassed span[aria-hidden="true"]')?.textContent ||
              document.querySelector('.playbackTimeline__timePassed')?.textContent ||
              '';

            const durationSec = toSeconds(durationText);
            const passedSec = toSeconds(passedText);
            if (!durationMs && durationSec) durationMs = Math.floor(durationSec * 1000);
            let positionMs = posMs || (passedSec ? Math.floor(passedSec * 1000) : 0);

            const slider = document.querySelector('.playbackTimeline__progressWrapper[role="progressbar"][aria-valuenow][aria-valuemax]');
            if (slider) {
              const maxVal = parseFloat(slider.getAttribute('aria-valuemax') || '0');
              const nowVal = parseFloat(slider.getAttribute('aria-valuenow') || '0');
              if (!durationMs && maxVal > 100) {
                durationMs = Math.floor(maxVal * 1000);
              }
              if (!positionMs && nowVal > 0 && maxVal > 0) {
                positionMs = Math.floor(nowVal * 1000);
              }
            }

            const trackHref = document.querySelector('.playbackSoundBadge__titleLink')?.getAttribute('href');
            const trackId = trackHref || window.location.pathname || title || 'unknown';

            return {
              trackId,
              title,
              artist,
              durationMs,
              positionMs,
              paused,
              ts: Date.now(),
            };
          };

          const pushUpdate = () => {
            const payload = grabMeta();
            if (!payload.title || !payload.artist || !payload.durationMs) {
              if (logCount < 5) {
                console.info('[MSCD] Missing metadata', {
                  title: payload.title,
                  artist: payload.artist,
                  durationMs: payload.durationMs,
                });
                logCount += 1;
              }
              return;
            }

            if (logCount < 5 || payload.trackId !== lastLoggedTrack) {
              console.info('[MSCD] playback payload', payload);
              logCount += 1;
              lastLoggedTrack = payload.trackId;
            }

            // avoid spamming identical payloads
            if (lastPayload &&
                lastPayload.trackId === payload.trackId &&
                lastPayload.title === payload.title &&
                lastPayload.artist === payload.artist &&
                Math.abs(lastPayload.positionMs - payload.positionMs) < 900 &&
                lastPayload.paused === payload.paused) {
              return;
            }
            lastPayload = payload;

            const body = JSON.stringify(payload);
            if (navigator.sendBeacon) {
              const blob = new Blob([body], { type: 'text/plain' });
              const sent = navigator.sendBeacon(endpoint, blob);
              if (sent) return;
            }

            fetch(endpoint, {
              method: 'POST',
              mode: 'no-cors',
              headers: { 'Content-Type': 'text/plain' },
              body,
            }).catch((err) => {
              console.warn('[MSCD] playback post failed', err);
            });
          };

          setInterval(pushUpdate, 2000);
          document.addEventListener('visibilitychange', () => {
            if (!document.hidden) pushUpdate();
          });
        };

        startScrobbleObserver();

        const settingsUrl = endpoint.replace(/\/playback$/, '/settings');

        const applySettings = (cfg) => {
          if (!cfg) return;
          if (typeof cfg.threshold === 'number') {
            const percent = Math.max(1, Math.min(100, Math.round(cfg.threshold * 100)));
            thresholdRow.slider.value = String(percent);
            thresholdRow.val.textContent = `${percent}%`;
          }
          if (typeof cfg.enable_scrobble === 'boolean') {
            scrobbleToggle.input.checked = cfg.enable_scrobble;
          }
          if (typeof cfg.enable_now_playing === 'boolean') {
            nowPlayingRow.input.checked = cfg.enable_now_playing;
          }
          if (typeof cfg.enable_notifications === 'boolean') {
            notifyRow.input.checked = cfg.enable_notifications;
          }
        };

        const gatherSettings = () => ({
          threshold: Math.max(0.01, Math.min(1, Number(thresholdRow.slider.value) / 100)),
          enable_scrobble: scrobbleToggle.input.checked,
          enable_now_playing: nowPlayingRow.input.checked,
          enable_notifications: notifyRow.input.checked,
        });

        const loadSettings = async () => {
          if (!settingsUrl) return;
          try {
            const res = await fetch(settingsUrl, { method: 'GET', mode: 'cors' });
            if (!res.ok) {
              console.warn('[MSCD] settings load failed', res.status);
              return;
            }
            const cfg = await res.json();
            applySettings(cfg);
          } catch (err) {
            console.warn('[MSCD] settings load error', err);
          }
        };

        const saveSettings = async () => {
          if (!settingsUrl) return;
          const payload = gatherSettings();
          try {
            const res = await fetch(settingsUrl, {
              method: 'POST',
              mode: 'cors',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(payload),
            });
            if (!res.ok) {
              console.warn('[MSCD] settings save failed', res.status);
              return;
            }
            const cfg = await res.json();
            applySettings(cfg);
          } catch (err) {
            console.warn('[MSCD] settings save error', err);
          }
        };
        slider?.addEventListener('change', () => {
          saveSettings();
        });

        scrobbleToggle.input.addEventListener('change', saveSettings);
        nowPlayingRow.input.addEventListener('change', saveSettings);
        notifyRow.input.addEventListener('change', saveSettings);
        loadSettings();
        console.info('[MSCD] Overlay injected');
        } catch (err) {
          console.warn('[MSCD] Overlay inject failed', err);
        }
      }

      const ready = () => {
        if (document.body) {
          inject();
        } else {
          console.info('[MSCD] Waiting for document.body');
          setTimeout(ready, 50);
        }
      };
      ready();
    })();
    "#;

  template
    .replace("{auth_url}", &auth_url)
    .replace("{key}", lastfm_key)
    .replace("{version}", version)
    .replace("{playback_url}", playback_url)
}

fn get_store(app: &tauri::AppHandle) -> Result<Arc<Store<tauri::Wry>>, String> {
  app.store(STORE_PATH).map_err(|e| e.to_string())
}

fn get_lastfm_session(app: &tauri::AppHandle) -> Option<LastfmSession> {
  let store = get_store(app).ok()?;
  store
    .get("session")
    .and_then(|v| serde_json::from_value::<LastfmSession>(v).ok())
}

fn load_scrobble_config(app: &tauri::AppHandle) -> ScrobbleConfig {
  let store = get_store(app).ok();
  if let Some(store) = store {
    if let Some(val) = store.get("scrobble_config") {
      if let Ok(cfg) = serde_json::from_value::<ScrobbleConfig>(val) {
        return cfg;
      }
    }
  }
  ScrobbleConfig {
    threshold: DEFAULT_THRESHOLD,
    enable_scrobble: true,
    enable_now_playing: true,
    enable_notifications: true,
  }
}

fn save_scrobble_config(app: &tauri::AppHandle, cfg: &ScrobbleConfig) -> Result<(), String> {
  let store = get_store(app)?;
  store.set(
    "scrobble_config",
    serde_json::to_value(cfg).map_err(|e| e.to_string())?,
  );
  store.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_external(app: tauri::AppHandle, url: String) -> Result<(), String> {
  app
    .opener()
    .open_url(url, None::<String>)
    .map_err(|e| e.to_string())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct LastfmSession {
  session_key: String,
  username: String,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct ScrobbleConfig {
  threshold: f32,
  enable_scrobble: bool,
  enable_now_playing: bool,
  enable_notifications: bool,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default)]
struct ScrobbleConfigUpdate {
  threshold: Option<f32>,
  enable_scrobble: Option<bool>,
  enable_now_playing: Option<bool>,
  enable_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackPayload {
  track_id: String,
  title: String,
  artist: String,
  #[serde(default)]
  album: Option<String>,
  duration_ms: u64,
  position_ms: u64,
  paused: bool,
  ts: u64,
}

#[derive(Debug, Default, Clone)]
struct TrackState {
  track_id: String,
  title: String,
  artist: String,
  album: Option<String>,
  duration_ms: u64,
  started_at: u64,
  listened_ms: u64,
  last_pos_ms: u64,
  scrobbled: bool,
  now_playing_sent: bool,
}

#[derive(Default)]
struct ScrobbleState {
  current: Option<TrackState>,
}

#[derive(Clone)]
struct PlaybackEndpoint(String);

fn millis_now() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_millis() as u64)
    .unwrap_or(0)
}

async fn handle_playback(
  app: tauri::AppHandle,
  state: &Arc<Mutex<ScrobbleState>>,
  payload: PlaybackPayload,
) -> Result<(), String> {
  let cfg = load_scrobble_config(&app);
  if !cfg.enable_scrobble {
    return Ok(());
  }

  if payload.duration_ms == 0 || payload.title.is_empty() || payload.artist.is_empty() {
    log::info!(
      "[Last.fm] report_playback skipped missing data title='{}' artist='{}' duration_ms={}",
      payload.title,
      payload.artist,
      payload.duration_ms
    );
    return Ok(());
  }

  let api_key = match lastfm_key() {
    Some(k) => k,
    None => {
      log::info!("[Last.fm] report_playback skipped: api key missing");
      return Ok(());
    }
  };
  let api_secret = match lastfm_secret() {
    Some(s) => s,
    None => {
      log::info!("[Last.fm] report_playback skipped: api secret missing");
      return Ok(());
    }
  };
  let session = match get_lastfm_session(&app) {
    Some(s) => s,
    None => {
      log::info!("[Last.fm] report_playback skipped: no session");
      return Ok(());
    }
  };

  let (now_playing_to_send, scrobble_to_send) = {
    let mut state_lock = state.lock().unwrap();
    let mut now_playing_to_send: Option<TrackState> = None;
    let mut scrobble_to_send: Option<TrackState> = None;

    let is_new_track = match &state_lock.current {
      Some(t) => t.track_id != payload.track_id,
      None => true,
    };

    if is_new_track {
      log::info!(
        "[Last.fm] new track detected: '{}' by '{}' ({} ms)",
        payload.title,
        payload.artist,
        payload.duration_ms
      );
      let mut t = TrackState {
        track_id: payload.track_id.clone(),
        title: payload.title.clone(),
        artist: payload.artist.clone(),
        album: payload.album.clone(),
        duration_ms: payload.duration_ms,
        started_at: millis_now().saturating_sub(payload.position_ms),
        listened_ms: 0,
        last_pos_ms: payload.position_ms,
        scrobbled: false,
        now_playing_sent: false,
      };
      if cfg.enable_now_playing {
        now_playing_to_send = Some(t.clone());
        t.now_playing_sent = true;
      }
      state_lock.current = Some(t);
    } else if let Some(current) = state_lock.current.as_mut() {
      let delta = if payload.position_ms > current.last_pos_ms {
        payload.position_ms - current.last_pos_ms
      } else {
        0
      };
      if !payload.paused {
        current.listened_ms = current.listened_ms.saturating_add(delta);
      }
      current.last_pos_ms = payload.position_ms;

      let threshold_ms = (current.duration_ms as f32 * cfg.threshold).round() as u64;
      if !current.scrobbled && current.listened_ms >= threshold_ms && current.duration_ms > 0 {
        current.scrobbled = true;
        scrobble_to_send = Some(current.clone());
        log::info!(
          "[Last.fm] threshold met for '{}' listened_ms={} threshold_ms={}",
          current.title,
          current.listened_ms,
          threshold_ms
        );
      }
    }

    (now_playing_to_send, scrobble_to_send)
  };

  if let Some(track) = now_playing_to_send {
    match send_now_playing(&session, &api_key, &api_secret, &track).await {
      Ok(_) => log::info!("[Last.fm] now playing sent for '{}'", track.title),
      Err(err) => log::warn!("[Last.fm] now playing failed: {}", err),
    }
  }
  if let Some(track) = scrobble_to_send {
    match send_scrobble(&session, &api_key, &api_secret, &track).await {
      Ok(_) => log::info!("[Last.fm] scrobbled '{}'", track.title),
      Err(err) => log::warn!("[Last.fm] scrobble failed: {}", err),
    }
  }

  Ok(())
}
async fn fetch_lastfm_session(api_key: &str, api_secret: &str, token: &str) -> Result<LastfmSession, String> {
  let sig_base = format!(
    "api_key{}methodauth.getSessiontoken{}{}",
    api_key, token, api_secret
  );
  let sig = format!("{:x}", md5::compute(sig_base.as_bytes()));
  log::info!("[Last.fm] Requesting session for token {}", token);

  #[derive(serde::Deserialize)]
  struct SessionResp {
    session: SessionInner,
  }
  #[derive(serde::Deserialize)]
  struct SessionInner {
    name: String,
    key: String,
  }

  let client = reqwest::Client::new();
  let url = "https://ws.audioscrobbler.com/2.0/";
  let res = client
    .get(url)
    .query(&[
      ("method", "auth.getSession"),
      ("api_key", api_key),
      ("token", token),
      ("api_sig", &sig),
      ("format", "json"),
    ])
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !res.status().is_success() {
    return Err(format!("Last.fm session request failed: {}", res.status()));
  }

  let body: SessionResp = res.json().await.map_err(|e| e.to_string())?;
  Ok(LastfmSession {
    session_key: body.session.key,
    username: body.session.name,
  })
}

#[tauri::command]
async fn get_lastfm_status(app: tauri::AppHandle) -> Result<Option<LastfmSession>, String> {
  let store = get_store(&app)?;
  if let Some(val) = store.get("session") {
    let parsed: LastfmSession = serde_json::from_value(val).map_err(|e| e.to_string())?;
    log::info!("[Last.fm] Returning stored session for user {}", parsed.username);
    Ok(Some(parsed))
  } else {
    log::info!("[Last.fm] No session stored");
    Ok(None)
  }
}

#[tauri::command]
async fn disconnect_lastfm(app: tauri::AppHandle) -> Result<(), String> {
  let store = get_store(&app)?;
  store.delete("session");
  store.save().map_err(|e| e.to_string())
}

fn sign_lastfm(params: &mut Vec<(&str, String)>, api_secret: &str) -> String {
  params.sort_by_key(|(k, _)| k.to_string());
  let mut base = String::new();
  for (k, v) in params.iter() {
    base.push_str(k);
    base.push_str(v);
  }
  base.push_str(api_secret);
  format!("{:x}", md5::compute(base.as_bytes()))
}

async fn lastfm_call(method: &str, params: Vec<(&str, String)>, api_key: &str, api_secret: &str, sk: &str) -> Result<(), String> {
  let mut params = params;
  params.push(("method", method.to_string()));
  params.push(("api_key", api_key.to_string()));
  params.push(("sk", sk.to_string()));
  let api_sig = sign_lastfm(&mut params.clone(), api_secret);

  let mut query: Vec<(&str, String)> = params;
  query.push(("api_sig", api_sig));
  query.push(("format", "json".to_string()));

  let client = reqwest::Client::new();
  let res = client
    .post("https://ws.audioscrobbler.com/2.0/")
    .form(&query)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !res.status().is_success() {
    return Err(format!("Last.fm call {} failed: {}", method, res.status()));
  }

  Ok(())
}

async fn send_now_playing(session: &LastfmSession, api_key: &str, api_secret: &str, track: &TrackState) -> Result<(), String> {
  lastfm_call(
    "track.updateNowPlaying",
    vec![
      ("track", track.title.clone()),
      ("artist", track.artist.clone()),
      ("duration", track.duration_ms.to_string()),
    ],
    api_key,
    api_secret,
    &session.session_key,
  )
  .await
}

async fn send_scrobble(session: &LastfmSession, api_key: &str, api_secret: &str, track: &TrackState) -> Result<(), String> {
  let ts = track.started_at as i64 / 1000;
  lastfm_call(
    "track.scrobble",
    vec![
      ("track[0]", track.title.clone()),
      ("artist[0]", track.artist.clone()),
      ("duration[0]", track.duration_ms.to_string()),
      ("timestamp[0]", ts.to_string()),
    ],
    api_key,
    api_secret,
    &session.session_key,
  )
  .await
}

#[tauri::command]
async fn complete_lastfm(app: tauri::AppHandle, url: String) -> Result<LastfmSession, String> {
  let parsed = Url::parse(&url).map_err(|e| e.to_string())?;
  let token = parsed
    .query_pairs()
    .find(|(k, _)| k == "token")
    .map(|(_, v)| v.to_string())
    .ok_or("missing token")?;

  log::info!("[Last.fm] Received callback with token {}", token);

  let api_key = lastfm_key().ok_or("LASTFM_API_KEY not set")?;
  let api_secret = lastfm_secret().ok_or("LASTFM_API_SECRET not set")?;

  let session = fetch_lastfm_session(&api_key, &api_secret, &token).await?;
  log::info!(
    "[Last.fm] Session established for user {}, key starts with {}***",
    session.username,
    session.session_key.chars().take(4).collect::<String>()
  );

  let store = get_store(&app)?;
  store.set(
    "session",
    serde_json::to_value(&session).map_err(|e| e.to_string())?,
  );
  store.save().map_err(|e| e.to_string())?;
  log::info!("[Last.fm] Session persisted to store");

  Ok(session)
}

#[tauri::command]
async fn report_playback(
  app: tauri::AppHandle,
  state: tauri::State<'_, Arc<Mutex<ScrobbleState>>>,
  payload: PlaybackPayload,
) -> Result<(), String> {
  handle_playback(app, &state, payload).await
}

#[cfg(debug_assertions)]
fn start_dev_callback_server(app: tauri::AppHandle) {
  let app_handle = app.clone();
  tauri::async_runtime::spawn(async move {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:35729").await {
      Ok(l) => {
        log::info!("[Last.fm] Dev callback server listening on {}", DEV_CALLBACK_URL);
        l
      }
      Err(err) => {
        log::warn!("[Last.fm] Failed to bind dev callback server: {}", err);
        return;
      }
    };

    loop {
      let (mut socket, _) = match listener.accept().await {
        Ok(s) => s,
        Err(err) => {
          log::warn!("[Last.fm] Accept failed: {}", err);
          continue;
        }
      };
      let app = app_handle.clone();
      tauri::async_runtime::spawn(async move {
        let mut buf = vec![0u8; 2048];
        let n = match socket.read(&mut buf).await {
          Ok(n) => n,
          Err(err) => {
            log::warn!("[Last.fm] Read failed: {}", err);
            return;
          }
        };
        let req = String::from_utf8_lossy(&buf[..n]);
        let mut token: Option<String> = None;
        if let Some(line) = req.lines().next() {
          if let Some(path) = line.split_whitespace().nth(1) {
            if let Some(q_idx) = path.find('?') {
              let query = &path[q_idx + 1..];
              for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                  if k == "token" {
                    token = Some(v.to_string());
                    break;
                  }
                }
              }
            }
          }
        }

        let response = if let Some(tok) = token {
          let url = format!("{}?token={}", DEV_CALLBACK_URL, tok);
          log::info!("[Last.fm] Dev callback received token {}", tok);
          if let Err(err) = complete_lastfm(app.clone(), url).await {
            log::warn!("[Last.fm] Dev callback processing failed: {}", err);
          }
          "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nYou can close this tab.\r\n"
        } else {
          log::warn!("[Last.fm] Dev callback missing token");
          "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nMissing token.\r\n"
        };

        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
      });
    }
  });
}

fn start_playback_server(
  app: tauri::AppHandle,
  state: Arc<Mutex<ScrobbleState>>,
) -> Option<String> {
  let state_for_server = state.clone();
  let app_handle = app.clone();
  let listener = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
  let port = listener.local_addr().ok()?.port();
  let local_url = format!("http://127.0.0.1:{}/playback", port);

  tauri::async_runtime::spawn(async move {
    let listener = match tokio::net::TcpListener::from_std(listener) {
      Ok(l) => l,
      Err(err) => {
        log::warn!("[Last.fm] Playback server start failed: {}", err);
        return;
      }
    };
    loop {
      let (mut socket, _) = match listener.accept().await {
        Ok(s) => s,
        Err(err) => {
          log::warn!("[Last.fm] Playback server accept failed: {}", err);
          continue;
        }
      };
      log::info!("[Last.fm] Playback server accepted connection");
      let app_clone = app_handle.clone();
      let state_clone = state_for_server.clone();
      tauri::async_runtime::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buf = vec![0u8; 8192];
        let mut total_read = 0usize;
        let mut content_length: Option<usize> = None;
        let mut method = String::new();
        let mut path = String::new();
        loop {
          let n = match socket.read(&mut buf[total_read..]).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(err) => {
              log::warn!("[Last.fm] Playback server read failed: {}", err);
              return;
            }
          };
          total_read += n;
          if total_read >= 4 {
            if let Some(idx) = twoway::find_bytes(&buf[..total_read], b"\r\n\r\n") {
              let headers = &buf[..idx];
              let header_str = String::from_utf8_lossy(headers);
              if let Some(line) = header_str.lines().next() {
                let mut parts = line.split_whitespace();
                method = parts.next().unwrap_or("").to_string();
                path = parts.next().unwrap_or("").to_string();
              }
              for line in headers.split(|b| *b == b'\n') {
                if let Some(pos) = line.iter().position(|b| *b == b':') {
                  let (name, val) = line.split_at(pos);
                  if name.eq_ignore_ascii_case(b"content-length") {
                    if let Ok(len_str) = std::str::from_utf8(&val[1..]).map(|s| s.trim()) {
                      if let Ok(len) = len_str.parse::<usize>() {
                        content_length = Some(len);
                      }
                    }
                  }
                }
              }
              let body_start = idx + 4;
              if method == "OPTIONS" {
                let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
                return;
              }

              let want_body = method == "POST";
              if want_body && content_length.is_none() {
                log::warn!("[Last.fm] Playback server missing content-length");
                let response = "HTTP/1.1 411 Length Required\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
                return;
              }

              if let Some(len) = content_length {
                let needed = body_start + len;
                while total_read < needed {
                  let n = match socket.read(&mut buf[total_read..]).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(err) => {
                      log::warn!("[Last.fm] Playback server read failed: {}", err);
                      return;
                    }
                  };
                  total_read += n;
                }
                let body = &buf[body_start..std::cmp::min(total_read, body_start + len)];
                if path == "/settings" {
                  if method == "POST" {
                    match serde_json::from_slice::<ScrobbleConfigUpdate>(body) {
                      Ok(update) => {
                        let mut cfg = load_scrobble_config(&app_clone);
                        if let Some(v) = update.threshold {
                          cfg.threshold = v.clamp(0.01, 1.0);
                        }
                        if let Some(v) = update.enable_scrobble {
                          cfg.enable_scrobble = v;
                        }
                        if let Some(v) = update.enable_now_playing {
                          cfg.enable_now_playing = v;
                        }
                        if let Some(v) = update.enable_notifications {
                          cfg.enable_notifications = v;
                        }
                        let _ = save_scrobble_config(&app_clone, &cfg);
                        let body = serde_json::to_string(&cfg).unwrap_or_else(|_| "{}".to_string());
                        let response = format!(
                          "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n{}",
                          body
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                        let _ = socket.shutdown().await;
                        return;
                      }
                      Err(err) => {
                        log::warn!("[Last.fm] Settings JSON parse failed: {}", err);
                      }
                    }
                  }
                } else if path == "/playback" {
                  log::info!("[Last.fm] Playback server received {} bytes", body.len());
                  match serde_json::from_slice::<PlaybackPayload>(body) {
                    Ok(payload) => {
                      if let Err(err) = handle_playback(app_clone.clone(), &state_clone, payload).await {
                        log::warn!("[Last.fm] handle_playback from server failed: {}", err);
                      }
                    }
                    Err(err) => {
                      log::warn!("[Last.fm] Playback server JSON parse failed: {}", err);
                    }
                  }
                }
              } else if method == "GET" && path == "/settings" {
                let cfg = load_scrobble_config(&app_clone);
                let body = serde_json::to_string(&cfg).unwrap_or_else(|_| "{}".to_string());
                let response = format!(
                  "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n{}",
                  body
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
                return;
              }
              break;
            }
          }
          if total_read == buf.len() {
            buf.resize(buf.len() * 2, 0);
          }
        }
        let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\nok").await;
        let _ = socket.shutdown().await;
      });
    }
  });

  Some(local_url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let key_for_overlay = lastfm_key().unwrap_or_else(|| "REPLACE_ME".to_string());
  let lastfm_cb = lastfm_callback();
  let context = tauri::generate_context!();
  let version = context
    .config()
    .version
    .clone()
    .unwrap_or_else(|| "0.0.0".to_string());

  let playback_url_holder = Arc::new(Mutex::new(String::new()));
  let playback_url_for_load = playback_url_holder.clone();
  let playback_url_for_setup = playback_url_holder.clone();

  let mut builder = tauri::Builder::default();

  if cfg!(debug_assertions) {
    builder = builder.plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    );
  }

  builder = builder
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_deep_link::init())
    .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
      log::info!("[Last.fm] Single-instance callback with argv: {:?}", argv);
      // On Windows, protocol handlers spawn a new instance; forward to the existing one.
      if let Some(url) = argv
        .iter()
        .find(|a| a.contains("mscd://"))
        .cloned()
      {
        log::info!("[Last.fm] Forwarding URL from secondary instance: {}", url);
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
          if let Err(err) = complete_lastfm(app_handle, url.clone()).await {
            log::warn!("[Last.fm] Failed to handle single-instance callback {}: {}", url, err);
          }
        });
      } else {
        log::info!("[Last.fm] No mscd:// URL found in single-instance argv");
      }
    }))
    .invoke_handler(tauri::generate_handler![
      open_external,
      complete_lastfm,
      get_lastfm_status,
      disconnect_lastfm,
      report_playback
    ])
    .setup(move |app| {
      app.manage(Arc::new(Mutex::new(ScrobbleState::default())));
      let scrobble_state = app.state::<Arc<Mutex<ScrobbleState>>>();
      if let Some(url) = start_playback_server(app.handle().clone(), scrobble_state.inner().clone()) {
        if let Ok(mut w) = playback_url_for_setup.lock() {
          *w = url.clone();
        }
        log::info!("[Last.fm] Playback server at {}", url);
      } else {
        log::warn!("[Last.fm] Failed to start playback server");
      }
      #[cfg(debug_assertions)]
      {
        start_dev_callback_server(app.handle().clone());
      }
      #[cfg(desktop)]
      {
        let handle = app.handle().clone();
        let deep = handle.deep_link();
        let _ = deep.register_all();

        if let Ok(Some(urls)) = deep.get_current() {
          log::info!("[Last.fm] deep_link get_current: {:?}", urls);
          for url in urls {
            let app_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
              if let Err(err) = complete_lastfm(app_handle, url.to_string()).await {
                log::warn!("Failed to handle Last.fm callback: {}", err);
              }
            });
          }
        }

        let handle_for_events = handle.clone();
        deep.on_open_url(move |event| {
          let urls = event.urls();
          log::info!("[Last.fm] deep_link on_open_url: {:?}", urls);
          for url in urls {
            let app_handle = handle_for_events.clone();
            tauri::async_runtime::spawn(async move {
              if let Err(err) = complete_lastfm(app_handle, url.to_string()).await {
                log::warn!("Failed to handle Last.fm callback: {}", err);
              }
            });
          }
        });
      }
      Ok(())
    })
    .on_page_load({
      let key_for_overlay = key_for_overlay.clone();
      let lastfm_cb = lastfm_cb.clone();
      let version = version.clone();
      move |window, _| {
        let playback_url = playback_url_for_load.lock().unwrap().clone();
        let script = build_overlay_script(&key_for_overlay, &lastfm_cb, &version, &playback_url);
      let _ = window.eval(&script);
      }
    });

  builder
    .run(context)
    .expect("error while running tauri application");
}
