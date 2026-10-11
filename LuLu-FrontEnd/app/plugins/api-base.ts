/**
 * Resolve the API origin at request time instead of baking an IP into the build.
 *
 * The LAN address comes from DHCP and has already changed twice in one session;
 * baking `NUXT_PUBLIC_API_BASE` means every lease change forces a rebuild and a
 * restart. Reading the hostname the client actually used (its `Location.origin`
 * on the client, the request's host header during SSR) always points at the
 * right machine, so `localhost`, any LAN IP, and static hosting all work from
 * one build. `NUXT_PUBLIC_API_BASE` still wins when set (static/other-origin
 * deployments).
 */
export default defineNuxtPlugin(() => {
    const config = useRuntimeConfig()
    if (config.public.apiBase) return

    let hostname = ''
    if (import.meta.client) {
        hostname = window.location.hostname
    } else {
        // SSR/prerender may run without a request event; leave apiBase empty
        // then - the client takes over with window.location.hostname.
        try {
            hostname = useRequestURL().hostname
        } catch {
            return
        }
    }
    if (hostname) config.public.apiBase = `http://${hostname}:4343/api`
})
