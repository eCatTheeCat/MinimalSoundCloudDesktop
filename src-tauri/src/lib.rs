use tauri::{Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Lightweight overlay injected into the SoundCloud page to prove we can
  // render UI elements even when loading a remote site directly.
  const OVERLAY_SCRIPT: &str = r#"
    (() => {
      if (window.__minimal_sc_overlay_installed) return;
      window.__minimal_sc_overlay_installed = true;

      const style = document.createElement('style');
      style.textContent = `
        .mscd-overlay {
          position: fixed;
          top: 10px;
          right: 10px;
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 10px;
          background: rgba(10, 12, 18, 0.9);
          border: 1px solid rgba(255,255,255,0.12);
          border-radius: 10px;
          z-index: 2147483647;
          box-shadow: 0 8px 30px rgba(0,0,0,0.35);
          backdrop-filter: blur(6px);
          color: #e9ecf5;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          pointer-events: auto;
        }
        .mscd-overlay button {
          height: 32px;
          padding: 0 12px;
          border-radius: 8px;
          border: 1px solid rgba(255,255,255,0.16);
          background: #1b202b;
          color: #e9ecf5;
          font-weight: 600;
          cursor: pointer;
          transition: background 120ms ease, border-color 120ms ease, transform 120ms ease;
        }
        .mscd-overlay button:hover { background: #252c3b; border-color: rgba(255,255,255,0.22); }
        .mscd-overlay button:active { transform: translateY(1px); }
        .mscd-overlay .pill {
          padding: 4px 8px;
          border-radius: 999px;
          background: rgba(255, 194, 74, 0.14);
          border: 1px solid rgba(255, 194, 74, 0.3);
          color: #ffc24a;
          font-size: 12px;
          letter-spacing: 0.05em;
          text-transform: uppercase;
        }
      `;
      document.head.appendChild(style);

      const overlay = document.createElement('div');
      overlay.className = 'mscd-overlay';

      const pill = document.createElement('span');
      pill.className = 'pill';
      pill.textContent = 'Preview overlay';

      const settingsBtn = document.createElement('button');
      settingsBtn.textContent = 'Settings';
      settingsBtn.onclick = () => alert('Settings placeholder – full UI will be injected later.');

      const darkBtn = document.createElement('button');
      darkBtn.textContent = 'Dark mode';
      darkBtn.onclick = () => alert('Dark mode toggle placeholder – will be wired later.');

      overlay.appendChild(pill);
      overlay.appendChild(settingsBtn);
      overlay.appendChild(darkBtn);

      document.body.appendChild(overlay);
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

      if let Some(main) = app.get_webview_window("main") {
        let script = OVERLAY_SCRIPT.to_string();
        main.clone().listen("tauri://page-load", move |_| {
          let _ = main.eval(script.as_str());
        });
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
