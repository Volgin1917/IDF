use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use ice_data_core::error::AppError;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::AppState;

/// n8n → наш API: NHL данные для синхронизации
#[derive(Deserialize)]
pub struct TeamSyncItem {
    abbreviation: String,
    team_name: Option<String>,
    location: Option<String>,
    conference: Option<String>,
    division: Option<String>,
    nhl_team_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct PlayerSyncItem {
    nhl_player_id: i32,
    first_name: String,
    last_name: String,
    full_name: String,
    position: String,
    team_abbreviation: Option<String>,
    season: Option<String>,
    games_played: Option<i32>,
    goals: Option<i32>,
    assists: Option<i32>,
    points: Option<i32>,
    plus_minus: Option<i32>,
    shots: Option<i32>,
    time_on_ice_per_game: Option<String>,
}

#[derive(Deserialize)]
pub struct NhlSyncPayload {
    teams: Option<Vec<TeamSyncItem>>,
    players: Option<Vec<PlayerSyncItem>>,
}

pub async fn nhl_sync(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NhlSyncPayload>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let mut teams_upserted = 0;
    let mut players_upserted = 0;
    let mut stats_upserted = 0;

    // Upsert teams
    if let Some(teams) = &payload.teams {
        for team in teams {
            let nhl_id = team.nhl_team_id.unwrap_or(0);
            let result = sqlx::query(
                r#"INSERT INTO teams (nhl_team_id, abbreviation, team_name, location, conference, division)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (abbreviation) DO UPDATE SET
                       team_name = COALESCE(NULLIF(EXCLUDED.team_name, ''), teams.team_name),
                       location = COALESCE(NULLIF(EXCLUDED.location, ''), teams.location),
                       conference = COALESCE(NULLIF(EXCLUDED.conference, ''), teams.conference),
                       division = COALESCE(NULLIF(EXCLUDED.division, ''), teams.division)"#,
            )
            .bind(nhl_id)
            .bind(&team.abbreviation)
            .bind(&team.team_name)
            .bind(&team.location)
            .bind(&team.conference)
            .bind(&team.division)
            .execute(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            if result.rows_affected() > 0 {
                teams_upserted += 1;
            }
        }
    }

    // Upsert players and their stats
    if let Some(players) = &payload.players {
        for p in players {
            let player = sqlx::query(
                r#"INSERT INTO players (nhl_player_id, first_name, last_name, full_name, position,
                                         current_team_abbreviation)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (nhl_player_id) DO UPDATE SET
                       first_name = EXCLUDED.first_name,
                       last_name = EXCLUDED.last_name,
                       full_name = EXCLUDED.full_name,
                       position = EXCLUDED.position,
                       current_team_abbreviation = EXCLUDED.current_team_abbreviation,
                       updated_at = NOW()
                   RETURNING id"#,
            )
            .bind(p.nhl_player_id)
            .bind(&p.first_name)
            .bind(&p.last_name)
            .bind(&p.full_name)
            .bind(&p.position)
            .bind(&p.team_abbreviation)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            if player.is_some() {
                players_upserted += 1;
            }

            // Upsert season stats if present
            if let Some(season) = &p.season {
                let _ = sqlx::query(
                    r#"INSERT INTO player_season_stats
                       (nhl_player_id, season, team_abbreviation, games_played, goals, assists,
                        points, plus_minus, shots, time_on_ice_per_game, player_id)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                           (SELECT id FROM players WHERE nhl_player_id = $1))
                       ON CONFLICT (nhl_player_id, season, team_abbreviation) DO UPDATE SET
                           games_played = EXCLUDED.games_played,
                           goals = EXCLUDED.goals,
                           assists = EXCLUDED.assists,
                           points = EXCLUDED.points,
                           plus_minus = EXCLUDED.plus_minus,
                           shots = EXCLUDED.shots,
                           time_on_ice_per_game = EXCLUDED.time_on_ice_per_game,
                           updated_at = NOW()"#,
                )
                .bind(p.nhl_player_id)
                .bind(season)
                .bind(&p.team_abbreviation)
                .bind(p.games_played.unwrap_or(0))
                .bind(p.goals.unwrap_or(0))
                .bind(p.assists.unwrap_or(0))
                .bind(p.points.unwrap_or(0))
                .bind(p.plus_minus.unwrap_or(0))
                .bind(p.shots.unwrap_or(0))
                .bind(&p.time_on_ice_per_game)
                .execute(&state.pool)
                .await;

                stats_upserted += 1;
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "teams_upserted": teams_upserted,
            "players_upserted": players_upserted,
            "stats_upserted": stats_upserted,
        })),
    ))
}

/// n8n → наш API: запуск батч-AI анализа
#[derive(Deserialize)]
pub struct AnalyzePlayersPayload {
    player_ids: Option<Vec<i32>>,
    #[allow(dead_code)]
    season: Option<String>,
    #[allow(dead_code)]
    analysis_type: Option<String>,
}

pub async fn analyze_players(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalyzePlayersPayload>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let limit = 50;
    let players = if let Some(ids) = &payload.player_ids {
        sqlx::query_as::<_, (i32, String)>(
            "SELECT nhl_player_id, full_name FROM players WHERE nhl_player_id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        sqlx::query_as::<_, (i32, String)>(
            "SELECT nhl_player_id, full_name FROM players WHERE is_active = true ORDER BY RANDOM() LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
    };

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "players": players.iter().map(|(id, name)| serde_json::json!({ "id": id, "name": name })).collect::<Vec<_>>(),
            "total": players.len(),
            "instructions": "Iterate over players and call GET /v1/players/{id}/ai-analysis?type=<analysis_type>"
        })),
    ))
}

/// n8n → наш API: сохранение собранных новостей
#[derive(Deserialize)]
pub struct NewsItem {
    title: String,
    url: String,
    source: String,
    published_at: String,
    summary: Option<String>,
    category: Option<String>,
    player_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct CollectNewsPayload {
    news: Vec<NewsItem>,
}

pub async fn collect_news(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CollectNewsPayload>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let mut inserted = 0;
    let mut errors = 0;

    for item in &payload.news {
        let result = sqlx::query(
            r#"INSERT INTO news_events (title, url, source, published_at, summary, category, player_ids)
               VALUES ($1, $2, $3, $4::TIMESTAMPTZ, $5, $6, $7)
               ON CONFLICT (url) DO UPDATE SET
                   title = EXCLUDED.title,
                   summary = COALESCE(NULLIF(EXCLUDED.summary, ''), news_events.summary),
                   updated_at = NOW()"#,
        )
        .bind(&item.title)
        .bind(&item.url)
        .bind(&item.source)
        .bind(&item.published_at)
        .bind(&item.summary)
        .bind(&item.category)
        .bind(&item.player_ids)
        .execute(&state.pool)
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => inserted += 1,
            Ok(_) => {} // duplicate, skip
            Err(e) => {
                tracing::warn!(url = item.url, error = %e, "Failed to insert news");
                errors += 1;
            }
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "ok",
            "inserted": inserted,
            "duplicates": payload.news.len() - inserted - errors,
            "errors": errors,
        })),
    ))
}
