<script setup lang="ts">
defineProps<{
  status: string
}>()

const { t } = useI18n()

function getStatusColor(status: string) {
  const configs: Record<string, string> = {
    pending: 'gray',
    generating: 'primary',
    completed: 'green',
    failed: 'red',
  }
  return configs[status] || 'gray'
}

function getStatusLabel(status: string) {
  const labels: Record<string, string> = {
    pending: 'Pending',
    generating: 'Generating',
    completed: 'Completed',
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
