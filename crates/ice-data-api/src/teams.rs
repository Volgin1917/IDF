use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use ice_data_core::error::AppError;
use ice_data_core::models::ApiResponse;
use serde_json::Value;
use sqlx::Row;

use crate::error::ApiError;
use crate::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Value>>>, ApiError> {
    let rows = sqlx::query(
        "SELECT nhl_team_id, abbreviation, team_name, location, conference, division
         FROM teams WHERE active = true ORDER BY team_name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let teams: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<i32, _>("nhl_team_id"),
                "abbreviation": r.get::<String, _>("abbreviation"),
                "team_name": r.get::<String, _>("team_name"),
                "location": r.get::<String, _>("location"),
                "conference": r.get::<Option<String>, _>("conference"),
                "division": r.get::<Option<String>, _>("division"),
            })
        })
        .collect();

    Ok(Json(ApiResponse { data: teams }))
}

pub async fn roster(
    State(state): State<Arc<AppState>>,
    Path(abbreviation): Path<String>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let team_abbrev = abbreviation.to_uppercase();

    let rows = sqlx::query(
        "SELECT nhl_player_id, full_name, position, jersey_number
         FROM players
         WHERE current_team_abbreviation = $1 AND is_active = true
         ORDER BY position, last_name",
    )
    .bind(&team_abbrev)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let roster: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "player_id": r.get::<i32, _>("nhl_player_id"),
                "name": r.get::<String, _>("full_name"),
                "position": r.get::<String, _>("position"),
                "jersey_number": r.get::<Option<i32>, _>("jersey_number"),
            })
        })
        .collect();

    Ok(Json(ApiResponse {
        data: serde_json::json!({
            "team": team_abbrev,
            "roster": roster,
        }),
    }))
}
