<script setup lang="ts">
import type { App, AgentStepStatus } from '~/types/apps'
import {
  Eye, Folder, LayoutGrid, Zap, Database, FileText,
  RefreshCw, Sparkles, Loader2, AlertTriangle, Plus,
  Check, Play, MessageSquare, Send, Bot, User, Circle, CheckCircle
} from '@lucide/vue'

definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { getApp, generateApp, createAppMessage } = useAppsApi()

const appId = computed(() => route.params.id as string || '')
const loading = ref(true)
const generating = ref(false)
const app = ref<App | null>(null)
const error = ref<string | null>(null)
const activeTab = ref('preview')
const currentStepIndex = ref(-1)
const showGeneratingModal = ref(false)
const followUpMessage = ref('')
const sendingFollowUp = ref(false)

// Get refreshApps from parent
const refreshApps = inject<() => Promise<void>>('refreshApps')

const tabItems = [
  { label: t('workspace.tabs.chat'), key: 'chat' },
  { label: t('workspace.tabs.preview'), key: 'preview' },
  { label: t('workspace.tabs.files'), key: 'files' },
  { label: t('workspace.tabs.pages'), key: 'pages' },
  { label: t('workspace.tabs.api'), key: 'api' },
  { label: t('workspace.tabs.models'), key: 'models' },
  { label: t('workspace.tabs.spec'), key: 'spec' },
]

const tabIcons: Record<string, any> = {
  preview: Eye,
  files: Folder,
  pages: LayoutGrid,
  api: Zap,
  models: Database,
  spec: FileText,
  chat: MessageSquare,
}

const canSendFollowUp = computed(() =>
  Boolean(followUpMessage.value.trim()) && !generating.value && !sendingFollowUp.value,
)

function getTabIcon(key: string) {
  return tabIcons[key] || FileText
}

async function loadApp() {
  loading.value = true
  error.value = null
  try {
    app.value = await getApp(appId.value)
    if (app.value.status === 'pending') {
      await handleGenerate()
    }
  } catch (e: any) {
    error.value = e?.message || t('workspace.loadFailed')
  } finally {
    loading.value = false
  }
}

async function handleGenerate() {
  if (!app.value || generating.value) return
  generating.value = true
  showGeneratingModal.value = true
  currentStepIndex.value = 0

  // Reset all steps status
  const steps = app.value.steps.map(step => ({
    ...step,
    status: 'waiting' as AgentStepStatus,
    startedAt: undefined,
    completedAt: undefined,
  }))
  app.value = { ...app.value, steps }

  // Animate first 6 steps (excluding finalize)
  const animateSteps = steps.length - 1
  for (let i = 0; i < animateSteps; i++) {
    currentStepIndex.value = i
    steps[i].status = 'running'
    app.value = { ...app.value, steps: [...steps] }

    await new Promise(resolve => setTimeout(resolve, 1200 + Math.random() * 800))

    steps[i].status = 'done'
    app.value = { ...app.value, steps: [...steps] }
  }

  // Start finalize step (loading state during API call)
  const finalizeIndex = steps.length - 1
  currentStepIndex.value = finalizeIndex
  steps[finalizeIndex].status = 'running'
  app.value = { ...app.value, steps: [...steps] }

  try {
    app.value = await generateApp(appId.value)
    // Mark finalize as done after API returns
    if (app.value) {
      app.value.steps[finalizeIndex].status = 'done'
      app.value = { ...app.value }
    }
    // Switch to preview tab after generation
    activeTab.value = 'preview'
  } catch (e: any) {
    // Mark finalize as error on failure
    if (app.value) {
      app.value.steps[finalizeIndex].status = 'error'
      app.value = { ...app.value }
    }
    try {
      const latestApp = await getApp(appId.value)
      app.value = latestApp
      error.value = latestApp.status === 'failed'
        ? null
        : e?.message || t('workspace.generateFailed')
    } catch {
      error.value = e?.message || t('workspace.generateFailed')
    }
  } finally {
    generating.value = false
    currentStepIndex.value = -1
    setTimeout(() => {
      showGeneratingModal.value = false
    }, 800)
    // Refresh sidebar apps list
    if (refreshApps) {
      await refreshApps()
    }
  }
}

async function handleFollowUp() {
  if (!app.value || !canSendFollowUp.value) return

  sendingFollowUp.value = true
  error.value = null

  try {
    const content = followUpMessage.value.trim()
    app.value = await createAppMessage(app.value.id, { content })
    followUpMessage.value = ''
    await handleGenerate()
  } catch (e: any) {
    error.value = e?.message || t('workspace.followUpFailed')
  } finally {
    sendingFollowUp.value = false
  }
}

onMounted(loadApp)
</script>

<template>
  <div class="flex-1 flex flex-col">
    <!-- Loading -->
    <template v-if="loading">
      <div class="flex-1 flex flex-col items-center justify-center">
        <Loader2 class="w-8 h-8 text-zinc-600 animate-spin mb-4" />
        <p class="text-sm text-zinc-500">{{ t('workspace.loading') }}</p>
      </div>
    </template>

    <!-- Error -->
    <template v-else-if="error || !app">
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
          <AlertTriangle class="w-6 h-6 text-red-400" />
        </div>
        <h2 class="text-lg font-medium mb-2">{{ t('workspace.unavailable') }}</h2>
        <p class="text-sm text-zinc-500 mb-6">{{ error || t('workspace.emptyDesc') }}</p>
        <UButton to="/" variant="outline">
          <Plus class="w-4 h-4 mr-1.5" />
          {{ t('nav.createNewApp') }}
        </UButton>
      </div>
    </template>

    <!-- Content -->
    <template v-else>
      <!-- Tabs -->
      <div class="border-b border-zinc-800">
        <div class="flex items-center gap-1 px-4 h-10">
          <button
            v-for="tab in tabItems"
            :key="tab.key"
            @click="activeTab = tab.key"
            :class="[
              'flex items-center gap-1.5 px-3 h-8 text-xs font-medium rounded-md transition-colors',
              activeTab === tab.key
                ? 'bg-zinc-800 text-zinc-100'
                : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50'
            ]"
          >
            <component :is="getTabIcon(tab.key)" class="w-3.5 h-3.5" />
            {{ tab.label }}
          </button>
          
          <div class="ml-auto">
            <UButton
              v-if="app.status === 'failed'"
              @click="handleGenerate"
              color="red"
              size="xs"
              variant="outline"
            >
              <RefreshCw class="w-3 h-3 mr-1" />
              {{ t('workspace.retry') }}
            </UButton>
          </div>
        </div>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <template v-if="activeTab === 'chat'">
          <div class="flex flex-col h-full max-w-3xl mx-auto">
            <!-- Messages -->
            <div class="flex-1 overflow-y-auto space-y-4 mb-4">
              <div
                v-for="msg in app.messages"
                :key="msg.id"
                class="flex gap-3"
              >
                <div :class="[
                  'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0',
                  msg.role === 'user' ? 'bg-zinc-800' :
                  msg.role === 'agent' ? 'bg-violet-500/10' :
                  msg.role === 'error' ? 'bg-red-500/10' : 'bg-zinc-800'
                ]">
                  <User v-if="msg.role === 'user'" class="w-4 h-4 text-zinc-400" />
                  <Bot v-else-if="msg.role === 'agent'" class="w-4 h-4 text-violet-400" />
                  <AlertTriangle v-else-if="msg.role === 'error'" class="w-4 h-4 text-red-400" />
                  <Circle v-else class="w-4 h-4 text-zinc-500" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-xs font-medium text-zinc-400">{{ msg.agentName || msg.role }}</span>
                  </div>
                  <p :class="[
                    'text-sm',
                    msg.role === 'error' ? 'text-red-400' : 'text-zinc-300'
                  ]">{{ msg.content }}</p>
                </div>
              </div>

              <!-- Generating indicator -->
              <div v-if="generating" class="flex gap-3">
                <div class="w-8 h-8 rounded-full bg-violet-500/10 flex items-center justify-center flex-shrink-0">
                  <Bot class="w-4 h-4 text-violet-400 animate-pulse" />
                </div>
                <div class="flex-1">
                  <p class="text-xs font-medium text-violet-400 mb-1">Agent</p>
                  <p class="text-sm text-zinc-400">{{ t('workspace.generatingDesc') }}</p>
                  <div class="mt-3 space-y-1.5">
                    <div
                      v-for="(step, index) in app.steps"
                      :key="step.id"
                      class="flex items-center gap-2"
                    >
                      <div v-if="step.status === 'running'" class="w-4 h-4 rounded-full border-2 border-violet-500 border-t-transparent animate-spin" />
                      <CheckCircle v-else-if="step.status === 'done'" class="w-4 h-4 text-emerald-500" />
                      <Circle v-else class="w-4 h-4 text-zinc-600" />
                      <span :class="[
                        'text-sm',
                        step.status === 'running' ? 'text-violet-400' :
                        step.status === 'done' ? 'text-zinc-400' : 'text-zinc-600'
                      ]">{{ t(`workspace.steps.${step.id}`) }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Empty state -->
              <div v-if="app.messages.length === 0 && !generating" class="text-center py-12">
                <MessageSquare class="w-12 h-12 text-zinc-700 mx-auto mb-3" />
                <p class="text-sm text-zinc-500">{{ t('workspace.chatEmpty') }}</p>
              </div>
            </div>

            <!-- Input -->
            <div class="border-t border-zinc-800 pt-4">
              <form @submit.prevent="handleFollowUp" class="flex gap-3">
                <UInput
                  v-model="followUpMessage"
                  :placeholder="t('workspace.followUpPlaceholder')"
                  color="gray"
                  class="flex-1"
                  size="lg"
                  :disabled="generating || sendingFollowUp"
                  @keydown.enter.prevent="handleFollowUp"
                />
                <UButton
                  type="submit"
                  color="primary"
                  size="lg"
                  :loading="sendingFollowUp || generating"
                  :disabled="!canSendFollowUp"
                >
                  <Send class="w-5 h-5" />
                </UButton>
              </form>
              <p class="text-xs text-zinc-600 mt-2">{{ t('workspace.followUpHint') }}</p>
            </div>
          </div>
        </template>

        <GeneratedResultTabs v-else :result="app.result" :active-tab="activeTab" />
      </div>
    </template>

    <!-- Generating Modal -->
    <UModal v-model="showGeneratingModal" :ui="{ width: 'max-w-md' }">
      <div class="p-6">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-full bg-violet-500/10 flex items-center justify-center">
            <Sparkles class="w-5 h-5 text-violet-400" />
          </div>
          <div>
            <h3 class="text-sm font-medium text-zinc-100">{{ t('workspace.generating') }}</h3>
            <p class="text-xs text-zinc-500">{{ app?.title }}</p>
          </div>
        </div>

        <div class="space-y-4">
          <div
            v-for="(step, index) in app?.steps || []"
            :key="step.id"
            class="flex items-start gap-3"
          >
            <div class="mt-0.5">
              <div v-if="step.status === 'running'" class="w-6 h-6 rounded-full border-2 border-violet-500 border-t-transparent animate-spin" />
              <div v-else-if="step.status === 'done'" class="w-6 h-6 rounded-full bg-emerald-500/10 flex items-center justify-center">
                <Check class="w-3.5 h-3.5 text-emerald-500" />
              </div>
              <div v-else class="w-6 h-6 rounded-full border border-zinc-700 flex items-center justify-center">
                <span class="text-xs text-zinc-600">{{ index + 1 }}</span>
              </div>
            </div>
            <div class="flex-1">
              <p :class="[
                'text-sm font-medium',
                step.status === 'running' ? 'text-violet-400' : 
                step.status === 'done' ? 'text-zinc-300' : 'text-zinc-600'
              ]">
                {{ t(`workspace.steps.${step.id}`) }}
              </p>
              <p class="text-xs text-zinc-600 mt-0.5">{{ t(`workspace.steps.${step.id}_desc`) }}</p>
            </div>
          </div>
        </div>

        <div class="mt-6 pt-4 border-t border-zinc-800">
          <div class="flex items-center gap-2">
            <div class="flex-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
              <div 
                class="h-full bg-violet-500 rounded-full transition-all duration-500"
                :style="{ width: `${((currentStepIndex + 1) / (app?.steps.length || 1)) * 100}%` }"
              />
            </div>
            <span class="text-xs text-zinc-500">{{ currentStepIndex + 1 }}/{{ app?.steps.length || 0 }}</span>
          </div>
        </div>
      </div>
    </UModal>
  </div>
</template>
