<script setup lang="ts">
import type { App } from '~/types/apps'

definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const route = useRoute()
const { getApp } = useAppsApi()

const appId = computed(() => route.params.id as string || '')
const loading = ref(true)
const app = ref<App | null>(null)
const error = ref<string | null>(null)

const hasPreview = computed(() => app.value?.status === 'completed' && app.value.result?.preview)

async function loadApp() {
  loading.value = true
  error.value = null
  try {
    app.value = await getApp(appId.value)
  } catch (e: any) {
    error.value = e?.message || t('previewPage.loadFailed')
  } finally {
    loading.value = false
  }
}

onMounted(loadApp)
</script>

<template>
  <div class="min-h-screen flex flex-col bg-zinc-950">
    <header class="border-b border-zinc-800 bg-zinc-950/80 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <UButton :to="`/app/${appId}/workspace`" variant="ghost" size="sm" color="gray">
            <ArrowLeft class="w-4 h-4 mr-1.5" />
            {{ t('nav.backWorkspace') }}
          </UButton>
          <div v-if="app" class="flex items-center gap-2">
            <span class="text-sm font-medium truncate max-w-[200px]">{{ app.title }}</span>
            <AppStatusBadge :status="app.status" />
          </div>
        </div>
      </div>
    </header>

    <main class="flex-1">
      <template v-if="loading">
        <div class="flex flex-col items-center justify-center py-24">
          <Loader2 class="w-8 h-8 text-zinc-600 animate-spin mb-4" />
          <p class="text-sm text-zinc-500">{{ t('previewPage.loading') }}</p>
        </div>
      </template>

      <template v-else-if="error || !app">
        <div class="flex flex-col items-center justify-center py-24">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
            <AlertTriangle class="w-6 h-6 text-red-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('previewPage.unavailable') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ error || t('previewPage.missingDesc') }}</p>
          <UButton to="/app/new" variant="outline">
            <Plus class="w-4 h-4 mr-1.5" />
            {{ t('nav.createNewApp') }}
          </UButton>
        </div>
      </template>

      <template v-else-if="app.status === 'pending'">
        <div class="flex flex-col items-center justify-center py-24">
          <div class="w-12 h-12 rounded-full bg-zinc-500/10 flex items-center justify-center mb-4">
            <Clock class="w-6 h-6 text-zinc-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('previewPage.notGenerated') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ t('previewPage.notGeneratedDesc') }}</p>
          <UButton :to="`/app/${appId}/workspace`" color="primary">
            <Wrench class="w-4 h-4 mr-1.5" />
            {{ t('nav.backWorkspace') }}
          </UButton>
        </div>
      </template>

      <template v-else-if="app.status === 'generating'">
        <div class="flex flex-col items-center justify-center py-24">
          <Loader2 class="w-8 h-8 text-violet-400 animate-spin mb-4" />
          <h2 class="text-lg font-medium mb-2">{{ t('previewPage.generating') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ t('previewPage.generatingDesc') }}</p>
          <UButton :to="`/app/${appId}/workspace`" variant="outline">
            <ArrowLeft class="w-4 h-4 mr-1.5" />
            {{ t('nav.backWorkspace') }}
          </UButton>
        </div>
      </template>

      <template v-else-if="app.status === 'failed'">
        <div class="flex flex-col items-center justify-center py-24">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
            <XCircle class="w-6 h-6 text-red-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('workspace.failed') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ t('previewPage.failedDesc') }}</p>
          <UButton :to="`/app/${appId}/workspace`" color="primary">
            <RefreshCw class="w-4 h-4 mr-1.5" />
            {{ t('previewPage.retryWorkspace') }}
          </UButton>
        </div>
      </template>

      <template v-else-if="!hasPreview">
        <div class="flex flex-col items-center justify-center py-24">
          <div class="w-12 h-12 rounded-full bg-zinc-500/10 flex items-center justify-center mb-4">
            <EyeOff class="w-6 h-6 text-zinc-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('previewPage.missing') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ t('previewPage.missingResultDesc') }}</p>
          <UButton :to="`/app/${appId}/workspace`" variant="outline">
            <ArrowLeft class="w-4 h-4 mr-1.5" />
            {{ t('nav.backWorkspace') }}
          </UButton>
        </div>
      </template>

      <template v-else>
        <PreviewMockApp :preview="app.result!.preview" :app-title="app.title" />
      </template>
    </main>
  </div>
</template>
