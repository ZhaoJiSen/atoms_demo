<script setup lang="ts">
import {
  Blocks, Plus, Loader2, MessageSquare, Pencil, Trash2,
  User, LogOut, LogIn, Server, FileText, Settings, Save,
  CheckCircle, AlertCircle, Zap
} from '@lucide/vue'

const { t, locale, setLocale, locales } = useI18n()
const { session, isAuthenticated, loadSession, login, logout, demoLogin } = useAuthApi()
const { request } = useApiRequest()
const router = useRouter()

const showLoginDialog = ref(false)
const showSettingsDialog = ref(false)
const saving = ref(false)
const testing = ref(false)
const configured = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)
const loggingIn = ref(false)

// Watch for logout to redirect to home
watch(isAuthenticated, (newVal) => {
  if (!newVal) {
    router.push('/')
  }
})

const settingsForm = ref({
  provider: 'mock' as 'mock' | 'gpt' | 'mimo',
  apiKey: '',
  model: '',
  baseUrl: '',
})

const providerOptions = [
  { label: 'Mock (免费，无需配置)', value: 'mock' },
  { label: 'OpenAI GPT', value: 'gpt' },
  { label: '小米 MIMO', value: 'mimo' },
]

const defaultModels: Record<string, string> = {
  mock: '',
  gpt: 'gpt-4',
  mimo: 'mimo-v2.5-pro',
}

const defaultBaseUrls: Record<string, string> = {
  mock: '',
  gpt: 'https://api.openai.com',
  mimo: 'https://token-plan-cn.xiaomimimo.com/v1',
}

const mimoModels = [
  'mimo-v2.5-pro',
  'mimo-v2.5',
  'mimo-v2.5-asr',
  'mimo-v2.5-tts',
  'mimo-v2-pro',
  'mimo-v2-omni',
  'mimo-v2-tts',
]

const localeOptions = computed(() => 
  locales.value.map(l => ({
    label: typeof l === 'string' ? l : l.name || l.code,
    value: typeof l === 'string' ? l : l.code,
  }))
)

const currentLocale = computed(() => locale.value)

async function loadSettings() {
  try {
    const data = await request<any>('/api/settings/ai')
    configured.value = data.configured
    if (data.configured) {
      settingsForm.value.provider = data.provider
      settingsForm.value.model = data.model
      settingsForm.value.baseUrl = data.baseUrl
    }
  } catch {
    // Settings not configured yet
  }
}

async function handleSaveSettings() {
  saving.value = true
  try {
    if (settingsForm.value.provider === 'mock') {
      // For mock, just mark as configured without sending to backend
      configured.value = true
      showSettingsDialog.value = false
      return
    }
    
    await request('/api/settings/ai', {
      method: 'PUT',
      body: {
        provider: settingsForm.value.provider,
        apiKey: settingsForm.value.apiKey,
        model: settingsForm.value.model || undefined,
        baseUrl: settingsForm.value.baseUrl || undefined,
      },
    })
    configured.value = true
    showSettingsDialog.value = false
  } catch (e: any) {
    console.error('Failed to save settings:', e)
  } finally {
    saving.value = false
  }
}

function onProviderChange() {
  settingsForm.value.model = defaultModels[settingsForm.value.provider] || ''
  settingsForm.value.baseUrl = defaultBaseUrls[settingsForm.value.provider] || ''
  testResult.value = null
}

async function handleTestApi() {
  if (settingsForm.value.provider === 'mock') {
    testResult.value = { success: true, message: t('settings.testSuccess') }
    return
  }
  
  testing.value = true
  testResult.value = null
  try {
    const baseUrl = (settingsForm.value.baseUrl || defaultBaseUrls[settingsForm.value.provider]).replace(/\/+$/, '')
    const model = settingsForm.value.model || defaultModels[settingsForm.value.provider]
    
    // Build correct URL
    const url = baseUrl.endsWith('/v1') 
      ? `${baseUrl}/chat/completions`
      : `${baseUrl}/v1/chat/completions`
    
    // Test by making a simple request
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${settingsForm.value.apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: model,
        messages: [{ role: 'user', content: 'Hello' }],
        max_tokens: 10,
      }),
    })
    
    if (response.ok) {
      testResult.value = { success: true, message: t('settings.testSuccess') }
    } else {
      const data = await response.json().catch(() => ({}))
      testResult.value = { success: false, message: data.error?.message || `HTTP ${response.status}: ${response.statusText}` }
    }
  } catch (e: any) {
    testResult.value = { success: false, message: e.message || t('settings.testFailed') }
  } finally {
    testing.value = false
  }
}

onMounted(async () => {
  await loadSession().catch(() => undefined)
  await loadSettings()
  
  // Listen for open-settings event from child components
  window.addEventListener('open-settings', () => {
    showSettingsDialog.value = true
  })
})

async function handleDemoLogin() {
  loggingIn.value = true
  try {
    await demoLogin()
    showLoginDialog.value = false
  } catch (e: any) {
    console.error('Demo login failed:', e)
  } finally {
    loggingIn.value = false
  }
}
</script>

<template>
  <div class="h-screen flex overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-72 border-r border-zinc-800 bg-zinc-950 flex flex-col">
      <div class="h-12 px-4 border-b border-zinc-800 flex items-center">
        <NuxtLink to="/" class="flex items-center gap-2">
          <div class="w-6 h-6 rounded bg-violet-600 flex items-center justify-center">
            <Blocks class="w-4 h-4 text-white" />
          </div>
          <span class="text-sm font-medium">{{ t('brand') }}</span>
        </NuxtLink>
      </div>

      <div class="flex-1 overflow-y-auto">
        <slot name="sidebar" />
      </div>

      <div class="p-4 border-t border-zinc-800">
        <template v-if="isAuthenticated">
          <div class="flex items-center gap-3 px-2">
            <div class="w-8 h-8 rounded-full bg-zinc-800 flex items-center justify-center flex-shrink-0">
              <User class="w-4 h-4 text-zinc-400" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium text-zinc-200 truncate">{{ session?.user?.displayName || 'User' }}</p>
              <p class="text-xs text-zinc-500 truncate">{{ session?.user?.email }}</p>
            </div>
            <UButton @click="logout()" variant="ghost" size="xs" color="gray" class="flex-shrink-0">
              <LogOut class="w-4 h-4" />
            </UButton>
          </div>
        </template>
        <template v-else>
          <UButton @click="showLoginDialog = true" color="primary" size="sm" block>
            <LogIn class="w-4 h-4 mr-1.5" />
            {{ t('auth.login') }}
          </UButton>
        </template>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 min-h-0 min-w-0 flex flex-col bg-zinc-950">
      <!-- Header -->
      <header class="border-b border-zinc-800 px-6 h-12 flex-shrink-0 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <UButton to="/" variant="ghost" size="xs" color="gray">
            <Home class="w-4 h-4" />
          </UButton>
          <slot name="header-actions" />
        </div>
        <div class="flex items-center gap-2">
          <slot name="header-right" />
          <UButton to="/notes" variant="ghost" size="xs" color="gray">
            <FileText class="w-3.5 h-3.5 mr-1" />
            {{ t('nav.notes') }}
          </UButton>
          <UButton to="/servers" variant="ghost" size="xs" color="gray">
            <Server class="w-3.5 h-3.5 mr-1" />
            {{ t('nav.servers') }}
          </UButton>
          <UButton @click="showSettingsDialog = true" variant="ghost" size="xs" color="gray">
            <Settings class="w-3.5 h-3.5 mr-1" />
            {{ t('nav.settings') }}
          </UButton>
          <div class="w-px h-4 bg-zinc-800 mx-1" />
          <USelectMenu
            v-model="currentLocale"
            :options="localeOptions"
            value-attribute="value"
            option-attribute="label"
            size="xs"
            color="gray"
            @update:model-value="(val: string) => setLocale(val)"
          >
            <template #label>
              <span class="text-xs">{{ currentLocale === 'zh' ? '中文' : 'EN' }}</span>
            </template>
          </USelectMenu>
        </div>
      </header>

      <slot />
    </main>

    <!-- Login Dialog -->
    <UModal v-model="showLoginDialog">
      <div class="p-6">
        <div class="text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-violet-500/10 flex items-center justify-center mx-auto mb-4">
            <LogIn class="w-6 h-6 text-violet-400" />
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">{{ t('auth.loginRequired') }}</h3>
          <p class="text-sm text-zinc-400">{{ t('auth.loginToCreate') }}</p>
        </div>
        <div class="space-y-3">
          <UButton @click="login('/')" color="primary" size="lg" block>
            <LogIn class="w-4 h-4 mr-2" />
            {{ t('auth.googleLogin') }}
          </UButton>
          <UButton @click="handleDemoLogin" color="gray" size="lg" block :loading="loggingIn">
            <User class="w-4 h-4 mr-2" />
            {{ t('auth.demoLogin') }}
          </UButton>
          <p class="text-xs text-zinc-600 text-center">{{ t('auth.demoLoginHint') }}</p>
        </div>
      </div>
    </UModal>

    <!-- Settings Dialog -->
    <UModal v-model="showSettingsDialog">
      <div class="p-6">
        <div class="flex items-center justify-between mb-6">
          <div>
            <h3 class="text-lg font-semibold text-zinc-100">{{ t('settings.title') }}</h3>
            <p class="text-sm text-zinc-400 mt-1">{{ t('settings.subtitle') }}</p>
          </div>
          <div class="flex items-center gap-2">
            <div :class="['w-2 h-2 rounded-full', configured ? 'bg-emerald-500' : 'bg-zinc-500']" />
            <span class="text-xs text-zinc-500">{{ configured ? t('settings.configured') : t('settings.notConfigured') }}</span>
          </div>
        </div>

        <div class="space-y-4">
          <UFormGroup :label="t('settings.provider')" required>
            <USelect
              v-model="settingsForm.provider"
              :options="providerOptions"
              color="gray"
              @update:model-value="onProviderChange"
            />
          </UFormGroup>

          <template v-if="settingsForm.provider !== 'mock'">
            <UFormGroup :label="t('settings.apiKey')" required>
              <UInput
                v-model="settingsForm.apiKey"
                type="password"
                :placeholder="t('settings.apiKeyPlaceholder')"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.apiKeyHelp') }}</p>
              </template>
            </UFormGroup>

            <UFormGroup :label="t('settings.model')">
              <USelect
                v-if="settingsForm.provider === 'mimo'"
                v-model="settingsForm.model"
                :options="mimoModels.map(m => ({ label: m, value: m }))"
                color="gray"
              />
              <UInput
                v-else
                v-model="settingsForm.model"
                :placeholder="defaultModels[settingsForm.provider]"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.modelHelp') }}</p>
              </template>
            </UFormGroup>

            <UFormGroup :label="t('settings.baseUrl')">
              <UInput
                v-model="settingsForm.baseUrl"
                :placeholder="defaultBaseUrls[settingsForm.provider]"
                color="gray"
              />
              <template #help>
                <p class="text-xs text-zinc-500 mt-1">{{ t('settings.baseUrlHelp') }}</p>
              </template>
            </UFormGroup>
          </template>

          <div v-else class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
            <p class="text-sm text-zinc-400">{{ t('settings.mockDesc') }}</p>
          </div>
        </div>

        <!-- Test Result -->
        <div v-if="testResult" class="mt-4 rounded-lg p-3" :class="testResult.success ? 'bg-emerald-900/20 border border-emerald-800/50' : 'bg-red-900/20 border border-red-800/50'">
          <div class="flex items-center gap-2">
            <CheckCircle v-if="testResult.success" class="w-4 h-4 text-emerald-400" />
            <AlertCircle v-else class="w-4 h-4 text-red-400" />
            <span :class="['text-sm', testResult.success ? 'text-emerald-300' : 'text-red-300']">{{ testResult.message }}</span>
          </div>
        </div>

        <div class="flex gap-3 mt-6">
          <UButton @click="handleTestApi" variant="outline" :loading="testing" class="flex-1 justify-center">
            <Zap class="w-4 h-4 mr-1.5" />
            {{ t('settings.testApi') }}
          </UButton>
          <UButton @click="handleSaveSettings" color="primary" :loading="saving" class="flex-1 justify-center">
            <Save class="w-4 h-4 mr-1.5" />
            {{ t('settings.save') }}
          </UButton>
        </div>
      </div>
    </UModal>
  </div>
</template>
