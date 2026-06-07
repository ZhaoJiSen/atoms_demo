# Notes Structuring Workflow

本文档约束 Notes 模块：帮助用户先梳理需求，再把笔记内容作为 AI 上下文
带入 App Builder。

## 业务目标

Notes 的价值不是普通文本编辑，而是让用户把想法变成可执行上下文。

目标路径：

```txt
进入 Notes
  -> 创建需求笔记
  -> 按结构梳理目标、用户、功能、数据和约束
  -> 选择笔记进入 prompt 或 conversation
  -> 创建或继续 App
```

## 当前基础形态

当前代码中的 `Note` 模型包含：

- `id`
- `title`
- `content`
- `appId`
- `createdAt`
- `updatedAt`

当前实现是基础 CRUD 和编辑体验：

- `GET /api/notes`
- `POST /api/notes`
- `GET /api/notes/:id`
- `PATCH /api/notes/:id`
- `DELETE /api/notes/:id`

Postgres 可用时，Notes 会持久化到 `notes` 表。

## 当前未完成

当前代码尚未实现：

- 多级目录。
- 结构化 blocks。
- 一篇笔记关联多个 App。
- 一键选择到 prompt。
- 一键插入 conversation。
- 学习模块页面。
- Notes 内容自动注入 AI 生成 prompt。

这些是后续扩展，不能在交付时描述成已完成。

## 目标结构化形态

后续应扩展为多级目录和结构化笔记：

- `folderId` 或 `parentId`：支持目录树。
- `kind`：区分普通笔记、需求笔记、学习内容。
- `blocks`：支持结构化块，例如目标、用户、功能、数据、约束、验收。
- `linkedAppIds`：支持一篇笔记关联多个 App。

结构化需求模板建议：

```txt
目标
目标用户
核心功能
页面结构
数据模型
接口需求
约束条件
验收标准
```

## 学习模块

Notes 可以包含学习模块，指导用户写出让 AI 高效执行的上下文。

学习模块重点：

- 如何描述目标和边界。
- 如何拆分功能和验收标准。
- 如何写类似 `.agent` 的项目规则。
- 如何避免只给一句模糊需求。
- 如何在 Notes 中沉淀可复用上下文。

## 与 App Builder 的关系

目标上，Notes 必须能进入 App Builder：

- 在创建 App 时选择一篇或多篇笔记作为上下文。
- 在 conversation 中插入笔记内容。
- 在 App 详情中展示关联笔记。

当前若只实现基础编辑器，交付说明必须写清：

- 已完成 Notes CRUD。
- 未完成多级目录。
- 未完成一键选择到 prompt。
- 未完成学习模块。

## 验收标准

基础验收：

- 可以创建笔记。
- 可以保存标题和内容。
- 可以删除笔记。
- 刷新后数据仍然存在。
- 未登录时写入被拦截。

扩展验收：

- 支持多级目录。
- 支持结构化模板。
- 支持一键带入 prompt 或 conversation。
- 学习模块能解释如何编写高质量 AI 上下文。
