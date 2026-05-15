use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use ice_data_core::error::AppError;
use ice_data_core::models::{ApiResponse, Player};
use serde::Deserialize;

use sqlx::Row;

use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i32>,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Player>>>, ApiError> {
    if query.q.len() < 2 {
        return Err(AppError::Validation("Query must be at least 2 characters".into()).into());
    }

    let limit = query.limit.unwrap_or(10).min(50);
    let players = ice_data_db::queries::search_players(&state.pool, &query.q, limit)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApiResponse { data: players }))
}

pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<Player>>, ApiError> {
    let player = ice_data_db::queries::get_player_by_nhl_id(&state.pool, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Player {id}")))?;

    Ok(Json(ApiResponse { data: player }))
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let _player = ice_data_db::queries::get_player_by_nhl_id(&state.pool, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Player {id}")))?;

    let rows = sqlx::query(
        "SELECT id, season, team_abbreviation, league,
                games_played, goals, assists, points, plus_minus,
                penalty_minutes, shots, shooting_percentage,
                time_on_ice_per_game, created_at, updated_at
         FROM player_season_stats
         WHERE nhl_player_id = $1
           AND ($2::TEXT IS NULL OR season = $2)
         ORDER BY season DESC",
    )
    .bind(id)
    .bind(&query.season)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let seasons: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({
            "season": r.get::<String, _>("season"),
            "team": r.get::<Option<String>, _>("team_abbreviation"),
            "games_played": r.get::<i32, _>("games_played"),
            "goals": r.get::<i32, _>("goals"),
            "assists": r.get::<i32, _>("assists"),
            "points": r.get::<i32, _>("points"),
            "plus_minus": r.get::<i32, _>("plus_minus"),
            "shots": r.get::<i32, _>("shots"),
        })
    }).collect();

    Ok(Json(ApiResponse { data: serde_json::json!({ "seasons": seasons }) }))
}

pub async fn get_analysis(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let rows = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT COALESCE(strengths, '[]'::jsonb) || COALESCE(weaknesses, '[]'::jsonb) as analysis_data
         FROM ai_analysis
         WHERE nhl_player_id = $1
           AND ($2::TEXT IS NULL OR season = $2)
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(id)
    .bind(query.season)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let data = rows.unwrap_or(serde_json::json!({"message": "No analysis available"}));
    Ok(Json(ApiResponse { data }))
}

#[derive(Deserialize)]
pub struct StatsQuery {
    season: Option<String>,
}

#[derive(Deserialize)]
pub struct AnalysisQuery {
    season: Option<String>,
}

#[derive(Deserialize)]
pub struct CompareRequest {
    player_ids: Vec<i32>,
    season: Option<String>,
}

pub async fn compare(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CompareRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.player_ids.len() < 2 || body.player_ids.len() > 5 {
        return Err(AppError::Validation(
            "Must compare between 2 and 5 players".into(),
        ).into());
    }

    let mut players_data = Vec::new();
    for pid in &body.player_ids {
        if let Some(player) = ice_data_db::queries::get_player_by_nhl_id(&state.pool, *pid)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            let rows = sqlx::query(
                "SELECT season, team_abbreviation, games_played, goals, assists, points
                 FROM player_season_stats
                 WHERE nhl_player_id = $1
                   AND ($2::TEXT IS NULL OR season = $2)
                 ORDER BY season DESC LIMIT 5",
            )
            .bind(pid)
            .bind(&body.season)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            let seasons: Vec<serde_json::Value> = rows.iter().map(|r| {
                serde_json::json!({
                    "season": r.get::<String, _>("season"),
                    "games_played": r.get::<i32, _>("games_played"),
                    "goals": r.get::<i32, _>("goals"),
                    "assists": r.get::<i32, _>("assists"),
                    "points": r.get::<i32, _>("points"),
                })
            }).collect();

            players_data.push(serde_json::json!({
                "player": player,
                "seasons": seasons
            }));
        }
    }

    Ok(Json(ApiResponse {
        data: serde_json::json!({ "comparison": players_data }),
    }))
}
