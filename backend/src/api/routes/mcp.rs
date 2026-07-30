use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{application::services::dynamic_tools, shared::AppError, AppState};

#[derive(Deserialize)]
struct ToolPayload {
    arguments: serde_json::Value,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/:name/call", post(call_tool))
}

async fn list_tools(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(dynamic_tools::list_tools(&state.db).await?))
}

async fn call_tool(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ToolPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(
        dynamic_tools::call_tool(&state.db, &name, payload.arguments).await?,
    ))
}
