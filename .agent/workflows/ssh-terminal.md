# SSH Terminal Workflow

本文档约束服务器模块：服务器连接管理、App 运维关联和 SSH / Xterm 终端。

## 业务目标

Atoms 类产品能做代码 Bug 修复、功能调整、构建和部署，但线上运维问题
需要额外入口。Servers 模块用于补齐这类场景。

目标路径：

```txt
Google OAuth 登录
  -> 进入 Servers
  -> 新增服务器连接
  -> 关联 App
  -> 打开 Terminal
  -> 建立 SSH 会话
  -> 查看状态或修改部署文件
  -> 断开连接
```

## 产品定位

Servers 是 App Builder 的延伸能力，不是完整 JumpServer。

必须体现：

- 服务器与 App 的关联方向。
- 用户显式连接 SSH。
- Xterm 交互终端。
- 运维问题入口。
- 高阶或付费能力的产品空间。

## 当前能力

当前代码提供：

- Servers CRUD。
- Server 状态。
- password / private key 两种认证方式。
- SSH terminal WebSocket 路由。
- 后端 SSH 会话建立。
- Xterm 输入、resize、disconnect 消息转发。
- SSH output、status、error 消息返回。
- 前端 Servers 列表页。
- 前端 Terminal 页。
- 响应不回显 password 或 privateKey。

当前未完成：

- App 绑定关系持久化。
- 凭据加密存储。
- 连接审计。
- 服务器命令授权策略。
- 多人协作会话。
- 文件管理器、日志面板、服务状态面板。

## Server 状态

```txt
disconnected -> connecting -> connected
disconnected -> connecting -> failed
connected -> disconnected
failed -> connecting -> connected
failed -> connecting -> failed
```

## 创建服务器连接

流程：

1. 前端校验 session。
2. 前端校验 name、host、username、authType。
3. password 认证校验 password。
4. private key 认证校验 privateKey。
5. 调用 `POST /api/servers`。
6. 后端保存服务器元数据。
7. 后端保存 SSH 凭据以支持真实连接。
8. 后端响应不得包含 password 或 privateKey。
9. 前端刷新列表。

## 打开终端

流程：

1. 用户点击 Terminal。
2. 进入 `/servers/:id/terminal`。
3. 页面调用 `GET /api/servers/:id`。
4. 页面展示服务器摘要和 disconnected 状态。
5. 用户显式点击 Connect。
6. 前端连接 `WebSocket /api/servers/:id/terminal`。
7. 后端校验 session 并建立 SSH 会话。
8. Xterm 输入通过 WebSocket 发送给后端。
9. SSH 输出通过 WebSocket 返回前端。
10. 用户点击 Disconnect 或离开页面后断开。

## Terminal 消息

客户端：

- `input`
- `resize`
- `disconnect`

服务端：

- `output`
- `status`
- `error`

## 数据安全

应持久化：

- server id
- name
- host
- port
- username
- authType
- appId 或 App 关联关系
- lastConnectedAt
- createdAt
- updatedAt

当前 Demo 为了真实连接也会保存：

- password 或 privateKey 对应的 secret。

不得回显：

- password
- privateKey
- passphrase
- one-time token

Postgres 模式下，当前凭据写入 `server_credentials` 表且未加密。该取舍
只适用于面试 Demo。生产化必须替换为加密存储、审计和授权方案。

## 验收标准

- `/servers` 有空状态、列表状态和错误状态。
- 可以创建服务器连接。
- 响应中没有 password 或 privateKey。
- `/servers/:id/terminal` 展示 Xterm 区域。
- Connect 由用户显式触发。
- 未登录保存或打开终端被拒绝。
- 连接失败有错误提示和 retry 入口。
- 输入能通过 WebSocket 写入 SSH channel。
- Disconnect 后状态回到 disconnected。
- Preview 页面没有承载 SSH 终端。
- App 绑定如未持久化，交付说明必须明确列为风险。
- 凭据明文持久化必须明确列为 Demo 风险。
