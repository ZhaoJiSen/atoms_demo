# Prompt Refinement Workflow

本文档约束 Prompt Select 能力：用户输入模糊想法后，系统生成贴近输入的
可选 prompt，帮助用户更快创建可执行的 App 需求。

## 业务目标

用户不一定知道自己想要什么。Prompt 模块要把模糊输入转成多个更清晰的
候选方向，让用户通过选择而不是重写来推进流程。

目标路径：

```txt
输入原始想法
  -> 生成贴近输入的 prompt 建议
  -> 用户选择其中一条
  -> 填充输入框或直接创建 App
  -> 进入 App Workspace
```

## 当前实现状态

当前产品界面可以展示 Select-style 建议，App 创建会使用用户最终提交的
idea。

当前后端尚未实现：

- `POST /api/prompts/suggestions`
- AI 生成 prompt 建议。
- Notes 内容参与 prompt 建议。

因此当前建议如果存在，应被视为前端规则化或静态过渡能力。交付时不能
把它描述成已由后端大模型生成。

## 建议质量

Prompt 建议必须：

- 贴近用户输入。
- 保留用户的行业、对象、功能关键词。
- 明确目标用户、核心功能或约束。
- 能直接用于 `POST /api/apps`。
- 避免泛化成固定模板。

示例：

```txt
用户输入：想做一个学习平台

建议：
1. 构建一个在线学习平台，包含课程管理、学习进度追踪和作业提交。
2. 构建一个面向自学用户的学习计划工具，支持目标拆解和每日复盘。
3. 构建一个课程运营后台，支持课程发布、学员管理和数据看板。
```

## 数据来源

Prompt 建议可以来自：

- 用户当前输入。
- 用户选择的 Notes。
- 当前 App conversation。
- 默认示例 prompt。
- AI Settings 配置后的真实 AI 调用。

未配置 AI 时，允许使用规则化 mock，但必须和输入文本相关。

## UI 要求

首页主输入区下方展示 Select-style 建议：

- 至少 3 条。
- 可点击。
- 选中态清晰。
- 支持重新生成或刷新建议。
- 选中后能填充输入框或直接创建 App。

## API 建议

建议新增：

```txt
POST /api/prompts/suggestions
```

请求：

```ts
interface PromptSuggestionRequest {
  input: string
  noteIds?: string[]
  appId?: string
}
```

响应：

```ts
interface PromptSuggestion {
  id: string
  label: string
  prompt: string
  reason: string
}
```

当前代码未实现该接口时，前端不得调用不存在的路由；可以先用页面内的
静态示例或规则化建议作为过渡，并在交付说明中声明差距。

## 验收标准

当前基础验收：

- 用户能看到建议项。
- 选择建议后输入框内容会更新。
- 使用建议创建 App 后，App idea 与所选 prompt 一致。
- 空输入时有默认示例或明确提示。

扩展验收：

- 用户输入后能看到相关建议。
- 建议不是完全固定模板。
- 建议可以来自后端 AI 调用。
- 可选 Notes 能影响建议内容。
