<script setup lang="ts">
definePageMeta({
  layout: 'default',
  ssr: false,
})

const { t } = useI18n()
const { request } = useApiRequest()

const loading = ref(true)
const saving = ref(false)
const configured = ref(false)

const form = ref({
  provider: 'gpt' as 'gpt' | 'mimo',
  apiKey: '',
  model: '',
  baseUrl: '',
})

const providerOptions = [
  { label: 'OpenAI GPT', value: 'gpt' },
  { label: 'MIMO', value: 'mimo' },
]

const defaultModels: Record<string, string> = {
  gpt: 'gpt-4',
  mimo: 'mimo-v2.5-pro',
}

const defaultBaseUrls: Record<string, string> = {
  gpt: 'https://api.openai.com',
  mimo: 'https://token-plan-cn.xiaomimimo.com/v1',
}

async function loadSettings() {
  loading.value = true
  try {
    const data = await request<any>('/api/settings/ai')
    configured.value = data.configured
    if (data.configured) {
      form.value.provider = data.provider
      form.value.model = data.model
      form.value.baseUrl = data.baseUrl
    }
  } catch {
    // Settings not configured yet
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saving.value = true
  try {
    await request('/api/settings/ai', {
      method: 'PUT',
      body: {
        provider: form.value.provider,
        apiKey: form.value.apiKey,
        model: form.value.model || undefined,
        baseUrl: form.value.baseUrl || undefined,
      },
    })
    configured.value = true
  } catch (e: any) {
    console.error('Failed to save settings:', e)
  } finally {
    saving.value = false
  }
}

function onProviderChange() {
  form.value.model = defaultModels[form.value.provider] || ''
  form.value.baseUrl = defaultBaseUrls[form.value.provider] || ''
}

onMounted(loadSettings)
</script>

<template>
  <NuxtLayout>
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-2xl mx-auto px-6 py-12">
        <div class="mb-8">
          <h1 class="text-2xl font-semibold text-zinc-100 mb-2">{{ t('settings.title') }}</h1>
          <p class="text-sm text-zinc-400">{{ t('settings.subtitle') }}</p>
        </div>

        <template v-if="loading">
          <div class="flex items-center justify-center py-12">
            <Loader2 class="w-6 h-6 text-zinc-600 animate-spin" />
          </div>
        </template>

        <template v-else>
          <!-- Status -->
          <div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4 mb-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div :class="['w-2 h-2 rounded-full', configured ? 'bg-emerald-500' : 'bg-zinc-500']" />
                <span class="text-sm">{{ t('settings.status') }}</span>
              </div>
              <UBadge :color="(configured ? 'green' : 'gray') as any" variant="subtle" size="xs">
                {{ configured ? t('settings.configured') : t('settings.notConfigured') }}
              </UBadge>
            </div>
          </div>

          <!-- Form -->
          <div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-6 space-y-6">
            <UFormGroup :label="t('settings.provider')" required>
              <USelect
                v-model="form.provider"
                :options="providerOptions"
                color="gray"
                @update:model-value="onProviderChange"
              />
            </UFormGroup>

            <UFormGroup :label="t('settings.apiKey')" required>
              <UInput
                v-model="form.apiKey"
                type="password"
                :placeholder="t('settings.apiKeyPlaceholder')"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.apiKeyHelp') }}</p>
              </template>
            </UFormGroup>

            <UFormGroup :label="t('settings.model')">
              <UInput
                v-model="form.model"
                :placeholder="defaultModels[form.provider]"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.modelHelp') }}</p>
              </template>
            </UFormGroup>

            <UFormGroup :label="t('settings.baseUrl')">
              <UInput
                v-model="form.baseUrl"
                :placeholder="defaultBaseUrls[form.provider]"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.baseUrlHelp') }}</p>
              </template>
            </UFormGroup>

            <div class="pt-4 border-t border-zinc-800">
              <UButton @click="handleSave" color="primary" :loading="saving">
                <Save class="w-4 h-4 mr-1.5" />
                {{ t('settings.save') }}
              </UButton>
            </div>
          </div>

          <!-- Info -->
          <div class="mt-6 rounded-lg border border-zinc-800 bg-zinc-900/50 p-6">
            <h3 class="text-sm font-medium text-zinc-200 mb-3">{{ t('settings.howItWorks') }}</h3>
            <div class="space-y-3 text-sm text-zinc-400">
              <p>{{ t('settings.info1') }}</p>
              <p>{{ t('settings.info2') }}</p>
              <p>{{ t('settings.info3') }}</p>
            </div>
          </div>
        </template>
      </div>
    </div>
  </NuxtLayout>
</template>
