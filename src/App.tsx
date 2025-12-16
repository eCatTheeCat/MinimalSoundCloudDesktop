import { useState } from 'react'
import './App.css'

type IconButtonProps = {
  label: string
  onClick?: () => void
}

function IconButton({ label, onClick }: IconButtonProps) {
  return (
    <button className="icon-button" onClick={onClick} type="button">
      {label}
    </button>
  )
}

function App() {
  const [isSettingsOpen, setSettingsOpen] = useState(false)

  return (
    <div className="app-shell">
      <header className="ribbon">
        <div className="brand">
          <span className="pill">Beta scaffold</span>
          <span className="title">Minimal SoundCloud Desktop</span>
        </div>
        <div className="ribbon-actions">
          <IconButton label="Dark mode" />
          <IconButton label="Settings" onClick={() => setSettingsOpen(true)} />
          <IconButton label="Minimize to tray" />
        </div>
      </header>

      <main className="main-surface">
        <iframe
          title="SoundCloud"
          src="https://soundcloud.com"
          className="sc-frame"
          allow="autoplay; clipboard-write; encrypted-media"
        />
      </main>

      {isSettingsOpen ? (
        <div className="modal-backdrop" onClick={() => setSettingsOpen(false)}>
          <div
            className="modal"
            role="dialog"
            aria-modal="true"
            aria-label="Settings (placeholder)"
            onClick={(e) => e.stopPropagation()}
          >
            <h2>Settings</h2>
            <p className="muted">
              Settings modal placeholder. This will host ad-block toggles, promoted-track skipping,
              Last.fm auth, scrobble threshold, notifications, and dark mode controls.
            </p>
            <button className="primary" type="button" onClick={() => setSettingsOpen(false)}>
              Close
            </button>
          </div>
        </div>
      ) : null}
    </div>
  )
}

export default App
