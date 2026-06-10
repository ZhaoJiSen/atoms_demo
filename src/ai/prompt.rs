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

The Vue files you generate are run live in a minimal in-browser sandbox.
The sandbox only provides the `vue` and `vue-router` runtimes. It has NO Tailwind,
NO UI component library, and NO CSS framework. Follow these rules so the app renders:
- Build real, working Vue 3 single-file components with <template>, <script setup>, and <style>.
- Style EVERY component with plain CSS inside its own <style> block. Do NOT use Tailwind or
  utility-class frameworks; class names like `flex`, `p-4`, `bg-zinc-900` will have no effect.
- Use normal, semantic class names and write the matching CSS rules yourself.
- Implement real interactivity (state, events, computed) so apps like a calculator or a
  game actually work; do not leave handlers empty.
- Only import from other generated files or from 'vue' / 'vue-router'. Do not import any
  package that is not generated.
- Provide a single clear entry: either `src/main.ts` that mounts the app, or a root
  `src/App.vue` / `src/pages/index.vue` component.
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
- Every Vue file must be a complete, runnable SFC: <template> + <script setup> + <style> with
  plain CSS (no Tailwind / no utility classes). Implement real interactivity, not placeholders.
- Provide one entry the sandbox can mount: src/main.ts (mounts the app) or src/App.vue / src/pages/index.vue.
- If the app has multiple pages, wire them with vue-router inside the generated code; in-app
  navigation must work without relying on the host.
- Include at least 2 preview sections.
- Include at least 1 primary action.
- Output JSON only."#
    )
}

/// Stage 1 (Planner): produce the product plan + a file manifest WITHOUT code.
/// The manifest's paths and descriptions drive the per-file code stage.
pub(super) fn build_plan_prompt(
    idea: &str,
    title: &str,
    conversation_context: &str,
    previous_result_json: Option<&str>,
) -> String {
    let previous_result = previous_result_json
        .map(|value| format!("\nExisting generated result to revise:\n{value}\n"))
        .unwrap_or_default();

    format!(
        r#"Plan an app prototype for this user idea. This is the PLANNING stage only.

Title: {title}
Idea: {idea}
Conversation and follow-up requirements:
{conversation_context}
{previous_result}

Return one JSON object with this exact shape (camelCase keys):

{{
  "productSpec": {{
    "name": "App name",
    "description": "Short product description",
    "targetUsers": ["user 1", "user 2"],
    "coreFeatures": ["feature 1", "feature 2", "feature 3"],
    "userFlow": ["step 1", "step 2", "step 3"]
  }},
  "pages": [
    {{ "path": "/", "name": "Dashboard", "description": "What this page is for", "components": ["MetricCard"] }}
  ],
  "apis": [
    {{ "method": "GET", "path": "/api/items", "description": "List items", "requestExample": null, "responseExample": {{"items": []}} }}
  ],
  "dataModels": [
    {{ "name": "Item", "description": "Core record", "fields": [ {{ "name": "id", "type": "string", "required": true, "description": "Unique ID" }} ] }}
  ],
  "fileStructure": [
    {{ "path": "src/pages/index.vue", "type": "file", "description": "Main page; what it renders and which components/files it imports" }},
    {{ "path": "src/components", "type": "directory", "description": "Shared components" }}
  ],
  "preview": {{
    "title": "Preview title",
    "description": "Preview description",
    "theme": "dark",
    "sections": [ {{ "id": "overview", "title": "Overview", "type": "stats", "content": "Summary", "items": [ {{ "title": "Active", "description": "Active records", "value": "24" }} ] }} ],
    "actions": [ {{ "label": "Create", "type": "primary" }} ]
  }}
}}

Rules for the PLANNING stage:
- fileStructure is a MANIFEST ONLY. Do NOT include any "content" field; code is written in a later stage.
- In each file node's "description", state its responsibility AND the exact relative paths it imports,
  so the later code stage produces consistent imports. Keep paths internally consistent.
- Include one clear entry: src/main.ts (mounts the app) or src/App.vue / src/pages/index.vue.
- If multiple pages exist, plan to wire them with vue-router in the generated code.
- Generate 3 to 5 pages, 4 to 6 APIs, 2 to 4 data models, 6 to 9 file nodes (at least 3 Vue files).
- The manifest MUST be import-complete: every component or module that any file will import
  has to be listed as its own file node. Do not plan a file that imports something not in the list.
- Prefer focused files, but never drop a component you intend to import just to keep the list short.
- Keep each page's core interactive logic (state, handlers, timers, game loops) INSIDE that
  page's own component. Use separate files only for genuinely shared or presentational pieces.
  For a single-screen app (calculator, game), plan ONE self-contained main component that works
  on its own rather than splitting its state across files.
- Include at least 2 preview sections and at least 1 primary action.
- Output JSON only."#
    )
}

/// System prompt for the per-file code stage: raw file content, no JSON.
pub(super) fn file_system_prompt() -> &'static str {
    r#"You write the full content of ONE source file for a Vue 3 app.
Return ONLY the raw file content. No JSON, no markdown code fences, no commentary.
For .vue files: a complete single-file component with <template>, <script setup>, and <style>.
The app runs in a minimal sandbox that provides ONLY the `vue` and `vue-router` runtimes:
- No Tailwind, no UI library, no CSS framework. Style with plain CSS in the <style> block using
  semantic class names; utility classes like `flex` or `p-4` have NO effect.
- Implement real, working interactivity (state, events, computed). No empty handlers, no TODOs.
- A page / entry component must be SELF-CONTAINED: define its own reactive state, event
  handlers, and any timers or game loops inside its own <script setup>. Do NOT rely on another
  file to hold a page's core state — for a single-screen app (calculator, game, etc.) put the
  whole thing in one component.
- When you DO import another project file, use the EXACT exported name and shape the plan
  implies; a mismatched import crashes at runtime. Prefer passing data to child components via
  props over sharing mutable state across files.
- Initialize ALL reactive state with a concrete default value at declaration, BEFORE the template
  reads it (e.g. `const game = reactive({ gridSize: 20, score: 0, snake: [] })`). The template must
  never read a property of something that could be undefined; if a value may be absent, guard it
  with `v-if`, optional chaining, or a default. Runtime errors like "Cannot read properties of
  undefined" mean the template rendered before its data existed — do not let that happen.
- A child component must declare every prop it uses via `defineProps`, with sensible defaults, and
  must not assume a parent passed something it did not.
- Keep the file focused and reasonably concise; do not pad it with unnecessary boilerplate.
- Only import from the provided project files or from 'vue' / 'vue-router'. NEVER import a file
  that is not in the provided list. If you need a small helper or component that is not listed,
  define it inline in THIS file instead of importing it.
"#
}

/// Stage 2 (Engineer): write the content of a single file given the plan + manifest.
pub(super) fn build_file_prompt(plan_json: &str, file_path: &str, file_description: &str) -> String {
    format!(
        r#"Write the complete content of this single file for the planned app.

Target file: {file_path}
File responsibility: {file_description}

Project context — the plan, the AVAILABLE FILES list, and the FULL SOURCE of any files that were
already generated. When you import one of those files, use its ACTUAL exported names and shapes
exactly as written below (do not guess a different API):
{plan_json}

Hard rule: you may ONLY import files that appear in the AVAILABLE FILES list above. If you would
need a component or module that is not listed, define it inline in this file — do NOT import a
path that does not exist.

Return ONLY the raw content of `{file_path}`. No fences, no JSON, no explanation."#
    )
}

