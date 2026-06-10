<script setup lang="ts">
definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const { createApp } = useAppsApi()
const { isAuthenticated, loadSession } = useAuthApi()

const idea = ref((route.query.idea as string) || '')
const creating = ref(false)
const error = ref<string | null>(null)

const charCount = computed(() => idea.value.length)
const isValid = computed(() => idea.value.trim().length >= 5)

const exampleIdeas = computed(() => [
  t('landing.examples.idea1'),
  t('landing.examples.idea2'),
  t('landing.examples.idea3'),
])

async function handleCreate() {
  if (!isValid.value) {
    error.value = t('newApp.ideaTooShort')
    return
  }
  if (!isAuthenticated.value) {
    error.value = t('auth.requiredCreateApp')
    return
  }
  creating.value = true
  error.value = null
  try {
    const app = await createApp({ idea: idea.value.trim() })
    router.push(`/app/${app.id}`)
  } catch (e: any) {
    error.value = e?.message || t('newApp.failed')
  } finally {
    creating.value = false
  }
}

function selectExample(text: string) {
  idea.value = text
}

onMounted(async () => {
  await loadSession().catch(() => undefined)
})
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
      <div class="mb-10">
        <UBadge color="primary" variant="subtle" size="sm" class="mb-4">{{ t('newApp.badge') }}</UBadge>
        <h1 class="text-2xl font-semibold mb-2">{{ t('newApp.title') }}</h1>
        <p class="text-zinc-400">{{ t('newApp.subtitle') }}</p>
      </div>

      <div class="mb-8">
        <div class="relative">
          <textarea
            v-model="idea"
            :placeholder="t('newApp.placeholder')"
            rows="6"
            maxlength="1000"
            class="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-4 py-3 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500/50 resize-none transition-all"
            @keydown.meta.enter="handleCreate"
            @keydown.ctrl.enter="handleCreate"
          />
          <div class="absolute bottom-3 right-3 text-xs text-zinc-600">
            {{ charCount }} / 1000
          </div>
        </div>
        <p v-if="error" class="mt-2 text-sm text-red-400 flex items-center gap-1.5">
          <AlertCircle class="w-4 h-4" />
          {{ error }}
        </p>
      </div>

      <div class="flex items-center gap-3 mb-12">
        <UButton
          @click="handleCreate"
          color="primary"
          :loading="creating"
          :disabled="!isValid"
        >
          <Sparkles class="w-4 h-4 mr-1.5" />
          {{ creating ? t('newApp.creating') : t('newApp.create') }}
        </UButton>
      </div>

      <div>
        <h2 class="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-4">{{ t('newApp.examples') }}</h2>
        <div class="grid gap-2">
          <button
            v-for="example in exampleIdeas"
            :key="example"
            @click="selectExample(example)"
            class="group flex items-center gap-3 p-3 rounded-lg border border-zinc-800 hover:border-zinc-700 hover:bg-zinc-900/50 transition-colors text-left"
          >
            <Lightbulb class="w-4 h-4 text-zinc-600 group-hover:text-violet-400 transition-colors flex-shrink-0" />
            <span class="text-sm text-zinc-400 group-hover:text-zinc-200 transition-colors">{{ example }}</span>
          </button>
        </div>
      </div>
    </main>
  </div>
</template>
