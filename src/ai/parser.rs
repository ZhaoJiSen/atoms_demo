use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    errors::{ApiError, ApiResult},
    models::GeneratedResult,
};

pub(super) fn parse_generated_result(response: &str) -> ApiResult<GeneratedResult> {
    parse_ai_json::<GeneratedResult>(response)
}

fn parse_ai_json<T: DeserializeOwned>(response: &str) -> ApiResult<T> {
    let json_text = extract_json(response).ok_or(ApiError::Internal("AI response is not JSON"))?;
    let value: Value = serde_json::from_str(&json_text).map_err(|error| {
        tracing::error!(%error, content = %json_text, "Failed to parse AI JSON");
        ApiError::Internal("Failed to parse AI generated JSON")
    })?;

    let candidate = value
        .get("result")
        .or_else(|| value.get("generatedResult"))
        .or_else(|| value.get("data"))
        .unwrap_or(&value)
        .clone();

    let candidate = normalize_generated_result(candidate);

    serde_json::from_value(candidate).map_err(|error| {
        tracing::error!(%error, content = %json_text, "Failed to decode generated result");
        ApiError::Internal("Failed to decode AI generated app")
    })
}

pub(super) fn extract_chat_content(body: &str) -> ApiResult<String> {
    let value: Value = serde_json::from_str(body).map_err(|error| {
        tracing::error!(%error, body = %body, "Failed to parse AI response JSON");
        ApiError::Internal("Failed to parse AI response")
    })?;

    if looks_like_generated_result(&value) {
        return Ok(body.into());
    }

    let direct_paths = [
        &["output_text"][..],
        &["content"][..],
        &["message", "content"][..],
        &["data", "output_text"][..],
        &["data", "content"][..],
        &["data", "message", "content"][..],
        &["data", "choices", "0", "message", "content"][..],
        &["data", "choices", "0", "text"][..],
        &["choices", "0", "message", "content"][..],
        &["choices", "0", "text"][..],
        &["choices", "0", "delta", "content"][..],
    ];

    for path in direct_paths {
        if let Some(content) = get_path(&value, path).and_then(value_to_text) {
            // Extract JSON from markdown if present
            return Ok(extract_json_from_markdown(&content));
        }
    }

    if let Some(content) = extract_from_openai_responses_output(&value) {
        return Ok(extract_json_from_markdown(&content));
    }

    tracing::error!(body = %body, "AI response JSON did not contain usable content");
    Err(ApiError::Internal("No content from AI response"))
}

fn extract_json_from_markdown(text: &str) -> String {
    // Try to extract JSON from markdown code block
    if let Some(json) = extract_fenced_json(text) {
        return json;
    }
    // Return as-is if no markdown found
    text.to_string()
}

fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        if let Ok(index) = key.parse::<usize>() {
            current = current.as_array()?.get(index)?;
        } else {
            current = current.get(*key)?;
        }
    }
    Some(current)
}

fn value_to_text(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.into());
        }
    }

    if let Some(parts) = value.as_array() {
        let text = parts
            .iter()
            .filter_map(|part| {
                part.as_str()
                    .map(str::to_owned)
                    .or_else(|| part.get("text").and_then(Value::as_str).map(str::to_owned))
                    .or_else(|| {
                        part.get("content")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
            })
            .collect::<Vec<_>>()
            .join("");

        if !text.trim().is_empty() {
            return Some(text);
        }
    }

    None
}

fn extract_from_openai_responses_output(value: &Value) -> Option<String> {
    let output = value.get("output")?.as_array()?;
    let text = output
        .iter()
        .filter_map(|item| item.get("content"))
        .flat_map(|content| content.as_array().into_iter().flatten())
        .filter_map(|part| {
            part.get("text")
                .and_then(Value::as_str)
                .or_else(|| part.get("content").and_then(Value::as_str))
        })
        .collect::<Vec<_>>()
        .join("");

    (!text.trim().is_empty()).then_some(text)
}

fn looks_like_generated_result(value: &Value) -> bool {
    value.get("productSpec").is_some()
        && value.get("pages").is_some()
        && value.get("apis").is_some()
        && value.get("dataModels").is_some()
        && value.get("fileStructure").is_some()
        && value.get("preview").is_some()
}

fn normalize_generated_result(mut value: Value) -> Value {
    if let Some(apis) = value.get_mut("apis").and_then(Value::as_array_mut) {
        for api in apis {
            uppercase_string_field(api, "method");
        }
    }

    if let Some(files) = value.get_mut("fileStructure").and_then(Value::as_array_mut) {
        for file in files {
            lowercase_string_field(file, "type");
        }
    }

    if let Some(preview) = value.get_mut("preview") {
        lowercase_string_field(preview, "theme");

        if let Some(sections) = preview.get_mut("sections").and_then(Value::as_array_mut) {
            for section in sections {
                lowercase_string_field(section, "type");
            }
        }

        if let Some(actions) = preview.get_mut("actions").and_then(Value::as_array_mut) {
            for action in actions {
                lowercase_string_field(action, "type");
            }
        }
    }

    value
}

fn uppercase_string_field(value: &mut Value, key: &str) {
    if let Some(field) = value.get(key).and_then(Value::as_str) {
        value[key] = Value::String(field.to_ascii_uppercase());
    }
}

fn lowercase_string_field(value: &mut Value, key: &str) {
    if let Some(field) = value.get(key).and_then(Value::as_str) {
        value[key] = Value::String(field.to_ascii_lowercase());
    }
}

fn extract_json(response: &str) -> Option<String> {
    let trimmed = response.trim();
    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return Some(trimmed.into());
    }

    if let Some(fenced) = extract_fenced_json(trimmed) {
        return Some(fenced);
    }

    extract_balanced_json(trimmed)
}

fn extract_fenced_json(text: &str) -> Option<String> {
    let fence_start = text.find("```")?;
    let after_start = &text[fence_start + 3..];
    let content_start = after_start
        .find('\n')
        .map(|index| index + 1)
        .unwrap_or_default();
    let after_language = &after_start[content_start..];
    let fence_end = after_language.find("```")?;
    Some(after_language[..fence_end].trim().into())
}

fn extract_balanced_json(text: &str) -> Option<String> {
    let (start_index, open) = text
        .char_indices()
        .find_map(|(index, ch)| (ch == '{' || ch == '[').then_some((index, ch)))?;
    let close = if open == '{' { '}' } else { ']' };
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in text[start_index..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let end = start_index + offset + ch.len_utf8();
                return Some(text[start_index..end].trim().into());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_markdown_fence() {
        let value: Value = parse_ai_json("```json\n{\"result\":{\"ok\":true}}\n```").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn extracts_balanced_json_from_text() {
        let value: Value = parse_ai_json("Here is JSON: {\"ok\": true} done").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn extracts_chat_content_from_mimo_style_data_wrapper() {
        let body = r#"{
          "data": {
            "choices": [
              {
                "message": {
                  "content": "{\"productSpec\":{\"name\":\"Demo\"}}"
                }
              }
            ]
          }
        }"#;

        let content = extract_chat_content(body).unwrap();
        assert!(content.contains("productSpec"));
    }

    #[test]
    fn extracts_chat_content_from_content_parts() {
        let body = r#"{
          "choices": [
            {
              "message": {
                "content": [
                  { "type": "text", "text": "{\"ok\":" },
                  { "type": "text", "text": "true}" }
                ]
              }
            }
          ]
        }"#;

        let content = extract_chat_content(body).unwrap();
        assert_eq!(content, "{\"ok\":true}");
    }
}
