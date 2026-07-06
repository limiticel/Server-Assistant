use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{shared::AppError, AppState};

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
    Ok(Json(state.orchestrator.list_tools().await?))
}

async fn call_tool(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ToolPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut arguments = payload.arguments;
    merge_tool_config_arguments(&state, &name, &mut arguments).await?;

    Ok(Json(state.orchestrator.call_tool(&name, arguments).await?))
}

async fn merge_tool_config_arguments(
    state: &AppState,
    tool_name: &str,
    arguments: &mut Value,
) -> Result<(), AppError> {
    let Some(target) = arguments.as_object_mut() else {
        return Ok(());
    };

    let config: Option<Value> = sqlx::query_scalar(
        "select config
         from mcp_tools
         where name = $1 and enabled = true
         limit 1",
    )
    .bind(tool_name)
    .fetch_optional(&state.db)
    .await?;

    let Some(config) = config else {
        return Ok(());
    };

    for key in [
        "username",
        "password",
        "host",
        "port",
        "db_host",
        "db_port",
        "db_user",
        "db_password",
        "db_name",
    ] {
        if target.contains_key(key) {
            continue;
        }
        if let Some(value) = config.get(key) {
            if value.is_string() || value.is_number() {
                target.insert(key.to_owned(), value.clone());
            }
        }
    }

    Ok(())
}
