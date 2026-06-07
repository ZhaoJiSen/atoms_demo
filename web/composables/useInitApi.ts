import type { DemoInitState, HealthResponse } from '~/types/apps'

export function useInitApi() {
  const { request } = useApiRequest()

  return {
    getHealth() {
      return request<HealthResponse>('/api/health')
    },
    getInit() {
      return request<DemoInitState>('/api/init')
    },
    completeInit() {
      return request<DemoInitState>('/api/init', {
        method: 'POST',
        body: {},
      })
    },
  }
}
