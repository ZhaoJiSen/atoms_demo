import type { Note } from '~/types/apps'

export function useNotesApi() {
  const { request } = useApiRequest()

  return {
    listNotes() {
      return request<Note[]>('/api/notes')
    },
    createNote(payload: { title: string; content?: string; appId?: string }) {
      return request<Note>('/api/notes', {
        method: 'POST',
        body: payload,
      })
    },
    getNote(id: string) {
      return request<Note>(`/api/notes/${encodeURIComponent(id)}`)
    },
    updateNote(id: string, payload: { title?: string; content?: string }) {
      return request<Note>(`/api/notes/${encodeURIComponent(id)}`, {
        method: 'PATCH',
        body: payload,
      })
    },
    deleteNote(id: string) {
      return request<{ success: boolean }>(`/api/notes/${encodeURIComponent(id)}`, {
        method: 'DELETE',
      })
    },
  }
}
