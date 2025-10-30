// src/chat.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

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
    // build OA messages
    let oa_msgs: Vec<OaIMsg> = payload
        .messages
        .iter()
        .map(|m| OaIMsg { role: &m.role, content: &m.content })
        .collect();

    let req_body = OaiReq {
        model: "gpt-5",
        messages: oa_msgs,
        max_tokens: Some(800),
    };

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&state.openai_api_key)
        .json(&req_body)
        .send()
        .await;

    let reply = match res {
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
                    tracing::warn!("OpenAI parse error: {}", e);
                    "(error parsing OpenAI response)".to_string()
                }
            }
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            tracing::warn!("OpenAI returned {}: {}", status, text);
            format!("(openai error {})", status)
        }
        Err(e) => {
            tracing::error!("network error calling OpenAI: {}", e);
            "(network error)".to_string()
        }
    };

    Json(ChatResponse { reply })
}
