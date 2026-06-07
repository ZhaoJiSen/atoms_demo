<script setup lang="ts">
const props = withDefaults(defineProps<{
  redirect?: string
  compact?: boolean
}>(), {
  redirect: '/auth/callback',
  compact: false,
})

const { t } = useI18n()
const { session, loading, isAuthenticated, loadSession, login, logout } = useAuthApi()

const userLabel = computed(() => session.value?.user?.displayName || session.value?.user?.email || t('auth.signedIn'))

async function handleLogout() {
  await logout()
}

onMounted(async () => {
  if (!session.value) {
    await loadSession().catch(() => undefined)
  }
})
</script>

<template>
  <div class="flex items-center gap-2">
    <template v-if="loading && !session">
      <Loader2 class="w-4 h-4 text-zinc-600 animate-spin" />
    </template>

    <template v-else-if="isAuthenticated">
      <div class="hidden sm:flex items-center gap-2 rounded-md border border-zinc-800 bg-zinc-900/50 px-2.5 py-1.5">
        <ShieldCheck class="w-3.5 h-3.5 text-emerald-400" />
        <div v-if="!compact" class="leading-none">
          <p class="text-xs text-zinc-200">{{ userLabel }}</p>
        </div>
      </div>
      <UButton :loading="loading" size="xs" variant="ghost" color="gray" @click="handleLogout">
        <LogOut class="w-3.5 h-3.5 mr-1" />
        {{ t('auth.logout') }}
      </UButton>
    </template>

    <template v-else>
      <UButton size="xs" color="primary" :loading="loading" @click="login(props.redirect)">
        <LogIn class="w-3.5 h-3.5 mr-1" />
        {{ t('auth.login') }}
      </UButton>
    </template>
  </div>
</template>
