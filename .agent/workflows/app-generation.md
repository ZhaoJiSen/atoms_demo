# App Generation Workflow

本文档约束核心业务流：从 prompt 到 App，再到 Workspace 和 Preview。

## 业务目标

完成一个可运行的 AI App Builder 闭环：

```txt
Google OAuth 登录
  -> 输入 idea
  -> 选择或采用 Prompt Select 建议
  -> 创建 App
  -> 进入 Workspace
  -> 触发生成
  -> 展示生成过程和结果
  -> 进入 Preview
```

生成过程可以使用 AI 设置，也可以使用 mock。无论哪种方式，结果必须和
用户输入有明显关系。

## 当前实现

当前代码已经支持：

- `POST /api/apps` 创建 `pending` App（id 唯一，含随机后缀）。
- `POST /api/apps/:id/messages` 追加用户消息。
- `GET /api/apps/:id/generate/stream` —— **SSE 流式分阶段生成（前端默认路径）**。
- `POST /api/apps/:id/generate` —— 同步全量生成（保留，主要给测试）。
- 进入 `/app/:id` 时若状态为 `pending` 或 `generating`（被中断）会自动触发生成。
- 生成过程内联展示：真实步骤进度、代码逐字写入 CodeMirror、文件树并发 spinner、无遮罩 Modal。
- Workspace 支持继续对话并重新生成（全量重生成，规划层参考旧 result + 最近 10 条消息）。
- AI Settings 存在时调用真实 provider；不存在时使用 mock。
- 生成完成写入 `GeneratedResult`，失败进入 `failed`。
- Workspace / Preview 使用同一份 App result 数据；Preview 真正运行该应用。

### 流式分阶段生成（SSE）

```txt
规划(Planner)    1 次 LLM：productSpec/pages/apis/dataModels/preview + 文件清单(无代码)
逐文件(Engineer) 按依赖分层(基础 .ts → 组件 → 页面/App → 入口 main)，
                 层内并行(并发上限 4)、层间串行；上层拿到下层真实代码做上下文 → 跨文件契约对齐
整合(Finalize)   组装完整 GeneratedResult 并持久化
```

SSE 事件：`step` / `partial` / `manifest` / `file_start` / `file_chunk` / `file_end` / `done` / `fail`
（详见 `api-contract.md`）。

当前代码尚未支持：

- 后端 Prompt Suggestions 接口。
- 将 Notes 内容自动拼入 AI prompt。
- 多 App 并发队列或后台任务持久化。
- 文件级 patch / diff apply（继续对话仍是整份覆盖）。
- 真实代码执行、构建或部署（但 Preview 会在前端沙箱里真实运行生成的应用）。

## 创建 App

入口可以是 `/` 或 `/app/new`。

流程：

1. 用户输入原始想法。
2. 系统展示贴近输入的 prompt 建议。
3. 用户选择建议或保留原始输入。
4. 前端校验 session 和非空 idea。
5. 调用 `POST /api/apps`。
6. 后端创建 `pending` App。
7. 前端进入 `/app/:id`，自动触发流式生成。

## 状态流转

App 状态：

```txt
pending -> generating -> completed
pending -> generating -> failed
failed -> generating -> completed
failed -> generating -> failed
completed -> generating -> completed
completed -> generating -> failed
```

禁止：

- `generating` 时重复触发生成。
- 无错误原因进入 `failed`。
- `completed` 后无说明回到 `pending`。

## Workspace

Workspace 必须展示：

- App title。
- idea。
- status。
- Agent steps。
- messages。
- Product Spec。
- Pages。
- API。
- Data Models。
- File Structure。
- Preview 入口。

Notes 如果已关联，应能作为上下文来源展示。当前 `Note.appId` 只支持基础
关联，尚未形成完整上下文注入链路。

## 后续对话

当前支持轻量后续对话：

```txt
用户输入追加需求
  -> POST /api/apps/:id/messages
  -> EventSource GET /api/apps/:id/generate/stream
  -> 后端规划阶段带上最近 10 条 messages + 完整旧 result 作为修订上下文
  -> 逐文件全量重写、整份覆盖
  -> 前端流式刷新 Workspace / Preview / File Structure
```

该能力用于展示 Atoms-like 的持续迭代体验。它不是完整 Agent 编排，也不执行
文件级 patch：每次再生成都**整份覆盖**旧 result。规划层“参考并尽量保留”旧结果，
但生成层是全量重写（逐文件阶段看不到旧版同名文件的代码）。

> 已知小问题（暂未修）：`build_generation_context` 在 `result` 清空后才判断
> `result.is_some()`，因此对话上下文总是写“首次生成”；修订信号实际通过规划提示词里的
> 旧 result JSON 传入，不影响功能但提示文案不准。

## AI Prompt Contract

真实 AI 生成必须遵守后端模型字段：

- `productSpec`
- `pages`
- `apis`
- `dataModels`
- `fileStructure`
- `preview`

枚举值必须使用：

- API method：`GET`、`POST`、`PATCH`、`DELETE`
- file type：`file`、`directory`
- preview theme：`dark`、`light`
- preview section type：`hero`、`stats`、`list`、`form`、`cards`、`table`
- preview action type：`primary`、`secondary`

生成代码必须能在前端沙箱里真实运行（vue3-sfc-loader），因此提示词强约束：

- 样式用组件自带 `<style>` 里的**纯 CSS**（沙箱无 Tailwind / UI 库）。
- 单屏应用（计算器、游戏）尽量做成**一个自包含组件**，核心状态/逻辑放进自己的
  `<script setup>`，避免跨文件共享可变状态。
- 只 import **AVAILABLE FILES 清单**里的文件；缺的就在本文件内联，**绝不 import 不存在的路径**。
- 导入已生成文件时用其**真实导出名/结构**（逐文件分层生成保证下层代码已作为上下文给到上层）。
- 规划阶段产出的 `fileStructure` 必须**import-complete**（凡要 import 的都得作为节点列出）。

沙箱侧的兜底：缺失模块用占位桩、`.js` 按 ES module 解析、localStorage 等用内存垫片，
单个坏文件不会让整个预览失败。

## 生成结果

`GeneratedResult` 至少包含：

- `productSpec`
- `pages`
- `apis`
- `dataModels`
- `fileStructure`
- `preview`

最低内容数量：

- `pages` 至少 2 个。
- `apis` 至少 2 个。
- `dataModels` 至少 1 个。
- `fileStructure` 至少 5 个节点。
- `preview.sections` 至少 2 个。
- `preview.actions` 至少 1 个 primary action。

## Preview

Preview 是 Workspace 内的一个 Tab，不触发生成流程。它**在沙箱 iframe 里真实运行**
生成的整套应用（vue3-sfc-loader 编译挂载），而不是固定 mock 布局。

特性：

- 运行整套 `fileStructure`，从入口（`main.*` → `App.vue` → `pages/index.vue`）挂载。
- 应用内部路由可跳转（vue-router 走 memory history，不改外层 URL、不影响 Atoms）。
- 文件流式到达时逐步构建；编辑代码触发重跑。
- 缺失/坏文件用占位桩兜底，不让整个预览失败。
- 运行时自托管在 `web/public/preview-runtime/`，不依赖外部 CDN。

Preview 不承载 SSH Terminal。沙箱不放 `allow-same-origin`/`allow-top-navigation`，
生成代码无法访问外层页面或 cookie。

## 异常场景

必须处理：

- idea 为空。
- 未登录创建或生成。
- App 不存在。
- API 请求失败。
- 生成中重复点击。
- AI 响应无法解析。
- 生成失败。
- completed 但 result 缺失。
- Preview 数据缺失。

## 验收标准

- 从首页到 Workspace 的创建链路可用。
- Prompt Select 能改善用户输入。
- 未登录写入有明确错误反馈。
- Generate 后状态最终进入 `completed` 或 `failed`。
- 追加对话后可以触发再生成。
- 再生成结果能体现最新用户修改需求。
- completed 状态下结果字段完整。
- Preview 是可交互原型。
- 错误状态有返回或重试入口。
