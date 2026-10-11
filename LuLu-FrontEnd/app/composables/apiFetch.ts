import { FetchError, ofetch } from 'ofetch'

/** Shape of every backend response body. */
type Envelope = { code: number; message: string; data?: unknown }

/**
 * Demo mode: answer every request from the in-browser mock instead of the
 * backend, so the app can be published to a static host with no server and no
 * database. The mock returns the same `{ code, message, data }` envelope, so
 * callers cannot tell the difference. With the flag off, nothing changes.
 */
const DEMO_MODE = import.meta.env.NUXT_PUBLIC_DEMO_MODE === 'true'

/**
 * Demo mode can also arrive at runtime through `runtimeConfig.public.demoMode`
 * (Nuxt exposes `NUXT_PUBLIC_*` under both), so accept either. The runtime read is
 * guarded because `apiFetch` is also called from stores, where the Nuxt context
 * may not be active.
 */
function demoEnabled(): boolean {
    if (DEMO_MODE) return true
    try {
        return useRuntimeConfig().public.demoMode === true || useRuntimeConfig().public.demoMode === 'true'
    } catch {
        return false
    }
}

/**
 * Same as `$fetch`, but never throws on an HTTP error status.
 *
 * The API now answers errors with their real status (401/403/404/500), and
 * `$fetch` throws on any non-2xx - which would surface as an unhandled
 * exception instead of the `response.code` check every caller already performs.
 * The backend keeps the same JSON envelope in every error body, so return that
 * body as-is: callers read `code`/`message` exactly as before.
 */
export function apiFetch<T = any>(url: string, options?: any): Promise<T> {
    // Resolved lazily so the mock's ~600 lines are not in the chunk for dev and
    // production builds that never touch them.
    if (demoEnabled()) {
        return import('./mockApi.js').then(({ mockRequest }) => mockRequest(url, options) as T)
    }

    return ofetch(url, options).catch((err: FetchError) => {
        const body = err?.data as Envelope | undefined
        if (body && typeof body === 'object' && typeof body.code === 'number') {
            return body as unknown as T
        }
        // Network failure or non-JSON body: synthesize the envelope so callers
        // still get a code and a message instead of an exception.
        return {
            code: err?.statusCode ?? 500,
            message: err?.message || 'Request failed',
            data: [],
        } as unknown as T
    })
}
