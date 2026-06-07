<script setup lang="ts">
import type { ServerConnection, CreateServerConnectionRequest, App } from '~/types/apps'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import {
  Plus, Loader2, Server, Terminal as TerminalIcon, Play, StopCircle,
  RefreshCw, X, Pencil, Trash2, AlertCircle
} from '@lucide/vue'

definePageMeta({
  layout: 'default',
  ssr: false,
})

const { t } = useI18n()
const { listServers, createServer, updateServer, deleteServer, terminalWebSocketUrl } = useServersApi()
const { listApps } = useAppsApi()

const loading = ref(true)
const servers = ref<ServerConnection[]>([])
const apps = ref<App[]>([])
const error = ref<string | null>(null)
const creating = ref(false)
const createError = ref<string | null>(null)
const showCreateDialog = ref(false)
const showDeleteConfirm = ref(false)
const deleteTargetId = ref<string | null>(null)
const editingServerId = ref<string | null>(null)
const editName = ref('')

// Context menu state
const contextMenuTarget = ref<ServerConnection | null>(null)
const contextMenuOpen = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })

// Terminal tabs
const terminalTabs = ref<Array<{ id: string; name: string; server: ServerConnection }>>([])
const activeTerminalTab = ref<string | null>(null)

// Terminal state
const terminals = ref<Map<string, { terminal: Terminal; fitAddon: FitAddon; socket: WebSocket | null; status: string }>>(new Map())

const selectedAppId = ref<string | undefined>(undefined)

const form = ref<CreateServerConnectionRequest>({
  name: '',
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: '',
  privateKey: '',
})

const isFormValid = computed(() => {
  return form.value.name.trim() &&
    form.value.host.trim() &&
    form.value.username.trim() &&
    (form.value.authType === 'password' ? form.value.password : form.value.privateKey)
})

const appOptions = computed(() => [
  { label: t('servers.noAppBinding'), value: '' },
  ...apps.value.map(app => ({ label: app.title, value: app.id }))
])

const virtualElement = computed(() => ({
  getBoundingClientRect: () => ({
    x: contextMenuPosition.value.x,
    y: contextMenuPosition.value.y,
    top: contextMenuPosition.value.y,
    left: contextMenuPosition.value.x,
    bottom: contextMenuPosition.value.y,
    right: contextMenuPosition.value.x,
    width: 0,
    height: 0,
  }),
}))

function openContextMenu(event: MouseEvent, server: ServerConnection) {
  event.preventDefault()
  contextMenuTarget.value = server
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  contextMenuOpen.value = true
}

function handleRename() {
  if (contextMenuTarget.value) {
    editingServerId.value = contextMenuTarget.value.id
    editName.value = contextMenuTarget.value.name
  }
  contextMenuOpen.value = false
}

function handleDeleteFromMenu() {
  if (contextMenuTarget.value) {
    deleteTargetId.value = contextMenuTarget.value.id
    showDeleteConfirm.value = true
  }
  contextMenuOpen.value = false
}

async function confirmRename(serverId: string) {
  if (!editName.value.trim()) return
  try {
    const updated = await updateServer(serverId, { name: editName.value.trim() })
    const index = servers.value.findIndex(s => s.id === updated.id)
    if (index > -1) {
      servers.value[index] = updated
    }
    // Update terminal tab name if exists
    const tabIndex = terminalTabs.value.findIndex(t => t.id === serverId)
    if (tabIndex > -1) {
      terminalTabs.value[tabIndex].name = updated.name
    }
    editingServerId.value = null
  } catch (e) {
    console.error('Failed to rename server:', e)
  }
}

function cancelRename() {
  editingServerId.value = null
  editName.value = ''
}

async function loadData() {
  loading.value = true
  error.value = null
  
  try {
    const [loadedServers, loadedApps] = await Promise.all([
      listServers().catch(() => []),
      listApps().catch(() => []),
    ])
    servers.value = loadedServers
    apps.value = loadedApps
  } catch (e: any) {
    console.error('Failed to load data:', e)
  }
  
  loading.value = false
}

async function handleCreate() {
  if (!isFormValid.value) {
    createError.value = t('servers.validation')
    return
  }

  creating.value = true
  createError.value = null
  try {
    const server = await createServer(form.value)
    servers.value.push(server)
    resetForm()
    showCreateDialog.value = false
  } catch (e: any) {
    createError.value = e?.message || t('servers.createFailed')
  } finally {
    creating.value = false
  }
}

function resetForm() {
  form.value = {
    name: '',
    host: '',
    port: 22,
    username: '',
    authType: 'password',
    password: '',
    privateKey: '',
  }
  selectedAppId.value = undefined
}

async function handleDelete() {
  if (!deleteTargetId.value) return
  try {
    await deleteServer(deleteTargetId.value)
    servers.value = servers.value.filter(s => s.id !== deleteTargetId.value)
    // Close terminal tab if exists
    closeTerminal(deleteTargetId.value)
  } catch (e) {
    console.error('Failed to delete server:', e)
  }
  showDeleteConfirm.value = false
  deleteTargetId.value = null
}

function getServerApps(server: ServerConnection) {
  return apps.value.slice(0, 3)
}

function updateServerStatus(serverId: string, status: string) {
  const index = servers.value.findIndex(s => s.id === serverId)
  if (index > -1) {
    servers.value[index] = { ...servers.value[index], status: status as any }
  }
}

// Terminal functions
function openTerminal(server: ServerConnection) {
  const existing = terminalTabs.value.find(tab => tab.id === server.id)
  if (existing) {
    activeTerminalTab.value = server.id
    return
  }

  terminalTabs.value.push({
    id: server.id,
    name: server.name,
    server,
  })
  activeTerminalTab.value = server.id

  nextTick(() => {
    initTerminal(server)
  })
}

function closeTerminal(serverId: string) {
  const terminalData = terminals.value.get(serverId)
  if (terminalData) {
    terminalData.socket?.close()
    terminalData.terminal.dispose()
    terminals.value.delete(serverId)
  }

  const index = terminalTabs.value.findIndex(tab => tab.id === serverId)
  if (index > -1) {
    terminalTabs.value.splice(index, 1)
    if (activeTerminalTab.value === serverId) {
      activeTerminalTab.value = terminalTabs.value[Math.min(index, terminalTabs.value.length - 1)]?.id || null
    }
  }
}

function initTerminal(server: ServerConnection) {
  const container = document.getElementById(`terminal-${server.id}`)
  if (!container) return

  const terminal = new Terminal({
    theme: {
      background: '#09090b',
      foreground: '#fafafa',
      cursor: '#8b5cf6',
      selectionBackground: 'rgba(139, 92, 246, 0.3)',
    },
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    fontSize: 13,
    lineHeight: 1.5,
    cursorBlink: true,
  })

  const fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(container)
  fitAddon.fit()

  terminal.writeln('\x1b[38;2;139;92;246m' + t('terminal.terminalReady') + '\x1b[0m')

  terminal.onData((data) => {
    const terminalData = terminals.value.get(server.id)
    if (terminalData?.socket?.readyState === WebSocket.OPEN) {
      terminalData.socket.send(JSON.stringify({ type: 'input', data }))
    }
  })

  terminals.value.set(server.id, {
    terminal,
    fitAddon,
    socket: null,
    status: 'disconnected',
  })

  window.addEventListener('resize', () => fitAddon.fit())
}

function connectTerminal(server: ServerConnection) {
  const terminalData = terminals.value.get(server.id)
  if (!terminalData) return

  terminalData.status = 'connecting'
  // Update sidebar server status
  updateServerStatus(server.id, 'connecting')
  
  const wsUrl = terminalWebSocketUrl(server.id)
  const socket = new WebSocket(wsUrl)

  socket.onopen = () => {
    terminalData.status = 'connected'
    // Update sidebar server status
    updateServerStatus(server.id, 'connected')
    socket.send(JSON.stringify({
      type: 'resize',
      cols: terminalData.terminal.cols,
      rows: terminalData.terminal.rows,
    }))
  }

  socket.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data)
      switch (msg.type) {
        case 'output':
          terminalData.terminal.write(msg.data)
          break
        case 'status':
          if (msg.status) {
            terminalData.status = msg.status
            // Update sidebar server status
            updateServerStatus(server.id, msg.status)
          }
          break
        case 'error':
          terminalData.terminal.writeln('\r\n\x1b[31m' + msg.error + '\x1b[0m')
          terminalData.status = 'failed'
          updateServerStatus(server.id, 'failed')
          break
      }
    } catch {
      terminalData.terminal.write(event.data)
    }
  }

  socket.onerror = () => {
    terminalData.status = 'failed'
    updateServerStatus(server.id, 'failed')
    terminalData.terminal.writeln('\r\n\x1b[31m' + t('terminal.failed') + '\x1b[0m')
  }

  socket.onclose = () => {
    if (terminalData.status === 'connected') {
      terminalData.terminal.writeln('\r\n\x1b[33m' + t('terminal.terminalClosed') + '\x1b[0m')
    }
    terminalData.status = 'disconnected'
    updateServerStatus(server.id, 'disconnected')
    terminalData.socket = null
  }

  terminalData.socket = socket
}

function disconnectTerminal(serverId: string) {
  const terminalData = terminals.value.get(serverId)
  if (terminalData?.socket) {
    terminalData.socket.send(JSON.stringify({ type: 'disconnect' }))
    terminalData.socket.close()
    terminalData.socket = null
    terminalData.status = 'disconnected'
  }
}

onMounted(loadData)

onUnmounted(() => {
  terminals.value.forEach((terminalData) => {
    terminalData.socket?.close()
    terminalData.terminal.dispose()
  })
  terminals.value.clear()
})
</script>

<template>
  <NuxtLayout>
    <template #sidebar>
      <div class="p-2">
        <UButton @click="showCreateDialog = true" color="primary" size="sm" block class="mb-2">
          <Plus class="w-4 h-4 mr-1.5" />
          {{ t('servers.create') }}
        </UButton>

        <div v-if="loading" class="flex items-center justify-center py-8">
          <Loader2 class="w-5 h-5 text-zinc-600 animate-spin" />
        </div>
        <div v-else-if="servers.length === 0" class="text-center py-8 px-4">
          <Server class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
          <p class="text-xs text-zinc-500">{{ t('servers.empty') }}</p>
        </div>
        <div v-else class="space-y-1">
          <div
            v-for="server in servers"
            :key="server.id"
            class="rounded-lg hover:bg-zinc-800/50 transition-colors"
            @contextmenu="openContextMenu($event, server)"
          >
            <!-- Rename Mode -->
            <div v-if="editingServerId === server.id" class="p-2">
              <UInput
                v-model="editName"
                size="sm"
                @keyup.enter="confirmRename(server.id)"
                @keyup.escape="cancelRename"
                autofocus
              />
              <div class="flex gap-1 mt-2">
                <UButton @click="confirmRename(server.id)" size="xs" color="primary" class="flex-1 justify-center">
                  {{ t('common.confirm') }}
                </UButton>
                <UButton @click="cancelRename" size="xs" variant="ghost" color="gray" class="flex-1 justify-center">
                  {{ t('common.cancel') }}
                </UButton>
              </div>
            </div>

            <!-- Normal Mode -->
            <button v-else @click="openTerminal(server)" class="w-full text-left px-3 py-2.5">
              <div class="flex items-center justify-between mb-1">
                <span class="text-sm font-medium text-zinc-200 truncate flex-1">{{ server.name }}</span>
                <ServerStatusBadge :status="server.status" />
              </div>
              <p class="text-xs text-zinc-500 font-mono truncate">{{ server.username }}@{{ server.host }}:{{ server.port }}</p>
            </button>
            <div class="px-3 pb-2">
              <div class="flex flex-wrap gap-1">
                <UBadge v-for="app in getServerApps(server)" :key="app.id" color="gray" variant="subtle" size="xs" class="cursor-pointer">
                  {{ app.title }}
                </UBadge>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template #header-actions>
      <!-- Terminal Tabs -->
      <div v-if="terminalTabs.length > 0" class="flex items-center gap-1">
        <div
          v-for="tab in terminalTabs"
          :key="tab.id"
          @click="activeTerminalTab = tab.id"
          :class="[
            'flex items-center gap-2 px-3 py-1.5 rounded-md text-xs transition-colors cursor-pointer',
            activeTerminalTab === tab.id
              ? 'bg-zinc-800 text-zinc-100'
              : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50'
          ]"
        >
          <TerminalIcon class="w-3 h-3" />
          <span class="max-w-[100px] truncate">{{ tab.name }}</span>
          <button @click.stop="closeTerminal(tab.id)" class="ml-1 text-zinc-600 hover:text-zinc-300">
            <X class="w-3 h-3" />
          </button>
        </div>
      </div>
    </template>

    <div class="flex-1 flex flex-col">
      <template v-if="terminalTabs.length === 0">
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <TerminalIcon class="w-16 h-16 text-zinc-700 mx-auto mb-4" />
            <h3 class="text-lg font-medium text-zinc-300 mb-2">{{ t('terminal.title') }}</h3>
            <p class="text-sm text-zinc-500">{{ t('terminal.selectServer') }}</p>
          </div>
        </div>
      </template>
      <template v-else>
        <div class="flex-1 p-4">
          <div
            v-for="tab in terminalTabs"
            :key="tab.id"
            v-show="activeTerminalTab === tab.id"
            class="h-full flex flex-col"
          >
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-sm font-medium text-zinc-200">{{ tab.server.name }}</span>
                <span class="text-xs text-zinc-500 font-mono">{{ tab.server.username }}@{{ tab.server.host }}:{{ tab.server.port }}</span>
                <ServerStatusBadge :status="terminals.get(tab.id)?.status || 'disconnected'" />
              </div>
              <div class="flex items-center gap-2">
                <UButton
                  v-if="terminals.get(tab.id)?.status === 'disconnected'"
                  @click="connectTerminal(tab.server)"
                  color="primary"
                  size="xs"
                >
                  <Play class="w-3 h-3 mr-1" />
                  {{ t('terminal.connect') }}
                </UButton>
                <UButton
                  v-else-if="terminals.get(tab.id)?.status === 'connecting'"
                  @click="disconnectTerminal(tab.id)"
                  color="gray"
                  size="xs"
                  loading
                >
                  {{ t('terminal.connecting') }}
                </UButton>
                <UButton
                  v-else-if="terminals.get(tab.id)?.status === 'connected'"
                  @click="disconnectTerminal(tab.id)"
                  color="red"
                  size="xs"
                  variant="outline"
                >
                  <StopCircle class="w-3 h-3 mr-1" />
                  {{ t('terminal.disconnect') }}
                </UButton>
                <UButton
                  v-else
                  @click="connectTerminal(tab.server)"
                  color="gray"
                  size="xs"
                >
                  <RefreshCw class="w-3 h-3 mr-1" />
                  {{ t('terminal.retry') }}
                </UButton>
              </div>
            </div>
            <div :id="`terminal-${tab.id}`" class="flex-1 rounded-lg border border-zinc-800 bg-zinc-950" />
          </div>
        </div>
      </template>
    </div>

    <!-- Context Menu -->
    <UContextMenu v-model="contextMenuOpen" :virtual-element="virtualElement">
      <div class="w-48 py-1">
        <button @click="handleRename" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100">
          <Pencil class="w-4 h-4" />
          {{ t('common.rename') }}
        </button>
        <button @click="handleDeleteFromMenu" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-zinc-800 hover:text-red-300">
          <Trash2 class="w-4 h-4" />
          {{ t('common.delete') }}
        </button>
      </div>
    </UContextMenu>

    <!-- Create Server Dialog -->
    <UModal v-model="showCreateDialog">
      <div class="p-6">
        <h3 class="text-lg font-semibold text-zinc-100 mb-4">{{ t('servers.formTitle') }}</h3>
        <div class="space-y-4">
          <UFormGroup :label="t('servers.name')" required>
            <UInput v-model="form.name" :placeholder="t('servers.name')" color="gray" />
          </UFormGroup>

          <UFormGroup :label="t('servers.bindApp')">
            <USelect v-model="selectedAppId" :options="appOptions" color="gray" />
          </UFormGroup>

          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2">
              <UFormGroup :label="t('servers.host')" required>
                <UInput v-model="form.host" placeholder="192.168.1.100" color="gray" />
              </UFormGroup>
            </div>
            <UFormGroup :label="t('servers.port')">
              <UInput v-model.number="form.port" type="number" placeholder="22" color="gray" />
            </UFormGroup>
          </div>

          <UFormGroup :label="t('servers.username')" required>
            <UInput v-model="form.username" placeholder="root" color="gray" />
          </UFormGroup>

          <UFormGroup :label="t('servers.authType')">
            <USelect
              v-model="form.authType"
              :options="[
                { label: t('servers.passwordAuth'), value: 'password' },
                { label: t('servers.privateKeyAuth'), value: 'private_key' }
              ]"
              color="gray"
            />
          </UFormGroup>

          <UFormGroup v-if="form.authType === 'password'" :label="t('servers.password')" required>
            <UInput v-model="form.password" type="password" color="gray" />
          </UFormGroup>

          <UFormGroup v-else :label="t('servers.privateKey')" required>
            <UTextarea v-model="form.privateKey" :rows="4" color="gray" />
          </UFormGroup>

          <p class="text-xs text-zinc-600">{{ t('servers.metadataOnly') }}</p>

          <p v-if="createError" class="text-sm text-red-400 flex items-center gap-1.5">
            <AlertCircle class="w-4 h-4" />
            {{ createError }}
          </p>

          <div class="flex gap-3">
            <UButton @click="showCreateDialog = false" variant="outline" class="flex-1 justify-center">
              {{ t('common.cancel') }}
            </UButton>
            <UButton @click="handleCreate" color="primary" :loading="creating" :disabled="!isFormValid" class="flex-1 justify-center">
              <Plus class="w-4 h-4 mr-1.5" />
              {{ creating ? t('servers.creating') : t('servers.create') }}
            </UButton>
          </div>
        </div>
      </div>
    </UModal>

    <!-- Delete Confirm Dialog -->
    <UModal v-model="showDeleteConfirm">
      <div class="p-6">
        <div class="text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mx-auto mb-4">
            <Trash2 class="w-6 h-6 text-red-400" />
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">{{ t('servers.deleteConfirm') }}</h3>
          <p class="text-sm text-zinc-400">{{ t('servers.deleteConfirmDesc') }}</p>
        </div>
        <div class="flex gap-3">
          <UButton @click="showDeleteConfirm = false; deleteTargetId = null" variant="outline" class="flex-1 justify-center">
            {{ t('common.cancel') }}
          </UButton>
          <UButton @click="handleDelete" color="red" class="flex-1 justify-center">
            <Trash2 class="w-4 h-4 mr-1.5" />
            {{ t('common.delete') }}
          </UButton>
        </div>
      </div>
    </UModal>
  </NuxtLayout>
</template>
