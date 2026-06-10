<script setup lang="ts">
import type { App, AgentStepStatus, GeneratedResult, FileNode } from '~/types/apps'
import {
  Eye, Folder, LayoutGrid, Zap, Database, FileText,
  RefreshCw, Loader2, AlertTriangle, Plus,
  MessageSquare, Send, Bot, User, Circle, CheckCircle
} from '@lucide/vue'

definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { getApp, createAppMessage } = useAppsApi()

const appId = computed(() => route.params.id as string || '')
const loading = ref(true)
const generating = ref(false)
const app = ref<App | null>(null)
const error = ref<string | null>(null)
const activeTab = ref('preview')
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
  } catch (e: any) {
    error.value = e?.message || t('workspace.loadFailed')
    return
  } finally {
    loading.value = false
  }
  // Keep the sidebar in sync when entering a workspace.
  if (refreshApps) refreshApps()
  // Auto-(re)generate when pending, or when stuck in `generating` from a
  // previous run that was interrupted (refresh / navigate away). Start AFTER
  // clearing `loading` (do not await) so the streaming UI is visible.
  if (app.value?.status === 'pending' || app.value?.status === 'generating') {
    handleGenerate()
  }
}

let streamFinished = false
let activeEs: EventSource | null = null
// Parallel streaming: many files write at once. Track each file's live text and
// which files are still being written; the editor shows the user's pick.
const streamingTexts = ref<Record<string, string>>({})
const streamingActive = ref<string[]>([])
const firstStreamingPath = ref<string | null>(null)

function buildStreamUrl() {
  const config = useRuntimeConfig()
  const base = String(config.public.apiBase || '')
  return `${base}/api/apps/${encodeURIComponent(appId.value)}/generate/stream`
}

function applyStep(payload: { id: string; status: AgentStepStatus; startedAt?: string; completedAt?: string }) {
  if (!app.value) return
  const steps = app.value.steps.map(step =>
    step.id === payload.id
      ? {
          ...step,
          status: payload.status,
          startedAt: payload.startedAt ?? step.startedAt,
          completedAt: payload.completedAt ?? step.completedAt,
        }
      : step,
  )
  app.value = { ...app.value, steps }
}

function applyPartial(payload: Partial<GeneratedResult>) {
  if (!app.value || !payload.productSpec || !payload.preview) return
  // Seed the result so spec/pages/api/model tabs populate immediately; files stream in next.
  app.value = {
    ...app.value,
    result: {
      productSpec: payload.productSpec,
      pages: payload.pages || [],
      apis: payload.apis || [],
      dataModels: payload.dataModels || [],
      preview: payload.preview,
      fileStructure: [],
    },
  }
}

function applyManifest(nodes: FileNode[]) {
  if (!app.value?.result) return
  // File structure is here — show the tree and the code being written.
  activeTab.value = 'files'
  app.value = { ...app.value, result: { ...app.value.result, fileStructure: nodes } }
}

function applyFileStart(path: string) {
  streamingTexts.value[path] = ''
  if (!streamingActive.value.includes(path)) {
    streamingActive.value = [...streamingActive.value, path]
  }
  // Default the editor to the first file that starts streaming.
  if (!firstStreamingPath.value) firstStreamingPath.value = path
}

function applyFileChunk(path: string, delta: string) {
  streamingTexts.value[path] = (streamingTexts.value[path] || '') + delta
}

function applyFileEnd(node: FileNode) {
  streamingTexts.value[node.path] = node.content ?? streamingTexts.value[node.path] ?? ''
  streamingActive.value = streamingActive.value.filter(p => p !== node.path)
  if (!app.value?.result) return
  // Completed file: fill its content so the live preview re-runs with it.
  const fileStructure = app.value.result.fileStructure.map(f =>
    f.path === node.path ? { ...f, content: node.content } : f,
  )
  if (!fileStructure.some(f => f.path === node.path)) {
    fileStructure.push(node)
  }
  app.value = { ...app.value, result: { ...app.value.result, fileStructure } }
}

// Consume the staged generation over Server-Sent Events.
function streamGenerate(): Promise<void> {
  return new Promise((resolve) => {
    streamFinished = false
    const es = new EventSource(buildStreamUrl(), { withCredentials: true })
    activeEs = es

    es.addEventListener('step', e => applyStep(JSON.parse((e as MessageEvent).data)))
    es.addEventListener('partial', e => applyPartial(JSON.parse((e as MessageEvent).data)))
    es.addEventListener('manifest', e => applyManifest(JSON.parse((e as MessageEvent).data).fileStructure || []))
    es.addEventListener('file_start', e => applyFileStart(JSON.parse((e as MessageEvent).data).path))
    es.addEventListener('file_chunk', (e) => {
      const d = JSON.parse((e as MessageEvent).data)
      applyFileChunk(d.path, d.delta || '')
    })
    es.addEventListener('file_end', e => applyFileEnd(JSON.parse((e as MessageEvent).data)))
    es.addEventListener('done', (e) => {
      streamFinished = true
      app.value = JSON.parse((e as MessageEvent).data)
      streamingActive.value = []
      firstStreamingPath.value = null
      streamingTexts.value = {}
      activeTab.value = 'preview'
      es.close()
      resolve()
    })
    es.addEventListener('fail', (e) => {
      streamFinished = true
      streamingActive.value = []
      firstStreamingPath.value = null
      const payload = JSON.parse((e as MessageEvent).data || '{}')
      error.value = payload.error || t('workspace.generateFailed')
      es.close()
      resolve()
    })
    // Native EventSource error: only meaningful before we finished (avoid auto-reconnect).
    es.onerror = () => {
      if (streamFinished) {
        es.close()
        return
      }
      streamFinished = true
      streamingActive.value = []
      firstStreamingPath.value = null
      es.close()
      error.value = error.value || t('workspace.generateFailed')
      resolve()
    }
  })
}

async function handleGenerate() {
  if (!app.value || generating.value) return
  generating.value = true
  error.value = null
  streamingTexts.value = {}
  streamingActive.value = []
  firstStreamingPath.value = null
  // Watch the code get written, then jump to the live preview when done.
  activeTab.value = 'files'

  // Reset to a clean generating state; streamed events refill steps + result.
  app.value = {
    ...app.value,
    status: 'generating',
    result: undefined,
    error: undefined,
    steps: app.value.steps.map(step => ({
      ...step,
      status: 'waiting' as AgentStepStatus,
      startedAt: undefined,
      completedAt: undefined,
    })),
  }
  // Reflect the in-progress app in the sidebar immediately.
  if (refreshApps) refreshApps()

  try {
    await streamGenerate()
  } finally {
    generating.value = false
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

// Warn before refreshing / closing the tab while generating.
function beforeUnloadHandler(e: BeforeUnloadEvent) {
  if (generating.value) {
    e.preventDefault()
    e.returnValue = ''
  }
}

// Confirm before navigating away within the app while generating.
onBeforeRouteLeave(() => {
  if (generating.value) {
    return window.confirm(t('workspace.leaveConfirm'))
  }
  return true
})

onMounted(() => {
  window.addEventListener('beforeunload', beforeUnloadHandler)
  loadApp()
})
onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', beforeUnloadHandler)
  streamFinished = true
  activeEs?.close()
})
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col">
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
      <div class="border-b border-zinc-800 flex-shrink-0">
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

      <!-- Inline generation progress (non-blocking; replaces the modal) -->
      <div
        v-if="generating"
        class="border-b border-zinc-800 bg-zinc-900/40 px-7 py-2 flex-shrink-0 flex items-center gap-4 overflow-x-auto"
      >
        <span class="text-xs text-violet-400 font-medium flex items-center gap-1.5 flex-shrink-0">
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          {{ t('workspace.generating') }}
        </span>
        <div class="flex items-center gap-3 flex-shrink-0">
          <div v-for="step in app.steps" :key="step.id" class="flex items-center gap-1.5">
            <CheckCircle v-if="step.status === 'done'" class="w-3.5 h-3.5 text-emerald-500" />
            <div v-else-if="step.status === 'running'" class="w-3 h-3 rounded-full border-2 border-violet-500 border-t-transparent animate-spin" />
            <Circle v-else class="w-3 h-3 text-zinc-600" />
            <span :class="[
              'text-xs whitespace-nowrap',
              step.status === 'running' ? 'text-violet-400' : step.status === 'done' ? 'text-zinc-400' : 'text-zinc-600',
            ]">{{ t(`workspace.steps.${step.id}`) }}</span>
          </div>
        </div>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 min-h-0 overflow-hidden p-6 flex flex-col">
        <template v-if="activeTab === 'chat'">
          <div class="flex-1 min-h-0 flex flex-col max-w-3xl mx-auto w-full">
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

        <div v-else class="flex-1 min-h-0 flex flex-col">
          <GeneratedResultTabs
            :result="app.result"
            :active-tab="activeTab"
            :streaming-texts="streamingTexts"
            :streaming-active="streamingActive"
            :default-streaming-path="firstStreamingPath"
            :generating="generating"
          />
        </div>
      </div>
    </template>
  </div>
</template>
