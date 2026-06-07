import type { App, CreateAppMessageRequest, CreateAppRequest } from '~/types/apps'

export function useAppsApi() {
  const { request } = useApiRequest()

  return {
    listApps() {
      return request<App[]>('/api/apps')
    },
    createApp(payload: CreateAppRequest) {
      return request<App>('/api/apps', {
        method: 'POST',
        body: payload,
      })
    },
    getApp(id: string) {
      return request<App>(`/api/apps/${encodeURIComponent(id)}`)
    },
    generateApp(id: string) {
      return request<App>(`/api/apps/${encodeURIComponent(id)}/generate`, {
        method: 'POST',
        body: {},
      })
    },
    createAppMessage(id: string, payload: CreateAppMessageRequest) {
      return request<App>(`/api/apps/${encodeURIComponent(id)}/messages`, {
        method: 'POST',
        body: payload,
      })
    },
    updateApp(id: string, payload: { title?: string; idea?: string }) {
      return request<App>(`/api/apps/${encodeURIComponent(id)}`, {
        method: 'PATCH',
        body: payload,
      })
    },
    deleteApp(id: string) {
      return request<{ success: boolean }>(`/api/apps/${encodeURIComponent(id)}`, {
        method: 'DELETE',
      })
    },
  }
}
