<script setup lang="ts">
import type { Note } from '~/types/apps'
import {
  Plus, Loader2, FileText, Save, Trash2, Pencil
} from '@lucide/vue'

definePageMeta({
  layout: 'default',
  ssr: false,
})

const { t } = useI18n()
const { listNotes, createNote, updateNote, deleteNote } = useNotesApi()

const loading = ref(true)
const notes = ref<Note[]>([])
const selectedNote = ref<Note | null>(null)
const editingContent = ref('')
const editingTitle = ref('')
const saving = ref(false)
const showDeleteConfirm = ref(false)
const deleteTargetId = ref<string | null>(null)
const editingNoteId = ref<string | null>(null)
const editTitle = ref('')

// Context menu state
const contextMenuTarget = ref<Note | null>(null)
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

function openContextMenu(event: MouseEvent, note: Note) {
  event.preventDefault()
  contextMenuTarget.value = note
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  contextMenuOpen.value = true
}

function handleRename() {
  if (contextMenuTarget.value) {
    editingNoteId.value = contextMenuTarget.value.id
    editTitle.value = contextMenuTarget.value.title
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

async function confirmRename(noteId: string) {
  if (!editTitle.value.trim()) return
  try {
    const updated = await updateNote(noteId, { title: editTitle.value.trim() })
    const index = notes.value.findIndex(n => n.id === updated.id)
    if (index > -1) {
      notes.value[index] = updated
    }
    if (selectedNote.value?.id === noteId) {
      selectedNote.value = updated
      editingTitle.value = updated.title
    }
    editingNoteId.value = null
  } catch (e) {
    console.error('Failed to rename note:', e)
  }
}

function cancelRename() {
  editingNoteId.value = null
  editTitle.value = ''
}

async function loadNotes() {
  loading.value = true
  try {
    notes.value = await listNotes()
    if (notes.value.length > 0 && !selectedNote.value) {
      selectNote(notes.value[0])
    }
  } catch {
    notes.value = []
  } finally {
    loading.value = false
  }
}

function selectNote(note: Note) {
  selectedNote.value = note
  editingTitle.value = note.title
  editingContent.value = note.content
}

async function handleCreate() {
  try {
    const note = await createNote({
      title: t('notes.newNote'),
      content: '',
    })
    notes.value.unshift(note)
    selectNote(note)
  } catch (e) {
    console.error('Failed to create note:', e)
  }
}

async function handleSave() {
  if (!selectedNote.value) return
  saving.value = true
  try {
    const updated = await updateNote(selectedNote.value.id, {
      title: editingTitle.value,
      content: editingContent.value,
    })
    const index = notes.value.findIndex(n => n.id === updated.id)
    if (index > -1) {
      notes.value[index] = updated
    }
    selectedNote.value = updated
  } catch (e) {
    console.error('Failed to save note:', e)
  } finally {
    saving.value = false
  }
}

async function handleDelete() {
  if (!deleteTargetId.value) return
  try {
    await deleteNote(deleteTargetId.value)
    notes.value = notes.value.filter(n => n.id !== deleteTargetId.value)
    if (selectedNote.value?.id === deleteTargetId.value) {
      selectedNote.value = notes.value[0] || null
      if (selectedNote.value) {
        selectNote(selectedNote.value)
      } else {
        editingTitle.value = ''
        editingContent.value = ''
      }
    }
  } catch (e) {
    console.error('Failed to delete note:', e)
  }
  showDeleteConfirm.value = false
  deleteTargetId.value = null
}

onMounted(loadNotes)
</script>

<template>
  <NuxtLayout>
    <template #sidebar>
      <div class="p-2">
        <UButton @click="handleCreate" color="primary" size="sm" block class="mb-2">
          <Plus class="w-4 h-4 mr-1.5" />
          {{ t('notes.create') }}
        </UButton>

        <div v-if="loading" class="flex items-center justify-center py-8">
          <Loader2 class="w-5 h-5 text-zinc-600 animate-spin" />
        </div>
        <div v-else-if="notes.length === 0" class="text-center py-8 px-4">
          <FileText class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
          <p class="text-xs text-zinc-500">{{ t('notes.empty') }}</p>
        </div>
        <div v-else class="space-y-1">
          <div
            v-for="note in notes"
            :key="note.id"
            :class="[
              'group rounded-lg transition-colors',
              selectedNote?.id === note.id ? 'bg-zinc-800' : 'hover:bg-zinc-800/50'
            ]"
            @contextmenu="openContextMenu($event, note)"
          >
            <!-- Rename Mode -->
            <div v-if="editingNoteId === note.id" class="p-2">
              <UInput
                v-model="editTitle"
                size="sm"
                @keyup.enter="confirmRename(note.id)"
                @keyup.escape="cancelRename"
                autofocus
              />
              <div class="flex gap-1 mt-2">
                <UButton @click="confirmRename(note.id)" size="xs" color="primary" class="flex-1 justify-center">
                  {{ t('common.confirm') }}
                </UButton>
                <UButton @click="cancelRename" size="xs" variant="ghost" color="gray" class="flex-1 justify-center">
                  {{ t('common.cancel') }}
                </UButton>
              </div>
            </div>

            <!-- Normal Mode -->
            <button
              v-else
              @click="selectNote(note)"
              class="w-full text-left px-3 py-2.5"
            >
              <p class="text-sm font-medium text-zinc-200 truncate">{{ note.title }}</p>
              <p class="text-xs text-zinc-500 truncate mt-1">{{ note.content || t('notes.noContent') }}</p>
            </button>
          </div>
        </div>
      </div>
    </template>

    <div class="flex-1 flex flex-col">
      <template v-if="selectedNote">
        <!-- Note Header -->
        <div class="border-b border-zinc-800 px-6 h-12 flex items-center justify-between">
          <UInput
            v-model="editingTitle"
            variant="none"
            class="flex-1 max-w-md"
            :placeholder="t('notes.titlePlaceholder')"
          />
          <div class="flex items-center gap-2">
            <UButton @click="handleSave" color="primary" size="xs" :loading="saving">
              <Save class="w-3 h-3 mr-1" />
              {{ t('notes.save') }}
            </UButton>
            <UButton @click="deleteTargetId = selectedNote?.id; showDeleteConfirm = true" variant="ghost" color="red" size="xs">
              <Trash2 class="w-3 h-3" />
            </UButton>
          </div>
        </div>

        <!-- Editor -->
        <div class="flex-1 p-6">
          <UTextarea
            v-model="editingContent"
            :placeholder="t('notes.contentPlaceholder')"
            :rows="20"
            color="gray"
            class="w-full h-full"
          />
        </div>
      </template>

      <template v-else>
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <FileText class="w-16 h-16 text-zinc-700 mx-auto mb-4" />
            <h3 class="text-lg font-medium text-zinc-300 mb-2">{{ t('notes.title') }}</h3>
            <p class="text-sm text-zinc-500 mb-6">{{ t('notes.selectOrCreate') }}</p>
            <UButton @click="handleCreate" color="primary">
              <Plus class="w-4 h-4 mr-1.5" />
              {{ t('notes.create') }}
            </UButton>
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

    <!-- Delete Confirm Dialog -->
    <UModal v-model="showDeleteConfirm">
      <div class="p-6">
        <div class="text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mx-auto mb-4">
            <Trash2 class="w-6 h-6 text-red-400" />
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">{{ t('notes.deleteConfirm') }}</h3>
          <p class="text-sm text-zinc-400">{{ t('notes.deleteConfirmDesc') }}</p>
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
