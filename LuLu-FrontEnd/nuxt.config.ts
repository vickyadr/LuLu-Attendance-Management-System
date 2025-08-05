// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
    ssr: false,
  hooks: {
    'prerender:routes'({ routes }) {
      routes.clear() // Do not generate any routes (except the defaults)
    }
  },
  runtimeConfig: {
    public: {
      apiBase: 'http://127.0.0.1:8080/api',
    }
  },
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
    'v-gsap-nuxt',
    '@dargmuesli/nuxt-cookie-control',
    '@vesp/nuxt-fontawesome',
  ]
})