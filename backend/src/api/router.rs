use axum::{routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::AppState;

use super::routes::{admin, auth, chat, gateway, mcp, settings};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/auth", auth::routes())
        .nest("/api", chat::routes())
        .nest("/api/admin", admin::routes())
        .nest("/api/mcp", mcp::routes())
        .nest("/api/settings", settings::routes())
        .nest("/v1", gateway::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
