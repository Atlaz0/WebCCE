use axum::{extract::State, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub conversation_id: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Serialize)]
struct OaIMsg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OaiReq<'a> {
    model: &'a str,
    messages: Vec<OaIMsg<'a>>,
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OaiChoice {
    message: Option<OaiMessage>,
}
#[derive(Deserialize)]
struct OaiMessage {
    content: Option<String>,
}
#[derive(Deserialize)]
struct OaiResp {
    choices: Vec<OaiChoice>,
}

pub async fn handle_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    // Build OA-compatible messages
    let oa_msgs: Vec<OaIMsg> = payload
        .messages
        .iter()
        .map(|m| OaIMsg { role: &m.role, content: &m.content })
        .collect();

    let req_body = OaiReq {
        model: "gpt-5-nano", // kept as requested
        messages: oa_msgs,
        max_tokens: Some(800),
    };

    // Log request for debugging
    let body_json = serde_json::to_string(&req_body).unwrap_or_else(|_| "(serialize error)".into());
    info!("OpenAI request url=https://api.openai.com/v1/chat/completions key_present={}", !state.openai_api_key.is_empty());
    info!("OpenAI request body: {}", body_json);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&state.openai_api_key)
        .json(&req_body)
        .send()
        .await;

    // Handle response
    let reply_text = match res {
        Ok(r) if r.status().is_success() => {
            match r.json::<OaiResp>().await {
                Ok(parsed) => {
                    parsed
                        .choices
                        .into_iter()
                        .next()
                        .and_then(|c| c.message.and_then(|m| m.content))
                        .unwrap_or_else(|| "(no reply)".to_string())
                }
                Err(e) => {
                    warn!("OpenAI parse error: {}", e);
                    "(error parsing OpenAI response)".to_string()
                }
            }
        }

        Ok(mut r) => {
            // return full body for debugging
            let status = r.status();
            let body = r.text().await.unwrap_or_else(|_| "(no body)".to_string());
            warn!("OpenAI returned {}: {}", status, body);
            return Json(ChatResponse { reply: format!("openai {}: {}", status, body) });
        }

        Err(e) => {
            error!("network error calling OpenAI: {}", e);
            "(network error)".to_string()
        }
    };

    Json(ChatResponse { reply: reply_text })
}