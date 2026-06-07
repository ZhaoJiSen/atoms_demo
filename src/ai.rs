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
    prompt::{build_app_generation_prompt, system_prompt},
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
    let response = call_ai(&client, settings, &prompt).await?;
    parse_generated_result(&response)
}

async fn call_ai(client: &Client, settings: &AiSettings, prompt: &str) -> ApiResult<String> {
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
                content: system_prompt().into(),
            },
            ChatMessage {
                role: "user".into(),
                content: prompt.into(),
            },
        ],
        temperature: 0.2,
        max_tokens: 8192,
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
        eprintln!("AI request failed with status {status}: {body}");
        return Err(ApiError::Internal("AI provider returned an error"));
    }

    extract_chat_content(&body)
}
