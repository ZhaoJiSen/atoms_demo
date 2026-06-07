pub(super) fn system_prompt() -> &'static str {
    r#"You are an AI app builder data generator.
Return valid JSON only. Do not wrap the JSON in markdown. Do not add explanations.
The JSON must match the requested schema exactly.
The Rust backend deserializes the response by exact field names, so missing or renamed fields will fail.
The top-level required fields are:
productSpec, pages, apis, dataModels, fileStructure, preview.
Use camelCase keys.
Use only these enum values:
- API method: GET, POST, PATCH, DELETE
- file type: file, directory
- preview theme: dark, light
- preview section type: hero, stats, list, form, cards, table
- preview action type: primary, secondary
Every generated file node with type "file" should include a realistic "content" string.
Vue files should be small Vue single-file components that can be displayed in a demo.
"#
}

pub(super) fn build_app_generation_prompt(
    idea: &str,
    title: &str,
    conversation_context: &str,
    previous_result_json: Option<&str>,
) -> String {
    let previous_result = previous_result_json
        .map(|value| format!("\nExisting generated result to revise:\n{value}\n"))
        .unwrap_or_default();

    format!(
        r#"Generate a complete app prototype data payload for this user idea.

Title: {title}
Idea: {idea}
Conversation and follow-up requirements:
{conversation_context}
{previous_result}

Return one JSON object with this exact TypeScript-compatible shape:

{{
  "productSpec": {{
    "name": "App name",
    "description": "Short product description",
    "targetUsers": ["target user 1", "target user 2"],
    "coreFeatures": ["feature 1", "feature 2", "feature 3"],
    "userFlow": ["step 1", "step 2", "step 3"]
  }},
  "pages": [
    {{
      "path": "/",
      "name": "Dashboard",
      "description": "What this page is for",
      "components": ["MetricCard", "ItemList"]
    }}
  ],
  "apis": [
    {{
      "method": "GET",
      "path": "/api/items",
      "description": "List items",
      "requestExample": null,
      "responseExample": {{"items": []}}
    }}
  ],
  "dataModels": [
    {{
      "name": "Item",
      "description": "Core record",
      "fields": [
        {{
          "name": "id",
          "type": "string",
          "required": true,
          "description": "Unique ID"
        }}
      ]
    }}
  ],
  "fileStructure": [
    {{
      "path": "src/pages/index.vue",
      "type": "file",
      "description": "Main page",
      "content": "<template>...</template>"
    }},
    {{
      "path": "src/components",
      "type": "directory",
      "description": "Shared components"
    }}
  ],
  "preview": {{
    "title": "Preview title",
    "description": "Preview description",
    "theme": "dark",
    "sections": [
      {{
        "id": "overview",
        "title": "Overview",
        "type": "stats",
        "content": "Summary",
        "items": [
          {{
            "title": "Active",
            "description": "Active records",
            "value": "24"
          }}
        ]
      }}
    ],
    "actions": [
      {{
        "label": "Create",
        "type": "primary"
      }}
    ]
  }}
}}

Content rules:
- Make every field relevant to the user's idea.
- If follow-up requirements are present, revise the existing app instead of ignoring history.
- Preserve useful parts of the previous result unless the latest user message asks to change them.
- Generate 3 to 5 pages.
- Generate 4 to 6 APIs.
- Generate 2 to 4 data models.
- Generate 8 to 12 file nodes.
- Include at least 3 Vue files with "content".
- Include at least 2 preview sections.
- Include at least 1 primary action.
- Output JSON only."#
    )
}
