import type { AuthSession } from '~/types/apps'

export function useAuthApi() {
  const config = useRuntimeConfig()
  const { request } = useApiRequest()
  const session = useState<AuthSession | null>('auth-session', () => null)
  const loading = useState('auth-loading', () => false)
  const error = useState<string | null>('auth-error', () => null)

  async function loadSession() {
    loading.value = true
    error.value = null
    try {
      session.value = await request<AuthSession>('/api/auth/me')
      return session.value
    } catch (e: any) {
      error.value = e?.message || 'Failed to load auth session'
      session.value = null
      throw e
    } finally {
      loading.value = false
    }
  }

  function login(redirect = '/auth/callback') {
    if (!import.meta.client) return

    const baseURL = String(config.public.apiBase || window.location.origin)
    const url = new URL('/api/auth/login', baseURL)
    url.searchParams.set('redirect', redirect)
    window.location.href = url.toString()
  }

  async function logout() {
    loading.value = true
    error.value = null
    try {
      await request<{ success: boolean }>('/api/auth/logout', {
        method: 'POST',
      })
      session.value = {
        authenticated: false,
        provider: {
          id: 'google',
          name: 'Google',
          mode: 'google',
        },
      }
    } catch (e: any) {
      error.value = e?.message || 'Failed to sign out'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function demoLogin() {
    loading.value = true
    error.value = null
    try {
      session.value = await request<AuthSession>('/api/auth/demo', {
        method: 'POST',
      })
      return session.value
    } catch (e: any) {
      error.value = e?.message || 'Failed to login as demo user'
      throw e
    } finally {
      loading.value = false
    }
  }

  const isAuthenticated = computed(() => session.value?.authenticated === true)

  return {
    session,
    loading,
    error,
    isAuthenticated,
    loadSession,
    login,
    logout,
    demoLogin,
  }
}
