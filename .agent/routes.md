# Routes

本文档约束前端页面路由、后端 API 路由和模块跳转关系。

## 前端路由

当前 Nuxt 页面结构包含：

```txt
web/pages/
  index.vue
  index/app/[id].vue      # App 工作台，嵌在首页布局内，路由为 /app/:id
  initialize.vue
  auth/callback.vue
  app/new.vue
  notes/index.vue
  servers/index.vue
  servers/[id]/terminal.vue
  settings/index.vue
```

App 工作台已统一为 `pages/index/app/[id].vue`（路由 `/app/:id`，常驻首页侧边栏布局）。
原先的 `app/[id]/workspace.vue` 与 `app/[id]/preview.vue` 已删除，不要再恢复重复入口。

页面职责：

- `/`：主工作台入口，包含侧边栏 App 列表、中心 prompt 输入和建议项。
- `/initialize`：Demo 初始化页面。
- `/auth/callback`：Google OAuth callback 结果页。
- `/app/new`：创建 App 页面，创建后跳转 `/app/:id`。
- `/app/:id`：Agent 工作台，含 Tab（Preview / Files / Pages / API / Models / Spec / Chat）。
  Preview 直接运行生成的应用，Files 在生成时实时显示代码写入。
- `/notes`：笔记列表与编辑器。
- `/servers`：服务器连接列表与配置。
- `/servers/:id/terminal`：SSH / Xterm 终端页。
- `/settings`：AI provider 设置。

## 当前主布局

主布局不是营销官网，而是工作台：

```txt
Top Bar
  Notes / Servers / Language / Settings

Left Sidebar
  Create New App
  App List
  Login / User State

Main Panel
  Prompt Textarea
  Select-style Prompt Suggestions
  Create App Button
```

## 页面跳转

标准路径：

```txt
/
  -> select or edit prompt
  -> POST /api/apps
  -> /app/:id
  -> EventSource GET /api/apps/:id/generate/stream  (流式分阶段生成)
  -> 生成中：Files Tab 看代码实时写入；完成后自动切到 Preview Tab
```

`/app/:id` 进入时若状态为 `pending` 或 `generating`（上次被中断）会自动触发生成。

笔记路径：

```txt
/notes
  -> create or edit note
  -> optional appId association
  -> future: select note as prompt context
```

服务器路径：

```txt
/servers
  -> create server
  -> /servers/:id/terminal
  -> WebSocket /api/servers/:id/terminal
```

Server 与 App 绑定仍是产品目标，当前路径中不要假装已完成持久化绑定。

登录路径：

```txt
/api/auth/login
  -> Google OAuth
  -> /api/auth/callback
  -> /auth/callback
  -> GET /api/auth/me
```

## 后端 API 路由

当前后端 API 使用 `/api` 前缀：

```txt
GET    /api/health
GET    /api/init
POST   /api/init

GET    /api/auth/login
GET    /api/auth/callback
POST   /api/auth/logout
POST   /api/auth/demo
GET    /api/auth/me

GET    /api/apps
POST   /api/apps
GET    /api/apps/:id
PATCH  /api/apps/:id
DELETE /api/apps/:id
POST   /api/apps/:id/messages
POST   /api/apps/:id/generate
GET    /api/apps/:id/generate/stream   # SSE 流式分阶段生成（前端默认走这条）

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

## 待补路由建议

Prompt Select 目前应作为产品必备能力继续补齐，建议新增：

```txt
POST /api/prompts/suggestions
```

请求输入用户原始想法和可选 note 上下文，响应返回可选择的 prompt 建议。

结构化 Notes 后续建议新增：

```txt
GET    /api/note-folders
POST   /api/note-folders
PATCH  /api/note-folders/:id
DELETE /api/note-folders/:id
```

Server 与 App 绑定后续可以选择：

```txt
POST   /api/apps/:id/servers/:serverId
DELETE /api/apps/:id/servers/:serverId
```

这些是后续扩展建议，不能在当前代码尚未实现时写入前端调用。
