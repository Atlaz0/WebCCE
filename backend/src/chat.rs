use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use openai::chat::{ChatCompletionMessage, ChatCompletionRequest};
use crate::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    conversation_id: String,
    messages: Vec<ChatCompletionMessage>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    reply: String,
}

pub async fn handle_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    // Example using async OpenAI client
    let resp = state
        .openai
        .chat()
        .create(ChatCompletionRequest {
            model: "gpt-5".to_string(),
            messages: payload.messages,
            ..Default::default()
        })
        .await
        .expect("chat request failed");

    let reply = resp
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_else(|| "(no reply)".to_string());

    Json(ChatResponse { reply })
}
