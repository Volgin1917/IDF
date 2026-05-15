use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use ice_data_core::error::AppError;
use ice_data_core::models::ApiResponse;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::analytics_engine;
use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct LeadersQuery {
    metric: String,
    season: Option<String>,
    position: Option<String>,
    limit: Option<i32>,
    min_games: Option<i32>,
}

pub async fn leaders(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LeadersQuery>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(200);
    let min_games = query.min_games.unwrap_or(20);
    let season = query.season.unwrap_or_else(current_season);

    let metric_col = match query.metric.as_str() {
        "points" => "pss.points",
        "goals" => "pss.goals",
        "assists" => "pss.assists",
        "games" => "pss.games_played",
        "plus_minus" => "pss.plus_minus",
        "shots" => "pss.shots",
        "points_per_game" => "pss.points_per_game",
        "shooting_percentage" => "pss.shooting_percentage",
        m => return Err(AppError::Validation(format!("Unknown metric: {m}")).into()),
    };

    // Use COALESCE for NULL-safe ordering
    let query_str = format!(
        "SELECT p.nhl_player_id, p.full_name, p.position, p.current_team_abbreviation,
                pss.season, pss.games_played, pss.goals, pss.assists, pss.points,
                pss.plus_minus, pss.shots, pss.points_per_game, pss.shooting_percentage,
                pss.advanced_metrics
         FROM players p
         JOIN player_season_stats pss ON p.nhl_player_id = pss.nhl_player_id
         WHERE pss.season = $1
           AND pss.league = 'NHL'
           AND pss.games_played >= $2
           AND ($3::TEXT IS NULL OR p.position = $3)
         ORDER BY COALESCE({metric_col}, 0) DESC
         LIMIT $4",
    );

    let rows = sqlx::query(&query_str)
        .bind(&season)
        .bind(min_games)
        .bind(&query.position)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let leaders: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let adv: Option<Value> = r.get("advanced_metrics");
            serde_json::json!({
                "rank": i + 1,
                "player_id": r.get::<i32, _>("nhl_player_id"),
                "name": r.get::<String, _>("full_name"),
                "team": r.get::<Option<String>, _>("current_team_abbreviation"),
                "position": r.get::<String, _>("position"),
                "value": r.get::<Option<i32>, _>(query.metric.as_str()).unwrap_or(0),
                "games": r.get::<Option<i32>, _>("games_played").unwrap_or(0),
                "advanced": adv,
            })
        })
        .collect();

    Ok(Json(ApiResponse {
        data: serde_json::json!({ "metric": query.metric, "season": season, "leaders": leaders }),
    }))
}

pub async fn player_timeline(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let rows = sqlx::query(
        "SELECT season, games_played, goals, assists, points, plus_minus, shots,
                time_on_ice_per_game, advanced_metrics
         FROM player_season_stats
         WHERE nhl_player_id = $1 AND league = 'NHL'
         ORDER BY season",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let seasons: Vec<Value> = rows
        .iter()
        .map(|r| {
            let adv: Option<Value> = r.get("advanced_metrics");
            serde_json::json!({
                "season": r.get::<String, _>("season"),
                "games": r.get::<i32, _>("games_played"),
                "goals": r.get::<i32, _>("goals"),
                "assists": r.get::<i32, _>("assists"),
                "points": r.get::<i32, _>("points"),
                "plus_minus": r.get::<i32, _>("plus_minus"),
                "shots": r.get::<i32, _>("shots"),
                "toi_per_game": r.get::<Option<String>, _>("time_on_ice_per_game"),
                "advanced_metrics": adv,
            })
        })
        .collect();

    Ok(Json(ApiResponse {
        data: serde_json::json!({ "player_id": id, "seasons": seasons }),
    }))
}

#[derive(Deserialize)]
pub struct RecalcQuery {
    player_id: Option<i32>,
    season: Option<String>,
}

pub async fn recalculate(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RecalcQuery>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let count = if let Some(pid) = query.player_id {
        recalc_player(&state.pool, pid, query.season.as_deref()).await?
    } else {
        recalc_all(&state.pool, query.season.as_deref()).await?
    };

    Ok(Json(ApiResponse {
        data: serde_json::json!({ "recalculated": count, "status": "ok" }),
    }))
}

async fn recalc_player(pool: &PgPool, player_id: i32, season: Option<&str>) -> Result<i32, ApiError> {
    let rows = sqlx::query(
        r#"SELECT pss.id, pss.games_played, pss.goals, pss.assists, pss.points,
                  pss.plus_minus, pss.shots, pss.time_on_ice_per_game,
                  p.position
           FROM player_season_stats pss
           JOIN players p ON p.nhl_player_id = pss.nhl_player_id
           WHERE pss.nhl_player_id = $1
             AND ($2::TEXT IS NULL OR pss.season = $2)"#,
    )
    .bind(player_id)
    .bind(season)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if rows.is_empty() {
        return Err(AppError::NotFound(format!("Stats for player {player_id}")).into());
    }

    let mut count = 0;
    for r in &rows {
        let stats_id: i64 = r.get("id");
        let games: i32 = r.get("games_played");
        let goals: i32 = r.get("goals");
        let assists: i32 = r.get("assists");
        let points: i32 = r.get("points");
        let plus_minus: i32 = r.get("plus_minus");
        let shots: i32 = r.get("shots");
        let toi: Option<String> = r.get("time_on_ice_per_game");
        let position: String = r.get("position");

        // Try to fetch existing corsi/fenwick from raw_api_data or advanced_metrics
        let existing = sqlx::query("SELECT advanced_metrics, raw_api_data FROM player_season_stats WHERE id = $1")
            .bind(stats_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let (corsi_for, corsi_against, fenwick_for, fenwick_against) =
            if let Some(row) = existing {
                let raw: Option<Value> = row.get("raw_api_data");
                match raw {
                    Some(r) => parse_shot_attempts(&r),
                    None => (None, None, None, None),
                }
            } else {
                (None, None, None, None)
            };

        let ppg = analytics_engine::points_per_game(games, points);

        let advanced = analytics_engine::compute_advanced_metrics(
            games, goals, assists, points, shots, plus_minus,
            toi.as_deref(), &position,
            corsi_for, corsi_against, fenwick_for, fenwick_against,
        );

        sqlx::query(
            "UPDATE player_season_stats SET
                points_per_game = $1, advanced_metrics = $2, calculated_at = NOW()
             WHERE id = $3",
        )
        .bind(ppg)
        .bind(&advanced)
        .bind(stats_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        count += 1;
    }

    // Cascade: update player's search ranking hint
    sqlx::query("UPDATE players SET last_api_sync = NOW() WHERE nhl_player_id = $1")
        .bind(player_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(count)
}

async fn recalc_all(pool: &PgPool, season: Option<&str>) -> Result<i32, ApiError> {
    let players: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT nhl_player_id FROM player_season_stats
         WHERE $1::TEXT IS NULL OR season = $1",
    )
    .bind(season)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut total = 0;
    for pid in players {
        total += recalc_player(pool, pid, season).await
            .map_err(|e| AppError::Internal(format!("Failed for player {pid}: {}", e.0)))?;
    }
    Ok(total)
}

fn parse_shot_attempts(raw: &Value) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let cf = raw.get("corsiFor").and_then(|v| v.as_f64());
    let ca = raw.get("corsiAgainst").and_then(|v| v.as_f64());
    let ff = raw.get("fenwickFor").and_then(|v| v.as_f64());
    let fa = raw.get("fenwickAgainst").and_then(|v| v.as_f64());
    (cf, ca, ff, fa)
}

fn current_season() -> String {
    let year = chrono::Utc::now().format("%Y").to_string();
    let y: i32 = year.parse().unwrap_or(2025);
    format!("{}{}", y - 1, y)
}
