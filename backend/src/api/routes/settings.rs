use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{shared::AppError, AppState};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatContextSettings {
    pub compaction_enabled: bool,
    pub max_messages: usize,
    pub keep_last_messages: usize,
    pub max_summary_chars: usize,
}

impl Default for ChatContextSettings {
    fn default() -> Self {
        Self {
            compaction_enabled: true,
            max_messages: 80,
            keep_last_messages: 24,
            max_summary_chars: 8000,
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/chat-context",
        get(get_chat_context).put(update_chat_context),
    )
}

pub async fn load_chat_context_settings(state: &AppState) -> Result<ChatContextSettings, AppError> {
    let value: Option<Value> =
        sqlx::query_scalar("select value from settings where key = 'chat_context'")
            .fetch_optional(&state.db)
            .await?;

    let Some(value) = value else {
        return Ok(ChatContextSettings::default());
    };

    Ok(serde_json::from_value(value).unwrap_or_default())
}

async fn get_chat_context(
    State(state): State<AppState>,
) -> Result<Json<ChatContextSettings>, AppError> {
    Ok(Json(load_chat_context_settings(&state).await?))
}

async fn update_chat_context(
    State(state): State<AppState>,
    Json(payload): Json<ChatContextSettings>,
) -> Result<Json<ChatContextSettings>, AppError> {
    let settings = validate_chat_context(payload)?;
    let value = serde_json::to_value(&settings)
        .map_err(|err| AppError::Internal(format!("failed to serialize settings: {err}")))?;

    sqlx::query(
        "insert into settings (key, value)
         values ('chat_context', $1)
         on conflict (key) do update set value = excluded.value, updated_at = now()",
    )
    .bind(value)
    .execute(&state.db)
    .await?;

    Ok(Json(settings))
}

fn validate_chat_context(
    mut settings: ChatContextSettings,
) -> Result<ChatContextSettings, AppError> {
    if settings.max_messages < 4 || settings.max_messages > 200 {
        return Err(AppError::Validation(
            "max_messages must be between 4 and 200".to_owned(),
        ));
    }

    if settings.keep_last_messages < 2 || settings.keep_last_messages > settings.max_messages {
        return Err(AppError::Validation(
            "keep_last_messages must be between 2 and max_messages".to_owned(),
        ));
    }

    if settings.max_summary_chars < 500 || settings.max_summary_chars > 20000 {
        return Err(AppError::Validation(
            "max_summary_chars must be between 500 and 20000".to_owned(),
        ));
    }

    settings.keep_last_messages = settings.keep_last_messages.min(settings.max_messages);
    Ok(settings)
}
