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
        { name: 'description', content: 'LuLu Attendance System — kelola karyawan, shift, jadwal, perangkat, dan laporan absensi real-time.' },
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
      apiBase: 'http://10.55.54.222:4343/api',
    }
  },
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
    'v-gsap-nuxt',
    '@dargmuesli/nuxt-cookie-control',
    '@vesp/nuxt-fontawesome',
  ],
  cookieControl: {
    cookieOptions: { path: '/', sameSite: 'lax', secure: true },
    cookieExpiryOffsetMs: 1000 * 60 * 60 * 24 * 365,
  },
})