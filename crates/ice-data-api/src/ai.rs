use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use ice_data_ai::{AiService, AnalysisType};
use ice_data_core::error::AppError;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct AiQuery {
    season: Option<String>,
    #[serde(rename = "type")]
    analysis_type: Option<String>,
    depth: Option<String>,
}

pub async fn analyze(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(query): Query<AiQuery>,
) -> Result<Json<Value>, ApiError> {
    let player = sqlx::query_as::<_, ice_data_core::models::Player>(
        "SELECT id, nhl_player_id, first_name, last_name, full_name, position,
                shoot_catches, height_cm, weight_lbs, birth_date, birth_city,
                birth_country, nationality, current_team_abbreviation, jersey_number,
                is_active, created_at, updated_at
         FROM players WHERE nhl_player_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ice_data_core::error::AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Player {id}")))?;

    let analysis_type = match query.analysis_type.as_deref() {
        Some("potential") => AnalysisType::Potential,
        Some("strengths") => AnalysisType::Strengths,
        _ => AnalysisType::Full,
    };

    let ai = AiService::new(state.pool.clone());

    let map_err = |e: String| ApiError(AppError::Internal(e));

    let result = if query.depth.as_deref() == Some("deep") {
        let full = ai.get_or_create_analysis(&player, query.season.as_deref(), &AnalysisType::Full).await.map_err(map_err)?;
        let potential = ai.get_or_create_analysis(&player, query.season.as_deref(), &AnalysisType::Potential).await.map_err(map_err)?;

        serde_json::json!({
            "player_id": id,
            "analysis": {
                "strengths": full.get("strengths"),
                "weaknesses": full.get("weaknesses"),
                "potential": potential.get("most_likely_outcome"),
                "summary": full.get("summary"),
                "depth": "deep",
                "model": "gpt-4",
            }
        })
    } else {
        let mut analysis = ai.get_or_create_analysis(&player, query.season.as_deref(), &analysis_type).await.map_err(map_err)?;
        analysis["depth"] = Value::String("standard".into());
        serde_json::json!({
            "player_id": id,
            "analysis": analysis
        })
    };

    Ok(Json(result))
}
