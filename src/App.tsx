import './App.css'

const checklist = [
  'Tauri + React shell scaffolded',
  'Run `npm run tauri:dev` to launch the desktop shell',
  'App window + tray + settings modal to be added next',
]

function App() {
  return (
    <main className="app">
      <header className="hero">
        <p className="eyebrow">Minimal SoundCloud Desktop</p>
        <h1>Wrapper shell is ready</h1>
        <p className="lede">
          This build is the starting point for the ad-free SoundCloud wrapper with
          Last.fm scrobbling. Next steps live here in the React layer; the Tauri
          backend is already wired up.
        </p>
      </header>

      <section className="card">
        <h2>What&apos;s in this scaffold</h2>
        <ul className="list">
          {checklist.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </section>
    </main>
  )
}

export default App
