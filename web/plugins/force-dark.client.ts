// Dark-only app: force the color mode to dark at runtime. The nuxt.config
// `colorMode.preference` is only the *default* — a previously stored
// `nuxt-color-mode` value (system/light) would override it, which is why some
// Nuxt UI components rendered light. Setting the preference here overwrites any
// stale stored value so `.dark` is always applied to <html>.
export default defineNuxtPlugin(() => {
  const colorMode = useColorMode()
  colorMode.preference = 'dark'
})
