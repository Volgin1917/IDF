mod ai;
mod analytics;
mod analytics_engine;
mod cache;
mod error;
mod middleware;
mod news;
mod players;
mod teams;
mod webhooks;

use std::sync::Arc;

use axum::{routing::get, routing::post, Json, Router};
use ice_data_core::config::AppConfig;
use ice_data_core::models::ApiResponse;
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub cache: cache::CacheManager,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = AppConfig::from_env();

    let pool = ice_data_db::pool::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    ice_data_db::migrations::run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrated successfully");

    let state = Arc::new(AppState {
        pool,
        cache: cache::CacheManager::new(config.cache_ttl_seconds),
        config: config.clone(),
    });

    let v1_routes = Router::new()
        .route("/players/search", get(players::search))
        .route("/players/{id}", get(players::get_by_id))
        .route("/players/{id}/stats", get(players::get_stats))
        .route("/players/{id}/analysis", get(players::get_analysis))
        .route("/players/{id}/ai-analysis", get(ai::analyze))
        .route("/players/compare", post(players::compare))
        .route("/teams", get(teams::list))
        .route("/teams/{abbreviation}/roster", get(teams::roster))
        .route("/news", get(news::list))
        .route("/analytics/leaders", get(analytics::leaders))
        .route("/analytics/player/{id}/timeline", get(analytics::player_timeline))
        .route("/analytics/recalculate", get(analytics::recalculate));

    let webhook_routes = Router::new()
        .route("/webhook/nhl-sync", post(webhooks::nhl_sync))
        .route("/webhook/analyze-players", post(webhooks::analyze_players))
        .route("/webhook/collect-news", post(webhooks::collect_news))
        .layer(axum::middleware::from_fn(middleware::webhook_auth::webhook_auth));

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/v1", v1_routes)
        .nest("/v1", webhook_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = std::net::SocketAddr::new(
        config.host.parse().expect("Invalid host address"),
        config.port,
    );
    tracing::info!("ICE DATA FORGE API listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        data: json!({
            "status": "ok",
            "service": "ice-data-api",
            "version": env!("CARGO_PKG_VERSION"),
        }),
    })
}
