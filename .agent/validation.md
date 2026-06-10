# Validation

本文档约束 Atoms Demo 的验收标准、测试方式和交付口径。

## 完成定义

声明完成前必须确认：

- 产品边界符合 `.agent/product.md`。
- 页面和跳转符合 `.agent/routes.md`。
- API 字段符合 `.agent/api-contract.md`。
- 权限符合 `.agent/permission.md`。
- UI 符合 `.agent/ui-rules.md`。
- 相关 workflow 已阅读并覆盖异常状态。
- 已运行与改动范围匹配的验证命令。
- 未验证项已经明确说明原因。

## 当前代码功能清单

当前可验证功能：

- Rust Axum API 启动和健康检查。
- Demo 初始化状态读取和完成。
- Google OAuth 登录、callback、session 查询和 logout。
- App CRUD、message 追加和 generate。
- mock 生成与真实 AI 生成二选一。
- AI 返回解析为 `GeneratedResult`。
- GeneratedResult 分区展示所需字段。
- Notes 基础 CRUD。
- Servers 基础 CRUD。
- SSH WebSocket 终端消息转发。
- Postgres migration 覆盖当前核心表。
- AI Settings 运行时配置和脱敏响应。
- Workspace 轻量后续对话和基于历史的再生成。

当前未完整落地：

- Prompt Suggestions 后端接口。
- Notes 多级目录和结构化块。
- Notes 一键带入 prompt / conversation。
- Server 与 App 的持久化绑定。
- AI Settings Postgres 持久化。
- 全量读取接口的 owner 校验。
- SSH 凭据生产级加密和审计。

## 核心业务验收

标准路径：

```txt
/
  -> 输入 idea
  -> 选择或采用 prompt 建议
  -> POST /api/apps
  -> /app/:id  (自动触发)
  -> EventSource GET /api/apps/:id/generate/stream
  -> Files Tab 看代码实时写入 -> 完成后自动进 Preview Tab(真实运行)
```

必须验证：

- 首页是工作台布局，不是纯静态介绍页。
- idea 为空时有错误反馈。
- prompt 建议可被选择或采用。
- 未登录时受保护写入有明确拦截。
- 登录后可以创建 App。
- Workspace 展示 steps、messages、status 和 result。
- 生成过程内联流式展示：步骤进度、代码逐字写入 CodeMirror、文件树并发 spinner（无遮罩 Modal）。
- Preview 为真实运行的应用（vue3-sfc-loader 沙箱），内部路由可跳转、外部隔离。
- Workspace 支持追加后续需求并触发再生成（流式、整份覆盖）。
- 再生成后 result、文件结构和 Preview 使用新结果刷新。
- generating 被中断后，重新进入 `/app/:id` 能自动重跑；离开/刷新有二次确认。
- completed 后可以进入 Preview。
- failed 时有错误原因和 retry 入口。

## AI 生成验收

必须验证：

- 未配置 AI Settings 时 mock 生成可用。
- 配置 AI Settings 后走真实 AI 调用。
- 已有 result 时，再生成需要带上历史 messages 和旧 result 上下文。
- 响应必须解析出 `productSpec`、`pages`、`apis`、`dataModels`、
  `fileStructure`、`preview`。
- `fileStructure` 的 file 节点可包含 `content`。
- AI provider 返回 wrapper 或 fenced JSON 时解析器仍可工作。
- 解析失败时 App 进入 `failed`，并返回清晰错误。

## Notes 验收

当前基础验收：

- 可以进入 `/notes`。
- 可以创建、编辑、保存和删除笔记。
- 空状态、加载状态和保存状态清晰。
- 后端 Notes API 返回 camelCase 字段。
- Notes 需要已登录 session。

目标扩展验收：

- 支持多级目录或结构化分组。
- 可以把笔记一键带入 prompt 或 conversation。
- 提供 AI 高可用上下文学习模块。

如果目标扩展未完成，必须在交付说明中标为未完成或后续优先级。

## Servers 验收

必须验证：

- 可以进入 `/servers`。
- 可以创建、更新、删除服务器连接配置。
- 响应不回显 password 或 privateKey。
- Terminal 页面展示 Xterm 区域。
- Connect 由用户显式触发。
- WebSocket 断开或连接失败有反馈。
- 服务器模块不混入 Preview。
- 与 App 的绑定关系若未完全落地，必须在风险中说明。
- Postgres 模式下凭据可用于真实 SSH 连接，但要声明明文存储风险。

## Auth 验收

必须验证：

- `GET /api/auth/me` 未登录返回 `authenticated: false`。
- `GET /api/auth/login` 使用 Google OAuth 配置。
- 未配置 Google OAuth 时返回清晰错误。
- callback 成功后设置 HttpOnly session cookie。
- logout 后受保护操作恢复未登录状态。
- 任何响应都不返回 OAuth token 或 session token。

## API 验收

必须检查：

- `GET /api/health`
- `GET /api/init`
- `POST /api/init`
- `GET /api/apps`
- `POST /api/apps`
- `GET /api/apps/:id`
- `PATCH /api/apps/:id`
- `DELETE /api/apps/:id`
- `POST /api/apps/:id/messages`
- `POST /api/apps/:id/generate`
- `GET /api/notes`
- `POST /api/notes`
- `GET /api/notes/:id`
- `PATCH /api/notes/:id`
- `DELETE /api/notes/:id`
- `GET /api/servers`
- `POST /api/servers`
- `GET /api/servers/:id`
- `PATCH /api/servers/:id`
- `DELETE /api/servers/:id`
- `GET /api/servers/:id/terminal`
- `GET /api/settings/ai`
- `PUT /api/settings/ai`

## 推荐验证命令

根据改动范围运行：

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
pnpm -C web typecheck
pnpm -C web build
markdownlint .agent/**/*.md .agent/*.md docs/*.md
```

当前 `web/package.json` 没有 lint script，不能报告 `pnpm -C web lint` 通过。

纯文档改动可以只运行 markdownlint，并在最终报告中说明未运行代码构建。

## 最终报告要求

最终报告必须包含：

- 修改的文档。
- 当前完成程度。
- 未完成或风险。
- 验证命令和结果。
- 未运行代码构建或测试的原因。
