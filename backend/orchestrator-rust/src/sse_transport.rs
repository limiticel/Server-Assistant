use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use axum::response::sse::Event;
use serde_json::{json, Value};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::auth::UserContext;
use crate::registry::{Registry, Tool};

pub type SseEvent = Result<Event, Infallible>;

pub struct SseSessions {
    next_id: AtomicU64,
    sessions: Mutex<HashMap<String, Session>>,
}

struct Session {
    ctx: UserContext,
    tx: UnboundedSender<SseEvent>,
}

impl SseSessions {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn create(&self, ctx: UserContext) -> (String, UnboundedReceiver<SseEvent>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).to_string();
        let (tx, rx) = unbounded_channel();

        self.sessions
            .lock()
            .expect("sse sessions mutex poisoned")
            .insert(id.clone(), Session { ctx, tx });

        (id, rx)
    }

    pub fn remove(&self, session_id: &str) {
        self.sessions
            .lock()
            .expect("sse sessions mutex poisoned")
            .remove(session_id);
    }

    pub fn handle_message(
        &self,
        session_id: &str,
        payload: Value,
        registry: &Registry,
        server_name: &str,
        server_version: &str,
    ) -> Result<(), String> {
        let (ctx, tx) = {
            let sessions = self.sessions.lock().expect("sse sessions mutex poisoned");
            let session = sessions
                .get(session_id)
                .ok_or_else(|| "SSE session not found".to_string())?;

            (session.ctx.clone(), session.tx.clone())
        };

        let Some(response) = handle_json_rpc(payload, registry, &ctx, server_name, server_version)
        else {
            return Ok(());
        };

        let data = serde_json::to_string(&response).map_err(|err| err.to_string())?;
        if tx
            .send(Ok(Event::default().event("message").data(data)))
            .is_err()
        {
            self.remove(session_id);
            return Err("SSE session is closed".to_string());
        }

        Ok(())
    }
}

pub(crate) fn handle_json_rpc(
    payload: Value,
    registry: &Registry,
    ctx: &UserContext,
    server_name: &str,
    server_version: &str,
) -> Option<Value> {
    let method = payload
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let id = payload.get("id").cloned();

    if method.starts_with("notifications/") {
        return None;
    }

    let Some(id) = id else {
        return None;
    };

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": false }
            },
            "serverInfo": {
                "name": server_name,
                "version": server_version
            }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": registry
                .tools_for_role(&ctx.role)
                .into_iter()
                .map(tool_to_mcp_json)
                .collect::<Vec<_>>()
        })),
        "tools/call" => call_tool(
            payload.get("params").cloned().unwrap_or_else(|| json!({})),
            registry,
            ctx,
        ),
        _ => Err(json_rpc_error(
            -32601,
            format!("Method not found: {method}"),
        )),
    };

    match result {
        Ok(result) => json_rpc_response(id, result),
        Err(error) => json_rpc_response_error(id, error),
    }
    .into()
}

fn call_tool(params: Value, registry: &Registry, ctx: &UserContext) -> Result<Value, Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| json_rpc_error(-32602, "Missing tool name"))?;

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let tool = registry
        .find_for_role(name, &ctx.role)
        .ok_or_else(|| json_rpc_error(-32602, format!("Tool not found: {name}")))?;

    if let Some(missing) = missing_required_argument(&tool, &arguments) {
        return Err(json_rpc_error(
            -32602,
            format!("Missing required parameter: {missing}"),
        ));
    }

    let args_keys: Vec<String> = arguments
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let start = std::time::Instant::now();
    let result = (tool.handler)(&arguments, registry);
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let success = result.is_ok();

    crate::auth::log_tool_call(ctx, name, &args_keys, duration_ms, success);

    let result = result.map_err(|message| json_rpc_error(-32000, message))?;

    let text = match result {
        Value::String(text) => text,
        value => serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
    };

    Ok(json!({
        "content": [
            { "type": "text", "text": text }
        ],
        "isError": false
    }))
}

fn tool_to_mcp_json(tool: Tool) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "inputSchema": tool.input_schema,
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

fn json_rpc_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn json_rpc_response_error(id: Value, error: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": error,
    })
}

fn json_rpc_error(code: i64, message: impl Into<String>) -> Value {
    json!({
        "code": code,
        "message": message.into(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::auth::UserContext;
    use crate::config::Settings;
    use crate::tools::build_registry;

    use super::SseSessions;

    #[test]
    fn initialize_message_is_accepted() {
        let settings = Settings::from_env();
        let registry = build_registry(&settings);
        let sessions = SseSessions::new();
        let (session_id, _rx) = sessions.create(admin_ctx());

        let result = sessions.handle_message(
            &session_id,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {}
            }),
            &registry,
            "test-server",
            "0.1.0",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn tools_call_executes_registered_tool() {
        let settings = Settings::from_env();
        let registry = build_registry(&settings);
        let sessions = SseSessions::new();
        let (session_id, _rx) = sessions.create(admin_ctx());

        let result = sessions.handle_message(
            &session_id,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "buscar_cliente",
                    "arguments": { "query": "Empresa" }
                }
            }),
            &registry,
            "test-server",
            "0.1.0",
        );

        assert!(result.is_ok());
    }

    fn admin_ctx() -> UserContext {
        UserContext {
            user_id: "dev".to_string(),
            role: "admin".to_string(),
            api_key: "test".to_string(),
            connected_at: std::time::SystemTime::now(),
        }
    }
}
