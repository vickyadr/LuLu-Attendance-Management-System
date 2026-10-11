import { ref, onUnmounted } from 'vue'

/**
 * Live transaction feed over WebSocket.
 *
 * The backend broadcasts an `enroll` event to every connected client each time
 * a device uploads a batch. This composable owns one connection per component
 * instance, reconnects on drop, and exposes a reactive `status` so the UI can
 * say what the socket is actually doing instead of a hardcoded label.
 *
 * `onEvent` receives the raw parsed frame. Callers decide what to do - the live
 * table refetches, a toast might just announce.
 */
export function useLiveFeed(onEvent: (data: any) => void) {
    const status = ref<'connecting' | 'open' | 'reconnecting' | 'closed'>('connecting')
    const lastEvent = ref(0)

    let socket: WebSocket | null = null
    let retryTimer: ReturnType<typeof setTimeout> | null = null
    let retryDelay = 1000
    let disposed = false

    const token = useCookie('XSRF-TOKEN')
    const config = useRuntimeConfig()

    // The API base is http(s)://host:4343/api; the socket wants ws(s)://host:4343.
    // Deriving it keeps this working across LAN IP changes, the same reason the
    // REST calls resolve their origin at runtime.
    function wsUrl() {
        const origin = (config.public.apiBase as string) || ''
        const u = origin.replace(/^http/, 'ws').replace(/\/api\/?$/, '')
        return `${u}/api/ws?token=${encodeURIComponent(token.value || '')}`
    }

    // Demo mode has no server to hold a socket open: the mock API runs in the
    // page, and a static host cannot upgrade a connection. Opening a real socket
    // here would just fail and re-fail on a 15s backoff, so simulate the feed on
    // a timer instead. The event shape is the one the real backend broadcasts.
    const DEMO_MODE = (config.public as any).demoMode === true
    let demoTimer: ReturnType<typeof setInterval> | null = null

    function startDemoFeed() {
        status.value = 'open'
        let seq = 1
        demoTimer = setInterval(() => {
            seq += 1
            lastEvent.value = Date.now()
            onEvent({ enroll: { enroll_id: 10000 + seq } })
        }, 8000)
    }

    function connect() {
        if (disposed) return
        if (DEMO_MODE) {
            startDemoFeed()
            return
        }
        status.value = retryTimer ? 'reconnecting' : 'connecting'
        try {
            socket = new WebSocket(wsUrl())
        } catch {
            scheduleRetry()
            return
        }

        socket.onopen = () => {
            status.value = 'open'
            retryDelay = 1000
        }

        socket.onmessage = (ev) => {
            try {
                const data = JSON.parse(ev.data)
                lastEvent.value = Date.now()
                onEvent(data)
            } catch {
                // A frame we cannot parse is not worth tearing the feed down for.
            }
        }

        socket.onclose = () => {
            socket = null
            if (disposed) return
            status.value = 'closed'
            scheduleRetry()
        }

        socket.onerror = () => {
            // `onclose` always follows `onerror`, so retry lives there only.
            socket?.close()
        }
    }

    function scheduleRetry() {
        if (disposed || retryTimer) return
        status.value = 'reconnecting'
        retryTimer = setTimeout(() => {
            retryTimer = null
            retryDelay = Math.min(retryDelay * 2, 15000)
            connect()
        }, retryDelay)
    }

    function send(payload: unknown) {
        if (socket?.readyState === WebSocket.OPEN) {
            socket.send(typeof payload === 'string' ? payload : JSON.stringify(payload))
            return true
        }
        return false
    }

    function close() {
        disposed = true
        if (retryTimer) clearTimeout(retryTimer)
        retryTimer = null
        if (demoTimer) clearInterval(demoTimer)
        demoTimer = null
        socket?.close()
        socket = null
        status.value = 'closed'
    }

    onUnmounted(close)
    connect()

    return { status, lastEvent, send, close }
}
