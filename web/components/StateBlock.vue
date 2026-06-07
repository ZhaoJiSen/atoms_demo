<script setup lang="ts">
const props = defineProps<{
  title?: string
  description?: string
  tone?: 'neutral' | 'success' | 'error' | 'info'
}>()

const toneConfig = computed(() => {
  const configs = {
    neutral: { border: 'border-zinc-800', bg: 'bg-zinc-900/50', iconBg: 'bg-zinc-800', iconColor: 'text-zinc-400', icon: 'Info' },
    success: { border: 'border-emerald-800/50', bg: 'bg-emerald-900/10', iconBg: 'bg-emerald-500/10', iconColor: 'text-emerald-400', icon: 'CheckCircle' },
    error: { border: 'border-red-800/50', bg: 'bg-red-900/10', iconBg: 'bg-red-500/10', iconColor: 'text-red-400', icon: 'AlertCircle' },
    info: { border: 'border-violet-800/50', bg: 'bg-violet-900/10', iconBg: 'bg-violet-500/10', iconColor: 'text-violet-400', icon: 'Info' },
  }
  return configs[props.tone || 'neutral']
})
</script>

<template>
  <div :class="[toneConfig.border, toneConfig.bg, 'rounded-lg border p-4']">
    <div class="flex items-start gap-3">
      <div :class="[toneConfig.iconBg, 'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0']">
        <component :is="toneConfig.icon" :class="toneConfig.iconColor" class="w-4 h-4" />
      </div>
      <div class="flex-1 min-w-0">
        <h3 v-if="title" class="text-sm font-medium mb-1">{{ title }}</h3>
        <p v-if="description" class="text-sm text-zinc-400">{{ description }}</p>
        <div v-if="$slots.default" class="mt-3">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>
