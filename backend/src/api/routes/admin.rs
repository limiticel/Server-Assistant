use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{shared::AppError, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/:id", axum::routing::delete(delete_provider))
        .route("/models", get(list_models).post(create_model))
        .route(
            "/models/:id/persona",
            axum::routing::put(update_model_persona),
        )
        .route(
            "/models/:id/tools",
            get(list_model_tools).put(update_model_tools),
        )
        .route("/mcp-tools", get(list_mcp_tools).post(create_mcp_tool))
        .route(
            "/mcp-tools/:id",
            axum::routing::delete(delete_mcp_tool).put(update_mcp_tool),
        )
        .route("/:resource", get(resource_table))
}

#[derive(Deserialize)]
struct ProviderPayload {
    name: String,
    base_url: String,
    api_key: Option<String>,
    default_model: Option<String>,
    provider_type: String,
    openai_compatible: Option<bool>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct ModelPayload {
    provider_id: Uuid,
    name: String,
    context_window: Option<i32>,
    input_price: Option<f64>,
    output_price: Option<f64>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct McpToolPayload {
    name: String,
    description: String,
    tool_type: String,
    input_schema: Value,
    config: Value,
    response_schema: Option<Value>,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
struct ModelToolsPayload {
    tool_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
struct ModelPersonaPayload {
    assistant_name: Option<String>,
    personality: Option<String>,
    temperament: Option<String>,
    pre_prompt: Option<String>,
    pre_prompt_limit: Option<i32>,
}

async fn dashboard(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let users: (i64,) = sqlx::query_as("select count(*) from users")
        .fetch_one(&state.db)
        .await?;
    let chats: (i64,) = sqlx::query_as("select count(*) from conversations")
        .fetch_one(&state.db)
        .await?;
    let messages: (i64,) = sqlx::query_as("select count(*) from messages")
        .fetch_one(&state.db)
        .await?;
    let billable: (Option<i64>, Option<i64>, Option<i64>, Option<String>) = sqlx::query_as(
        "select
           sum(t.prompt_tokens)::bigint,
           sum(t.completion_tokens)::bigint,
           sum(t.total_tokens)::bigint,
           coalesce(sum(t.estimated_cost), 0)::text
         from token_usage t
         left join providers p on p.id::text = t.provider
         where lower(coalesce(p.provider_type, t.provider)) in ('openai', 'anthropic', 'claude')
            or lower(coalesce(p.name, t.provider)) = 'openai'",
    )
    .fetch_one(&state.db)
    .await?;

    let billable_by_provider: serde_json::Value = sqlx::query_scalar(
        "select coalesce(json_agg(row_to_json(t)), '[]'::json)
         from (
           select
             coalesce(p.name, token_usage.provider) as provider,
             lower(coalesce(p.provider_type, token_usage.provider)) as provider_type,
             sum(token_usage.prompt_tokens)::bigint as prompt_tokens,
             sum(token_usage.completion_tokens)::bigint as completion_tokens,
             sum(token_usage.total_tokens)::bigint as total_tokens,
             coalesce(sum(token_usage.estimated_cost), 0)::text as estimated_cost
           from token_usage
           left join providers p on p.id::text = token_usage.provider
           where lower(coalesce(p.provider_type, token_usage.provider)) in ('openai', 'anthropic', 'claude')
              or lower(coalesce(p.name, token_usage.provider)) = 'openai'
           group by coalesce(p.name, token_usage.provider), lower(coalesce(p.provider_type, token_usage.provider))
           order by total_tokens desc
         ) t",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "general": {
            "users": users.0,
            "conversations": chats.0,
            "messages": messages.0,
            "configured_env_providers": state.providers.names()
        },
        "billable_usage": {
            "scope": "openai_and_claude_only",
            "prompt_tokens": billable.0.unwrap_or(0),
            "completion_tokens": billable.1.unwrap_or(0),
            "total_tokens": billable.2.unwrap_or(0),
            "estimated_cost": billable.3.unwrap_or_else(|| "0".to_owned()),
            "by_provider": billable_by_provider
        }
    })))
}

async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: serde_json::Value = sqlx::query_scalar(
        "select coalesce(json_agg(t), '[]'::json)
         from (
           select id, name, base_url, default_model, provider_type, openai_compatible,
                  active, health_status, api_key_cipher is not null as has_api_key,
                  created_at, updated_at
           from providers
           order by created_at desc
         ) t",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn create_provider(
    State(state): State<AppState>,
    Json(payload): Json<ProviderPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("provider name is required".to_owned()));
    }
    if payload.base_url.trim().is_empty() {
        return Err(AppError::Validation("base_url is required".to_owned()));
    }

    let id = Uuid::new_v4();
    let api_key = payload
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    sqlx::query(
        "insert into providers
         (id, name, base_url, api_key_cipher, default_model, provider_type, openai_compatible, active, health_status)
         values ($1, $2, $3, $4, $5, $6, $7, $8, 'unchecked')",
    )
    .bind(id)
    .bind(payload.name.trim())
    .bind(payload.base_url.trim().trim_end_matches('/'))
    .bind(api_key)
    .bind(payload.default_model.as_deref().map(str::trim).filter(|value| !value.is_empty()))
    .bind(payload.provider_type.trim())
    .bind(payload.openai_compatible.unwrap_or(true))
    .bind(payload.active.unwrap_or(true))
    .execute(&state.db)
    .await?;

    if let Some(model) = payload
        .default_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sqlx::query("insert into models (id, provider_id, name, active) values ($1, $2, $3, true)")
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(model)
            .execute(&state.db)
            .await?;
    }

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

async fn delete_provider(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("delete from providers where id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({
        "deleted": true,
        "id": id
    })))
}

async fn list_models(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let rows: serde_json::Value = sqlx::query_scalar(
        "select coalesce(json_agg(t), '[]'::json)
         from (
           select m.id, m.provider_id, p.name as provider_name, m.name, m.context_window,
                  m.input_price, m.output_price, m.active,
                  m.assistant_name, m.personality, m.temperament, m.pre_prompt, m.pre_prompt_limit,
                  m.created_at, m.updated_at,
                  coalesce(
                    (
                      select json_agg(json_build_object(
                        'id', t.id,
                        'name', t.name,
                        'description', t.description,
                        'enabled', t.enabled
                      ) order by t.name)
                      from model_mcp_tools mt
                      join mcp_tools t on t.id = mt.tool_id
                      where mt.model_id = m.id
                    ),
                    '[]'::json
                  ) as tools
           from models m
           left join providers p on p.id = m.provider_id
           order by m.created_at desc
         ) t",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn list_model_tools(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: serde_json::Value = sqlx::query_scalar(
        "select coalesce(json_agg(t), '[]'::json)
         from (
           select tools.id, tools.name, tools.description, tools.tool_type, tools.enabled,
                  mt.model_id is not null as assigned
           from mcp_tools tools
           left join model_mcp_tools mt on mt.tool_id = tools.id and mt.model_id = $1
           order by tools.name
         ) t",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn update_model_tools(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<ModelToolsPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut transaction = state.db.begin().await?;

    sqlx::query("delete from model_mcp_tools where model_id = $1")
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    for tool_id in payload.tool_ids {
        sqlx::query(
            "insert into model_mcp_tools (model_id, tool_id)
             select $1, $2
             where exists (select 1 from models where id = $1)
               and exists (select 1 from mcp_tools where id = $2)",
        )
        .bind(id)
        .bind(tool_id)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(Json(serde_json::json!({ "saved": true, "model_id": id })))
}

async fn update_model_persona(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<ModelPersonaPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = payload.pre_prompt_limit.unwrap_or(2000).clamp(200, 12000);
    let pre_prompt = clean_optional_text(payload.pre_prompt, limit as usize);

    let result = sqlx::query(
        "update models
         set assistant_name = $2,
             personality = $3,
             temperament = $4,
             pre_prompt = $5,
             pre_prompt_limit = $6,
             updated_at = now()
         where id = $1",
    )
    .bind(id)
    .bind(clean_optional_text(payload.assistant_name, 80))
    .bind(clean_optional_text(payload.personality, 600))
    .bind(clean_optional_text(payload.temperament, 600))
    .bind(pre_prompt)
    .bind(limit)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({
        "id": id,
        "updated": true
    })))
}

fn clean_optional_text(value: Option<String>, max_chars: usize) -> Option<String> {
    value
        .map(|text| text.trim().chars().take(max_chars).collect::<String>())
        .filter(|text| !text.is_empty())
}

async fn create_model(
    State(state): State<AppState>,
    Json(payload): Json<ModelPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("model name is required".to_owned()));
    }

    let id = Uuid::new_v4();
    sqlx::query(
        "insert into models
         (id, provider_id, name, context_window, input_price, output_price, active)
         values ($1, $2, $3, $4, $5::numeric, $6::numeric, $7)",
    )
    .bind(id)
    .bind(payload.provider_id)
    .bind(payload.name.trim())
    .bind(payload.context_window)
    .bind(payload.input_price.unwrap_or(0.0).to_string())
    .bind(payload.output_price.unwrap_or(0.0).to_string())
    .bind(payload.active.unwrap_or(true))
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

async fn list_mcp_tools(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: serde_json::Value = sqlx::query_scalar(
        "select coalesce(json_agg(t), '[]'::json)
         from (
           select id, name, description, tool_type, input_schema, config,
                  response_schema, enabled, created_at, updated_at
           from mcp_tools
           order by created_at desc
         ) t",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn create_mcp_tool(
    State(state): State<AppState>,
    Json(payload): Json<McpToolPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let name = payload.name.trim();
    let description = payload.description.trim();
    let tool_type = payload.tool_type.trim();

    if name.is_empty() {
        return Err(AppError::Validation("tool name is required".to_owned()));
    }
    if description.is_empty() {
        return Err(AppError::Validation(
            "tool description is required".to_owned(),
        ));
    }
    if !matches!(tool_type, "physical" | "abstract") {
        return Err(AppError::Validation(
            "tool_type must be physical or abstract".to_owned(),
        ));
    }
    if !payload.input_schema.is_object() {
        return Err(AppError::Validation(
            "input_schema must be a JSON object".to_owned(),
        ));
    }
    if !payload.config.is_object() {
        return Err(AppError::Validation(
            "config must be a JSON object".to_owned(),
        ));
    }

    let id = Uuid::new_v4();
    sqlx::query(
        "insert into mcp_tools
         (id, name, description, tool_type, input_schema, config, response_schema, enabled)
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(tool_type)
    .bind(payload.input_schema)
    .bind(payload.config)
    .bind(
        payload
            .response_schema
            .unwrap_or_else(|| serde_json::json!({})),
    )
    .bind(payload.enabled.unwrap_or(true))
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

async fn update_mcp_tool(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<McpToolPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let name = payload.name.trim();
    let description = payload.description.trim();
    let tool_type = payload.tool_type.trim();

    if name.is_empty() {
        return Err(AppError::Validation("tool name is required".to_owned()));
    }
    if description.is_empty() {
        return Err(AppError::Validation(
            "tool description is required".to_owned(),
        ));
    }
    if !matches!(tool_type, "physical" | "abstract") {
        return Err(AppError::Validation(
            "tool_type must be physical or abstract".to_owned(),
        ));
    }
    if !payload.input_schema.is_object() {
        return Err(AppError::Validation(
            "input_schema must be a JSON object".to_owned(),
        ));
    }
    if !payload.config.is_object() {
        return Err(AppError::Validation(
            "config must be a JSON object".to_owned(),
        ));
    }

    let result = sqlx::query(
        "update mcp_tools
         set name = $2,
             description = $3,
             tool_type = $4,
             input_schema = $5,
             config = $6,
             response_schema = $7,
             enabled = $8,
             updated_at = now()
         where id = $1",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(tool_type)
    .bind(payload.input_schema)
    .bind(payload.config)
    .bind(
        payload
            .response_schema
            .unwrap_or_else(|| serde_json::json!({})),
    )
    .bind(payload.enabled.unwrap_or(true))
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_mcp_tool(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("delete from mcp_tools where id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}

async fn resource_table(
    Path(resource): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table = match resource.as_str() {
        "users" => "users",
        "providers" => "providers",
        "models" => "models",
        "personalities" => "personalities",
        "token-usage" => "token_usage",
        "mcp-tools" => "mcp_tools",
        _ => return Err(AppError::NotFound),
    };

    let sql = format!("select coalesce(json_agg(t), '[]'::json) from (select * from {table} order by created_at desc limit 100) t");
    let rows: serde_json::Value = sqlx::query_scalar(&sql).fetch_one(&state.db).await?;
    Ok(Json(rows))
}
