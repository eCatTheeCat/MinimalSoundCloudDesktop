fn build_overlay_script(lastfm_key: &str, lastfm_callback: &str) -> String {
  let auth_url = format!(
    "https://www.last.fm/api/auth/?api_key={}&cb={}",
    lastfm_key, lastfm_callback
  );

  let template = r#"
    (() => {
      if (window.__minimal_sc_overlay_installed) return;
      window.__minimal_sc_overlay_installed = true;

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
            min-height: 44px;
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
            height: 32px;
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

          const statusWrap = document.createElement('span');
          statusWrap.innerHTML = 'Status: <strong>Not connected</strong>';

          const toggleWrap = document.createElement('div');
          toggleWrap.className = 'toggle';
          toggleWrap.style.gap = '12px';

          const btn = document.createElement('button');
          btn.id = 'mscd-lastfm-connect';
          btn.textContent = 'Connect in browser';
          if (keyMissing) btn.disabled = true;

          toggleWrap.append(btn);
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

          return { row, btn, warnNode, authInfo };
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
        const nowPlayingRow = makeToggleRow('Send "Now Playing"');
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

        const connectBtn = lf.btn;
        console.info('[MSCD] Connect button present:', !!connectBtn, 'key missing:', keyMissing);
        connectBtn?.addEventListener('click', () => {
          console.info('[MSCD] Connect clicked');
          if (keyMissing) {
            console.warn('[MSCD] Key missing; connect disabled');
            return;
          }

          const fallbackOpen = () => {
            const opened = window.open(authUrl, '_blank', 'noopener,noreferrer');
            if (!opened) {
              console.warn('[MSCD] window.open blocked, trying anchor click');
              const anchor = document.createElement('a');
              anchor.href = authUrl;
              anchor.target = '_blank';
              anchor.rel = 'noopener noreferrer';
              shadow.appendChild(anchor);
              anchor.click();
              shadow.removeChild(anchor);
            }
            if (!opened) {
              console.warn('[MSCD] Fallback to same-window navigation');
              window.location.href = authUrl;
            }
          };

          try {
            const tauri = (window).__TAURI__;
            const invoke = tauri?.invoke ?? tauri?.core?.invoke;
            if (invoke) {
              console.info('[MSCD] Opening via open_external command', authUrl);
              Promise.resolve(invoke('open_external', { url: authUrl }))
                .then(() => console.info('[MSCD] open_external success'))
                .catch((err) => {
                  console.warn('[MSCD] open_external failed; falling back', err);
                  fallbackOpen();
                });
              return;
            } else {
              console.warn('[MSCD] __TAURI__.invoke unavailable; falling back to window.open');
            }
          } catch (e) {
            console.warn('TAURI shell unavailable, falling back to window.open', e);
          }

          fallbackOpen();
        });

        const slider = thresholdRow.slider;
        const label = thresholdRow.val;
        slider?.addEventListener('input', () => {
          label.textContent = `${slider.value}%`;
        });

        shadow.append(shell, backdrop);
      }

      const ready = () => document.body ? inject() : setTimeout(ready, 50);
      ready();
    })();
    "#;

  template
    .replace("{auth_url}", &auth_url)
    .replace("{key}", lastfm_key)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let lastfm_key = option_env!("LASTFM_API_KEY").unwrap_or("fca939e737410506a2c49ec7ee49ba68");
  let lastfm_callback = option_env!("LASTFM_CALLBACK").unwrap_or("mscd://lastfm-callback");
  let script = build_overlay_script(lastfm_key, lastfm_callback);

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![open_external])
    .setup(|app| {
      if cfg!(debug_assertions) {
        let _ = app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      let _ = app.handle().plugin(tauri_plugin_opener::init());
      Ok(())
    })
    .on_page_load(move |window, _| {
      let _ = window.eval(&script);
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
async fn open_external(app: tauri::AppHandle, url: String) -> Result<(), String> {
  app.opener().open_url(url, None::<String>).map_err(|e| e.to_string())
}
