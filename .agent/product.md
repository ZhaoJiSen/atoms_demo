# 产品说明

本项目是一个面向面试交付的 Atoms Demo。它不是营销页，也不是完整
Atoms 复刻，而是一个可运行的 AI App Builder 工作台：用户把模糊想法
输入系统，系统帮助用户创建 App、调用大模型生成应用方案，并通过笔记
和服务器模块补齐真实使用场景。

文档必须区分三类状态：

- 已实现：当前代码已经具备并可以验证。
- 受控简化：为 Demo 交付刻意做小的部分。
- 后续扩展：产品方向明确，但当前代码尚未完整落地。

## 产品定位

本 Demo 的定位是“从想法到应用，只需一次对话”的 AI 工作台。

当前布局以暗色工具台为主：

- 左侧是 App 工作区列表和创建入口。
- 中心是自然语言输入区和 Select 形式的 prompt 建议展示。
- 顶部导航提供笔记、服务器、语言和 AI 设置等能力入口。
- 登录入口位于侧边栏底部，用 Google OAuth 建立最小用户会话。

## 当前已实现

### App Builder

App Builder 是主流程。当前代码支持：

- App 创建、列表、详情、更新和删除（App id 唯一，含随机后缀防撞）。
- Generate 流程和状态流转。
- **SSE 流式分阶段生成**：规划 → 逐文件（按依赖分层、层内并行、并发限流）→ 整合。
- 生成过程内联可视化：真实步骤进度、代码逐字写入 CodeMirror、文件树并发 spinner、无遮罩 Modal。
- **Preview 在前端沙箱里真实运行**生成的整套应用（vue3-sfc-loader），内部路由可跳转、外部隔离。
- 未配置 AI 时使用 mock 生成；配置后调用真实 Chat Completions 风格接口（规划用整段 JSON，
  逐文件用流式 token）。
- AI 返回内容解析支持常见 wrapper、Markdown fence、夹杂文本 JSON 和流式 delta。
- 提示词强约束：纯 CSS 样式、单屏应用自包含、只 import 存在的文件、用真实导出名。
- Workspace 支持继续对话（追加消息后流式再生成）。
- 再生成参考最近 10 条 messages 和上一版 result，**整份覆盖**为完整新版结果（非文件级 diff）。
- 生成被中断（刷新/离开）可在返回时自动重跑；离开/刷新有二次确认。
- 后端接入 `tracing` 日志（`RUST_LOG` 调级别）。

### 登录与初始化

当前代码不实现注册流程。登录使用 Google OAuth 最小闭环：

- 登录入口。
- OAuth callback。
- state 校验。
- HttpOnly session cookie。
- 当前用户查询。
- Logout。

初始化流程用于显性展示 Demo 准备状态，而不是创建账号。

### 笔记模块

当前代码提供基础 Notes CRUD：

- `title`
- `content`
- 可选 `appId`
- 创建、读取、更新、删除。
- Postgres 可用时持久化。

笔记模块当前用于帮助用户先梳理想法。多级目录、结构化块和一键带入 prompt / conversation 仍属于后续扩展。

### 服务器模块

服务器模块用于补齐 Atoms 类产品不直接覆盖的线上运维问题。

当前代码支持：

- ServerConnection 创建、读取、更新、删除。
- password / private key 两种认证类型。
- 响应不回显 password 或 privateKey。
- WebSocket 终端路由。
- 后端通过 SSH 建立会话，转发 input、resize、output、status 和 error。
- 前端提供服务器列表和 Xterm 终端页面。

当前没有完整落地 Server 与 App 的持久化绑定关系。

### AI 设置

当前代码提供 AI Provider 配置入口：

- provider：`gpt` 或 `mimo`
- apiKey
- model
- baseUrl

AI Settings 当前保存在运行时内存中，接口响应只返回脱敏 `apiKey`。

## 受控简化

当前 Demo 为了在有限时间内可交付，做了以下简化：

- 不实现注册、组织、团队、RBAC、审批流或计费系统。
- 不真实执行用户生成的代码。
- 不真实替用户部署到线上平台。
- 不访问用户本地任意文件。
- 不把 SSH 终端混入 App Preview。
- AI Settings 保存在内存中，重启后需要重新配置。
- SSH 凭据在 Postgres 模式下写入 `server_credentials` 表，用于 Demo 连接。
- 后续对话是轻量再生成，不是完整多 Agent 编排、流式输出或文件级 patch。

## 风险与边界

当前代码仍需在交付说明中明确：

- 部分读取接口按 ID 查询，owner 校验不完整，存在 Demo 级数据隔离风险。
- `GET /api/servers` 和 `GET /api/servers/:id` 当前未要求登录。
- AI Settings 当前未要求登录，且只在内存保存。
- Postgres 模式下 SSH 凭据是明文持久化，生产化必须加密、审计和轮转。
- Server 与 App 的绑定关系尚未持久化。
- Prompt Suggestions 当前主要是前端体验方向，后端接口尚未实现。

## 核心闭环

```txt
进入工作台
  -> 查看初始化与登录状态
  -> 输入原始想法
  -> 选择或采用 prompt 建议
  -> 创建 App
  -> 进入 Agent Workspace
  -> 触发生成
  -> 查看分区结果和 Preview
  -> 使用 Notes 沉淀需求上下文
  -> 需要运维时进入 Servers 并打开 SSH Terminal
```

## 继续投入方向

后续扩展优先级：

1. 补齐 Prompt Select 的 AI 化建议接口和一键采用流程。
2. 将 Notes 从普通文本扩展为多级目录、结构化字段和 App 关联上下文。
3. 将 Servers 与 App 建立明确绑定关系。
4. 为 SSH 凭据增加加密存储、权限校验、审计和过期机制。
5. 收紧所有读取接口的数据归属校验。
6. 增加在线可访问链接、部署说明和演示数据初始化脚本。
