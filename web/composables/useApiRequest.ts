import type { ErrorResponse } from '~/types/apps'

export class ApiRequestError extends Error {
  statusCode?: number

  constructor(message: string, statusCode?: number) {
    super(message)
    this.name = 'ApiRequestError'
    this.statusCode = statusCode
  }
}

export function normalizeApiError(error: unknown): ApiRequestError {
  if (error instanceof ApiRequestError) {
    return error
  }

  if (typeof error === 'object' && error !== null) {
    const fetchError = error as {
      statusCode?: number
      status?: number
      data?: Partial<ErrorResponse>
      message?: string
    }
    return new ApiRequestError(
      fetchError.data?.error || fetchError.message || 'Request failed',
      fetchError.statusCode || fetchError.status,
    )
  }

  return new ApiRequestError('Request failed')
}

export function useApiRequest() {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiBase || ''

  async function request<T>(path: string, options: Parameters<typeof $fetch<T>>[1] = {}) {
    try {
      return await $fetch<T>(path, {
        baseURL,
        credentials: 'include',
        ...options,
      })
    } catch (error) {
      throw normalizeApiError(error)
    }
  }

  function websocketUrl(path: string) {
    const apiBase = String(baseURL || '')
    const origin = import.meta.client ? window.location.origin : 'http://localhost:3001'
    const defaultWsBase = (() => {
      if (!import.meta.client) return origin

      const current = new URL(origin)
      if (!apiBase && (current.port === '3000' || current.port === '3002')) {
        current.port = '3001'
      }
      return current.toString()
    })()
    const url = new URL(path, apiBase || defaultWsBase)
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
    return url.toString()
  }

  return { request, websocketUrl }
}
