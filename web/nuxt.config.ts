export default defineNuxtConfig({
  compatibilityDate: '2026-06-05',
  modules: ['@nuxt/ui', '@nuxtjs/i18n', '@nuxtjs/tailwindcss'],
  css: ['~/assets/css/main.css', '@xterm/xterm/css/xterm.css'],
  devtools: { enabled: true },
  ssr: false,
  experimental: {
    appManifest: false,
  },
  typescript: {
    strict: true,
    typeCheck: true,
  },
  runtimeConfig: {
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || '',
    },
  },
  nitro: {
    devProxy: {
      '/api': {
        target: 'http://localhost:3001/api',
        changeOrigin: true,
      },
    },
  },
  i18n: {
    strategy: 'no_prefix',
    defaultLocale: 'zh',
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: 'atoms_demo_locale',
      redirectOn: 'root',
    },
    locales: [
      { code: 'zh', name: '中文', file: 'zh.json' },
      { code: 'en', name: 'English', file: 'en.json' },
    ],
    langDir: 'locales',
  },
  app: {
    head: {
      title: 'Atoms Demo',
      meta: [
        {
          name: 'description',
          content: 'A compact AI app builder demo with agent workspace and generated preview.',
        },
      ],
    },
  },
})