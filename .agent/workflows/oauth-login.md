# OAuth Login Workflow

本文档约束 Atoms Demo 的 Google OAuth 登录闭环。

在实现登录入口、callback、logout、当前用户、session、cookie 或受保护
操作前，Codex 必须先阅读本文档。

## 1. 业务目标

业务目标是提供一个可运行、可验收的 OAuth 登录 Demo：

```txt
未登录用户
  -> 点击 Sign in
  -> GET /api/auth/login
  -> Google OAuth callback
  -> 设置 HttpOnly session cookie
  -> GET /api/auth/me 返回当前用户
  -> 允许创建 App / 保存服务器 / 打开终端
  -> Logout 清除 session
```

## 2. 当前实现

当前代码支持：

- 读取 Google OAuth 环境变量。
- 创建 Google authorize URL。
- 设置 OAuth state cookie 和 redirect cookie。
- callback 校验 state。
- 用 code 换 Google access token。
- 读取 Google userinfo。
- 创建后端 session。
- session cookie 使用 HttpOnly。
- Postgres 模式下 session 写入 `sessions` 表。
- 内存模式下 session 写入运行时 state。
- logout 删除 session 并清除 cookie。
- `/api/auth/me` 返回认证状态和用户展示信息。

当前不实现：

- 注册流程。
- 组织、团队、RBAC、ACL、计费权限。
- refresh token 管理。
- session 过期时间。
- 生产级 Secure cookie 自动区分。

## 3. 产品边界

必须实现：

- 登录入口。
- OAuth callback。
- 当前 session 查询。
- logout。
- Google 标识。
- 未登录受保护操作提示。

可以简化：

- 需要配置 Google OAuth Provider。
- session 可以保存在 Postgres 或后端内存。
- 本地 HTTP Demo 可以不设置 Secure cookie。

禁止：

- 实现注册流程。
- 实现组织、团队、RBAC、ACL、计费权限。
- 将 OAuth token、authorization code 或 session token 暴露到前端。
- 使用 localStorage 保存 session token。
- 在 URL 中泄露 session token。

## 4. 当前受保护操作

当前代码已保护：

- `POST /api/apps`
- `GET /api/apps`
- `PATCH /api/apps/:id`
- `DELETE /api/apps/:id`
- `POST /api/apps/:id/messages`
- `POST /api/apps/:id/generate`
- `GET /api/notes`
- `POST /api/notes`
- `GET /api/notes/:id`
- `PATCH /api/notes/:id`
- `DELETE /api/notes/:id`
- `POST /api/servers`
- `PATCH /api/servers/:id`
- `DELETE /api/servers/:id`
- `WebSocket /api/servers/:id/terminal`

当前仍需收紧：

- `GET /api/apps/:id`
- `GET /api/servers`
- `GET /api/servers/:id`
- `GET /api/settings/ai`
- `PUT /api/settings/ai`

未登录时返回：

```json
{ "error": "Authentication required" }
```

## 5. Session 与 Cookie

- session cookie 名称为 `atoms_demo_session`。
- cookie 必须设置 `HttpOnly`。
- cookie 必须设置 `SameSite=Lax`。
- 本地 HTTP Demo 可以不设置 `Secure`。
- OAuth state cookie 名称为 `atoms_demo_oauth_state`。
- OAuth redirect cookie 名称为 `atoms_demo_oauth_redirect`。
- logout 必须清除 session cookie 并删除服务端 session。

## 6. UI 行为

所有入口页或工作台页应提供轻量登录状态：

- 未登录：显示 Google 和 Sign in。
- 已登录：显示用户显示名、邮箱或 provider 摘要，并提供 Logout。
- callback 成功：显示登录成功，可以继续到 App Builder 或 Servers。
- callback 失败：显示失败原因和重新登录入口。
- 受保护操作被拦截：显示需要登录的错误状态。

## 7. 验收标准

Reviewer 必须验证：

- `/api/auth/me` 未登录返回 `authenticated: false`。
- `/api/auth/login` 发起登录并设置 state cookie。
- `/api/auth/callback` 校验 state，成功后设置 session cookie。
- `/api/auth/me` 登录后返回用户资料，不返回 token。
- `/api/auth/logout` 清除 session。
- 未登录时创建 App、保存服务器、打开终端被拒绝。
- UI 明确标识 Google。
- 没有 token 存入 localStorage 或出现在 URL / JSON 响应中。

如果读取类接口仍未保护，最终交付报告必须列为安全风险。
