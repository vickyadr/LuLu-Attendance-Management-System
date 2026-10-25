import { FetchError, ofetch } from 'ofetch'

/** Shape of every backend response body. `error` is the i18n key the backend
 *  wants rendered (`payroll.payslipsExistPeriod`); absent means "show `message`
 *  as-is". Key equals the locale path, so no lookup table on either side. */
type Envelope = { code: number; message: string; data?: unknown; error?: string }

/**
 * Resolve `envelope.error` through i18n and write the result into `message`.
 * vue-i18n echoes an unresolved key back verbatim, so that check is the
 * "no such translation" guard. No i18n context (store outside setup, teardown)
 * throws and leaves `message` untouched: callers keep the English fallback.
 */
function translate(envelope: Envelope): Envelope {
    const key = envelope.error
    if (!key) return envelope
    try {
        const localized = useI18n().t(key)
        const text = String(localized ?? '')
        if (!text || text === key) return envelope
        return { ...envelope, message: text }
    } catch {
        return envelope
    }
}

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
        let body = err?.data as Envelope | string | undefined
        // Some framework errors (the 413 body-size guard) arrive as text/plain
        // even though the body is JSON: parse it before giving up on the envelope.
        if (typeof body === 'string') {
            try {
                body = JSON.parse(body) as Envelope
            } catch {
                /* not JSON: fall through to the synthesized envelope below */
            }
        }
        if (body && typeof body === 'object' && typeof (body as Envelope).code === 'number') {
            return translate(body as Envelope) as unknown as T
        }
        const status = err?.statusCode ?? 500
        return {
            code: status,
            message: status === 413 ? 'Payload too large' : (err?.message || 'Request failed'),
            data: [],
        } as unknown as T
    })
}
