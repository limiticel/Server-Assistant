use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    application::services::chat_service, domain::ai::ChatMessage, shared::AppError, AppState,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatContextSettings {
    pub compaction_enabled: bool,
    pub max_messages: usize,
    pub keep_last_messages: usize,
    pub max_summary_chars: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileSummarySettings {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
}

impl Default for ProfileSummarySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: String::new(),
            model: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileSummaryState {
    pub settings: ProfileSummarySettings,
    pub summary: String,
    pub generated_at: Option<DateTime<Utc>>,
}

impl Default for ProfileSummaryState {
    fn default() -> Self {
        Self {
            settings: ProfileSummarySettings::default(),
            summary: String::new(),
            generated_at: None,
        }
    }
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
    Router::new()
        .route(
            "/chat-context",
            get(get_chat_context).put(update_chat_context),
        )
        .route(
            "/profile-summary",
            get(get_profile_summary).put(update_profile_summary),
        )
        .route("/profile-summary/generate", post(generate_profile_summary))
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

async fn get_profile_summary(
    State(state): State<AppState>,
) -> Result<Json<ProfileSummaryState>, AppError> {
    Ok(Json(load_profile_summary_state(&state).await?))
}

async fn update_profile_summary(
    State(state): State<AppState>,
    Json(payload): Json<ProfileSummarySettings>,
) -> Result<Json<ProfileSummaryState>, AppError> {
    let mut current = load_profile_summary_state(&state).await?;
    current.settings = validate_profile_summary_settings(payload)?;
    save_profile_summary_state(&state, &current).await?;
    Ok(Json(current))
}

async fn generate_profile_summary(
    State(state): State<AppState>,
) -> Result<Json<ProfileSummaryState>, AppError> {
    let mut current = load_profile_summary_state(&state).await?;
    current.settings = validate_profile_summary_settings(current.settings)?;

    if let Some(generated_at) = current.generated_at {
        if generated_at.date_naive() == Utc::now().date_naive()
            && !current.summary.trim().is_empty()
        {
            return Ok(Json(current));
        }
    }

    let transcript = load_profile_summary_source(&state).await?;
    if transcript.trim().is_empty() {
        current.summary =
            "Ainda nao ha conversas suficientes para montar um resumo diario do usuario."
                .to_owned();
        current.generated_at = Some(Utc::now());
        save_profile_summary_state(&state, &current).await?;
        return Ok(Json(current));
    }

    let messages = vec![
        ChatMessage {
            role: "system".to_owned(),
            content: "Voce cria um resumo diario de perfil do usuario com base nas conversas recentes. Escreva em portugues do Brasil, em 1 a 3 paragrafos curtos. O resumo deve ser util para continuidade: objetivos do usuario, preferencias, decisoes recentes, pendencias e contexto importante. Nao seja longo demais e nao invente informacoes.".to_owned(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
        ChatMessage {
            role: "user".to_owned(),
            content: format!("Gere o resumo diario do usuario com base neste historico recente:\n\n{transcript}"),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    ];

    let response = chat_service::complete_chat(
        &state,
        &current.settings.provider,
        &current.settings.model,
        messages,
    )
    .await?;

    current.summary = limit_summary(response.content, 1400);
    current.generated_at = Some(Utc::now());
    save_profile_summary_state(&state, &current).await?;
    Ok(Json(current))
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

async fn load_profile_summary_state(state: &AppState) -> Result<ProfileSummaryState, AppError> {
    let value: Option<Value> =
        sqlx::query_scalar("select value from settings where key = 'profile_summary'")
            .fetch_optional(&state.db)
            .await?;

    let Some(value) = value else {
        return Ok(ProfileSummaryState::default());
    };

    Ok(serde_json::from_value(value).unwrap_or_default())
}

async fn save_profile_summary_state(
    state: &AppState,
    settings: &ProfileSummaryState,
) -> Result<(), AppError> {
    let value = serde_json::to_value(settings)
        .map_err(|err| AppError::Internal(format!("failed to serialize profile summary: {err}")))?;

    sqlx::query(
        "insert into settings (key, value)
         values ('profile_summary', $1)
         on conflict (key) do update set value = excluded.value, updated_at = now()",
    )
    .bind(value)
    .execute(&state.db)
    .await?;

    Ok(())
}

fn validate_profile_summary_settings(
    settings: ProfileSummarySettings,
) -> Result<ProfileSummarySettings, AppError> {
    if settings.enabled && (settings.provider.trim().is_empty() || settings.model.trim().is_empty())
    {
        return Err(AppError::Validation(
            "provider and model are required for profile summary".to_owned(),
        ));
    }

    Ok(ProfileSummarySettings {
        enabled: settings.enabled,
        provider: settings.provider.trim().to_owned(),
        model: settings.model.trim().to_owned(),
    })
}

async fn load_profile_summary_source(state: &AppState) -> Result<String, AppError> {
    let rows: Vec<(String, String, chrono::DateTime<Utc>)> = sqlx::query_as(
        "select role, content, created_at
         from messages
         where role in ('user', 'assistant')
         order by created_at desc
         limit 120",
    )
    .fetch_all(&state.db)
    .await?;

    let mut lines = Vec::with_capacity(rows.len());
    for (role, content, created_at) in rows.into_iter().rev() {
        let clean = content.split_whitespace().collect::<Vec<_>>().join(" ");
        if clean.is_empty() {
            continue;
        }

        lines.push(format!(
            "[{}] {}: {}",
            created_at.format("%Y-%m-%d %H:%M"),
            role,
            truncate_chars(&clean, 900)
        ));
    }

    Ok(truncate_chars(&lines.join("\n"), 18000))
}

fn limit_summary(summary: String, max_chars: usize) -> String {
    let clean = summary.trim().to_owned();
    if clean.chars().count() <= max_chars {
        return clean;
    }

    truncate_chars(&clean, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}
