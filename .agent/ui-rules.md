# UI 视觉风格

本文档约束 Atoms Demo 的界面风格、布局和交互要求。

## 设计基调

UI 是工具型 SaaS / Developer Tool 工作台，不是营销官网。

风格要求：

- 暗色优先。
- 信息密度适中。
- 紫色只作为主操作和少量强调色。
- 布局克制、清晰、可扫描。
- 不使用大面积渐变、装饰图形或无意义动画。
- 优先使用 Nuxt UI、Tailwind CSS 和 lucide 图标。

## 当前主界面

首页应呈现为工作台：

```txt
Left Sidebar
  Logo / Product Name
  Create New App
  App List or Empty State
  Login / User State

Top Bar
  Notes
  Servers
  Language
  AI Settings

Main
  H1: 从想法到应用，只需一次对话
  Prompt Textarea
  Create App Button
  Select-style Prompt Suggestions
```

这个页面是用户的第一屏核心体验，不应退回到介绍型 Landing。

## Prompt Select

Prompt Select 是重要 UX 亮点。

要求：

- 用户输入后展示贴近输入的建议。
- 建议以可点击 Select / List 形式呈现。
- 每条建议要能直接填入输入框或作为创建 App 的 prompt。
- 默认建议可以存在，但用户输入后必须优先根据输入生成相关建议。
- 建议文案要具体，不要只写“创建一个应用”。

## Notes UI

Notes 页面用于帮助用户梳理需求和沉淀上下文。

当前基础形态：

- 左侧笔记列表。
- 主区标题和内容编辑器。
- 创建、保存、删除操作。

目标扩展形态：

- 多级目录或文档树。
- 结构化笔记模板，例如目标、用户、功能、数据、约束、验收。
- 一键选择笔记进入 App 创建或 conversation。
- 学习模块，指导用户编写高质量 `.agent` 风格上下文。

## Servers UI

Servers 是运维延伸能力，必须和 App Builder 形成清晰区分。

要求：

- 显示服务器连接列表、空状态、错误状态。
- 表单必须包含 name、host、port、username、authType。
- 敏感凭据输入后不得在详情中明文展示。
- 服务器应能展示与 App 的关联关系。
- Terminal 页面以 Xterm 为主体，不放在小卡片里伪装。
- Connect 必须由用户显式点击触发。

## Workspace UI

Workspace 必须展示：

- App title。
- 用户 idea。
- 当前状态。
- Agent steps。
- Agent messages。
- 生成结果分区。
- Preview 入口。
- Retry 或失败说明。

生成结果必须分区展示，不允许堆成一段文本。

推荐分区：

- Product Spec
- Pages
- API
- Data Models
- File Structure
- Preview

## Preview UI

Preview 是可交互原型，不是静态截图或纯文本说明。

必须包含：

- 应用标题和描述。
- 至少两个内容区块。
- Mock 列表、卡片、表格、表单或统计之一。
- 至少一个可点击操作。
- 返回 Workspace 入口。

## 状态反馈

所有核心交互必须有状态反馈：

- loading
- saving
- generating
- connected
- disconnected
- empty
- error
- unauthorized
- retry

## 禁止事项

- 把首页做成纯营销页。
- 用静态内容假装真实交互。
- 把 SSH Terminal 放进 App Preview。
- 明文展示 SSH 密码、私钥、OAuth token 或 session token。
- 所有逻辑都堆在页面文件里且无法维护。
- 忽略窄屏布局导致文本或控件重叠。
