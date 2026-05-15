use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use ice_data_core::error::AppError;
use ice_data_core::models::ApiResponse;
use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;

use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct NewsQuery {
    player_id: Option<i32>,
    team: Option<String>,
    days: Option<i32>,
    limit: Option<i32>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, ApiError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let days = query.days.unwrap_or(30);

    let rows = sqlx::query(
        "SELECT id, title, source, url, published_at, category, sentiment
         FROM news_events
         WHERE ($1::INT IS NULL OR player_ids @> ARRAY[$1]::INTEGER[])
           AND ($2::TEXT IS NULL OR team_abbreviations @> ARRAY[$2]::TEXT[])
           AND published_at >= NOW() - ($3 || ' days')::INTERVAL
         ORDER BY published_at DESC
         LIMIT $4",
    )
    .bind(query.player_id)
    .bind(&query.team)
    .bind(days.to_string())
    .bind(limit as i64)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let news: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "title": r.get::<String, _>("title"),
                "source": r.get::<String, _>("source"),
                "url": r.get::<String, _>("url"),
                "published_at": r.get::<chrono::DateTime<chrono::Utc>, _>("published_at"),
                "category": r.get::<Option<String>, _>("category"),
                "sentiment": r.get::<Option<String>, _>("sentiment"),
            })
        })
        .collect();

    Ok(Json(ApiResponse { data: news }))
}
