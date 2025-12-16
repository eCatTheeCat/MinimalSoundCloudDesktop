#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Lightweight overlay injected into the SoundCloud page to prove we can
  // render UI elements even when loading a remote site directly.
  const OVERLAY_SCRIPT: &str = r#"
    (() => {
      if (window.__minimal_sc_overlay_installed) return;
      window.__minimal_sc_overlay_installed = true;

      function inject() {
        const host = document.createElement('div');
        host.id = 'mscd-overlay-host';
        host.style.position = 'fixed';
        host.style.top = '10px';
        host.style.right = '10px';
        host.style.zIndex = '2147483647';
        host.style.pointerEvents = 'none';
        document.body.appendChild(host);

        const shadow = host.attachShadow({ mode: 'open' });

        const style = document.createElement('style');
        style.textContent = `
          :host {
            all: initial;
          }
          * { box-sizing: border-box; }
          .shell {
            pointer-events: auto;
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 10px;
            background: rgba(10, 12, 18, 0.92);
            border: 1px solid rgba(255,255,255,0.12);
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.35);
            backdrop-filter: blur(8px);
            color: #e9ecf5;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          }
          .pill {
            padding: 4px 8px;
            border-radius: 999px;
            background: rgba(255, 194, 74, 0.12);
            border: 1px solid rgba(255, 194, 74, 0.28);
            color: #ffc24a;
            font-size: 12px;
            letter-spacing: 0.05em;
            text-transform: uppercase;
            white-space: nowrap;
          }
          .actions {
            display: flex;
            gap: 8px;
          }
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
          .modal-backdrop.open {
            display: flex;
            pointer-events: auto;
          }
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
          .row {
            display: flex;
            justify-content: space-between;
            gap: 8px;
            align-items: center;
          }
          .row + .row { margin-top: 4px; }
          .muted { color: #a7acb8; font-size: 13px; }
          .toggle {
            display: flex;
            align-items: center;
            gap: 8px;
          }
          .toggle input {
            accent-color: #3c57ff;
          }
          .slider {
            width: 160px;
          }
          .section h3 {
            margin: 0;
            font-size: 15px;
          }
          .section {
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 10px;
            padding: 12px;
            background: rgba(255,255,255,0.03);
          }
          .modal header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 8px;
          }
          .close {
            height: 32px;
            padding: 0 10px;
          }
        `;
        shadow.appendChild(style);

        let settingsOpen = false;
        let darkMode = true;

        const shell = document.createElement('div');
        shell.className = 'shell';

        const pill = document.createElement('span');
        pill.className = 'pill';
        pill.textContent = 'Minimal SC Desktop';

        const actions = document.createElement('div');
        actions.className = 'actions';

        const btnSettings = document.createElement('button');
        btnSettings.textContent = 'Settings';

        const btnDark = document.createElement('button');
        btnDark.textContent = 'Dark mode';

        const btnTray = document.createElement('button');
        btnTray.textContent = 'Minimize to tray';

        actions.append(btnSettings, btnDark, btnTray);
        shell.append(pill, actions);

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

        const secPlayback = document.createElement('div');
        secPlayback.className = 'section';
        const s1Title = document.createElement('h3');
        s1Title.textContent = 'Playback & Ads';
        const adRow = document.createElement('div');
        adRow.className = 'row';
        adRow.innerHTML = `<span>Skip audio ads</span><label class="toggle"><input type="checkbox" checked /></label>`;
        const promoRow = document.createElement('div');
        promoRow.className = 'row';
        promoRow.innerHTML = `<span>Skip promoted tracks</span><label class="toggle"><input type="checkbox" checked /></label>`;
        secPlayback.append(s1Title, adRow, promoRow);

        const secScrobble = document.createElement('div');
        secScrobble.className = 'section';
        const s2Title = document.createElement('h3');
        s2Title.textContent = 'Scrobbling';
        const thresholdRow = document.createElement('div');
        thresholdRow.className = 'row';
        thresholdRow.innerHTML = `
          <span>Scrobble threshold</span>
          <div class="toggle">
            <input class="slider" type="range" min="1" max="100" value="50" />
            <span class="muted" id="mscd-threshold-label">50%</span>
          </div>
        `;
        const nowPlayingRow = document.createElement('div');
        nowPlayingRow.className = 'row';
        nowPlayingRow.innerHTML = `<span>Send "Now Playing"</span><label class="toggle"><input type="checkbox" checked /></label>`;
        const notifyRow = document.createElement('div');
        notifyRow.className = 'row';
        notifyRow.innerHTML = `<span>Show scrobble notifications</span><label class="toggle"><input type="checkbox" checked /></label>`;
        secScrobble.append(s2Title, thresholdRow, nowPlayingRow, notifyRow);

        const secLastfm = document.createElement('div');
        secLastfm.className = 'section';
        const s3Title = document.createElement('h3');
        s3Title.textContent = 'Last.fm';
        const lfRow = document.createElement('div');
        lfRow.className = 'row';
        lfRow.innerHTML = `<span>Status: <strong>Not connected</strong></span><button>Connect</button>`;
        secLastfm.append(s3Title, lfRow);

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

        const slider = thresholdRow.querySelector('.slider');
        const label = thresholdRow.querySelector('#mscd-threshold-label');
        slider?.addEventListener('input', () => {
          label.textContent = `${slider.value}%`;
        });

        shadow.append(shell, backdrop);
      }

      const ready = () => document.body ? inject() : setTimeout(ready, 50);
      ready();
    })();
  "#;

  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .on_page_load(|window, _payload| {
      let script = OVERLAY_SCRIPT;
      // Fire-and-forget; CSP does not block eval() injected via Tauri.
      let _ = window.eval(script);
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
