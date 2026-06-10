mod parser;
mod prompt;

use reqwest::Client;
use serde::Serialize;

use crate::{
    errors::{ApiError, ApiResult},
    models::{AiSettings, GeneratedResult},
};

use self::{
    parser::{extract_chat_content, parse_generated_result},
    prompt::{
        build_app_generation_prompt, build_file_prompt, build_plan_prompt, file_system_prompt,
        system_prompt,
    },
};

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

pub(crate) async fn generate_with_ai(
    settings: &AiSettings,
    idea: &str,
    title: &str,
    conversation_context: &str,
    previous_result_json: Option<&str>,
) -> ApiResult<GeneratedResult> {
    let client = Client::new();
    let prompt =
        build_app_generation_prompt(idea, title, conversation_context, previous_result_json);
    tracing::info!(
        provider = %settings.provider,
        model = %settings.model,
        is_revision = previous_result_json.is_some(),
        prompt_len = prompt.len(),
        "Starting AI generation"
    );
    let response = call_ai(&client, settings, system_prompt(), &prompt).await?;
    tracing::debug!(response_len = response.len(), "AI raw response received");
    let result = parse_generated_result(&response)?;
    tracing::info!(
        pages = result.pages.len(),
        apis = result.apis.len(),
        data_models = result.data_models.len(),
        file_nodes = result.file_structure.len(),
        "AI generation parsed successfully"
    );
    Ok(result)
}

/// Stage 1: produce the product plan + file manifest (no file content).
pub(crate) async fn plan_app(
    settings: &AiSettings,
    idea: &str,
    title: &str,
    conversation_context: &str,
    previous_result_json: Option<&str>,
) -> ApiResult<GeneratedResult> {
    let client = Client::new();
    let prompt = build_plan_prompt(idea, title, conversation_context, previous_result_json);
    tracing::info!(model = %settings.model, "Planning stage: requesting product plan + manifest");
    let response = call_ai(&client, settings, system_prompt(), &prompt).await?;
    let plan = parse_generated_result(&response)?;
    tracing::info!(
        pages = plan.pages.len(),
        files = plan.file_structure.len(),
        "Planning stage complete"
    );
    Ok(plan)
}

/// Stage 2 (streaming): open an upstream streaming completion for one file.
/// Returns the raw response so the caller can forward token deltas as they arrive.
pub(crate) async fn open_file_stream(
    settings: &AiSettings,
    plan_json: &str,
    file_path: &str,
    file_description: &str,
) -> ApiResult<reqwest::Response> {
    let client = Client::new();
    let base = settings.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    };
    let request = ChatRequest {
        model: settings.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: file_system_prompt().into(),
            },
            ChatMessage {
                role: "user".into(),
                content: build_file_prompt(plan_json, file_path, file_description),
            },
        ],
        temperature: 0.2,
        max_tokens: 8192,
        stream: true,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|_| ApiError::Internal("AI request failed"))?;

    if !response.status().is_success() {
        tracing::error!(status = %response.status(), file = %file_path, "Streaming file request failed");
        return Err(ApiError::Internal("AI provider returned an error"));
    }
    Ok(response)
}

/// Parse one upstream SSE line and pull the incremental content delta, if any.
pub(crate) fn extract_stream_delta(line: &str) -> Option<String> {
    let data = line.strip_prefix("data:")?.trim();
    if data.is_empty() || data == "[DONE]" {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(data).ok()?;
    value
        .get("choices")?
        .get(0)?
        .get("delta")?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

/// Strip a leading/trailing markdown code fence if the provider added one.
pub(crate) fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        // Drop the first line (``` or ```vue) and the trailing fence.
        let after_lang = rest.splitn(2, '\n').nth(1).unwrap_or("");
        let body = after_lang.strip_suffix("```").unwrap_or(after_lang);
        return body.trim_end_matches('`').trim().to_string();
    }
    trimmed.to_string()
}

async fn call_ai(
    client: &Client,
    settings: &AiSettings,
    system: &str,
    prompt: &str,
) -> ApiResult<String> {
    let base = settings.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    };

    let request = ChatRequest {
        model: settings.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: system.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: prompt.into(),
            },
        ],
        temperature: 0.2,
        max_tokens: 8192,
        stream: false,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|_| ApiError::Internal("AI request failed"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|_| ApiError::Internal("Failed to read AI response"))?;

    if !status.is_success() {
        tracing::error!(%status, body = %body, "AI provider returned an error");
        return Err(ApiError::Internal("AI provider returned an error"));
    }

    extract_chat_content(&body)
}
