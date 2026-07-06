mod api;
mod application;
mod auth;
mod config;
mod domain;
mod infrastructure;
mod shared;
mod telemetry;

use std::net::SocketAddr;

use api::router;
use config::Settings;
use infrastructure::{database::connect_pool, providers::ProviderRegistry};
use telemetry::init_tracing;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub db: sqlx::PgPool,
    pub providers: ProviderRegistry,
    pub orchestrator: infrastructure::mcp::OrchestratorClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()
        .or_else(|_| dotenvy::from_filename("../.env"))
        .ok();
    let settings = Settings::from_env()?;
    init_tracing(&settings);

    let db = connect_pool(&settings.database_url).await?;
    let providers = ProviderRegistry::from_settings(&settings);
    let orchestrator = infrastructure::mcp::OrchestratorClient::new(
        settings.mcp_orchestrator_url.clone(),
        settings.mcp_orchestrator_api_key.clone(),
    );

    let state = AppState {
        settings: settings.clone(),
        db,
        providers,
        orchestrator,
    };

    let app = router::build(state);
    let addr: SocketAddr = format!("{}:{}", settings.app_host, settings.app_port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "server-assistant backend started");
    axum::serve(listener, app).await?;
    Ok(())
}
