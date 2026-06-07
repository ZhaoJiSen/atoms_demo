<script setup lang="ts">
definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const router = useRouter()
const { getHealth, getInit, completeInit } = useInitApi()

const loading = ref(true)
const completing = ref(false)
const apiHealthy = ref(false)
const initStatus = ref<'not_initialized' | 'ready'>('not_initialized')
const error = ref<string | null>(null)

const isReady = computed(() => initStatus.value === 'ready')

async function loadStatus() {
  loading.value = true
  error.value = null
  try {
    const [health, init] = await Promise.all([getHealth(), getInit()])
    apiHealthy.value = health.status === 'ok'
    initStatus.value = init.status
  } catch (e: any) {
    error.value = e?.message || t('init.errorTitle')
    apiHealthy.value = false
  } finally {
    loading.value = false
  }
}

async function handleComplete() {
  completing.value = true
  try {
    await completeInit()
    initStatus.value = 'ready'
  } catch (e: any) {
    error.value = e?.message || t('init.errorTitle')
  } finally {
    completing.value = false
  }
}

onMounted(loadStatus)
</script>

<template>
  <div class="min-h-screen">
    <header class="border-b border-zinc-800">
      <div class="max-w-3xl mx-auto px-6 h-14 flex items-center">
        <UButton to="/" variant="ghost" size="sm" color="gray">
          <ArrowLeft class="w-4 h-4 mr-1.5" />
          {{ t('nav.backHome') }}
        </UButton>
      </div>
    </header>

    <main class="max-w-3xl mx-auto px-6 py-16">
      <template v-if="loading">
        <div class="flex flex-col items-center justify-center py-24">
          <Loader2 class="w-8 h-8 text-zinc-600 animate-spin mb-4" />
          <p class="text-sm text-zinc-500">{{ t('init.loading') }}</p>
          <p class="text-xs text-zinc-600 mt-1">{{ t('init.loadingDesc') }}</p>
        </div>
      </template>

      <template v-else-if="error && !apiHealthy">
        <div class="flex flex-col items-center justify-center py-24">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
            <AlertTriangle class="w-6 h-6 text-red-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('init.errorTitle') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ error }}</p>
          <UButton @click="loadStatus" variant="outline">
            <RefreshCw class="w-4 h-4 mr-1.5" />
            {{ t('common.retry') }}
          </UButton>
        </div>
      </template>

      <template v-else>
        <div class="mb-10">
          <UBadge color="primary" variant="subtle" size="sm" class="mb-4">{{ t('init.badge') }}</UBadge>
          <h1 class="text-2xl font-semibold mb-2">{{ t('init.title') }}</h1>
          <p class="text-zinc-400">{{ t('init.subtitle') }}</p>
        </div>

        <div class="grid gap-4 mb-10">
          <div class="flex items-center justify-between p-4 rounded-lg border border-zinc-800 bg-zinc-900/50">
            <div class="flex items-center gap-3">
              <div :class="['w-2 h-2 rounded-full', apiHealthy ? 'bg-emerald-500' : 'bg-red-500']" />
              <span class="text-sm">{{ t('init.apiHealth') }}</span>
            </div>
            <UBadge :color="(apiHealthy ? 'green' : 'red') as any" variant="subtle" size="sm">
              {{ apiHealthy ? t('common.healthy') : t('common.unavailable') }}
            </UBadge>
          </div>

          <div class="flex items-center justify-between p-4 rounded-lg border border-zinc-800 bg-zinc-900/50">
            <div class="flex items-center gap-3">
              <div :class="['w-2 h-2 rounded-full', isReady ? 'bg-emerald-500' : 'bg-zinc-500']" />
              <span class="text-sm">{{ t('init.statusLabel') }}</span>
            </div>
            <UBadge :color="(isReady ? 'green' : 'gray') as any" variant="subtle" size="sm">
              {{ isReady ? t('init.status.ready') : t('init.status.not_initialized') }}
            </UBadge>
          </div>

          <div class="flex items-center justify-between p-4 rounded-lg border border-zinc-800 bg-zinc-900/50">
            <div class="flex items-center gap-3">
              <User class="w-4 h-4 text-zinc-500" />
              <span class="text-sm">{{ t('init.noRegistration') }}</span>
            </div>
            <UBadge color="primary" variant="subtle" size="sm">{{ t('common.yes') }}</UBadge>
          </div>

        </div>

        <div class="mb-10">
          <h2 class="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-4">{{ t('init.stepsTitle') }}</h2>
          <div class="grid gap-2">
            <div
              v-for="(step, index) in ['one', 'two', 'three', 'four']"
              :key="step"
              class="flex items-center gap-3 p-3 rounded-lg border border-zinc-800"
            >
              <span class="w-6 h-6 rounded-full bg-zinc-800 flex items-center justify-center text-xs text-zinc-500">
                {{ index + 1 }}
              </span>
              <span class="text-sm text-zinc-300">{{ t(`init.steps.${step}`) }}</span>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <template v-if="!isReady">
            <UButton @click="handleComplete" color="primary" :loading="completing">
              <Check class="w-4 h-4 mr-1.5" />
              {{ t('init.complete') }}
            </UButton>
          </template>
          <template v-else>
            <UButton to="/app/new" color="primary">
              <ArrowRight class="w-4 h-4 mr-1.5" />
              {{ t('init.continueBuilder') }}
            </UButton>
          </template>
          <UButton to="/servers" variant="outline">
            <Server class="w-4 h-4 mr-1.5" />
            {{ t('init.continueServers') }}
          </UButton>
        </div>
      </template>
    </main>
  </div>
</template>
