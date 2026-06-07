# API Contract

本文档约束 Atoms Demo 的前后端 API 契约。所有 JSON 字段使用 camelCase。

## API Scope

当前 API 支撑：

- 初始化。
- Google OAuth session。
- App CRUD、对话和生成。
- Notes CRUD。
- Servers CRUD 与 SSH Terminal。
- AI 设置。

后端不真实执行生成代码，也不真实部署应用。

Prompt Select 建议是产品目标，但当前没有后端接口。

## 通用规则

请求体和响应体使用 JSON：

```http
Content-Type: application/json
```

错误响应统一：

```json
{ "error": "Authentication required" }
```

受保护接口未登录时返回 `401`。

## 已实现 API

```txt
GET    /api/health
GET    /api/init
POST   /api/init

GET    /api/auth/login
GET    /api/auth/callback
POST   /api/auth/logout
GET    /api/auth/me

GET    /api/apps
POST   /api/apps
GET    /api/apps/:id
PATCH  /api/apps/:id
DELETE /api/apps/:id
POST   /api/apps/:id/messages
POST   /api/apps/:id/generate

GET    /api/notes
POST   /api/notes
GET    /api/notes/:id
PATCH  /api/notes/:id
DELETE /api/notes/:id

GET    /api/servers
POST   /api/servers
GET    /api/servers/:id
PATCH  /api/servers/:id
DELETE /api/servers/:id
GET    /api/servers/:id/terminal

GET    /api/settings/ai
PUT    /api/settings/ai
```

## App Model

```ts
type AppStatus = 'pending' | 'generating' | 'completed' | 'failed'
type AgentStepStatus = 'waiting' | 'running' | 'done' | 'error'
type AgentMessageRole = 'user' | 'agent' | 'system' | 'error'

interface App {
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
```

`GeneratedResult` 必须包含：

- `productSpec`
- `pages`
- `apis`
- `dataModels`
- `fileStructure`
- `preview`

```ts
interface GeneratedResult {
  productSpec: ProductSpec
  pages: GeneratedPage[]
  apis: GeneratedApi[]
  dataModels: DataModel[]
  fileStructure: FileNode[]
  preview: PreviewData
}
```

`fileStructure` 中的文件节点可以携带 AI 生成的文件代码：

```ts
interface FileNode {
  path: string
  type: 'file' | 'directory'
  description?: string
  content?: string
}
```

`content` 只用于 `type: 'file'` 的节点。前端应优先展示该字段；缺失时才
使用占位内容。

## App Generation

`POST /api/apps/:id/generate` 当前行为：

- 如果 App 已是 `generating`，返回 `409`。
- 如果配置了 AI Settings，调用真实 Chat Completions 风格接口。
- 如果未配置 AI Settings，使用 mock 生成。
- 生成 prompt 会带上最近的 conversation messages。
- App 已有 result 时，会把上一版 result 作为可修订上下文传给 AI。
- AI 必须返回完整的新版 `GeneratedResult`，而不是局部 diff。
- AI 响应会被解析为 `GeneratedResult`。
- 解析器允许 provider 返回以下包裹形式：
  - 直接返回 `GeneratedResult`
  - `result`
  - `generatedResult`
  - `data`
  - OpenAI / Mimo 风格 `choices[0].message.content`
  - Markdown fenced JSON
  - 文本中夹带的平衡 JSON

AI 生成必须返回 Rust 后端可反序列化的字段，不允许缺失或改名。

`POST /api/apps/:id/messages` 用于追加后续对话：

```ts
interface CreateAppMessageRequest {
  content: string
}
```

轻量多轮流程：

```txt
POST /api/apps/:id/messages
POST /api/apps/:id/generate
```

后端会保存用户消息，再根据历史消息和旧 result 重新生成完整结果。

## Notes Model

当前代码模型：

```ts
interface Note {
  id: string
  title: string
  content: string
  appId?: string
  createdAt: string
  updatedAt: string
}
```

当前接口：

```txt
GET    /api/notes
POST   /api/notes
GET    /api/notes/:id
PATCH  /api/notes/:id
DELETE /api/notes/:id
```

`POST /api/notes`：

```ts
interface CreateNoteRequest {
  title: string
  content?: string
  appId?: string
}
```

`PATCH /api/notes/:id`：

```ts
interface UpdateNoteRequest {
  title?: string
  content?: string
}
```

后续结构化目标：

- 增加 `parentId` 或 `folderId` 支持多级目录。
- 增加 `kind` 区分普通笔记、需求笔记和学习模块。
- 增加“选择到 prompt / conversation”的明确接口或前端流程。

## Server Model

```ts
type ServerAuthType = 'password' | 'private_key'
type ServerConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'failed'

interface ServerConnection {
  id: string
  name: string
  host: string
  port: number
  username: string
  authType: ServerAuthType
  status: ServerConnectionStatus
  lastConnectedAt?: string
  error?: string
  createdAt: string
  updatedAt: string
}
```

创建服务器：

```ts
interface CreateServerConnectionRequest {
  name: string
  host: string
  port?: number
  username: string
  authType: 'password' | 'private_key'
  password?: string
  privateKey?: string
}
```

服务器响应不得包含：

- `password`
- `privateKey`
- OAuth token
- session token

当前 Postgres 模式下，SSH 凭据会写入 `server_credentials` 表，用于真实
SSH 连接。该实现只适合 Demo，生产化必须改为加密存储。

Server 与 App 的绑定是产品目标。若实现字段，应使用 `appId` 或关联表表达，
不得只依赖前端临时映射。

## Terminal WebSocket

客户端消息：

```ts
type TerminalClientMessage =
  | { type: 'input'; data: string }
  | { type: 'resize'; cols: number; rows: number }
  | { type: 'disconnect' }
```

服务端消息：

```ts
type TerminalServerMessage =
  | { type: 'output'; data: string }
  | { type: 'status'; status: ServerConnectionStatus }
  | { type: 'error'; error: string }
```

WebSocket 路由：

```txt
GET /api/servers/:id/terminal
```

该路由必须通过 session 校验，连接由用户显式触发。

## Auth API

```txt
GET  /api/auth/login
GET  /api/auth/callback
POST /api/auth/logout
GET  /api/auth/me
```

`GET /api/auth/me` 返回：

```ts
interface AuthSession {
  authenticated: boolean
  provider: {
    id: string
    name: string
    mode: 'demo' | 'google'
  }
  user?: {
    id: string
    displayName: string
    email: string
    provider: AuthSession['provider']
    createdAt: string
  }
  expiresAt?: string
}
```

## AI Settings API

```txt
GET /api/settings/ai
PUT /api/settings/ai
```

配置请求：

```ts
interface UpdateAiSettingsRequest {
  provider: 'gpt' | 'mimo'
  apiKey: string
  model?: string
  baseUrl?: string
}
```

响应不得回显真实 `apiKey`，只能返回脱敏值。

当前 AI Settings 是内存态，不持久化到 Postgres。

## Prompt Suggestions API

Prompt Select 是产品必备方向，建议接口：

```txt
POST /api/prompts/suggestions
```

建议请求：

```ts
interface PromptSuggestionRequest {
  input: string
  noteIds?: string[]
  appId?: string
}
```

当前代码未实现该接口。前端若展示建议，只能使用页面内规则化建议或静态
建议，不能调用不存在的后端路由。
