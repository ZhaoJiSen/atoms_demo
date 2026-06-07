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

- `POST /api/apps` 创建 `pending` App。
- `POST /api/apps/:id/messages` 追加用户消息。
- `POST /api/apps/:id/generate` 触发生成。
- `generating` 状态禁止重复生成。
- Workspace 支持用户追加后续需求并重新生成。
- 再生成会把最近对话和上一版 result 传给 AI。
- AI Settings 存在时调用真实 provider。
- AI Settings 不存在时使用 mock。
- 生成完成后写入 `GeneratedResult`。
- 生成失败时进入 `failed`。
- Workspace / Preview 使用同一份 App result 数据。

当前代码尚未支持：

- 后端 Prompt Suggestions 接口。
- 将 Notes 内容自动拼入 AI prompt。
- 多 App 并发队列或后台任务持久化。
- 流式 token 输出。
- 文件级 patch / diff apply。
- 真实代码执行、构建或部署。

## 创建 App

入口可以是 `/` 或 `/app/new`。

流程：

1. 用户输入原始想法。
2. 系统展示贴近输入的 prompt 建议。
3. 用户选择建议或保留原始输入。
4. 前端校验 session 和非空 idea。
5. 调用 `POST /api/apps`。
6. 后端创建 `pending` App。
7. 前端进入 `/app/:id/workspace`。

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
  -> POST /api/apps/:id/generate
  -> 后端带上历史 messages 和旧 result
  -> AI 返回完整新版 GeneratedResult
  -> 前端刷新 Workspace / Preview / File Structure
```

该能力用于展示 Atoms-like 的持续迭代体验。它不是完整 Agent 编排，也不
执行文件级 patch；每次再生成都以完整结果覆盖旧 result。

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

文件节点建议携带 `content`，尤其是 Vue 文件，方便前端文件结构和预览模块
展示真实代码。

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

Preview 页面只展示生成应用原型，不触发生成流程。

必须包含：

- 应用标题。
- 应用描述。
- Mock 数据。
- 基础交互。
- 返回 Workspace 入口。

Preview 不承载 SSH Terminal，不执行用户生成代码的危险操作。

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
