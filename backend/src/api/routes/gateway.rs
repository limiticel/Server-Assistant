use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::ai::{ChatRequest, EmbeddingRequest},
    shared::AppError,
    AppState,
};

#[derive(Deserialize)]
struct GatewayChatRequest {
    model: String,
    messages: Vec<crate::domain::ai::ChatMessage>,
    provider: Option<String>,
    stream: Option<bool>,
}

#[derive(Serialize)]
struct Model {
    id: String,
    object: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/models", get(models))
        .route("/chat/completions", post(chat_completions))
        .route("/embeddings", post(embeddings))
}

async fn models(State(state): State<AppState>) -> Json<serde_json::Value> {
    let data: Vec<Model> = state
        .providers
        .names()
        .into_iter()
        .map(|id| Model {
            id,
            object: "model",
        })
        .collect();
    Json(serde_json::json!({ "object": "list", "data": data }))
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<GatewayChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = payload.provider.unwrap_or_else(|| "openai".to_owned());
    let response = state
        .providers
        .get(&provider)?
        .chat(ChatRequest {
            model: payload.model.clone(),
            messages: payload.messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: payload.stream.unwrap_or(false),
            tools: None,
            tool_choice: None,
        })
        .await?;

    Ok(Json(serde_json::json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "choices": [{ "index": 0, "message": { "role": "assistant", "content": response.content }, "finish_reason": "stop" }],
        "usage": { "prompt_tokens": response.prompt_tokens, "completion_tokens": response.completion_tokens, "total_tokens": response.prompt_tokens + response.completion_tokens }
    })))
}

async fn embeddings(
    State(state): State<AppState>,
    Json(payload): Json<EmbeddingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let response = state.providers.get("openai")?.embeddings(payload).await?;
    let data: Vec<_> = response.embeddings.into_iter().enumerate().map(|(index, embedding)| {
        serde_json::json!({ "object": "embedding", "index": index, "embedding": embedding })
    }).collect();
    Ok(Json(serde_json::json!({ "object": "list", "data": data })))
}
