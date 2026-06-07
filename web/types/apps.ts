export type AppStatus = 'pending' | 'generating' | 'completed' | 'failed'

export type InitStatus = 'not_initialized' | 'ready'

export type ServerConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'failed'

export type AgentStepStatus = 'waiting' | 'running' | 'done' | 'error'

export type AgentMessageRole = 'user' | 'agent' | 'system' | 'error'

export interface App {
  id: string
  title: string
  idea: string
  status: AppStatus
  steps: AgentStep[]
  messages: AgentMessage[]
  result?: GeneratedResult
  error?: string
  createdAt: string
  updatedAt: string
}

export interface AgentStep {
  id: string
  name: string
  description: string
  status: AgentStepStatus
  startedAt?: string
  completedAt?: string
}

export interface AgentMessage {
  id: string
  role: AgentMessageRole
  agentName?: string
  content: string
  createdAt: string
}

export interface GeneratedResult {
  productSpec: ProductSpec
  pages: GeneratedPage[]
  apis: GeneratedApi[]
  dataModels: DataModel[]
  fileStructure: FileNode[]
  preview: PreviewData
}

export interface ProductSpec {
  name: string
  description: string
  targetUsers: string[]
  coreFeatures: string[]
  userFlow: string[]
}

export interface GeneratedPage {
  path: string
  name: string
  description: string
  components: string[]
}

export interface GeneratedApi {
  method: 'GET' | 'POST' | 'PATCH' | 'DELETE'
  path: string
  description: string
  requestExample?: unknown
  responseExample?: unknown
}

export interface DataModel {
  name: string
  description: string
  fields: DataModelField[]
}

export interface DataModelField {
  name: string
  type: string
  required: boolean
  description: string
}

export interface FileNode {
  path: string
  type: 'file' | 'directory'
  description?: string
  content?: string
}

export interface PreviewData {
  title: string
  description: string
  theme: 'dark' | 'light'
  sections: PreviewSection[]
  actions: PreviewAction[]
}

export interface PreviewSection {
  id: string
  title: string
  type: 'hero' | 'stats' | 'list' | 'form' | 'cards' | 'table'
  content: string
  items?: PreviewItem[]
}

export interface PreviewItem {
  title: string
  description?: string
  value?: string
}

export interface PreviewAction {
  label: string
  type: 'primary' | 'secondary'
}

export interface ErrorResponse {
  error: string
}

export interface CreateAppRequest {
  idea: string
}

export interface CreateAppMessageRequest {
  content: string
}

export interface HealthResponse {
  status: 'ok' | string
}

export interface DemoInitState {
  status: InitStatus
  apiHealthy: boolean
  completedAt?: string
  createdAt: string
  updatedAt: string
}

export interface OAuthProviderSummary {
  id: 'google' | 'demo'
  name: string
  mode: 'google' | 'demo'
}

export interface OAuthUser {
  id: string
  displayName: string
  email: string
  provider: OAuthProviderSummary
  createdAt: string
}

export interface AuthSession {
  authenticated: boolean
  provider: OAuthProviderSummary
  user?: OAuthUser
  expiresAt?: string
}

export interface ServerConnection {
  id: string
  name: string
  host: string
  port: number
  username: string
  authType: 'password' | 'private_key'
  status: ServerConnectionStatus
  lastConnectedAt?: string
  error?: string
  createdAt: string
  updatedAt: string
}

export interface CreateServerConnectionRequest {
  name: string
  host: string
  port?: number
  username: string
  authType: 'password' | 'private_key'
  password?: string
  privateKey?: string
}

export type UpdateServerConnectionRequest = Partial<CreateServerConnectionRequest>

export type TerminalClientMessage =
  | { type: 'input', data: string }
  | { type: 'resize', cols: number, rows: number }
  | { type: 'disconnect' }

export type TerminalServerMessage =
  | { type: 'output', data: string }
  | { type: 'status', status: ServerConnectionStatus }
  | { type: 'error', error: string }

export interface Note {
  id: string
  title: string
  content: string
  appId?: string
  createdAt: string
  updatedAt: string
}
