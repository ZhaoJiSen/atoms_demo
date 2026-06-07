# Implementer Role

Implementer 负责根据 Planner 的计划完成代码实现，并保持 diff 小、行为清晰、可验证。

## 1. 实现前检查

Implementer 开始修改前必须确认：

- Planner 已说明需求类型和涉及文档
- API 字段与 `.agent/api-contract.md` 一致
- 路由与 `.agent/routes.md` 一致
- UI 实现与 `.agent/ui-rules.md` 和 `DESIGN.md` 一致
- 权限边界与 `.agent/permission.md` 一致

如果发现计划与文档冲突，必须先记录冲突并采用文档优先的最小实现。

## 2. 实现原则

必须遵守：

- 保持 Demo 范围，不实现真实云容器、真实部署、真实 LLM 或真实代码执行
- 前端核心数据通过 API 获取
- 后端返回稳定 JSON 结构
- 错误响应统一为 `{ "error": "..." }`
- 状态覆盖 `pending`、`generating`、`completed`、`failed`
- Step 状态覆盖 `waiting`、`running`、`done`、`error`
- UI 必须覆盖 loading、empty、error、success 或 completed 状态
- 不引入新依赖，除非用户明确要求

## 3. 前端实现要求

前端代码位于 `web/`。

建议结构：

```txt
web/pages/
web/components/
web/composables/
web/types/
```

实现时应：

- 使用 Nuxt 文件路由
- 优先使用 Nuxt UI 组件
- 将 API 请求封装到 composable
- 将 TypeScript 类型集中到 `types` 或 composable 附近
- 避免所有逻辑堆在页面文件
- 为核心操作提供 loading 和 error 状态

## 4. 后端实现要求

后端入口为 `src/main.rs`。

实现时应：

- 使用 Axum 提供 `/api` 路由
- 使用 Serde 输出 camelCase JSON 字段
- 使用内存 HashMap 或本地 JSON 保存 Demo 数据
- 将生成逻辑保持为可读的 mock 规则
- 不返回 panic、堆栈或内部路径
- 对不存在的 App 返回 `404`
- 对重复 generate 返回 `409`

## 5. 代码质量要求

Implementer 应保持：

- 小文件优先，必要时拆模块
- 命名清晰，避免过度抽象
- 数据模型前后端一致
- 删除无用代码
- 不修改无关文件
- 不重构根目录结构，除非 Planner 已说明原因

## 6. 交付给 Reviewer

实现完成后，Implementer 必须说明：

- 修改了哪些文件
- 实现了哪些页面、接口或数据结构
- 哪些状态和异常已覆盖
- 运行了哪些验证命令
- 哪些内容未验证或存在风险
