use std::env;

use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Settings {
    pub app_env: String,
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub cors_origins: Vec<String>,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub ollama_base_url: String,
    pub mcp_orchestrator_url: String,
    pub mcp_orchestrator_api_key: Option<String>,
    pub rust_log: String,
}

impl Settings {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            app_env: env_or("APP_ENV", "development"),
            app_host: env_or("APP_HOST", "0.0.0.0"),
            app_port: env_or("APP_PORT", "8080")
                .parse()
                .context("APP_PORT invalid")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET is required")?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .context("JWT_REFRESH_SECRET is required")?,
            cors_origins: env_or("CORS_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .collect(),
            openai_api_key: optional_env("OPENAI_API_KEY"),
            anthropic_api_key: optional_env("ANTHROPIC_API_KEY"),
            gemini_api_key: optional_env("GEMINI_API_KEY"),
            deepseek_api_key: optional_env("DEEPSEEK_API_KEY"),
            ollama_base_url: env_or("OLLAMA_BASE_URL", "http://localhost:11434"),
            mcp_orchestrator_url: env_or("MCP_ORCHESTRATOR_URL", "http://localhost:8016"),
            mcp_orchestrator_api_key: optional_env("MCP_ORCHESTRATOR_API_KEY"),
            rust_log: env_or("RUST_LOG", "info"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}
