<script setup lang="ts">
definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { loadSession } = useAuthApi()

const status = computed(() => String(route.query.status || 'success'))
const redirectTarget = computed(() => {
  const value = String(route.query.redirect || '/')
  return value.startsWith('/') && !value.startsWith('//') ? value : '/'
})

onMounted(async () => {
  try {
    await loadSession()
  } catch {
    // ignore
  }
  
  // 直接跳转
  router.push(redirectTarget.value)
})
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-zinc-950">
    <div class="flex flex-col items-center">
      <Loader2 class="w-8 h-8 text-zinc-600 animate-spin mb-4" />
      <p class="text-sm text-zinc-500">{{ t('auth.callbackLoading') }}</p>
    </div>
  </div>
</template>
