# Agent 工作索引

本文档是 Codex 执行需求时的总入口，用于说明文档阅读顺序、任务路由规则和角色协作流程。

Codex 在开始实现前，必须先根据当前任务类型读取对应文档。不得在未读取相关上下文文档的情况下直接修改代码。

在执行任何任务前，Codex 必须先阅读：

- `product.md`

`product.md` 用于说明本 Demo 的产品定位、业务边界、核心闭环和成功标准。

## 1. 文档阅读规则

根据任务类型，按以下规则读取文档：

- 做页面结构前，先读 `architecture.md` 和 `routes.md`
- 做接口联调前，先读 `api-contract.md`
- 做权限相关需求前，先读 `permission.md`
- 做 UI 或交互前，先读 `ui-rules.md`
- 做验收前，先读 `validation.md`
- 做具体业务流时，再进入 `workflows/` 下对应文档
- 做 Prompt 建议、Select prompt 或创建 App 输入体验前，先读
  `workflows/prompt-refinement.md`
- 做 Notes、需求梳理、结构化上下文或学习模块前，先读
  `workflows/notes-structuring.md`
- 做 SSH、终端、服务器连接或 App 运维扩展前，先读
  `workflows/ssh-terminal.md`

如果一个需求同时涉及多个类型，需要同时读取对应文档。

例如：

- 页面 + 接口：读取 `architecture.md`、`routes.md`、`api-contract.md`
- 页面 + 权限 + UI：读取 `architecture.md`、`routes.md`、`permission.md`、`ui-rules.md`
- 业务流 + 验收：读取 `workflows/` 下对应文档和 `validation.md`
- Prompt Select：读取 `api-contract.md`、`ui-rules.md`、
  `workflows/prompt-refinement.md`
- Notes：读取 `api-contract.md`、`permission.md`、`ui-rules.md`、
  `workflows/notes-structuring.md`
- SSH 终端：读取 `api-contract.md`、`permission.md`、`ui-rules.md`、
  `workflows/ssh-terminal.md`

## 2. Agent Roles

Codex 执行本需求时，默认拆分为三个角色：

- Planner：负责理解需求、读取相关上下文、拆解任务并输出实现计划
- Implementer：负责根据 Planner 的计划进行代码实现
- Reviewer：负责根据 validation.md 和相关上下文文档进行验收

默认执行流程：

```text
Planner -> Implementer -> Reviewer
```

角色详细规范见：

- roles/planner.md
- roles/implementer.md
- roles/reviewer.md

## 3. 执行要求

Codex 在执行需求时，必须明确说明：

- 当前需求属于哪类任务
- 已读取哪些上下文文档
- 本次计划修改哪些页面、组件、接口、权限或业务流程
- 实际修改了哪些内容
- 验收依据是什么
- 是否存在风险、边界情况或未确认问题

如果任务较小，可以合并角色输出，但不能完全跳过角色流程。

合并输出时，仍然需要包含：

```text
Planner + Implementer + Reviewer
```

并说明：

- 需求理解
- 涉及文档
- 实现内容
- 验收结果
- 风险项

## 4. 禁止事项

Codex 执行需求时，禁止以下行为：

- 未读取相关文档就直接修改代码
- 擅自改变接口字段、路由结构、权限规则或 UI 规范
- 只实现功能，不补充异常状态、空状态、加载状态和权限边界
- 只修改代码，不说明修改依据和验收方式
- 跳过 Reviewer 验收阶段
- 与现有文档冲突时直接猜测实现

如果发现需求与现有文档冲突，必须先说明冲突点，并给出建议处理方式。

## 5. 业务流入口

当需求涉及具体业务流程时，需要进入 workflows/ 目录查找对应业务文档。如果找不到对应业务流文档，Codex 需要在 Planner 阶段说明：

- 当前缺少哪个业务流文档
- 根据现有代码和上下文推断出的业务流程
- 哪些部分需要用户或维护者确认

当前核心业务流文档：

- `workflows/app-generation.md`：创建 App、触发 Agent 生成、查看 Workspace 和 Preview 的完整闭环
- `workflows/prompt-refinement.md`：把用户模糊输入转成 Select 形式 prompt 建议
- `workflows/notes-structuring.md`：笔记梳理、结构化上下文和学习模块
- `workflows/ssh-terminal.md`：服务器连接管理、SSH 会话和 Xterm 交互终端
