<script setup lang="ts">
import {
  Sparkles, Lightbulb, AlertCircle, LogIn
} from '@lucide/vue'

definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const router = useRouter()
const { createApp } = useAppsApi()
const { isAuthenticated, login } = useAuthApi()

const refreshApps = inject<() => Promise<void>>('refreshApps')

const idea = ref('')
const creating = ref(false)
const error = ref<string | null>(null)
const showLoginDialog = ref(false)

const charCount = computed(() => idea.value.length)
const isValid = computed(() => idea.value.trim().length >= 5)

const exampleIdeas = computed(() => [
  t('landing.examples.idea1'),
  t('landing.examples.idea2'),
  t('landing.examples.idea3'),
])

function handleTextareaClick() {
  if (!isAuthenticated.value) {
    showLoginDialog.value = true
  }
}

async function handleCreate() {
  if (!isValid.value) {
    error.value = t('newApp.ideaTooShort')
    return
  }
  if (!isAuthenticated.value) {
    showLoginDialog.value = true
    return
  }
  creating.value = true
  error.value = null
  try {
    const app = await createApp({ idea: idea.value.trim() })
    if (refreshApps) {
      await refreshApps()
    }
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
</script>

<template>
  <div class="flex-1 flex items-center justify-center p-8">
    <div class="w-full max-w-2xl">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-semibold tracking-tight mb-3 text-zinc-100">
          {{ t('landing.title') }}
        </h1>
        <p class="text-sm text-zinc-500">
          {{ t('landing.subtitle') }}
        </p>
      </div>

      <div class="relative mb-4">
        <UTextarea
          v-model="idea"
          :placeholder="t('newApp.placeholder')"
          :rows="10"
          :maxlength="1000"
          size="lg"
          color="gray"
          class="w-full"
          @keydown.meta.enter="handleCreate"
          @keydown.ctrl.enter="handleCreate"
          @focus="handleTextareaClick"
        />
        <div class="absolute bottom-3 right-4 flex items-center gap-3">
          <span class="text-xs text-zinc-600">{{ charCount }}/1000</span>
        </div>
      </div>

      <p v-if="error" class="mb-4 text-sm text-red-400 flex items-center gap-1.5">
        <AlertCircle class="w-4 h-4" />
        {{ error }}
      </p>

      <div class="flex items-center gap-3 mb-8">
        <UButton
          @click="handleCreate"
          color="primary"
          size="lg"
          :loading="creating"
          :disabled="!isValid"
          class="flex-1 justify-center"
        >
          <Sparkles class="w-4 h-4 mr-2" />
          {{ creating ? t('newApp.creating') : t('newApp.create') }}
        </UButton>
      </div>

      <div class="space-y-2">
        <button
          v-for="example in exampleIdeas"
          :key="example"
          @click="selectExample(example)"
          class="w-full flex items-center gap-3 px-4 py-3 rounded-lg border border-zinc-800 hover:border-zinc-700 hover:bg-zinc-900 text-left transition-colors"
        >
          <Lightbulb class="w-4 h-4 text-zinc-500 flex-shrink-0" />
          <span class="text-sm text-zinc-400">{{ example }}</span>
        </button>
      </div>
    </div>

    <!-- Login Dialog -->
    <UModal v-model="showLoginDialog">
      <div class="p-6">
        <div class="text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-violet-500/10 flex items-center justify-center mx-auto mb-4">
            <LogIn class="w-6 h-6 text-violet-400" />
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">{{ t('auth.loginRequired') }}</h3>
          <p class="text-sm text-zinc-400">{{ t('auth.loginToCreate') }}</p>
        </div>
        <div class="space-y-3">
          <UButton @click="login('/')" color="primary" size="lg" block>
            <LogIn class="w-4 h-4 mr-2" />
            {{ t('auth.googleLogin') }}
          </UButton>
          <UButton @click="showLoginDialog = false" variant="ghost" size="lg" block color="gray">
            {{ t('common.cancel') }}
          </UButton>
        </div>
      </div>
    </UModal>
  </div>
</template>
