import type {
  CreateServerConnectionRequest,
  ServerConnection,
  TerminalClientMessage,
  UpdateServerConnectionRequest,
} from '~/types/apps'

export function useServersApi() {
  const { request, websocketUrl } = useApiRequest()

  return {
    listServers() {
      return request<ServerConnection[]>('/api/servers')
    },
    createServer(payload: CreateServerConnectionRequest) {
      return request<ServerConnection>('/api/servers', {
        method: 'POST',
        body: payload,
      })
    },
    getServer(id: string) {
      return request<ServerConnection>(`/api/servers/${encodeURIComponent(id)}`)
    },
    updateServer(id: string, payload: UpdateServerConnectionRequest) {
      return request<ServerConnection>(`/api/servers/${encodeURIComponent(id)}`, {
        method: 'PATCH',
        body: payload,
      })
    },
    deleteServer(id: string) {
      return request<{ success: boolean }>(`/api/servers/${encodeURIComponent(id)}`, {
        method: 'DELETE',
      })
    },
    terminalWebSocketUrl(id: string) {
      return websocketUrl(`/api/servers/${encodeURIComponent(id)}/terminal`)
    },
    encodeTerminalMessage(message: TerminalClientMessage) {
      return JSON.stringify(message)
    },
  }
}
