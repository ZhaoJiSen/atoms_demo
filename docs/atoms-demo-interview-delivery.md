# Atoms Demo 面试说明文档

## 实现思路与关键取舍

本 Demo 的目标不是完整复刻 Atoms，而是在有限时间内做出一个可运行、
可交互、可解释的 AI App Builder 工作台。

整体思路是围绕“从想法到应用，只需一次对话”构建主闭环：

```txt
用户输入想法
  -> 选择更清晰的 prompt
  -> 创建 App
  -> Agent 生成应用方案
  -> 查看 Workspace 和 Preview
  -> 用 Notes 梳理上下文
  -> 用 Servers 接入运维场景
```

关键取舍：

- 后端选择 Rust + Axum，优先保证 API 结构、错误处理、WebSocket 和会话
  边界清晰。
- 前端选择 Nuxt，优先做真实工作台，而不是营销式静态页面。
- 登录不做注册流程，只做 Google OAuth 最小闭环，降低账号体系复杂度。
- 生成能力允许 mock 和真实 AI 设置并存，优先保证 Demo 可运行。
- 运维能力不做完整 JumpServer，只做 Servers + SSH / Xterm 的延伸能力。
- Notes 先做基础 CRUD，再把结构化、多级目录和学习模块作为明确扩展方向。

这个取舍对应面试方的评估维度：

- 完成度：优先完成主流程、登录、持久化接口和可交互页面。
- 工程思维：把 App、Notes、Servers、Auth、AI Settings 拆成独立模块。
- 用户体验：首页就是工作台，用户可以直接输入、选择 prompt、创建应用。
- 创新性：用 Notes 解决“用户不知道怎么描述需求”的问题，用 Servers
  补齐 Atoms 不覆盖的线上运维入口。
- 可交付性：用 `.agent` 文档约束范围、API、路由、权限、UI 和验收标准。

## 当前完成程度

### 已完成或已具备基础能力

- 主工作台布局：左侧 App 列表、中心 prompt 输入、顶部 Notes / Servers
  等导航。
- App 模块：创建、列表、更新、删除、追加消息、触发生成、Workspace、
  Preview。
- Agent 结果结构：Product Spec、Pages、API、Data Models、File Structure、
  Preview。
- Google OAuth：登录入口、callback、logout、`/api/auth/me` 当前会话查询。
- 初始化：`/api/init` 和初始化页面。
- AI 设置：支持 provider、apiKey、model、baseUrl，并对返回值做密钥脱敏。
- Notes 基础模块：前端笔记页面和后端 Notes CRUD 接口。
- Servers 基础模块：服务器 CRUD、Terminal 页面和 WebSocket terminal 路由。
- Postgres 接入：已有 `DATABASE_URL` 连接、用户、session、apps、servers
  等表的 migration 和 JSONB 持久化策略。
- `.agent` 文档：已更新产品、架构、路由、API、权限、UI、验收和业务流。

### 尚未完全完成或需要继续补齐

- Prompt Select 目前仍需补齐真正的后端 suggestions 接口，当前文档建议为
  `POST /api/prompts/suggestions`。
- Notes 当前是基础文本笔记，还没有多级目录、结构化 blocks、学习模块和
  一键带入 prompt / conversation 的完整闭环。
- `src/db.rs` 已有 Notes 的查询和写入函数，但 migration 中尚未创建
  `notes` 表；启用 Postgres 后 Notes 可能遇到缺表问题。
- Server 与 App 的绑定在产品上已经明确，但当前模型中还需要固化 `appId`
  或关联表，避免只靠前端临时关联。
- AI Settings 如果只保存在运行时内存中，重启后会丢失；生产化需要加密
  存储和权限保护。
- SSH 能力还不是完整堡垒机方案，缺少审计、授权、命令控制、凭据加密和
  会话回放。
- Demo 还需要补充可测试的在线访问地址和最终 GitHub 仓库链接。

## 如果继续投入时间，我会如何扩展

### P0：交付稳定性

- 补齐 `notes` 表 migration，保证 Notes 在 Postgres 模式下真实持久化。
- 校验 Apps、Notes、Servers 的 owner 查询，避免不同用户通过 ID 串数据。
- 为 AI Settings 加 session 校验和持久化策略，真实密钥必须加密或只在
  安全运行时保存。
- 补充一键启动和演示数据初始化说明，确保面试方可以稳定复现。

### P1：Prompt 与 Notes 闭环

- 实现 `POST /api/prompts/suggestions`。
- 根据用户输入和选中的 Notes 生成 3 到 5 条 Select prompt。
- 支持把 Notes 一键插入创建 App 的 prompt 或 Workspace conversation。
- 给 Notes 增加结构化模板：目标、用户、功能、数据、约束、验收。

### P2：服务器运维能力

- 在 ServerConnection 中增加 `appId`，或新增 App-Server 关联表。
- Terminal 页面增加常用运维入口，例如 `systemctl status`、日志路径提示、
  部署文件位置记录。
- 增加连接失败诊断：认证失败、网络不可达、host key、权限不足。
- 将 SSH 凭据迁移到加密存储，并增加最小审计日志。

### P3：产品化与创新增强

- 增加“AI 高可用上下文学习模块”，教用户如何写 `.agent` 风格的需求文档。
- 为生成结果增加导出能力，例如导出 PRD、API 草案或文件结构。
- 增加 App 的版本历史，让用户比较每次 prompt 迭代后的变化。
- 增加在线部署链接和只读演示账号，提升可交付性。

## 交付说明

当前交付重点是展示真实可用的 Demo 方向和工程拆解能力。代码层面仍有
少量需要补齐的稳定性工作，尤其是 Notes migration、Server-App 绑定和
Prompt Select 后端接口。

如果作为面试提交，建议在最终文档的“结果回收处”附上：

- Demo 可访问地址。
- GitHub 源码地址。
- 本说明文档。
- 已知风险和后续优先级。
