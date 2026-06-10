use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use serde_json::{Value, json};
use tokio::sync::{Semaphore, mpsc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    auth::{callback, demo_login, login, logout, me, require_auth},
    errors::{ApiError, ApiResult},
    init::{complete_init, get_init},
    mock::{
        default_steps, derive_title, generate_mock_result, generated_messages,
        should_mock_generation_fail,
    },
    models::{
        AgentMessage, AgentMessageRole, AgentStepStatus, AiSettings, App, AppStatus,
        CreateAppMessageRequest, CreateAppRequest, CreateNoteRequest, FileNode, FileNodeType,
        GeneratedResult, Note, UpdateAiSettingsRequest, UpdateAppRequest, UpdateNoteRequest,
    },
    servers::{create_server, delete_server, get_server, list_servers, terminal_ws, update_server},
    state::AppState,
    time::now_iso,
};

pub(crate) fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/init", get(get_init).post(complete_init))
        .route("/api/auth/login", get(login))
        .route("/api/auth/callback", get(callback))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/auth/demo", post(demo_login))
        .route("/api/apps", get(list_apps).post(create_app))
        .route(
            "/api/apps/{id}",
            get(get_app).patch(update_app).delete(delete_app),
        )
        .route("/api/apps/{id}/messages", post(create_app_message))
        .route("/api/apps/{id}/generate", post(generate_app))
        .route("/api/apps/{id}/generate/stream", get(generate_app_stream))
        .route("/api/servers", get(list_servers).post(create_server))
        .route(
            "/api/servers/{id}",
            get(get_server).patch(update_server).delete(delete_server),
        )
        .route("/api/servers/{id}/terminal", get(terminal_ws))
        .route("/api/notes", get(list_notes).post(create_note))
        .route(
            "/api/notes/{id}",
            get(get_note).patch(update_note).delete(delete_note),
        )
        .route(
            "/api/settings/ai",
            get(get_ai_settings).put(update_ai_settings),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// Unique app id. A sequential prefix keeps ids readable, but a random suffix
/// prevents collisions with already-persisted apps after the in-memory counter
/// resets on restart (which previously overwrote other users' apps).
fn generate_app_id(seq: u64) -> String {
    use rand::{Rng, distributions::Alphanumeric};
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    format!("app_{seq:03}{suffix}")
}

async fn create_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAppRequest>,
) -> ApiResult<(StatusCode, Json<App>)> {
    let user = require_auth(&state, &headers).await?;

    let idea = payload.idea.trim();
    if idea.is_empty() {
        return Err(ApiError::BadRequest("Idea is required"));
    }

    let id_number = state.next_id.fetch_add(1, Ordering::Relaxed);
    let id = generate_app_id(id_number);
    let timestamp = now_iso();
    let title = derive_title(idea);
    let app = App {
        id: id.clone(),
        title,
        idea: idea.to_owned(),
        status: AppStatus::Pending,
        steps: default_steps(),
        messages: vec![
            AgentMessage {
                id: "msg_001".into(),
                role: AgentMessageRole::User,
                agent_name: None,
                content: idea.to_owned(),
                created_at: timestamp.clone(),
            },
            AgentMessage {
                id: "msg_002".into(),
                role: AgentMessageRole::System,
                agent_name: None,
                content: "App project created. Ready to generate.".into(),
                created_at: timestamp.clone(),
            },
        ],
        result: None,
        error: None,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    if let Some(pool) = &state.db {
        crate::db::put_app(pool, &user.id, &app).await?;
    } else {
        let mut apps = state
            .apps
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access app state"))?;
        apps.insert(id, app.clone());
    }

    Ok((StatusCode::CREATED, Json(app)))
}

async fn list_apps(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<App>>> {
    let user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        return Ok(Json(crate::db::list_apps(pool, &user.id).await?));
    }

    let apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;

    let mut app_list: Vec<App> = apps.values().cloned().collect();
    app_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(Json(app_list))
}

async fn get_app(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<App>> {
    if let Some(pool) = &state.db {
        let app = crate::db::get_app(pool, &id)
            .await?
            .ok_or(ApiError::NotFound("App not found"))?;
        return Ok(Json(app));
    }

    let apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;
    let app = apps
        .get(&id)
        .cloned()
        .ok_or(ApiError::NotFound("App not found"))?;

    Ok(Json(app))
}

async fn update_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAppRequest>,
) -> ApiResult<Json<App>> {
    let user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        let mut app = crate::db::get_app(pool, &id)
            .await?
            .ok_or(ApiError::NotFound("App not found"))?;

        if let Some(title) = payload.title {
            app.title = title;
        }
        if let Some(idea) = payload.idea {
            app.idea = idea;
        }
        app.updated_at = now_iso();

        crate::db::put_app(pool, &user.id, &app).await?;
        return Ok(Json(app));
    }

    let mut apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;

    let app = apps
        .get_mut(&id)
        .ok_or(ApiError::NotFound("App not found"))?;

    if let Some(title) = payload.title {
        app.title = title;
    }
    if let Some(idea) = payload.idea {
        app.idea = idea;
    }
    app.updated_at = now_iso();

    Ok(Json(app.clone()))
}

async fn delete_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        let deleted = crate::db::delete_app(pool, &id).await?;
        if !deleted {
            return Err(ApiError::NotFound("App not found"));
        }
        return Ok(Json(json!({ "success": true })));
    }

    let mut apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;

    if apps.remove(&id).is_none() {
        return Err(ApiError::NotFound("App not found"));
    }

    Ok(Json(json!({ "success": true })))
}

async fn create_app_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<CreateAppMessageRequest>,
) -> ApiResult<Json<App>> {
    let user = require_auth(&state, &headers).await?;

    let content = payload.content.trim();
    if content.is_empty() {
        return Err(ApiError::BadRequest("Message content is required"));
    }

    let mut app = load_app_for_update(&state, &id).await?;

    let timestamp = now_iso();
    let message_number = app.messages.len() + 1;
    app.messages.push(AgentMessage {
        id: format!("msg_{message_number:03}"),
        role: AgentMessageRole::User,
        agent_name: None,
        content: content.to_owned(),
        created_at: timestamp.clone(),
    });
    app.updated_at = timestamp;

    save_app(&state, &user.id, &app).await?;
    Ok(Json(app))
}

async fn generate_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<App>> {
    let user = require_auth(&state, &headers).await?;

    let (title, idea, conversation_context, previous_result_json, mock_generation_context) = {
        let mut app = load_app_for_update(&state, &id).await?;

        if app.status == AppStatus::Generating {
            return Err(ApiError::Conflict("App is already generating"));
        }

        let started_at = now_iso();
        app.status = AppStatus::Generating;
        app.error = None;
        app.result = None;
        app.updated_at = started_at.clone();
        app.steps = default_steps();

        if let Some(first_step) = app.steps.first_mut() {
            first_step.status = AgentStepStatus::Running;
            first_step.started_at = Some(started_at);
        }

        let title = app.title.clone();
        let idea = app.idea.clone();
        let conversation_context = build_generation_context(&app);
        let previous_result_json = app
            .result
            .as_ref()
            .and_then(|result| serde_json::to_string_pretty(result).ok());
        let mock_generation_context = build_mock_generation_context(&app);
        save_app(&state, &user.id, &app).await?;
        (
            title,
            idea,
            conversation_context,
            previous_result_json,
            mock_generation_context,
        )
    };

    if should_mock_generation_fail(&mock_generation_context) {
        let mut app = load_app_for_update(&state, &id).await?;

        let failed_at = now_iso();
        tracing::warn!(app_id = %id, "Generation failed (mock failure trigger)");
        if let Some(first_step) = app.steps.first_mut() {
            first_step.status = AgentStepStatus::Error;
            first_step.completed_at = Some(failed_at.clone());
        }
        app.status = AppStatus::Failed;
        app.error = Some("Failed to generate app".into());
        app.updated_at = failed_at.clone();
        app.messages.push(AgentMessage {
            id: "msg_error_001".into(),
            role: AgentMessageRole::Error,
            agent_name: Some("Reviewer Agent".into()),
            content: "Mock generation failed before producing a preview.".into(),
            created_at: failed_at,
        });
        save_app(&state, &user.id, &app).await?;

        return Err(ApiError::Internal("Failed to generate app"));
    }

    // Check if AI settings are configured
    let ai_settings = state
        .ai_settings
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access AI settings"))?
        .clone();

    let result = if let Some(settings) = ai_settings {
        // Use real AI generation
        tracing::info!(app_id = %id, title = %title, "Generating app with AI provider");
        match crate::ai::generate_with_ai(
            &settings,
            &idea,
            &title,
            &conversation_context,
            previous_result_json.as_deref(),
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                tracing::error!(app_id = %id, error = ?e, "AI generation failed");
                let mut app = load_app_for_update(&state, &id).await?;
                app.status = AppStatus::Failed;
                app.error = Some("AI generation failed".to_string());
                app.updated_at = now_iso();
                save_app(&state, &user.id, &app).await?;
                return Err(e);
            }
        }
    } else {
        // Use mock generation
        tracing::info!(app_id = %id, title = %title, "Generating app with mock (no AI settings)");
        generate_mock_result(&title, &mock_generation_context)
    };

    let mut app = load_app_for_update(&state, &id).await?;

    let completed_at = now_iso();
    for step in &mut app.steps {
        if step.started_at.is_none() {
            step.started_at = Some(completed_at.clone());
        }
        step.status = AgentStepStatus::Done;
        step.completed_at = Some(completed_at.clone());
    }
    let next_message_index = app.messages.len() + 1;
    app.messages.extend(generated_messages(next_message_index));
    app.result = Some(result);
    app.status = AppStatus::Completed;
    app.updated_at = completed_at;
    save_app(&state, &user.id, &app).await?;
    tracing::info!(app_id = %id, "Generation completed");

    Ok(Json(app))
}

fn sse_event(name: &str, data: Value) -> Result<Event, Infallible> {
    Ok(Event::default().event(name).data(data.to_string()))
}

fn is_code_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next(),
        Some("vue" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "css")
    )
}

/// Dependency tier for ordering file generation. Lower tiers are generated first
/// and their real code is passed as context to higher tiers that import them:
/// 0 foundations (.ts/utils/composables/stores) → 1 components → 2 pages/App → 3 entry.
fn file_tier(path: &str) -> u8 {
    if path.ends_with("main.ts") || path.ends_with("main.js") || path.ends_with("main.mjs") {
        return 3;
    }
    if path.ends_with("App.vue") || path.contains("/pages/") || path.starts_with("pages/") {
        return 2;
    }
    if path.contains("/components/") || path.starts_with("components/") {
        return 1;
    }
    0
}

fn template_file_content(path: &str) -> String {
    match path.rsplit('.').next() {
        Some("json") => "{}\n".to_string(),
        Some("md") => format!("# {path}\n"),
        _ => format!("// {path}\n"),
    }
}

fn fallback_file_content(path: &str) -> String {
    if path.ends_with(".vue") {
        "<template>\n  <div class=\"placeholder\">Generation failed for this component.</div>\n</template>\n\n<style scoped>\n.placeholder { padding: 16px; color: #a1a1aa; }\n</style>\n".to_string()
    } else {
        template_file_content(path)
    }
}

/// Messages produced by a per-file streaming task, merged into the SSE stream.
enum FileMsg {
    Start(String),
    Chunk(String, String),
    End(FileNode),
}

/// Max number of files generated concurrently, to stay under provider rate limits.
const MAX_CONCURRENT_FILES: usize = 4;

/// Generate one file's content, streaming token deltas to `tx` as they arrive.
/// Runs as its own task so multiple files generate in parallel. Returns early if
/// the receiver is gone (client disconnected) to avoid wasted provider calls.
async fn stream_one_file(
    settings: Option<AiSettings>,
    plan_json: Arc<String>,
    node: FileNode,
    tx: mpsc::Sender<FileMsg>,
    semaphore: Arc<Semaphore>,
) {
    // Wait for a concurrency slot before starting (rate limiting). Holding the
    // permit for the file's lifetime caps simultaneous in-flight generations.
    let _permit = match semaphore.acquire().await {
        Ok(permit) => permit,
        Err(_) => return,
    };

    let path = node.path.clone();
    if tx.send(FileMsg::Start(path.clone())).await.is_err() {
        return;
    }
    let desc = node.description.clone().unwrap_or_default();

    let content: String = if let Some(existing) = node.content.clone() {
        // Mock path: stream known content in chunks for a typing effect.
        let mut i = 0usize;
        while i < existing.len() {
            let mut end = (i + 48).min(existing.len());
            while !existing.is_char_boundary(end) {
                end += 1;
            }
            if tx
                .send(FileMsg::Chunk(path.clone(), existing[i..end].to_string()))
                .await
                .is_err()
            {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(22)).await;
            i = end;
        }
        existing
    } else if let Some(settings) = &settings {
        if is_code_file(&path) {
            match crate::ai::open_file_stream(settings, &plan_json, &path, &desc).await {
                Ok(mut resp) => {
                    let mut acc = String::new();
                    let mut buf = String::new();
                    loop {
                        match resp.chunk().await {
                            Ok(Some(bytes)) => {
                                buf.push_str(&String::from_utf8_lossy(&bytes));
                                while let Some(nl) = buf.find('\n') {
                                    let line: String = buf.drain(..=nl).collect();
                                    if let Some(delta) =
                                        crate::ai::extract_stream_delta(line.trim_end())
                                    {
                                        acc.push_str(&delta);
                                        if tx
                                            .send(FileMsg::Chunk(path.clone(), delta))
                                            .await
                                            .is_err()
                                        {
                                            return;
                                        }
                                    }
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                    if let Some(delta) = crate::ai::extract_stream_delta(buf.trim_end()) {
                        acc.push_str(&delta);
                        let _ = tx.send(FileMsg::Chunk(path.clone(), delta)).await;
                    }
                    crate::ai::strip_code_fences(&acc)
                }
                Err(_) => fallback_file_content(&path),
            }
        } else {
            template_file_content(&path)
        }
    } else {
        template_file_content(&path)
    };

    let filled = FileNode {
        path,
        node_type: FileNodeType::File,
        description: node.description.clone(),
        content: Some(content),
    };
    let _ = tx.send(FileMsg::End(filled)).await;
}

async fn persist_failed_app(state: &AppState, user_id: &str, id: &str, message: &str) {
    if let Ok(mut app) = load_app_for_update(state, id).await {
        let failed_at = now_iso();
        if let Some(first_step) = app.steps.first_mut() {
            first_step.status = AgentStepStatus::Error;
            first_step.completed_at = Some(failed_at.clone());
        }
        app.status = AppStatus::Failed;
        app.error = Some(message.to_string());
        app.updated_at = failed_at.clone();
        app.messages.push(AgentMessage {
            id: "msg_error_stream".into(),
            role: AgentMessageRole::Error,
            agent_name: Some("Reviewer Agent".into()),
            content: message.to_string(),
            created_at: failed_at,
        });
        let _ = save_app(state, user_id, &app).await;
    }
}

async fn finalize_streamed_app(
    state: &AppState,
    user_id: &str,
    id: &str,
    result: GeneratedResult,
) -> ApiResult<App> {
    let mut app = load_app_for_update(state, id).await?;
    let completed_at = now_iso();
    for step in &mut app.steps {
        if step.started_at.is_none() {
            step.started_at = Some(completed_at.clone());
        }
        step.status = AgentStepStatus::Done;
        step.completed_at = Some(completed_at.clone());
    }
    let next_message_index = app.messages.len() + 1;
    app.messages.extend(generated_messages(next_message_index));
    app.result = Some(result);
    app.status = AppStatus::Completed;
    app.updated_at = completed_at;
    save_app(state, user_id, &app).await?;
    Ok(app)
}

/// Streamed, staged generation. Emits SSE events as each agent stage completes:
/// `step` (status changes), `partial` (plan), `file` (one file ready), `done`, `error`.
async fn generate_app_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match require_auth(&state, &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };
    let user_id = user.id.clone();

    // Prelude: flip the app to `generating` and snapshot the generation context.
    let (title, idea, conversation_context, previous_result_json, mock_generation_context) = {
        let mut app = match load_app_for_update(&state, &id).await {
            Ok(a) => a,
            Err(e) => return e.into_response(),
        };
        // Note: we intentionally do NOT 409 on `Generating` here. If a previous
        // stream was interrupted (client navigated away/refreshed), the app is
        // left stuck in `Generating`; re-entering must be able to restart it.
        // Capture revision context BEFORE clearing the previous result.
        let previous_result_json = app
            .result
            .as_ref()
            .and_then(|r| serde_json::to_string_pretty(r).ok());

        let started_at = now_iso();
        app.status = AppStatus::Generating;
        app.error = None;
        app.result = None;
        app.updated_at = started_at;
        app.steps = default_steps();

        let title = app.title.clone();
        let idea = app.idea.clone();
        let conversation_context = build_generation_context(&app);
        let mock_generation_context = build_mock_generation_context(&app);
        if let Err(e) = save_app(&state, &user_id, &app).await {
            return e.into_response();
        }
        (
            title,
            idea,
            conversation_context,
            previous_result_json,
            mock_generation_context,
        )
    };

    let ai_settings = match state.ai_settings.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return ApiError::Internal("Failed to access AI settings").into_response(),
    };
    let mock_should_fail = should_mock_generation_fail(&mock_generation_context);

    let stream = async_stream::stream! {
        let analyze_started = now_iso();
        yield sse_event("step", json!({"id":"analyze_idea","status":"running","startedAt":analyze_started}));

        if mock_should_fail {
            persist_failed_app(&state, &user_id, &id, "Mock generation failed before producing a preview.").await;
            yield sse_event("fail", json!({"error":"Failed to generate app"}));
            return;
        }

        // Stage 1 — Planner: product plan + file manifest.
        let plan = if let Some(settings) = &ai_settings {
            match crate::ai::plan_app(settings, &idea, &title, &conversation_context, previous_result_json.as_deref()).await {
                Ok(p) => p,
                Err(_) => {
                    persist_failed_app(&state, &user_id, &id, "AI planning failed").await;
                    yield sse_event("fail", json!({"error":"AI planning failed"}));
                    return;
                }
            }
        } else {
            generate_mock_result(&title, &mock_generation_context)
        };

        let plan_done = now_iso();
        for sid in ["analyze_idea", "plan_product", "design_pages", "define_api"] {
            yield sse_event("step", json!({"id":sid,"status":"done","completedAt":plan_done.clone()}));
        }
        yield sse_event("partial", json!({
            "productSpec": serde_json::to_value(&plan.product_spec).unwrap_or(Value::Null),
            "pages": serde_json::to_value(&plan.pages).unwrap_or(Value::Null),
            "apis": serde_json::to_value(&plan.apis).unwrap_or(Value::Null),
            "dataModels": serde_json::to_value(&plan.data_models).unwrap_or(Value::Null),
            "preview": serde_json::to_value(&plan.preview).unwrap_or(Value::Null),
        }));
        // The file tree renders immediately from the manifest; contents stream in next.
        yield sse_event("manifest", json!({
            "fileStructure": serde_json::to_value(&plan.file_structure).unwrap_or(Value::Null),
        }));

        // Stage 2 — Engineer: generate files in dependency LAYERS. Foundations are
        // written first (utils/.ts → components → pages/App → main), and each layer's
        // REAL generated code is fed as context to the layers above it, so cross-file
        // APIs always line up. Files within a layer still stream in parallel.
        yield sse_event("step", json!({"id":"generate_files","status":"running","startedAt":now_iso()}));
        let available_paths = plan
            .file_structure
            .iter()
            .filter(|n| matches!(n.node_type, FileNodeType::File))
            .map(|n| n.path.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let base_context = format!(
            "{}\n\nAVAILABLE FILES — you may import ONLY these exact paths; never import a file that is not in this list:\n{}",
            serde_json::to_string_pretty(&plan).unwrap_or_default(),
            available_paths,
        );

        // Bucket files into dependency tiers (0 = foundations … 3 = entry).
        let mut tiers: [Vec<FileNode>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for node in &plan.file_structure {
            if matches!(node.node_type, FileNodeType::File) {
                tiers[file_tier(&node.path) as usize].push(node.clone());
            }
        }

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FILES));
        let mut accumulated = base_context;
        let mut generated: HashMap<String, FileNode> = HashMap::new();

        for tier in tiers.iter() {
            if tier.is_empty() {
                continue;
            }
            let ctx = Arc::new(accumulated.clone());
            let (tx, mut rx) = mpsc::channel::<FileMsg>(64);
            for node in tier {
                tokio::spawn(stream_one_file(
                    ai_settings.clone(),
                    ctx.clone(),
                    node.clone(),
                    tx.clone(),
                    semaphore.clone(),
                ));
            }
            drop(tx); // channel closes once this tier's tasks finish

            while let Some(msg) = rx.recv().await {
                match msg {
                    FileMsg::Start(path) => {
                        yield sse_event("file_start", json!({"path": path}));
                    }
                    FileMsg::Chunk(path, delta) => {
                        yield sse_event("file_chunk", json!({"path": path, "delta": delta}));
                    }
                    FileMsg::End(node) => {
                        yield sse_event("file_end", serde_json::to_value(&node).unwrap_or(Value::Null));
                        generated.insert(node.path.clone(), node);
                    }
                }
            }

            // Feed this tier's real code to the layers that depend on it.
            for node in tier {
                if let Some(content) = generated.get(&node.path).and_then(|f| f.content.as_ref()) {
                    accumulated.push_str(&format!(
                        "\n\n--- ALREADY GENERATED FILE: {} ---\n{}\n",
                        node.path, content
                    ));
                }
            }
        }

        // Reassemble in the original plan order (tasks complete out of order).
        let mut filled_files: Vec<FileNode> = Vec::new();
        for node in &plan.file_structure {
            if matches!(node.node_type, FileNodeType::File) {
                filled_files.push(generated.remove(&node.path).unwrap_or_else(|| FileNode {
                    path: node.path.clone(),
                    node_type: FileNodeType::File,
                    description: node.description.clone(),
                    content: Some(fallback_file_content(&node.path)),
                }));
            } else {
                filled_files.push(FileNode {
                    path: node.path.clone(),
                    node_type: FileNodeType::Directory,
                    description: node.description.clone(),
                    content: None,
                });
            }
        }
        yield sse_event("step", json!({"id":"generate_files","status":"done","completedAt":now_iso()}));

        // Stage 3 — preview + finalize.
        yield sse_event("step", json!({"id":"build_preview","status":"running","startedAt":now_iso()}));
        yield sse_event("step", json!({"id":"build_preview","status":"done","completedAt":now_iso()}));
        yield sse_event("step", json!({"id":"finalize","status":"running","startedAt":now_iso()}));

        let final_result = GeneratedResult {
            product_spec: plan.product_spec,
            pages: plan.pages,
            apis: plan.apis,
            data_models: plan.data_models,
            file_structure: filled_files,
            preview: plan.preview,
        };

        match finalize_streamed_app(&state, &user_id, &id, final_result).await {
            Ok(app) => {
                yield sse_event("step", json!({"id":"finalize","status":"done","completedAt":now_iso()}));
                yield sse_event("done", serde_json::to_value(&app).unwrap_or(Value::Null));
            }
            Err(_) => {
                yield sse_event("fail", json!({"error":"Failed to save generated app"}));
            }
        }
    };

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

async fn load_app_for_update(state: &AppState, id: &str) -> ApiResult<App> {
    if let Some(pool) = &state.db {
        return crate::db::get_app(pool, id)
            .await?
            .ok_or(ApiError::NotFound("App not found"));
    }

    let apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;
    apps.get(id)
        .cloned()
        .ok_or(ApiError::NotFound("App not found"))
}

async fn save_app(state: &AppState, user_id: &str, app: &App) -> ApiResult<()> {
    if let Some(pool) = &state.db {
        return crate::db::put_app(pool, user_id, app).await;
    }

    let mut apps = state
        .apps
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access app state"))?;
    apps.insert(app.id.clone(), app.clone());
    Ok(())
}

fn build_generation_context(app: &App) -> String {
    let mut context = String::from("Conversation history, oldest to newest:\n");

    for message in app
        .messages
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        let role = match &message.role {
            AgentMessageRole::User => "user",
            AgentMessageRole::Agent => "agent",
            AgentMessageRole::System => "system",
            AgentMessageRole::Error => "error",
        };
        context.push_str("- ");
        context.push_str(role);
        context.push_str(": ");
        context.push_str(message.content.trim());
        context.push('\n');
    }

    if app.result.is_some() {
        context.push_str("\nThe app already has a generated result. Treat the latest user messages as revision requests and return a complete updated result.\n");
    } else {
        context.push_str(
            "\nThis is the first generation. Return a complete result for the initial idea.\n",
        );
    }

    context
}

fn build_mock_generation_context(app: &App) -> String {
    let mut parts = vec![app.idea.trim().to_owned()];
    parts.extend(
        app.messages
            .iter()
            .filter(|message| matches!(&message.role, AgentMessageRole::User))
            .skip(1)
            .map(|message| format!("追加需求：{}", message.content.trim())),
    );
    parts.join("\n")
}

// Notes handlers
async fn list_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Note>>> {
    let user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        return Ok(Json(crate::db::list_notes(pool, &user.id).await?));
    }

    let notes = state
        .notes
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access notes state"))?;

    let mut note_list: Vec<Note> = notes.values().cloned().collect();
    note_list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(Json(note_list))
}

async fn create_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNoteRequest>,
) -> ApiResult<(StatusCode, Json<Note>)> {
    let user = require_auth(&state, &headers).await?;

    let title = payload.title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("Title is required"));
    }

    let id_number = state.next_note_id.fetch_add(1, Ordering::Relaxed);
    let id = format!("note_{id_number:03}");
    let timestamp = now_iso();

    let note = Note {
        id: id.clone(),
        title: title.to_owned(),
        content: payload.content.unwrap_or_default(),
        app_id: payload.app_id,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    if let Some(pool) = &state.db {
        crate::db::put_note(pool, &user.id, &note).await?;
    } else {
        let mut notes = state
            .notes
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access notes state"))?;
        notes.insert(id, note.clone());
    }

    Ok((StatusCode::CREATED, Json(note)))
}

async fn get_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Note>> {
    let _user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        let note = crate::db::get_note(pool, &id)
            .await?
            .ok_or(ApiError::NotFound("Note not found"))?;
        return Ok(Json(note));
    }

    let notes = state
        .notes
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access notes state"))?;
    let note = notes
        .get(&id)
        .cloned()
        .ok_or(ApiError::NotFound("Note not found"))?;

    Ok(Json(note))
}

async fn update_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNoteRequest>,
) -> ApiResult<Json<Note>> {
    let user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        let mut note = crate::db::get_note(pool, &id)
            .await?
            .ok_or(ApiError::NotFound("Note not found"))?;

        if let Some(title) = payload.title {
            note.title = title;
        }
        if let Some(content) = payload.content {
            note.content = content;
        }
        note.updated_at = now_iso();

        crate::db::put_note(pool, &user.id, &note).await?;
        return Ok(Json(note));
    }

    let mut notes = state
        .notes
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access notes state"))?;

    let note = notes
        .get_mut(&id)
        .ok_or(ApiError::NotFound("Note not found"))?;

    if let Some(title) = payload.title {
        note.title = title;
    }
    if let Some(content) = payload.content {
        note.content = content;
    }
    note.updated_at = now_iso();

    Ok(Json(note.clone()))
}

async fn delete_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let _user = require_auth(&state, &headers).await?;

    if let Some(pool) = &state.db {
        let deleted = crate::db::delete_note(pool, &id).await?;
        if !deleted {
            return Err(ApiError::NotFound("Note not found"));
        }
        return Ok(Json(json!({ "success": true })));
    }

    let mut notes = state
        .notes
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access notes state"))?;

    if notes.remove(&id).is_none() {
        return Err(ApiError::NotFound("Note not found"));
    }

    Ok(Json(json!({ "success": true })))
}

// AI Settings handlers
async fn get_ai_settings(
    State(state): State<AppState>,
    _headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let settings = state
        .ai_settings
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access AI settings"))?;

    match settings.as_ref() {
        Some(s) => Ok(Json(json!({
            "configured": true,
            "provider": s.provider,
            "model": s.model,
            "baseUrl": s.base_url,
            "apiKey": "***"
        }))),
        None => Ok(Json(json!({
            "configured": false,
            "provider": null,
            "model": null,
            "baseUrl": null,
            "apiKey": null
        }))),
    }
}

async fn update_ai_settings(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<UpdateAiSettingsRequest>,
) -> ApiResult<Json<Value>> {
    let model = payload.model.unwrap_or_else(|| match payload.provider {
        crate::models::AiProvider::Gpt => "gpt-4".to_string(),
        crate::models::AiProvider::Mimo => "mimo-v2.5-pro".to_string(),
    });

    let base_url = payload.base_url.unwrap_or_else(|| match payload.provider {
        crate::models::AiProvider::Gpt => "https://api.openai.com".to_string(),
        crate::models::AiProvider::Mimo => "https://token-plan-cn.xiaomimimo.com/v1".to_string(),
    });

    let settings = crate::models::AiSettings {
        provider: payload.provider,
        api_key: payload.api_key,
        model,
        base_url,
    };

    let mut ai_settings = state
        .ai_settings
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access AI settings"))?;
    *ai_settings = Some(settings.clone());

    Ok(Json(json!({
        "configured": true,
        "provider": settings.provider,
        "model": settings.model,
        "baseUrl": settings.base_url,
        "apiKey": "***"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, HeaderValue, Method, Request, header::COOKIE},
        response::{IntoResponse, Response},
    };
    use tower::ServiceExt;

    use crate::{
        models::{OAuthProviderMode, OAuthProviderSummary, OAuthUser},
        time::iso_from_unix_seconds,
    };

    #[tokio::test]
    async fn creates_app_with_camel_case_json() {
        let state = AppState::default();
        let cookie = seed_session(&state);
        let app = build_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/apps")
                    .header("content-type", "application/json")
                    .header("cookie", cookie)
                    .body(Body::from(json!({ "idea": "健身打卡应用" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_json(response).await;
        assert_eq!(body["status"], "pending");
        assert!(body.get("createdAt").is_some());
        assert!(body.get("created_at").is_none());
        assert_eq!(body["steps"][0]["status"], "waiting");
    }

    #[tokio::test]
    async fn protected_create_app_requires_auth() {
        let response = create_app(
            State(AppState::default()),
            HeaderMap::new(),
            Json(CreateAppRequest {
                idea: "健身打卡应用".into(),
            }),
        )
        .await
        .unwrap_err()
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            to_json(response).await,
            json!({ "error": "Authentication required" })
        );
    }

    #[tokio::test]
    async fn app_message_endpoint_persists_user_conversation() {
        let state = AppState::default();
        let created = create_test_app(state.clone()).await;
        let app_id = created["id"].as_str().unwrap().to_owned();

        let response = create_app_message(
            State(state.clone()),
            auth_headers(&state),
            Path(app_id),
            Json(CreateAppMessageRequest {
                content: "继续补充一个报表页面".into(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.last().unwrap()["role"], "user");
        assert_eq!(messages.last().unwrap()["content"], "继续补充一个报表页面");
    }

    #[tokio::test]
    async fn google_oauth_login_requires_provider_config() {
        let app = build_router(AppState::default());
        let login_response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/auth/login?redirect=/app/new")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(login_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            to_json(login_response).await,
            json!({ "error": "GOOGLE_OAUTH_CLIENT_ID is required" })
        );
    }

    #[tokio::test]
    async fn init_flow_moves_from_not_initialized_to_ready() {
        let app = build_router(AppState::default());
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/init")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["status"], "not_initialized");
        assert_eq!(body["apiHealthy"], true);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/init")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        assert_eq!(body["status"], "ready");
        assert_eq!(body["apiHealthy"], true);
        assert!(body.get("completedAt").is_some());
    }

    #[tokio::test]
    async fn server_crud_never_returns_sensitive_credentials() {
        let state = AppState::default();
        let cookie = seed_session(&state);
        let app = build_router(state);
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/servers")
                    .header("content-type", "application/json")
                    .header("cookie", cookie.clone())
                    .body(Body::from(
                        json!({
                            "name": "Demo Server",
                            "host": "10.0.0.10",
                            "username": "deploy",
                            "authType": "password",
                            "password": "secret"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);
        let created = to_json(create_response).await;
        assert_eq!(created["id"], "srv_001");
        assert_eq!(created["port"], 22);
        assert_eq!(created["status"], "disconnected");
        assert!(created.get("password").is_none());
        assert!(created.get("privateKey").is_none());

        let server_id = created["id"].as_str().unwrap();
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/servers/{server_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let fetched = to_json(get_response).await;
        assert_eq!(fetched["host"], "10.0.0.10");
        assert!(fetched.get("password").is_none());

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri(format!("/api/servers/{server_id}"))
                    .header("content-type", "application/json")
                    .header("cookie", cookie.clone())
                    .body(Body::from(json!({ "name": "Prod Server" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let updated = to_json(update_response).await;
        assert_eq!(updated["name"], "Prod Server");
        assert!(updated.get("password").is_none());
        assert!(updated.get("privateKey").is_none());

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/servers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let list = to_json(list_response).await;
        assert_eq!(list.as_array().unwrap().len(), 1);
        assert!(list[0].get("password").is_none());

        let delete_response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/servers/{server_id}"))
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::OK);
        assert_eq!(to_json(delete_response).await, json!({ "success": true }));
    }

    #[tokio::test]
    async fn server_validation_uses_error_shape() {
        let state = AppState::default();
        let cookie = seed_session(&state);
        let app = build_router(state);
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/servers")
                    .header("content-type", "application/json")
                    .header("cookie", cookie)
                    .body(Body::from(
                        json!({
                            "name": "Missing Credential",
                            "host": "10.0.0.11",
                            "username": "deploy",
                            "authType": "password"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            to_json(create_response).await,
            json!({ "error": "SSH password is required" })
        );
    }

    #[tokio::test]
    async fn generate_returns_completed_result() {
        let state = AppState::default();
        let created = create_test_app(state.clone()).await;
        let response = generate_app(
            State(state.clone()),
            auth_headers(&state),
            Path(created["id"].as_str().unwrap().to_owned()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_json(response).await;
        assert_eq!(body["status"], "completed");
        assert_eq!(body["steps"][0]["status"], "done");
        assert!(body["result"]["productSpec"].is_object());
        assert!(body["result"]["pages"].as_array().unwrap().len() >= 2);
        assert!(body["result"]["apis"].as_array().unwrap().len() >= 2);
        assert!(!body["result"]["dataModels"].as_array().unwrap().is_empty());
        assert!(body["result"]["fileStructure"].as_array().unwrap().len() >= 5);
        assert!(
            body["result"]["preview"]["sections"]
                .as_array()
                .unwrap()
                .len()
                >= 2
        );
        assert!(
            body["result"]["preview"]["actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| action["type"] == "primary")
        );
    }

    #[tokio::test]
    async fn follow_up_generation_uses_conversation_without_duplicate_message_ids() {
        let state = AppState::default();
        let created = create_test_app(state.clone()).await;
        let app_id = created["id"].as_str().unwrap().to_owned();

        let _ = generate_app(
            State(state.clone()),
            auth_headers(&state),
            Path(app_id.clone()),
        )
        .await
        .unwrap();

        let _ = create_app_message(
            State(state.clone()),
            auth_headers(&state),
            Path(app_id.clone()),
            Json(CreateAppMessageRequest {
                content: "把应用调整为企业权限管理场景".into(),
            }),
        )
        .await
        .unwrap();

        let response = generate_app(
            State(state.clone()),
            auth_headers(&state),
            Path(app_id.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_json(response).await;
        let messages = body["messages"].as_array().unwrap();
        let mut ids = messages
            .iter()
            .map(|message| message["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), messages.len());
        assert!(
            body["result"]["productSpec"]["description"]
                .as_str()
                .unwrap()
                .contains("企业权限管理")
        );

        let stored_app = {
            let apps = state.apps.lock().unwrap();
            apps.get(&app_id).cloned().unwrap()
        };
        let context = build_generation_context(&stored_app);
        assert!(context.contains("把应用调整为企业权限管理场景"));
        assert!(context.contains("complete updated result"));
    }

    #[tokio::test]
    async fn missing_app_returns_404_error_shape() {
        let response = get_app(State(AppState::default()), Path("missing".into()))
            .await
            .unwrap_err()
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(to_json(response).await, json!({ "error": "App not found" }));
    }

    #[tokio::test]
    async fn generating_app_rejects_duplicate_generate() {
        let state = AppState::default();
        let created = create_test_app(state.clone()).await;
        let app_id = created["id"].as_str().unwrap().to_owned();
        {
            let mut apps = state.apps.lock().unwrap();
            let app = apps.get_mut(&app_id).unwrap();
            app.status = AppStatus::Generating;
        }

        let response = generate_app(State(state.clone()), auth_headers(&state), Path(app_id))
            .await
            .unwrap_err()
            .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            to_json(response).await,
            json!({ "error": "App is already generating" })
        );
    }

    #[tokio::test]
    async fn mock_failure_sets_failed_app_and_error_step() {
        let state = AppState::default();
        let response = create_app(
            State(state.clone()),
            auth_headers(&state),
            Json(CreateAppRequest {
                idea: "测试 [mock-fail]".into(),
            }),
        )
        .await
        .unwrap()
        .into_response();
        let created = to_json(response).await;
        let app_id = created["id"].as_str().unwrap().to_owned();

        let response = generate_app(
            State(state.clone()),
            auth_headers(&state),
            Path(app_id.clone()),
        )
        .await
        .unwrap_err()
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            to_json(response).await,
            json!({ "error": "Failed to generate app" })
        );

        let response = get_app(State(state), Path(app_id))
            .await
            .unwrap()
            .into_response();
        let body = to_json(response).await;
        assert_eq!(body["status"], "failed");
        assert_eq!(body["steps"][0]["status"], "error");
        assert_eq!(body["error"], "Failed to generate app");
    }

    #[tokio::test]
    async fn empty_idea_returns_400_error_shape() {
        let state = AppState::default();
        let response = create_app(
            State(state.clone()),
            auth_headers(&state),
            Json(CreateAppRequest { idea: " ".into() }),
        )
        .await
        .unwrap_err()
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            to_json(response).await,
            json!({ "error": "Idea is required" })
        );
    }

    #[test]
    fn formats_unix_seconds_as_utc_iso() {
        assert_eq!(iso_from_unix_seconds(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_unix_seconds(1_704_067_200), "2024-01-01T00:00:00Z");
    }

    async fn create_test_app(state: AppState) -> Value {
        let response = create_app(
            State(state.clone()),
            auth_headers(&state),
            Json(CreateAppRequest {
                idea: "帮我生成一个健身打卡应用，支持记录训练".into(),
            }),
        )
        .await
        .unwrap()
        .into_response();

        to_json(response).await
    }

    fn auth_headers(state: &AppState) -> HeaderMap {
        let cookie = seed_session(state);
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(&cookie).unwrap());
        headers
    }

    fn seed_session(state: &AppState) -> String {
        let token = "test_session";
        let provider = OAuthProviderSummary {
            id: "google".into(),
            name: "Google".into(),
            mode: OAuthProviderMode::Google,
        };
        let user = OAuthUser {
            id: "google_test_user".into(),
            display_name: "Google Test User".into(),
            email: "google.user@example.com".into(),
            provider,
            created_at: "2026-06-05T12:00:00Z".into(),
        };
        state
            .auth_sessions
            .lock()
            .unwrap()
            .insert(token.into(), user);
        format!("atoms_demo_session={token}")
    }

    async fn to_json(response: Response) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }
}
