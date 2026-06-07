<script setup lang="ts">
import type { AgentMessage } from '~/types/apps'

defineProps<{
  messages: AgentMessage[]
}>()

const roleIcons: Record<string, string> = {
  user: 'User',
  agent: 'Sparkles',
  system: 'Settings',
  error: 'AlertTriangle',
}

const roleColors: Record<string, string> = {
  user: 'bg-zinc-800 text-zinc-400',
  agent: 'bg-violet-500/10 text-violet-400',
  system: 'bg-zinc-800 text-zinc-500',
  error: 'bg-red-500/10 text-red-400',
}

function formatTime(dateStr: string) {
  const date = new Date(dateStr)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div class="space-y-3 max-h-[40vh] overflow-y-auto">
    <div
      v-for="msg in messages"
      :key="msg.id"
      class="flex gap-3"
    >
      <div :class="[roleColors[msg.role] || roleColors.system, 'w-7 h-7 rounded-full flex items-center justify-center flex-shrink-0']">
        <component :is="roleIcons[msg.role] || roleIcons.system" class="w-3.5 h-3.5" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-xs font-medium">{{ msg.agentName || msg.role }}</span>
          <span class="text-xs text-zinc-600">{{ formatTime(msg.createdAt) }}</span>
        </div>
        <p class="text-sm text-zinc-300 leading-relaxed">{{ msg.content }}</p>
      </div>
    </div>
    <div v-if="messages.length === 0" class="text-xs text-zinc-600 text-center py-4">
      {{ $t('common.notGenerated') }}
    </div>
  </div>
</template>
