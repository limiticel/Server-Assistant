use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;

use axum::http::HeaderMap;
use tracing::info;

use crate::config::Settings;

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub role: String,
    pub api_key: String,
    pub connected_at: SystemTime,
}

pub fn get_request_context(
    headers: &HeaderMap,
    remote_addr: Option<SocketAddr>,
    settings: &Settings,
) -> Option<UserContext> {
    let api_key = extract_api_key(headers);
    if let Some(ctx) = authenticate(api_key.as_deref(), settings) {
        return Some(ctx);
    }

    if settings.debug && is_loopback(remote_addr) {
        tracing::warn!("DEBUG fallback auth allowed for local request");
        return Some(UserContext {
            user_id: "dev".to_string(),
            role: "admin".to_string(),
            api_key: "debug".to_string(),
            connected_at: SystemTime::now(),
        });
    }

    None
}

pub fn log_tool_call(
    ctx: &UserContext,
    tool_name: &str,
    args_keys: &[String],
    duration_ms: f64,
    success: bool,
) {
    info!(
        user_id = %ctx.user_id,
        role = %ctx.role,
        tool = tool_name,
        ?args_keys,
        duration_ms,
        success,
        "tool_call"
    );
}

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

fn authenticate(api_key: Option<&str>, settings: &Settings) -> Option<UserContext> {
    let key = api_key?;
    let map = api_key_map(settings);
    let profile = map.get(key)?;

    Some(UserContext {
        user_id: profile.0.clone(),
        role: profile.1.clone(),
        api_key: key.to_string(),
        connected_at: SystemTime::now(),
    })
}

fn api_key_map(settings: &Settings) -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();

    for raw in &settings.api_keys {
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() == 3 {
            map.insert(
                parts[2].to_string(),
                (parts[0].to_string(), parts[1].to_string()),
            );
        }
    }

    map
}

fn is_loopback(remote_addr: Option<SocketAddr>) -> bool {
    match remote_addr.map(|addr| addr.ip()) {
        Some(IpAddr::V4(ip)) => ip.is_loopback(),
        Some(IpAddr::V6(ip)) => ip.is_loopback(),
        None => true,
    }
}
