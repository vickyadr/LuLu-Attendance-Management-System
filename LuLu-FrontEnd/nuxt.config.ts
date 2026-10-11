// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },
  ssr: false,
  devServer: { host: '0.0.0.0', port: 3232 },
  nitro: { preset: 'node-server' },
  css: ['~/assets/css/green-theme.css'],
  app: {
    head: {
      title: "LuLu Attendance System",
      meta: [
        { name: 'theme-color', content: '#059669' },
        { name: 'description', content: 'LuLu Attendance System - kelola karyawan, shift, jadwal, perangkat, dan laporan absensi real-time.' },
      ],
      link: [
        { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
        { rel: 'icon', type: 'image/png', sizes: '16x16', href: '/favicon-16x16.png' },
        { rel: 'icon', type: 'image/png', sizes: '32x32', href: '/favicon-32x32.png' },
        { rel: 'icon', type: 'image/png', sizes: '96x96', href: '/lulu-96.png' },
        { rel: 'apple-touch-icon', sizes: '180x180', href: '/apple-touch-icon.png' },
        { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@500;700&family=Plus+Jakarta+Sans:wght@600;700;800&display=swap' },
      ],
    }
  },
  hooks: {
    'prerender:routes'({ routes }) {
      routes.clear()
    }
  },
  runtimeConfig: {
    public: {
      // Same-origin by default: the browser resolves a relative URL against
      // whatever host served the page, so LAN and localhost both work without
      // hardcoding an IP. Override with NUXT_PUBLIC_API_BASE when the API is
      // on a different origin (e.g. static hosting on another domain).
      apiBase: process.env.NUXT_PUBLIC_API_BASE || '',
      // Demo build: NUXT_PUBLIC_DEMO_MODE=true swaps the real backend for the
      // in-browser mock (app/composables/mockApi.js) so the app can be published
      // to a static host with no server and no database. Unset in dev/production.
      demoMode: process.env.NUXT_PUBLIC_DEMO_MODE === 'true',
    }
  },
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
    '@nuxtjs/i18n',
    'v-gsap-nuxt',
    '@dargmuesli/nuxt-cookie-control',
    '@vesp/nuxt-fontawesome',
  ],
  i18n: {
    // Locale files live in the repository as plain JSON: a translation
    // contribution is a pull request that adds or edits one file, so review,
    // merge conflicts and attribution all work the way contributors expect.
    // See i18n/locales/CONTRIBUTING.md for the guide aimed at people who
    // are not comfortable with tooling.
    defaultLocale: 'en',
    strategy: 'no_prefix',
    // The app is an SPA (ssr: false), so there is no server render to localise
    // and no request-time locale to resolve. Reading the browser's language
    // would surprise an operator who set Indonesian here and English on their
    // desktop, so the choice is explicit and persisted in localStorage.
    detectBrowserLanguage: false,
    locales: [
      { code: 'en', language: 'en', name: 'English', file: 'en.json' },
      { code: 'id', language: 'id', name: 'Bahasa Indonesia', file: 'id.json' },
    ],
    langDir: 'locales',
    lazy: true,
    // Missing key in a translation falls back to the default locale instead of
    // rendering the raw key, and warns in the console so the gap is visible.
    fallbackLocale: 'en',
    missingWarn: false,
    fallbackWarn: false,
    // Pluralisation and interpolation helpers are available to templates as
    // $t / $rt, with the full vue-i18n composer under `useI18n()`.
    bundle: { optimizeTranslationDirective: false },
  },
  cookieControl: {
    cookieOptions: { path: '/', sameSite: 'lax', secure: true },
    cookieExpiryOffsetMs: 1000 * 60 * 60 * 24 * 365,
  },
})