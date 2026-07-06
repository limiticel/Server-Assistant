mod auth;
mod config;
mod http;
mod rate_limit;
mod registry;
mod sse_transport;
mod stdio_transport;
mod tools;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use tracing::info;

use crate::config::Settings;
use crate::http::router;
use crate::rate_limit::RateLimiter;
use crate::registry::Registry;
use crate::sse_transport::SseSessions;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub registry: Arc<Registry>,
    pub rate_limiter: Arc<RateLimiter>,
    pub sse_sessions: Arc<SseSessions>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()
        .or_else(|_| dotenvy::from_filename("../.env"))
        .or_else(|_| dotenvy::from_filename("../../.env"))
        .ok();

    let settings = Settings::from_env();
    init_logging(&settings);

    let registry = Arc::new(tools::build_registry(&settings));

    // Suporte a stdio (para Claude Desktop e uso local)
    // Ative com: TRANSPORT=stdio ./mcp-orchestrator-rust
    // ou passe --stdio como argumento
    let use_stdio = settings.transport == "stdio" || std::env::args().any(|a| a == "--stdio");

    if use_stdio {
        return stdio_transport::run(&registry, &settings.server_name, &settings.server_version)
            .await;
    }

    let rate_limiter = Arc::new(RateLimiter::new(
        settings.rate_limit_per_minute,
        settings.rate_limit_burst,
    ));
    let sse_sessions = Arc::new(SseSessions::new());

    let state = AppState {
        settings: settings.clone(),
        registry,
        rate_limiter,
        sse_sessions,
    };

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port)
        .parse()
        .context("invalid HOST/PORT")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "starting mcp-orchestrator-rust (HTTP+SSE)");

    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

fn init_logging(settings: &Settings) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| settings.log_level.clone().into());

    if settings.log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
