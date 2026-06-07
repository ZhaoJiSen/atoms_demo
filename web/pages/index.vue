<script setup lang="ts">
import type { App } from '~/types/apps'
import {
  Plus, Loader2, MessageSquare, Pencil, Trash2,
  User, LogOut, LogIn, Server, FileText, Blocks
} from '@lucide/vue'

definePageMeta({
  layout: 'default',
  ssr: false,
})

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const { listApps, updateApp, deleteApp } = useAppsApi()

const apps = ref<App[]>([])
const appsLoading = ref(false)
const editingApp = ref<App | null>(null)
const editTitle = ref('')
const showDeleteConfirm = ref(false)
const deleteTargetId = ref<string | null>(null)

// Context menu state
const contextMenuTarget = ref<App | null>(null)
const contextMenuOpen = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })

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

function openContextMenu(event: MouseEvent, app: App) {
  event.preventDefault()
  contextMenuTarget.value = app
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  contextMenuOpen.value = true
}

function handleRename() {
  if (contextMenuTarget.value) {
    startRename(contextMenuTarget.value)
  }
  contextMenuOpen.value = false
}

function handleDelete() {
  if (contextMenuTarget.value) {
    showDeleteConfirm.value = true
    deleteTargetId.value = contextMenuTarget.value.id
  }
  contextMenuOpen.value = false
}

async function loadApps() {
  appsLoading.value = true
  try {
    apps.value = await listApps()
  } catch {
    apps.value = []
  } finally {
    appsLoading.value = false
  }
}

function goToApp(app: App) {
  router.push(`/app/${app.id}`)
}

// Provide refresh function to child pages
provide('refreshApps', loadApps)

function startRename(app: App) {
  editingApp.value = app
  editTitle.value = app.title
  contextMenuOpen.value = false
}

async function confirmRename() {
  if (!editingApp.value || !editTitle.value.trim()) return
  try {
    await updateApp(editingApp.value.id, { title: editTitle.value.trim() })
    editingApp.value.title = editTitle.value.trim()
    editingApp.value = null
  } catch (e) {
    console.error('Rename failed:', e)
  }
}

function cancelRename() {
  editingApp.value = null
  editTitle.value = ''
}

async function confirmDelete(id: string) {
  try {
    await deleteApp(id)
    apps.value = apps.value.filter(a => a.id !== id)
    if (route.params.id === id) {
      router.push('/')
    }
  } catch (e) {
    console.error('Delete failed:', e)
  }
  showDeleteConfirm.value = false
  deleteTargetId.value = null
}

onMounted(loadApps)
</script>

<template>
  <NuxtLayout>
    <template #sidebar>
      <div class="p-2">
        <UButton to="/" color="primary" size="sm" block class="mb-2">
          <Plus class="w-4 h-4 mr-1.5" />
          {{ t('nav.createNewApp') }}
        </UButton>

        <div v-if="appsLoading" class="flex items-center justify-center py-8">
          <Loader2 class="w-5 h-5 text-zinc-600 animate-spin" />
        </div>
        <div v-else-if="apps.length === 0" class="text-center py-8 px-4">
          <MessageSquare class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
          <p class="text-xs text-zinc-500">{{ t('workspace.empty') }}</p>
        </div>
        <div v-else class="space-y-1">
          <div
            v-for="app in apps"
            :key="app.id"
            :class="[
              'group rounded-lg transition-colors',
              route.params.id === app.id ? 'bg-zinc-800' : 'hover:bg-zinc-800/50'
            ]"
            @contextmenu="openContextMenu($event, app)"
          >
            <div v-if="editingApp?.id === app.id" class="p-2">
              <UInput v-model="editTitle" size="sm" @keyup.enter="confirmRename" @keyup.escape="cancelRename" autofocus />
              <div class="flex gap-1 mt-2">
                <UButton @click="confirmRename" size="xs" color="primary" class="flex-1 justify-center">
                  {{ t('common.confirm') }}
                </UButton>
                <UButton @click="cancelRename" size="xs" variant="ghost" color="gray" class="flex-1 justify-center">
                  {{ t('common.cancel') }}
                </UButton>
              </div>
            </div>
            <div v-else class="px-3 py-2.5 cursor-pointer" @click="goToApp(app)">
              <div class="flex items-center justify-between mb-1">
                <span class="text-sm font-medium text-zinc-200 truncate flex-1">{{ app.title }}</span>
                <AppStatusBadge :status="app.status" />
              </div>
              <p class="text-xs text-zinc-500 truncate">{{ app.idea }}</p>
            </div>
          </div>
        </div>
      </div>
    </template>

    <NuxtPage />

    <UContextMenu v-model="contextMenuOpen" :virtual-element="virtualElement">
      <div class="w-48 py-1">
        <button @click="handleRename" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100">
          <Pencil class="w-4 h-4" />
          {{ t('common.rename') }}
        </button>
        <button @click="handleDelete" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-zinc-800 hover:text-red-300">
          <Trash2 class="w-4 h-4" />
          {{ t('common.delete') }}
        </button>
      </div>
    </UContextMenu>

    <UModal v-model="showDeleteConfirm">
      <div class="p-6">
        <div class="text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mx-auto mb-4">
            <Trash2 class="w-6 h-6 text-red-400" />
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">{{ t('app.deleteConfirm') }}</h3>
          <p class="text-sm text-zinc-400">{{ t('app.deleteConfirmDesc') }}</p>
        </div>
        <div class="flex gap-3">
          <UButton @click="showDeleteConfirm = false; deleteTargetId = null" variant="outline" class="flex-1 justify-center">
            {{ t('common.cancel') }}
          </UButton>
          <UButton @click="confirmDelete(deleteTargetId!)" color="red" class="flex-1 justify-center">
            <Trash2 class="w-4 h-4 mr-1.5" />
            {{ t('common.delete') }}
          </UButton>
        </div>
      </div>
    </UModal>
  </NuxtLayout>
</template>
