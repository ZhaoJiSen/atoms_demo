# 项目架构

本文档约束 Atoms Demo 的目录结构、前后端职责、数据边界和扩展取舍。

## 当前目录

```txt
.agent/
docs/
src/
web/
Cargo.toml
docker-compose.yml
```

- `.agent/`：Agent harness 需求与执行约束。
- `docs/`：交付说明和面试材料。
- `src/`：Rust Axum 后端。
- `web/`：Nuxt 前端。
- `docker-compose.yml`：本地 Postgres 等开发基础设施。

不得为实现单个功能随意重构根目录。

## 前端职责

前端位于 `web/`，技术栈为 Nuxt、TypeScript、Tailwind CSS、Nuxt UI、
CodeMirror、Vue SFC loader 和 Xterm。

前端负责：

- 暗色工作台布局、侧边栏、顶部导航和语言切换。
- 首页 prompt 输入、建议项展示和创建 App 入口。
- App Workspace、GeneratedResult Tabs、文件结构和代码查看。
- Preview 原型展示和 sandbox / SFC 预览能力。
- Notes 列表、编辑器和后续结构化入口。
- Servers 列表、服务器表单和 Xterm 终端页。
- Google OAuth 登录状态展示。
- AI 设置表单。
- 调用后端 `/api` 接口。

前端不得绕过 API 用硬编码数据完成核心业务闭环。

## 后端职责

后端位于 `src/`，技术栈为 Rust、Axum、Tokio、Serde、SQLx、Reqwest、
SSH2 和 Tower HTTP。

后端负责：

- `/api/health` 健康检查。
- Demo 初始化状态。
- Google OAuth 登录、callback、session 和 logout。
- App CRUD、对话消息追加和生成流程。
- AI / mock 生成选择。
- AI 返回内容解析和 `GeneratedResult` 反序列化。
- Notes CRUD。
- Servers CRUD。
- SSH / WebSocket 终端代理。
- AI 设置读写。
- Postgres 可用时的数据持久化。

## Rust 模块边界

当前 Rust 代码按职责分割：

- `main.rs`：启动、环境变量和路由挂载。
- `routes.rs`：App、Notes、AI Settings 和主 API 路由。
- `auth.rs`：Google OAuth、session cookie 和当前用户。
- `db.rs`：Postgres migration 与 CRUD。
- `models.rs`：API request / response / domain model。
- `state.rs`：内存态和 Postgres state。
- `mock.rs`：无 AI 配置时的 mock 生成。
- `ai.rs`：AI 调用主流程。
- `ai/prompt.rs`：内置系统提示词和生成提示词。
- `ai/parser.rs`：AI 响应提取、JSON 解析和枚举标准化。
- `servers.rs`：服务器连接 CRUD 和凭据校验。
- `servers/terminal.rs`：SSH WebSocket 终端会话。

新增 Rust 功能时应优先复用这些边界，不要把所有逻辑堆回 `routes.rs`。

## 数据模型边界

当前代码中的核心模型包括：

- `App`：App 项目、状态、steps、messages、result。
- `GeneratedResult`：产品方案、页面、API、数据模型、文件结构和 preview。
- `OAuthUser` / `AuthSession`：Google OAuth 最小会话。
- `Note`：笔记标题、内容、可选 `appId`。
- `ServerConnection`：服务器连接非敏感元数据。
- `ServerCredential`：SSH password 或 private key，仅后端使用。
- `AiSettings`：AI provider、model、baseUrl 和密钥配置。

## 持久化策略

Postgres migration 当前覆盖：

- `demo_init`
- `users`
- `sessions`
- `apps`
- `servers`
- `server_credentials`
- `notes`

内存模式用于快速 Demo 和测试；Postgres 模式用于实际持久化。

当前持久化事实：

- App、Notes、Servers、OAuth users 和 sessions 支持 Postgres。
- Server 凭据在 Postgres 模式下写入 `server_credentials`。
- AI Settings 当前只保存在内存，不进 Postgres。

## 敏感信息规则

必须遵守：

- OAuth token 不返回前端。
- session token 只通过 HttpOnly cookie 传递。
- AI Settings 响应只返回脱敏 `apiKey`。
- Server 响应不返回 password 或 privateKey。

当前 Demo 风险：

- SSH 凭据在 Postgres 中明文保存，只能作为面试 Demo 取舍。
- 生产化必须增加加密、审计、授权、过期和轮转。

## 模块关系

```txt
Prompt Select
  -> App Builder
  -> Workspace / Preview

Notes
  -> Prompt / Conversation Context
  -> App

Servers
  -> App Binding
  -> SSH Terminal

AI Settings
  -> App Generation
  -> Prompt Suggestion

Conversation
  -> App Messages
  -> Regenerate Full Result
```

## 工程取舍

本项目选择 Rust + Axum 作为后端，是为了展示稳定 API、显式错误处理、
Postgres 持久化和 WebSocket 能力；选择 Nuxt 是为了快速搭建真实工作台
体验。

需要控制复杂度：

- 不引入微服务。
- 不引入完整组织权限系统。
- 不把运维能力做成完整堡垒机。
- 不把笔记做成独立知识库产品。
- 不为了展示 AI 而牺牲可运行性。

## 当前风险

- Notes 当前是基础文本模型，尚未完全实现多级目录。
- Server 与 App 的绑定在产品上需要明确，但当前模型仍需进一步固化字段。
- AI Settings 只存内存，刷新服务后需要重新配置。
- 部分读取接口未按 user_id 限制，需要继续收紧数据隔离。
- Postgres migration 已覆盖当前模块，但生产级 migration 管理仍未引入。
