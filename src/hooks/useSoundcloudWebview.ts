import { useEffect, useRef } from 'react'
import { appWindow } from '@tauri-apps/api/window'
import { Webview } from '@tauri-apps/api/webview'

type Bounds = {
  x: number
  y: number
  width: number
  height: number
}

const RIBBON_HEIGHT = 54 // keep in sync with CSS

/**
 * Creates and maintains a native webview pointing at SoundCloud, sized to the host element.
 * This avoids iframe sandbox issues and keeps us able to inject scripts later.
 */
export function useSoundcloudWebview(hostRef: React.RefObject<HTMLElement | null>) {
  const webviewRef = useRef<Webview | null>(null)
  const resizeObserverRef = useRef<ResizeObserver | null>(null)

  useEffect(() => {
    let disposed = false

    async function ensureWebview() {
      if (disposed || webviewRef.current) return

      const host = hostRef.current
      if (!host) return

      const { x, y, width, height } = getHostBounds(host)

      const webview = await Webview.create({
        label: 'soundcloud-webview',
        url: 'https://soundcloud.com',
        bounds: { x, y, width, height },
        devtools: false,
        visible: true,
        focus: true,
        transparent: false,
        resizable: true,
        acceptFirstMouse: true,
      })

      webviewRef.current = webview
    }

    function getHostBounds(host: HTMLElement): Bounds {
      const rect = host.getBoundingClientRect()
      const scale = window.devicePixelRatio || 1
      // Translate logical CSS pixels to physical for native webview bounds.
      return {
        x: Math.round(rect.x * scale),
        y: Math.round((rect.y + RIBBON_HEIGHT) * scale), // offset to match ribbon height in title bar area
        width: Math.round(rect.width * scale),
        height: Math.round(rect.height * scale),
      }
    }

    function attachResizeObserver() {
      if (resizeObserverRef.current || !hostRef.current) return
      resizeObserverRef.current = new ResizeObserver(async () => {
        const host = hostRef.current
        const webview = webviewRef.current
        if (!host || !webview) return
        const next = getHostBounds(host)
        await webview.setBounds(next)
      })
      resizeObserverRef.current.observe(hostRef.current)
    }

    function attachWindowResize() {
      const handler = async () => {
        const host = hostRef.current
        const webview = webviewRef.current
        if (!host || !webview) return
        const next = getHostBounds(host)
        await webview.setBounds(next)
      }
      window.addEventListener('resize', handler)
      return () => window.removeEventListener('resize', handler)
    }

    ensureWebview()
    attachResizeObserver()
    const detachWindowResize = attachWindowResize()

    return () => {
      disposed = true
      detachWindowResize?.()
      resizeObserverRef.current?.disconnect()
      resizeObserverRef.current = null
      if (webviewRef.current) {
        webviewRef.current?.destroy()
        webviewRef.current = null
      }
    }
  }, [hostRef])
}
