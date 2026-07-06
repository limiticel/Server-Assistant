use std::env;

#[derive(Clone, Debug)]
pub struct Settings {
    // Server
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub server_name: String,
    pub server_version: String,
    // Auth
    pub secret_key: String,
    pub api_keys: Vec<String>,
    pub jwt_algorithm: String,
    pub jwt_expire_minutes: u32,
    // CORS
    pub cors_origins: Vec<String>,
    // Rate limiting
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
    // Database / Redis
    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub database_url: Option<String>,
    pub redis_url: String,
    // Logging
    pub log_level: String,
    pub log_format: String,
    // Modules
    pub enable_sales: bool,
    pub enable_devtools: bool,
    pub enable_internal_systems: bool,
    pub enable_server_admin_tools: bool,
    pub rag_activate: bool,
    // Transport: "http" | "stdio"
    pub transport: String,
}

impl Settings {
    pub fn from_env() -> Self {
        Self {
            host: env_string("HOST", "0.0.0.0"),
            port: env_parse("PORT", 8016),
            debug: env_bool("DEBUG", true),
            server_name: env_string("SERVER_NAME", "corporate-mcp-server"),
            server_version: env_string("SERVER_VERSION", "1.0.0"),
            secret_key: env_string("SECRET_KEY", "change-me-in-production"),
            api_keys: env_list("API_KEYS"),
            jwt_algorithm: env_string("JWT_ALGORITHM", "HS256"),
            jwt_expire_minutes: env_parse("JWT_EXPIRE_MINUTES", 480),
            cors_origins: env_list_or("CORS_ORIGINS", vec!["*".to_string()]),
            rate_limit_per_minute: env_parse("RATE_LIMIT_PER_MINUTE", 60),
            rate_limit_burst: env_parse("RATE_LIMIT_BURST", 10),
            database_host: env_string("DATABASE_HOST", "localhost"),
            database_port: env_parse("DATABASE_PORT", 5432),
            database_user: env_string("DATABASE_USER", "user"),
            database_password: env_string("DATABASE_PASSWORD", "password"),
            database_name: env_string("DATABASE_NAME", "mcp_database"),
            database_url: env::var("DATABASE_URL").ok(),
            redis_url: env_string("REDIS_URL", "redis://localhost:6379/0"),
            log_level: env_string("LOG_LEVEL", "info").to_lowercase(),
            log_format: env_string("LOG_FORMAT", "text").to_lowercase(),
            enable_sales: env_bool("ENABLE_SALES", true),
            enable_devtools: env_bool("ENABLE_DEVTOOLS", true),
            enable_internal_systems: env_bool("ENABLE_INTERNAL_SYSTEMS", true),
            enable_server_admin_tools: env_bool("ENABLE_SERVER_ADMIN_TOOLS", true),
            rag_activate: env_bool("RAG_ACTIVATE", false),
            transport: env_string("TRANSPORT", "http").to_lowercase(),
        }
    }
}

fn env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn env_parse<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<T>().ok())
        .unwrap_or(default)
}

fn env_list(key: &str) -> Vec<String> {
    env::var(key)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn env_list_or(key: &str, default: Vec<String>) -> Vec<String> {
    let values = env_list(key);
    if values.is_empty() {
        default
    } else {
        values
    }
}
