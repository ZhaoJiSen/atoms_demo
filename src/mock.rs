use serde_json::json;

use crate::{
    models::{
        AgentMessage, AgentMessageRole, AgentStep, AgentStepStatus, ApiMethod, DataModel,
        DataModelField, FileNode, FileNodeType, GeneratedApi, GeneratedPage, GeneratedResult,
        PreviewAction, PreviewActionType, PreviewData, PreviewItem, PreviewSection,
        PreviewSectionType, PreviewTheme, ProductSpec,
    },
    time::now_iso,
};

pub(crate) fn default_steps() -> Vec<AgentStep> {
    vec![
        AgentStep {
            id: "analyze_idea".into(),
            name: "Analyze idea".into(),
            description: "理解用户输入的应用想法".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "plan_product".into(),
            name: "Plan product".into(),
            description: "生成产品目标、用户角色和核心功能".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "design_pages".into(),
            name: "Design pages".into(),
            description: "设计页面结构和组件组成".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "define_api".into(),
            name: "Define API".into(),
            description: "生成接口草案和数据模型".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "generate_files".into(),
            name: "Generate files".into(),
            description: "生成模拟文件结构".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "build_preview".into(),
            name: "Build preview".into(),
            description: "生成可交互的预览数据".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
        AgentStep {
            id: "finalize".into(),
            name: "Finalize".into(),
            description: "整合生成结果并完成部署准备".into(),
            status: AgentStepStatus::Waiting,
            started_at: None,
            completed_at: None,
        },
    ]
}

pub(crate) fn generated_messages(start_index: usize) -> Vec<AgentMessage> {
    let timestamp = now_iso();

    vec![
        AgentMessage {
            id: format!("msg_{start_index:03}"),
            role: AgentMessageRole::Agent,
            agent_name: Some("Planner Agent".into()),
            content: "我已经理解你的需求，并生成了产品计划。".into(),
            created_at: timestamp.clone(),
        },
        AgentMessage {
            id: format!("msg_{:03}", start_index + 1),
            role: AgentMessageRole::Agent,
            agent_name: Some("Designer Agent".into()),
            content: "页面结构和预览数据已经准备完成。".into(),
            created_at: timestamp.clone(),
        },
        AgentMessage {
            id: format!("msg_{:03}", start_index + 2),
            role: AgentMessageRole::Agent,
            agent_name: Some("Backend Agent".into()),
            content: "接口草案、数据模型和模拟文件结构已经生成。".into(),
            created_at: timestamp,
        },
    ]
}

pub(crate) fn should_mock_generation_fail(idea: &str) -> bool {
    idea.contains("[mock-fail]")
}

pub(crate) fn derive_title(idea: &str) -> String {
    let cleaned = idea
        .trim()
        .trim_start_matches("帮我生成")
        .trim_start_matches("帮我做")
        .trim_start_matches("创建")
        .trim_start_matches("一个")
        .trim_start_matches("一款")
        .trim();

    let candidate: String = cleaned
        .chars()
        .take_while(|character| {
            !matches!(
                character,
                '，' | ',' | '。' | '.' | '；' | ';' | '\n' | '\r'
            )
        })
        .take(24)
        .collect();

    if candidate.trim().is_empty() {
        "AI App Demo".into()
    } else {
        candidate.trim().into()
    }
}

pub(crate) fn generate_mock_result(title: &str, idea: &str) -> GeneratedResult {
    let normalized_title = if title.is_empty() {
        "AI App Demo"
    } else {
        title
    };
    let focus = idea.trim();

    GeneratedResult {
        product_spec: ProductSpec {
            name: normalized_title.into(),
            description: format!(
                "{normalized_title} 是一个围绕“{focus}”构建的可交互 Web 应用原型。"
            ),
            target_users: vec![
                "需要快速验证想法的产品负责人".into(),
                "希望管理日常工作流的业务用户".into(),
                "负责落地 Demo 的全栈开发者".into(),
            ],
            core_features: vec![
                "核心数据仪表盘".into(),
                "记录创建与状态追踪".into(),
                "列表筛选和详情查看".into(),
                "关键操作快捷入口".into(),
            ],
            user_flow: vec![
                "进入首页查看摘要".into(),
                "创建一条新的业务记录".into(),
                "在列表中跟踪进度".into(),
                "打开详情完成后续操作".into(),
            ],
        },
        pages: vec![
            GeneratedPage {
                path: "/".into(),
                name: "Dashboard".into(),
                description: format!("展示 {normalized_title} 的关键指标、近期动态和快捷操作。"),
                components: vec![
                    "SummaryStats".into(),
                    "RecentActivityList".into(),
                    "QuickActions".into(),
                ],
            },
            GeneratedPage {
                path: "/records".into(),
                name: "Records".into(),
                description: "管理核心记录，支持创建、筛选和查看详情。".into(),
                components: vec![
                    "RecordForm".into(),
                    "RecordTable".into(),
                    "StatusFilter".into(),
                ],
            },
            GeneratedPage {
                path: "/settings".into(),
                name: "Settings".into(),
                description: "配置 Demo 原型中的偏好、展示字段和通知文案。".into(),
                components: vec!["PreferenceForm".into(), "FieldSelector".into()],
            },
        ],
        apis: vec![
            GeneratedApi {
                method: ApiMethod::Get,
                path: "/api/records".into(),
                description: "获取当前应用的核心记录列表。".into(),
                request_example: None,
                response_example: Some(json!({
                    "items": [
                        { "id": "rec_001", "title": normalized_title, "status": "active" }
                    ]
                })),
            },
            GeneratedApi {
                method: ApiMethod::Post,
                path: "/api/records".into(),
                description: "创建一条新的核心业务记录。".into(),
                request_example: Some(json!({ "title": normalized_title, "notes": focus })),
                response_example: Some(json!({
                    "id": "rec_002",
                    "title": normalized_title,
                    "status": "active"
                })),
            },
            GeneratedApi {
                method: ApiMethod::Patch,
                path: "/api/records/:id".into(),
                description: "更新记录状态或补充详情。".into(),
                request_example: Some(json!({ "status": "completed" })),
                response_example: Some(json!({ "success": true })),
            },
        ],
        data_models: vec![DataModel {
            name: "Record".into(),
            description: format!("{normalized_title} 中被追踪和管理的核心业务记录。"),
            fields: vec![
                DataModelField {
                    name: "id".into(),
                    field_type: "string".into(),
                    required: true,
                    description: "记录唯一 ID".into(),
                },
                DataModelField {
                    name: "title".into(),
                    field_type: "string".into(),
                    required: true,
                    description: "记录标题".into(),
                },
                DataModelField {
                    name: "status".into(),
                    field_type: "'active' | 'completed' | 'archived'".into(),
                    required: true,
                    description: "当前处理状态".into(),
                },
                DataModelField {
                    name: "notes".into(),
                    field_type: "string".into(),
                    required: false,
                    description: "从用户想法延展出的说明信息".into(),
                },
            ],
        }],
        file_structure: vec![
            FileNode {
                path: "pages/index.vue".into(),
                node_type: FileNodeType::File,
                description: Some("Dashboard 页面".into()),
                content: Some(format!(
                    r#"<template>
  <main class="min-h-screen bg-zinc-950 text-zinc-100 p-6">
    <section class="mx-auto max-w-5xl space-y-6">
      <div>
        <h1 class="text-3xl font-semibold">{normalized_title}</h1>
        <p class="mt-2 text-zinc-400">{focus}</p>
      </div>
      <div class="grid gap-4 md:grid-cols-3">
        <article class="rounded-lg border border-zinc-800 p-4">
          <p class="text-sm text-zinc-500">Active</p>
          <p class="mt-2 text-2xl font-semibold">12</p>
        </article>
        <article class="rounded-lg border border-zinc-800 p-4">
          <p class="text-sm text-zinc-500">Done</p>
          <p class="mt-2 text-2xl font-semibold">8</p>
        </article>
        <article class="rounded-lg border border-zinc-800 p-4">
          <p class="text-sm text-zinc-500">Rate</p>
          <p class="mt-2 text-2xl font-semibold">68%</p>
        </article>
      </div>
    </section>
  </main>
</template>"#
                )),
            },
            FileNode {
                path: "pages/records.vue".into(),
                node_type: FileNodeType::File,
                description: Some("记录管理页面".into()),
                content: Some(
                    r#"<template>
  <main class="space-y-4 p-6">
    <h1 class="text-2xl font-semibold">Records</h1>
    <RecordTable />
  </main>
</template>"#
                        .into(),
                ),
            },
            FileNode {
                path: "components/SummaryStats.vue".into(),
                node_type: FileNodeType::File,
                description: Some("关键指标组件".into()),
                content: Some(
                    r#"<template>
  <section class="grid gap-3 md:grid-cols-3">
    <div class="rounded border p-4">Summary</div>
  </section>
</template>"#
                        .into(),
                ),
            },
            FileNode {
                path: "components/RecordTable.vue".into(),
                node_type: FileNodeType::File,
                description: Some("记录列表组件".into()),
                content: Some(
                    r#"<template>
  <div class="rounded border p-4">
    <p>Record table</p>
  </div>
</template>"#
                        .into(),
                ),
            },
            FileNode {
                path: "server/api/records".into(),
                node_type: FileNodeType::Directory,
                description: Some("模拟记录 API 目录".into()),
                content: None,
            },
            FileNode {
                path: "types/record.ts".into(),
                node_type: FileNodeType::File,
                description: Some("Record 类型定义".into()),
                content: Some(
                    r#"export interface Record {
  id: string
  title: string
  status: 'active' | 'completed' | 'archived'
  notes?: string
}"#
                    .into(),
                ),
            },
        ],
        preview: PreviewData {
            title: normalized_title.into(),
            description: format!("围绕“{focus}”生成的可交互 Mock Preview。"),
            theme: PreviewTheme::Dark,
            sections: vec![
                PreviewSection {
                    id: "hero".into(),
                    title: "应用概览".into(),
                    section_type: PreviewSectionType::Hero,
                    content: format!("{normalized_title} 帮助用户把想法转化为可跟踪的工作流。"),
                    items: None,
                },
                PreviewSection {
                    id: "stats".into(),
                    title: "今日摘要".into(),
                    section_type: PreviewSectionType::Stats,
                    content: "当前 Demo 已准备好关键指标和最近动态。".into(),
                    items: Some(vec![
                        PreviewItem {
                            title: "活跃记录".into(),
                            description: Some("正在跟进的核心事项".into()),
                            value: Some("12".into()),
                        },
                        PreviewItem {
                            title: "完成率".into(),
                            description: Some("本周已完成记录占比".into()),
                            value: Some("68%".into()),
                        },
                    ]),
                },
                PreviewSection {
                    id: "activity".into(),
                    title: "近期动态".into(),
                    section_type: PreviewSectionType::List,
                    content: "展示最近创建、更新和完成的记录。".into(),
                    items: Some(vec![
                        PreviewItem {
                            title: "创建首个记录".into(),
                            description: Some(focus.chars().take(80).collect()),
                            value: Some("New".into()),
                        },
                        PreviewItem {
                            title: "补充详情".into(),
                            description: Some("为原型准备展示数据和状态标签。".into()),
                            value: Some("Draft".into()),
                        },
                    ]),
                },
            ],
            actions: vec![
                PreviewAction {
                    label: "新增记录".into(),
                    action_type: PreviewActionType::Primary,
                },
                PreviewAction {
                    label: "查看全部".into(),
                    action_type: PreviewActionType::Secondary,
                },
            ],
        },
    }
}
