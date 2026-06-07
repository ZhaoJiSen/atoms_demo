# Permission Rules

本文档约束 Atoms Demo 的权限、数据访问和安全边界。

## 当前权限模型

本 Demo 不实现注册、组织、团队、RBAC、审批流或计费权限。

登录使用 Google OAuth 最小闭环：

- `/api/auth/login`
- `/api/auth/callback`
- `/api/auth/me`
- `/api/auth/logout`

session 通过 HttpOnly cookie 传递。前端不得把 session token 放入
localStorage，也不得在 JSON 响应中接收 token。

## 匿名可访问

匿名用户可以访问：

- 首页工作台外观。
- 初始化页面。
- OAuth 登录入口。
- 健康检查。
- 部分只读演示内容。

匿名用户不能执行写入或敏感操作。

## 当前代码权限事实

当前已要求登录的接口：

```txt
POST   /api/apps
GET    /api/apps
PATCH  /api/apps/:id
DELETE /api/apps/:id
POST   /api/apps/:id/messages
POST   /api/apps/:id/generate

GET    /api/notes
POST   /api/notes
GET    /api/notes/:id
PATCH  /api/notes/:id
DELETE /api/notes/:id

POST   /api/servers
PATCH  /api/servers/:id
DELETE /api/servers/:id
GET    /api/servers/:id/terminal
```

当前尚未要求登录或 owner 校验不足的接口：

```txt
GET    /api/apps/:id
GET    /api/servers
GET    /api/servers/:id
GET    /api/settings/ai
PUT    /api/settings/ai
```

这些接口在 Demo 中可运行，但交付时必须列为安全边界风险。

## 目标权限规则

目标上，已登录用户可以：

- 创建、读取、更新、删除自己的 App。
- 追加 App conversation message。
- 触发 App generate。
- 创建、读取、更新、删除自己的 Notes。
- 选择 Notes 作为 prompt 或 conversation 上下文。
- 配置 AI Settings。
- 创建、读取、更新、删除自己的 ServerConnection。
- 显式打开 SSH / Xterm 终端。

目标上，以下接口都应要求已登录 session：

```txt
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

未登录响应：

```json
{ "error": "Authentication required" }
```

## 数据归属

Demo 阶段所有业务数据都应归属于当前 Google OAuth session。

必须避免：

- 用户 A 读取或修改用户 B 的 App。
- 用户 A 读取或修改用户 B 的 Notes。
- 用户 A 打开用户 B 的服务器终端。

当前代码中，部分 Postgres 查询仍按 ID 查询，缺少 `user_id` 条件。这是
下一轮需要收紧的安全点。

## 禁止事项

禁止：

- 实现真实注册或生产账号系统。
- 暴露 OAuth access token、refresh token、authorization code。
- 暴露 session token。
- 使用 localStorage 保存 session token。
- 读取用户本地任意文件。
- 执行用户生成出来的代码。
- 自动扫描服务器或内网。
- 未经用户点击自动建立 SSH 连接。
- 在 UI 中回显已保存的敏感凭据。
- 在错误响应中泄露 Rust panic、数据库错误、环境变量或绝对路径。

## OAuth 安全

- cookie 使用 HttpOnly。
- cookie 至少使用 `SameSite=Lax`。
- 本地 HTTP Demo 可以不设置 `Secure`，生产 HTTPS 必须设置 `Secure`。
- callback 必须校验 state。
- 登录 redirect 必须限制为站内路径。
- `/api/auth/me` 只能返回展示信息和认证状态。

## SSH 安全

服务器连接只允许用户显式创建和显式连接。

响应不得包含：

- password
- privateKey
- passphrase
- one-time token

当前 Demo 为了建立真实 SSH 会话，会保存凭据：

- 内存模式：凭据在运行时内存中。
- Postgres 模式：凭据在 `server_credentials` 表中明文保存。

这不是生产方案。生产化需要：

- 字段级加密。
- KMS 或环境密钥托管。
- 操作审计。
- 连接授权。
- 凭据过期和轮转。

## 验收规则

权限相关需求完成后必须检查：

- 未登录写入返回统一错误。
- OAuth token 和 session token 没有进入前端。
- Notes、Apps、Servers 的数据归属没有串用户。
- Server 响应不回显敏感凭据。
- Terminal 不会自动连接。
- AI Settings 不回显真实 API key。
- 错误响应不泄露内部实现。

如果当前代码未满足目标权限，最终报告必须明确列出差距。
