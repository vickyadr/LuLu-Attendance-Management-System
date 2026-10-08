import { FetchError, ofetch } from 'ofetch'

/** Shape of every backend response body. */
type Envelope = { code: number; message: string; data?: unknown }

const DEMO_MODE = import.meta.env.NUXT_PUBLIC_DEMO_MODE === 'true'

function demoEnabled(): boolean {
    if (DEMO_MODE) return true
    try {
        return useRuntimeConfig().public.demoMode === true || useRuntimeConfig().public.demoMode === 'true'
    } catch {
        return false
    }
}

export function apiFetch<T = any>(url: string, options?: any): Promise<T> {

    if (demoEnabled()) {
        return import('./mockApi.js').then(({ mockRequest }) => mockRequest(url, options) as T)
    }

    return ofetch(url, options).catch((err: FetchError) => {
        const body = err?.data as Envelope | undefined
        if (body && typeof body === 'object' && typeof body.code === 'number') {
            return body as unknown as T
        }

        return {
            code: err?.statusCode ?? 500,
            message: err?.message || 'Request failed',
            data: [],
        } as unknown as T
    })
}
