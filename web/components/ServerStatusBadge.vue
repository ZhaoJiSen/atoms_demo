<script setup lang="ts">
defineProps<{
  status: string
}>()

const { t } = useI18n()

function getStatusColor(status: string) {
  const configs: Record<string, string> = {
    disconnected: 'gray',
    connecting: 'primary',
    connected: 'green',
    failed: 'red',
  }
  return configs[status] || 'gray'
}

function getStatusLabel(status: string) {
  const labels: Record<string, string> = {
    disconnected: 'Disconnected',
    connecting: 'Connecting',
    connected: 'Connected',
    failed: 'Failed',
  }
  return labels[status] || status
}
</script>

<template>
  <UBadge :color="(getStatusColor(status) as any)" variant="subtle" size="xs">
    {{ getStatusLabel(status) }}
  </UBadge>
</template>
