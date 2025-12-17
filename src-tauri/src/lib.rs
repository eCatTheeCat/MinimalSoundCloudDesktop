use std::sync::Arc;

use serde::Deserialize;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::{Store, StoreExt};
use url::Url;

const STORE_PATH: &str = "lastfm.json";
const DEV_CALLBACK_URL: &str = "http://127.0.0.1:35729/callback";

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

fn build_overlay_script(lastfm_key: &str, lastfm_callback: &str) -> String {
  let auth_url = format!(
    "https://www.last.fm/api/auth/?api_key={}&cb={}",
    lastfm_key, lastfm_callback
  );

  let template = r#"
    (() => {
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
        status.textContent = 'Ad-free - Scrobbling-ready';
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
        const thresholdRow = makeSliderRow();
        const nowPlayingRow = makeToggleRow('Send \"Now Playing\"');
        const notifyRow = makeToggleRow('Show scrobble notifications');
        secScrobble.append(s2Title, thresholdRow.row, nowPlayingRow.row, notifyRow.row);

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
      }

      const ready = () => (document.body ? inject() : setTimeout(ready, 50));
      ready();
    })();
    "#;

  template
    .replace("{auth_url}", &auth_url)
    .replace("{key}", lastfm_key)
}

fn get_store(app: &tauri::AppHandle) -> Result<Arc<Store<tauri::Wry>>, String> {
  app.store(STORE_PATH).map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let key_for_overlay = lastfm_key().unwrap_or_else(|| "REPLACE_ME".to_string());
  let lastfm_cb = lastfm_callback();
  let script = build_overlay_script(&key_for_overlay, &lastfm_cb);

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
      disconnect_lastfm
    ])
    .setup(|app| {
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
    .on_page_load(move |window, _| {
      let _ = window.eval(&script);
    });

  builder
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
