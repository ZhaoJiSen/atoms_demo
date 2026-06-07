<script setup lang="ts">
import type { AgentStep } from '~/types/apps'

defineProps<{
  steps: AgentStep[]
}>()

const { t } = useI18n()

const stepIcons: Record<string, string> = {
  analyze_idea: 'Search',
  plan_product: 'ClipboardList',
  design_pages: 'Layout',
  define_api: 'Code',
  generate_files: 'FileCode',
  build_preview: 'Eye',
}
</script>

<template>
  <div class="space-y-2">
    <div
      v-for="step in steps"
      :key="step.id"
      class="flex items-start gap-2.5"
    >
      <div class="mt-0.5">
        <Loader2 v-if="step.status === 'running'" class="w-4 h-4 text-violet-400 animate-spin" />
        <CheckCircle v-else-if="step.status === 'done'" class="w-4 h-4 text-emerald-400" />
        <Circle v-else-if="step.status === 'waiting'" class="w-4 h-4 text-zinc-600" />
        <XCircle v-else class="w-4 h-4 text-red-400" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-xs font-medium truncate">{{ step.name }}</p>
        <p class="text-xs text-zinc-600 truncate">{{ step.description }}</p>
      </div>
    </div>
    <div v-if="steps.length === 0" class="text-xs text-zinc-600 text-center py-2">
      {{ t('common.notGenerated') }}
    </div>
  </div>
</template>
