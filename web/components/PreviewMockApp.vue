<script setup lang="ts">
import type { PreviewData, PreviewItem } from '~/types/apps'

const props = defineProps<{
  preview: PreviewData
  appTitle: string
}>()

const { t } = useI18n()
const activeFilter = ref('All')
const actionCount = ref(0)
const draftTitle = ref('')
const localItems = ref<PreviewItem[]>([])

const sectionsWithItems = computed(() => props.preview.sections.filter(section => section.items?.length))
const primaryAction = computed(() => props.preview.actions.find(action => action.type === 'primary') || props.preview.actions[0])

const listItems = computed(() => {
  const source = sectionsWithItems.value.flatMap(section => section.items || []).concat(localItems.value)
  if (activeFilter.value === 'Active') {
    return source.filter(item => item.value !== 'Done')
  }
  if (activeFilter.value === 'Done') {
    return source.filter(item => item.value === 'Done')
  }
  return source
})

function runPrimaryAction() {
  actionCount.value += 1
  const label = draftTitle.value.trim() || `Item ${actionCount.value}`
  localItems.value.unshift({
    title: label,
    description: 'Created via preview',
    value: actionCount.value % 2 === 0 ? 'Done' : 'New',
  })
  draftTitle.value = ''
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg font-semibold text-zinc-100">{{ preview.title }}</h3>
        <p class="text-sm text-zinc-400 mt-1">{{ preview.description }}</p>
      </div>
      <div class="flex gap-2">
        <UButton
          v-for="action in preview.actions"
          :key="action.label"
          :color="action.type === 'primary' ? 'primary' : 'gray'"
          :variant="action.type === 'primary' ? 'solid' : 'outline'"
          size="sm"
          @click="action.type === 'primary' ? runPrimaryAction() : null"
        >
          {{ action.label }}
        </UButton>
      </div>
    </div>

    <div class="grid gap-4 sm:grid-cols-2">
      <div
        v-for="section in preview.sections"
        :key="section.id"
        class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4"
      >
        <div class="mb-3">
          <span class="text-xs font-medium uppercase text-zinc-500">{{ section.type }}</span>
          <h4 class="text-sm font-medium text-zinc-200 mt-1">{{ section.title }}</h4>
          <p class="text-xs text-zinc-400 mt-1">{{ section.content }}</p>
        </div>

        <div v-if="section.type === 'stats' && section.items?.length" class="grid gap-2">
          <div
            v-for="item in section.items"
            :key="item.title"
            class="flex items-center justify-between p-2 rounded bg-zinc-800/50"
          >
            <span class="text-xs text-zinc-400">{{ item.title }}</span>
            <span class="text-sm font-medium text-zinc-200">{{ item.value || '-' }}</span>
          </div>
        </div>

        <div v-else-if="section.type === 'form'" class="flex gap-2">
          <UInput v-model="draftTitle" :placeholder="t('newApp.placeholder')" size="sm" class="flex-1" color="gray" />
          <UButton @click="runPrimaryAction" color="primary" size="sm">
            {{ primaryAction?.label || 'Add' }}
          </UButton>
        </div>

        <div v-else-if="section.items?.length" class="space-y-2">
          <div class="flex gap-1 mb-2">
            <UButton
              v-for="filter in ['All', 'Active', 'Done']"
              :key="filter"
              size="2xs"
              :color="activeFilter === filter ? 'primary' : 'gray'"
              :variant="activeFilter === filter ? 'solid' : 'ghost'"
              @click="activeFilter = filter"
            >
              {{ filter }}
            </UButton>
          </div>
          <div
            v-for="item in section.items"
            :key="item.title"
            class="flex items-center justify-between p-2 rounded bg-zinc-800/50"
          >
            <div>
              <p class="text-xs font-medium text-zinc-200">{{ item.title }}</p>
              <p v-if="item.description" class="text-xs text-zinc-500 mt-0.5">{{ item.description }}</p>
            </div>
            <UBadge :color="item.value === 'Done' ? 'green' : 'gray'" variant="subtle" size="xs">
              {{ item.value || 'Open' }}
            </UBadge>
          </div>
        </div>
      </div>
    </div>

    <div v-if="listItems.length > 0" class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
      <h4 class="text-sm font-medium text-zinc-200 mb-3">{{ t('workspace.tabs.preview') }} Items</h4>
      <div class="space-y-2">
        <div
          v-for="item in listItems"
          :key="`${item.title}-${item.value}`"
          class="flex items-center justify-between p-2 rounded bg-zinc-800/50"
        >
          <div>
            <p class="text-xs font-medium text-zinc-200">{{ item.title }}</p>
            <p v-if="item.description" class="text-xs text-zinc-500 mt-0.5">{{ item.description }}</p>
          </div>
          <UBadge :color="item.value === 'Done' ? 'green' : 'gray'" variant="subtle" size="xs">
            {{ item.value || 'Open' }}
          </UBadge>
        </div>
      </div>
    </div>
  </div>
</template>
