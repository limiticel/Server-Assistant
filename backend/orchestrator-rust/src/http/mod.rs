use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::auth::{get_request_context, UserContext};
use crate::registry::Tool;
use crate::AppState;

#[derive(Deserialize)]
struct ToolCallPayload {
    #[serde(default)]
    arguments: Value,
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/sse", get(handle_sse))
        .route("/messages", post(handle_messages))
        .route("/health", get(handle_health))
        .route("/tools", get(handle_tools_debug))
        .route("/api/all-routes", get(handle_all_routes))
        .route("/api/tools", get(handle_api_tools))
        .route("/api/tools/:name", get(handle_api_tool_detail))
        .route("/api/tools/:name/call", post(handle_api_tool_call))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn handle_health(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "server": state.settings.server_name,
        "version": state.settings.server_version,
        "tools_count": state.registry.list_registered_tools().len(),
    }))
}

async fn handle_tools_debug(State(state): State<AppState>) -> impl IntoResponse {
    if !state.settings.debug {
        return error("Forbidden", StatusCode::FORBIDDEN);
    }

    let tools: Vec<Value> = state
        .registry
        .tools_for_role("*")
        .into_iter()
        .map(|tool| json!({ "name": tool.name, "description": tool.description }))
        .collect();

    (StatusCode::OK, Json(Value::Array(tools))).into_response()
}

async fn handle_api_tools(
    State(state): State<AppState>,
    headers: HeaderMap,
    remote_addr: Option<ConnectInfo<SocketAddr>>,
) -> impl IntoResponse {
    let Some(ctx) = request_context(&headers, remote_addr, &state) else {
        return error("Unauthorized", StatusCode::UNAUTHORIZED);
    };

    let tools: Vec<Value> = state
        .registry
        .tools_for_role(&ctx.role)
        .into_iter()
        .map(tool_summary)
        .collect();

    (StatusCode::OK, Json(Value::Array(tools))).into_response()
}

async fn handle_api_tool_detail(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    remote_addr: Option<ConnectInfo<SocketAddr>>,
) -> impl IntoResponse {
    let Some(ctx) = request_context(&headers, remote_addr, &state) else {
        return error("Unauthorized", StatusCode::UNAUTHORIZED);
    };

    match state.registry.find_for_role(&name, &ctx.role) {
        Some(tool) => (StatusCode::OK, Json(tool_summary(tool))).into_response(),
        None => error("Not Found", StatusCode::NOT_FOUND),
    }
}

async fn handle_api_tool_call(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    remote_addr: Option<ConnectInfo<SocketAddr>>,
    body: String,
) -> impl IntoResponse {
    let Some(ctx) = request_context(&headers, remote_addr, &state) else {
        return error("Unauthorized", StatusCode::UNAUTHORIZED);
    };

    if !state.rate_limiter.is_allowed(&ctx.user_id) {
        return error("Too Many Requests", StatusCode::TOO_MANY_REQUESTS);
    }

    let Some(tool) = state.registry.find_for_role(&name, &ctx.role) else {
        return error("Not Found", StatusCode::NOT_FOUND);
    };

    let payload = parse_payload(&body);
    let arguments = payload.arguments;

    if let Some(missing) = missing_required_argument(&tool, &arguments) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Missing required parameter: {missing}") })),
        )
            .into_response();
    }

    let args_keys: Vec<String> = arguments
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let start = std::time::Instant::now();
    let call_result = (tool.handler)(&arguments, &state.registry);
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let success = call_result.is_ok();

    crate::auth::log_tool_call(&ctx, &name, &args_keys, duration_ms, success);

    match call_result {
        Ok(result) => (StatusCode::OK, Json(json!({ "result": result }))).into_response(),
        Err(message) => {
            (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
        }
    }
}

async fn handle_all_routes() -> impl IntoResponse {
    Json(json!([
        { "type": "route", "path": "/sse", "methods": ["GET"], "name": "handle_sse" },
        { "type": "route", "path": "/messages", "methods": ["POST"], "name": "handle_messages" },
        { "type": "route", "path": "/health", "methods": ["GET"], "name": "handle_health" },
        { "type": "route", "path": "/tools", "methods": ["GET"], "name": "handle_tools_debug" },
        { "type": "route", "path": "/api/all-routes", "methods": ["GET"], "name": "handle_all_routes" },
        { "type": "route", "path": "/api/tools", "methods": ["GET"], "name": "handle_api_tools" },
        { "type": "route", "path": "/api/tools/:name", "methods": ["GET"], "name": "handle_api_tool_detail" },
        { "type": "route", "path": "/api/tools/:name/call", "methods": ["POST"], "name": "handle_api_tool_call" }
    ]))
}

async fn handle_sse(
    State(state): State<AppState>,
    headers: HeaderMap,
    remote_addr: Option<ConnectInfo<SocketAddr>>,
) -> axum::response::Response {
    let Some(ctx) = request_context(&headers, remote_addr, &state) else {
        return error("Unauthorized", StatusCode::UNAUTHORIZED);
    };

    if !state.rate_limiter.is_allowed(&ctx.user_id) {
        return error("Too Many Requests", StatusCode::TOO_MANY_REQUESTS);
    }

    let (session_id, rx) = state.sse_sessions.create(ctx);
    let endpoint = format!("/messages?session_id={session_id}");

    let initial = Event::default().event("endpoint").data(endpoint);
    let stream = tokio_stream::once(Ok(initial)).chain(UnboundedReceiverStream::new(rx));

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

async fn handle_messages(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
    body: String,
) -> impl IntoResponse {
    let Some(session_id) = query
        .get("session_id")
        .or_else(|| query.get("sessionId"))
        .or_else(|| query.get("session"))
    else {
        return error("Missing session_id", StatusCode::BAD_REQUEST);
    };

    let payload = match serde_json::from_str::<Value>(&body) {
        Ok(payload) => payload,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid JSON payload", "details": err.to_string() })),
            )
                .into_response()
        }
    };

    match state.sse_sessions.handle_message(
        session_id,
        payload,
        &state.registry,
        &state.settings.server_name,
        &state.settings.server_version,
    ) {
        Ok(()) => (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" }))).into_response(),
        Err(message) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": message, "status": 404 })),
        )
            .into_response(),
    }
}

fn request_context(
    headers: &HeaderMap,
    remote_addr: Option<ConnectInfo<SocketAddr>>,
    state: &AppState,
) -> Option<UserContext> {
    get_request_context(
        headers,
        remote_addr.map(|ConnectInfo(addr)| addr),
        &state.settings,
    )
}

fn parse_payload(body: &str) -> ToolCallPayload {
    if body.trim().is_empty() {
        return ToolCallPayload {
            arguments: json!({}),
        };
    }

    serde_json::from_str::<ToolCallPayload>(body).unwrap_or(ToolCallPayload {
        arguments: json!({}),
    })
}

fn missing_required_argument(tool: &Tool, arguments: &Value) -> Option<String> {
    let required = tool.input_schema.get("required")?.as_array()?;

    for item in required {
        let Some(name) = item.as_str() else {
            continue;
        };

        if arguments.get(name).is_none() {
            return Some(name.to_string());
        }
    }

    None
}

fn tool_summary(tool: Tool) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.input_schema,
        "route": format!("/api/tools/{}/call", tool.name),
    })
}

fn error(message: &str, status: StatusCode) -> axum::response::Response {
    (
        status,
        Json(json!({
            "error": message,
            "status": status.as_u16(),
        })),
    )
        .into_response()
}
