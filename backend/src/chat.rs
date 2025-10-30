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

fn headers_to_string(hdrs: &reqwest::header::HeaderMap) -> String {
    hdrs.iter()
        .map(|(k, v)| {
            let v = v.to_str().unwrap_or("<non-utf8>");
            format!("{}: {}", k.as_str(), v)
        })
        .collect::<Vec<_>>()
        .join("\n")
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

    // Keep the requested model
    let req_body = OaiReq {
        model: "gpt-5-nano",
        messages: oa_msgs,
        max_tokens: Some(800),
    };

    // Serialize body for logs
    let body_json = serde_json::to_string_pretty(&req_body).unwrap_or_else(|_| "(serialize error)".into());

    // Log presence of key but do NOT print it
    info!("Preparing OpenAI request. key_present={}", !state.openai_api_key.is_empty());
    info!("OpenAI request URL: https://api.openai.com/v1/chat/completions");
    info!("OpenAI request body: {}", body_json);

    let client = reqwest::Client::new();

    // Send request and capture result
    let send_result = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&state.openai_api_key) // reqwest sets Authorization header internally
        .json(&req_body)
        .send()
        .await;

    // Build debug reply that we will return to caller when something goes wrong.
    let mut debug_parts: Vec<String> = Vec::new();
    debug_parts.push("=== DEBUG: OpenAI request ===".to_string());
    debug_parts.push("URL: https://api.openai.com/v1/chat/completions".to_string());
    debug_parts.push(format!("key_present: {}", !state.openai_api_key.is_empty()));
    debug_parts.push("REQUEST BODY:".to_string());
    debug_parts.push(body_json);

    // Evaluate response
    let final_reply = match send_result {
        Ok(resp) if resp.status().is_success() => {
            // success path: parse a normal OpenAI response
            match resp.json::<OaiResp>().await {
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
                    debug_parts.push(format!("ERROR parsing OpenAI JSON: {}", e));
                    // try to extract raw text too
                    let raw = resp.text().await.unwrap_or_else(|_| "(no body)".to_string());
                    debug_parts.push("RAW BODY:".to_string());
                    debug_parts.push(raw);
                    debug_parts.join("\n\n")
                }
            }
        }

        Ok(mut resp) => {
            // non-success status. Capture headers and body. Return detailed debug.
            let status = resp.status();
            let headers = headers_to_string(resp.headers());
            let body_text = resp.text().await.unwrap_or_else(|_| "(no body)".to_string());

            warn!("OpenAI returned status {}. Body: {}", status, body_text);

            debug_parts.push("=== OPENAI RESPONSE ===".to_string());
            debug_parts.push(format!("status: {}", status));
            debug_parts.push("response headers:".to_string());
            debug_parts.push(headers);
            debug_parts.push("response body:".to_string());
            debug_parts.push(body_text);

            // return the debug text so you see exact server-side response
            debug_parts.join("\n\n")
        }

        Err(e) => {
            // network error / client error
            error!("Network error calling OpenAI: {}", e);
            debug_parts.push(format!("NETWORK ERROR: {}", e));
            debug_parts.join("\n\n")
        }
    };

    Json(ChatResponse { reply: final_reply })
}
