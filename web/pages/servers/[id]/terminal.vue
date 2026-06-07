<script setup lang="ts">
import type { ServerConnection } from '~/types/apps'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

definePageMeta({
  ssr: false,
})

const { t } = useI18n()
const route = useRoute()
const { getServer, terminalWebSocketUrl } = useServersApi()
const { isAuthenticated, loadSession } = useAuthApi()

const serverId = computed(() => route.params.id as string || '')
const loading = ref(true)
const server = ref<ServerConnection | null>(null)
const error = ref<string | null>(null)
const connectionStatus = ref<'disconnected' | 'connecting' | 'connected' | 'failed'>('disconnected')
const terminalReady = ref(false)

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let socket: WebSocket | null = null

const terminalRef = ref<HTMLElement | null>(null)

async function loadServer() {
  loading.value = true
  error.value = null
  try {
    const [, loadedServer] = await Promise.all([loadSession().catch(() => undefined), getServer(serverId.value)])
    server.value = loadedServer
  } catch (e: any) {
    error.value = e?.message || t('terminal.loadFailed')
  } finally {
    loading.value = false
  }
}

function initTerminal() {
  if (!terminalRef.value || terminal) return

  terminal = new Terminal({
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

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(terminalRef.value)
  fitAddon.fit()

  terminal.writeln('\x1b[38;2;139;92;246m' + t('terminal.terminalReady') + '\x1b[0m')
  terminalReady.value = true

  terminal.onData((data) => {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'input', data }))
    }
  })

  window.addEventListener('resize', handleResize)
}

function handleResize() {
  if (fitAddon && terminal) {
    fitAddon.fit()
    if (socket?.readyState === WebSocket.OPEN && server.value) {
      socket.send(JSON.stringify({
        type: 'resize',
        cols: terminal.cols,
        rows: terminal.rows,
      }))
    }
  }
}

function connect() {
  if (!server.value || !terminal) return
  if (!isAuthenticated.value) {
    connectionStatus.value = 'failed'
    terminal.writeln('\r\n\x1b[31m' + t('auth.requiredTerminal') + '\x1b[0m')
    return
  }

  connectionStatus.value = 'connecting'
  const wsUrl = terminalWebSocketUrl(server.value.id)

  socket = new WebSocket(wsUrl)

  socket.onopen = () => {
    connectionStatus.value = 'connected'
    if (terminal && server.value) {
      socket?.send(JSON.stringify({
        type: 'resize',
        cols: terminal.cols,
        rows: terminal.rows,
      }))
    }
  }

  socket.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data)
      switch (msg.type) {
        case 'output':
          terminal?.write(msg.data)
          break
        case 'status':
          if (msg.status) connectionStatus.value = msg.status
          break
        case 'error':
          terminal?.writeln('\r\n\x1b[31m' + msg.error + '\x1b[0m')
          connectionStatus.value = 'failed'
          break
      }
    } catch {
      terminal?.write(event.data)
    }
  }

  socket.onerror = () => {
    connectionStatus.value = 'failed'
    terminal?.writeln('\r\n\x1b[31m' + t('terminal.failed') + '\x1b[0m')
  }

  socket.onclose = () => {
    if (connectionStatus.value === 'connected') {
      terminal?.writeln('\r\n\x1b[33m' + t('terminal.terminalClosed') + '\x1b[0m')
    }
    connectionStatus.value = 'disconnected'
    socket = null
  }
}

function disconnect() {
  if (socket) {
    socket.send(JSON.stringify({ type: 'disconnect' }))
    socket.close()
    socket = null
  }
  connectionStatus.value = 'disconnected'
}

onMounted(async () => {
  await loadServer()
  nextTick(() => {
    initTerminal()
  })
})

onUnmounted(() => {
  disconnect()
  window.removeEventListener('resize', handleResize)
  terminal?.dispose()
  terminal = null
  fitAddon = null
})
</script>

<template>
  <div class="min-h-screen flex flex-col bg-zinc-950">
    <header class="border-b border-zinc-800 bg-zinc-950/80 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <UButton to="/servers" variant="ghost" size="sm" color="gray">
            <ArrowLeft class="w-4 h-4 mr-1.5" />
            {{ t('terminal.returnServers') }}
          </UButton>
          <div v-if="server" class="flex items-center gap-2">
            <Server class="w-4 h-4 text-zinc-500" />
            <span class="text-sm font-medium">{{ server.name }}</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <ServerStatusBadge :status="connectionStatus" />
          <UButton
            v-if="connectionStatus === 'disconnected'"
            @click="connect"
            color="primary"
            size="sm"
          >
            <Play class="w-4 h-4 mr-1.5" />
            {{ t('terminal.connect') }}
          </UButton>
          <UButton
            v-else-if="connectionStatus === 'connecting'"
            @click="disconnect"
            color="gray"
            size="sm"
            loading
          >
            {{ t('terminal.connecting') }}
          </UButton>
          <UButton
            v-else-if="connectionStatus === 'connected'"
            @click="disconnect"
            color="red"
            size="sm"
            variant="outline"
          >
            <StopCircle class="w-4 h-4 mr-1.5" />
            {{ t('terminal.disconnect') }}
          </UButton>
          <UButton
            v-else
            @click="connect"
            color="gray"
            size="sm"
          >
            <RefreshCw class="w-4 h-4 mr-1.5" />
            {{ t('terminal.retry') }}
          </UButton>
        </div>
      </div>
    </header>

    <main class="flex-1 flex">
      <template v-if="loading">
        <div class="flex-1 flex flex-col items-center justify-center">
          <Loader2 class="w-8 h-8 text-zinc-600 animate-spin mb-4" />
          <p class="text-sm text-zinc-500">{{ t('common.loading') }}</p>
        </div>
      </template>

      <template v-else-if="error || !server">
        <div class="flex-1 flex flex-col items-center justify-center">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
            <AlertTriangle class="w-6 h-6 text-red-400" />
          </div>
          <h2 class="text-lg font-medium mb-2">{{ t('terminal.notFound') }}</h2>
          <p class="text-sm text-zinc-500 mb-6">{{ error }}</p>
          <UButton to="/servers" variant="outline">
            <ArrowLeft class="w-4 h-4 mr-1.5" />
            {{ t('terminal.returnServers') }}
          </UButton>
        </div>
      </template>

      <template v-else>
        <aside class="w-64 border-r border-zinc-800 p-4">
          <h3 class="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-4 flex items-center gap-2">
            <Info class="w-3.5 h-3.5" />
            {{ t('terminal.summary') }}
          </h3>
          <div class="space-y-3">
            <div>
              <span class="text-xs text-zinc-600">{{ t('servers.name') }}</span>
              <p class="text-sm">{{ server.name }}</p>
            </div>
            <div>
              <span class="text-xs text-zinc-600">{{ t('servers.host') }}</span>
              <p class="text-sm font-mono">{{ server.host }}</p>
            </div>
            <div>
              <span class="text-xs text-zinc-600">{{ t('servers.port') }}</span>
              <p class="text-sm font-mono">{{ server.port }}</p>
            </div>
            <div>
              <span class="text-xs text-zinc-600">{{ t('servers.username') }}</span>
              <p class="text-sm font-mono">{{ server.username }}</p>
            </div>
            <div>
              <span class="text-xs text-zinc-600">{{ t('servers.authType') }}</span>
              <p class="text-sm">{{ server.authType === 'password' ? t('servers.passwordAuth') : t('servers.privateKeyAuth') }}</p>
            </div>
          </div>
        </aside>

        <div class="flex-1 flex flex-col">
          <div ref="terminalRef" class="flex-1 p-4" />
        </div>
      </template>
    </main>
  </div>
</template>
