<script setup lang="ts">
defineProps<{
  status: string
}>()

const { t } = useI18n()

function getStatusColor(status: string) {
  const configs: Record<string, string> = {
    not_initialized: 'gray',
    ready: 'green',
  }
  return configs[status] || 'gray'
}
</script>

<template>
  <UBadge :color="(getStatusColor(status) as any)" variant="subtle" size="xs">
    {{ t(`init.status.${status}`) }}
  </UBadge>
</template>
