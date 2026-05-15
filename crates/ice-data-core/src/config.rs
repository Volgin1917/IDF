#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub nhl_api_base_url: String,
    pub nhl_api_rate_limit: u64,
    pub cache_ttl_seconds: u64,
    pub openai_api_key: String,
    pub openai_model: String,
    pub n8n_webhook_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            nhl_api_base_url: std::env::var("NHL_API_BASE_URL")
                .unwrap_or_else(|_| "https://api-web.nhle.com/v1".into()),
            nhl_api_rate_limit: std::env::var("NHL_API_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            cache_ttl_seconds: std::env::var("CACHE_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY must be set"),
            openai_model: std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4".into()),
            n8n_webhook_secret: std::env::var("N8N_WEBHOOK_SECRET")
                .unwrap_or_else(|_| "change-me-n8n-secret".into()),
        }
    }
}
