use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, patch, post},
    Json, Router,
};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::routes::settings::{load_chat_context_settings, ChatContextSettings},
    application::services::chat_service::{self, AgentEvent},
    domain::ai::ChatMessage,
    shared::AppError,
    AppState,
};

#[derive(Deserialize)]
struct NewConversation {
    title: Option<String>,
}

#[derive(Deserialize)]
struct SendMessage {
    provider: String,
    model: String,
    content: String,
}

#[derive(Serialize)]
struct ConversationResponse {
    id: Uuid,
    title: String,
}

#[derive(Serialize)]
struct MessageResponse {
    id: Uuid,
    role: String,
    content: String,
    provider: Option<String>,
    model: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/conversations/:id",
            patch(rename_conversation).delete(delete_conversation),
        )
        .route(
            "/chat/:conversation_id/messages",
            get(list_messages).post(send_message),
        )
        .route(
            "/chat/:conversation_id/messages/stream",
            post(stream_message),
        )
        .route("/chat/:conversation_id/stream", get(stream_chat))
}

async fn list_conversations(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationResponse>>, AppError> {
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("select id, title from conversations order by updated_at desc limit 50")
            .fetch_all(&state.db)
            .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title)| ConversationResponse { id, title })
            .collect(),
    ))
}

async fn create_conversation(
    State(state): State<AppState>,
    Json(payload): Json<NewConversation>,
) -> Result<Json<ConversationResponse>, AppError> {
    let id = Uuid::new_v4();
    let title = payload.title.unwrap_or_else(|| "Novo chat".to_owned());
    sqlx::query("insert into conversations (id, title) values ($1, $2)")
        .bind(id)
        .bind(&title)
        .execute(&state.db)
        .await?;
    Ok(Json(ConversationResponse { id, title }))
}

async fn rename_conversation(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<NewConversation>,
) -> Result<Json<ConversationResponse>, AppError> {
    let title = payload.title.unwrap_or_else(|| "Chat renomeado".to_owned());
    sqlx::query("update conversations set title = $2, updated_at = now() where id = $1")
        .bind(id)
        .bind(&title)
        .execute(&state.db)
        .await?;
    Ok(Json(ConversationResponse { id, title }))
}

async fn delete_conversation(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("delete from conversations where id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn list_messages(
    Path(conversation_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let rows: Vec<(
        Uuid,
        String,
        String,
        Option<String>,
        Option<String>,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "select id, role, content, provider, model, created_at
             from messages
             where conversation_id = $1
             order by created_at asc",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(
                |(id, role, content, provider, model, created_at)| MessageResponse {
                    id,
                    role,
                    content,
                    provider,
                    model,
                    created_at,
                },
            )
            .collect(),
    ))
}

async fn send_message(
    Path(conversation_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<SendMessage>,
) -> Result<Json<serde_json::Value>, AppError> {
    let messages = vec![ChatMessage {
        role: "user".into(),
        content: payload.content.clone(),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    let response =
        chat_service::complete_chat(&state, &payload.provider, &payload.model, messages).await?;

    sqlx::query("insert into messages (id, conversation_id, role, content, provider, model) values ($1, $2, 'user', $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(conversation_id)
        .bind(payload.content)
        .bind(&payload.provider)
        .bind(&payload.model)
        .execute(&state.db)
        .await?;

    sqlx::query("insert into messages (id, conversation_id, role, content, provider, model) values ($1, $2, 'assistant', $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(conversation_id)
        .bind(&response.content)
        .bind(&payload.provider)
        .bind(&payload.model)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "content": response.content })))
}

async fn stream_message(
    Path(conversation_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<SendMessage>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>>, AppError> {
    sqlx::query("insert into messages (id, conversation_id, role, content, provider, model) values ($1, $2, 'user', $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(conversation_id)
        .bind(&payload.content)
        .bind(&payload.provider)
        .bind(&payload.model)
        .execute(&state.db)
        .await?;

    let messages = recent_messages(&state, conversation_id).await?;
    let usage_messages = messages.clone();
    let context_settings = load_chat_context_settings(&state).await?;
    let original_message_count = messages.len();
    let (messages, was_compacted) = compact_messages(messages, &context_settings);
    let compacted_message_count = original_message_count.saturating_sub(messages.len());
    let mut upstream = chat_service::agent_stream_with_model_tools(
        state.clone(),
        payload.provider.clone(),
        payload.model.clone(),
        messages,
    )
    .await?;
    let db = state.db.clone();
    let provider = payload.provider.clone();
    let model = payload.model.clone();

    let stream = async_stream::stream! {
        let mut full_content = String::new();

        if was_compacted {
            yield Ok(Event::default().event("status").data(format!(
                "Conversa compactada: {} mensagens antigas foram resumidas para economizar contexto.",
                compacted_message_count
            )));
        }

        while let Some(item) = upstream.next().await {
            match item {
                Ok(AgentEvent::Status(message)) => {
                    yield Ok(Event::default().event("status").data(message));
                }
                Ok(AgentEvent::ToolStart { name, arguments }) => {
                    yield Ok(Event::default().event("tool_start").data(serde_json::json!({
                        "name": name,
                        "arguments": arguments
                    }).to_string()));
                }
                Ok(AgentEvent::ToolResult { name, result }) => {
                    yield Ok(Event::default().event("tool_result").data(serde_json::json!({
                        "name": name,
                        "result": result
                    }).to_string()));
                }
                Ok(AgentEvent::Delta(chunk)) => {
                    full_content.push_str(&chunk);
                    yield Ok(Event::default().event("delta").data(chunk));
                }
                Err(err) => {
                    yield Ok(Event::default().event("error").data(err.to_string()));
                    return;
                }
            }
        }

        let _ = sqlx::query("insert into messages (id, conversation_id, role, content, provider, model) values ($1, $2, 'assistant', $3, $4, $5)")
            .bind(Uuid::new_v4())
            .bind(conversation_id)
            .bind(&full_content)
            .bind(&provider)
            .bind(&model)
            .execute(&db)
            .await;

        chat_service::record_estimated_stream_usage(
            &db,
            &provider,
            &model,
            &usage_messages,
            &full_content,
        )
        .await;

        yield Ok(Event::default().event("done").data("[DONE]"));
    };

    Ok(Sse::new(stream))
}

async fn recent_messages(
    state: &AppState,
    conversation_id: Uuid,
) -> Result<Vec<ChatMessage>, AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "select role, content
         from (
           select role, content, created_at
           from messages
           where conversation_id = $1 and role in ('user', 'assistant')
           order by created_at desc
           limit 200
         ) t
         order by created_at asc",
    )
    .bind(conversation_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(role, content)| ChatMessage {
            role,
            content,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        })
        .collect())
}

fn compact_messages(
    messages: Vec<ChatMessage>,
    settings: &ChatContextSettings,
) -> (Vec<ChatMessage>, bool) {
    if !settings.compaction_enabled || messages.len() <= settings.max_messages {
        return (messages, false);
    }

    let keep_last = settings.keep_last_messages.min(messages.len());
    let split_at = messages.len().saturating_sub(keep_last);
    let older_messages = &messages[..split_at];
    let recent_messages = &messages[split_at..];
    let summary = build_compact_summary(older_messages, settings.max_summary_chars);

    let mut compacted = Vec::with_capacity(recent_messages.len() + 1);
    compacted.push(ChatMessage {
        role: "system".to_owned(),
        content: summary,
        name: None,
        tool_call_id: None,
        tool_calls: None,
    });
    compacted.extend(recent_messages.iter().cloned());

    (compacted, true)
}

fn build_compact_summary(messages: &[ChatMessage], max_chars: usize) -> String {
    let mut summary = format!(
        "Resumo compacto das {} mensagens anteriores da conversa. Use este resumo apenas como contexto; as mensagens mais recentes continuam completas.\n",
        messages.len()
    );

    for message in messages {
        let role = match message.role.as_str() {
            "user" => "Usuario",
            "assistant" => "Assistente",
            other => other,
        };
        let content = normalize_summary_text(&message.content);
        let line = format!("- {role}: {content}\n");

        if summary.len() + line.len() > max_chars {
            summary.push_str("- ...conteudo antigo restante omitido pelo compactador.\n");
            break;
        }

        summary.push_str(&line);
    }

    summary
}

fn normalize_summary_text(content: &str) -> String {
    const PER_MESSAGE_LIMIT: usize = 600;
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= PER_MESSAGE_LIMIT {
        return normalized;
    }

    normalized
        .chars()
        .take(PER_MESSAGE_LIMIT)
        .collect::<String>()
        + "..."
}

async fn stream_chat(
    Path(conversation_id): Path<Uuid>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let chunks = vec![
        format!("conversation:{conversation_id}"),
        "streaming habilitado".to_owned(),
        "[DONE]".to_owned(),
    ];
    Sse::new(stream::iter(
        chunks
            .into_iter()
            .map(|chunk| Ok(Event::default().data(chunk))),
    ))
}
