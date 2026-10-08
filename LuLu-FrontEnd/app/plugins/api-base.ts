export default defineNuxtPlugin(() => {
    const config = useRuntimeConfig()
    if (config.public.apiBase) return

    let hostname = ''
    if (import.meta.client) {
        hostname = window.location.hostname
    } else {
        try {
            hostname = useRequestURL().hostname
        } catch {
            return
        }
    }
    if (hostname) config.public.apiBase = `http://${hostname}:4343/api`
})
