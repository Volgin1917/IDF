use chrono::Utc;
use ice_data_core::models::Player;
use serde_json::Value;
use sqlx::{PgPool, Row};
use tracing::info;

mod client;
mod parser;
mod prompts;

pub use client::{AiClient, AiClientError};
use parser::{normalize_analysis, parse_json_response};

pub struct AiService {
    pub client: AiClient,
    pub pool: PgPool,
}

pub enum AnalysisType {
    Full,
    Potential,
    Strengths,
    Comparison,
}

impl AnalysisType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Potential => "potential",
            Self::Strengths => "strengths",
            Self::Comparison => "comparison",
        }
    }
}

impl AiService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            client: AiClient::from_env(),
            pool,
        }
    }

    /// Get cached analysis if available and fresh, or generate a new one.
    pub async fn get_or_create_analysis(
        &self,
        player: &Player,
        season: Option<&str>,
        analysis_type: &AnalysisType,
    ) -> Result<Value, String> {
        let type_str = analysis_type.as_str();

        // Check cache
        let row = sqlx::query(
            "SELECT id, strengths, weaknesses, potential, development_areas,
                    statistical_insights, full_report, confidence_score
             FROM ai_analysis
             WHERE nhl_player_id = $1
               AND ($2::TEXT IS NULL OR season = $2)
               AND analysis_type = $3
               AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(player.nhl_player_id)
        .bind(season)
        .bind(type_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

        if let Some(r) = row {
            info!(
                player_id = player.nhl_player_id,
                analysis_type = type_str,
                "Returning cached AI analysis"
            );
            return Ok(serde_json::json!({
                "strengths": r.get::<Option<Value>, _>("strengths"),
                "weaknesses": r.get::<Option<Value>, _>("weaknesses"),
                "potential": r.get::<Option<Value>, _>("potential"),
                "development_areas": r.get::<Option<Value>, _>("development_areas"),
                "statistical_insights": r.get::<Option<Value>, _>("statistical_insights"),
                "full_report": r.get::<Option<String>, _>("full_report"),
                "confidence_score": r.get::<Option<f64>, _>("confidence_score"),
                "model": "gpt-4",
                "cached": true,
            }));
        }

        // Fetch stats for prompting
        let stats = sqlx::query(
            "SELECT season, team_abbreviation, league, games_played, goals, assists, points,
                    plus_minus, penalty_minutes, shots, shooting_percentage, time_on_ice_per_game
             FROM player_season_stats
             WHERE nhl_player_id = $1
               AND ($2::TEXT IS NULL OR season = $2)
             ORDER BY season DESC LIMIT 10",
        )
        .bind(player.nhl_player_id)
        .bind(season)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

        let seasons: Vec<ice_data_core::models::SeasonStats> = stats
            .iter()
            .map(|r| {
                use sqlx::Row;
                ice_data_core::models::SeasonStats {
                    id: 0,
                    player_id: player.id,
                    season: r.get("season"),
                    team_abbreviation: r.get("team_abbreviation"),
                    league: r.get("league"),
                    games_played: r.get("games_played"),
                    goals: r.get("goals"),
                    assists: r.get("assists"),
                    points: r.get("points"),
                    plus_minus: r.get("plus_minus"),
                    penalty_minutes: r.get("penalty_minutes"),
                    shots: r.get("shots"),
                    shooting_percentage: r.get("shooting_percentage"),
                    time_on_ice_per_game: r.get("time_on_ice_per_game"),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
            .collect();

        let analysis_type_for_prompt = analysis_type;

        let (system_prompt, user_prompt, max_tokens) = match analysis_type_for_prompt {
            AnalysisType::Full => (
                prompts::SYSTEM_PROMPT,
                prompts::build_analysis_prompt(player, &seasons),
                1024,
            ),
            AnalysisType::Potential => (
                prompts::SYSTEM_PROMPT,
                prompts::build_potential_prompt(player, &seasons),
                1024,
            ),
            AnalysisType::Strengths => (
                prompts::SYSTEM_PROMPT,
                prompts::build_strengths_prompt(player, &seasons),
                1024,
            ),
            AnalysisType::Comparison => {
                return Err("Comparison requires multiple players, use analyze_comparison".into());
            }
        };

        let (raw_response, _tokens) = self
            .client
            .chat_completion(system_prompt, &user_prompt, max_tokens)
            .await
            .map_err(|e| format!("OpenAI error: {e}"))?;

        let mut parsed = parse_json_response(&raw_response)?;
        normalize_analysis(&mut parsed, type_str);

        // Store in DB
        self.store_analysis(player, season, type_str, &parsed).await?;

        parsed["model"] = Value::String("gpt-4".into());
        parsed["cached"] = Value::Bool(false);

        Ok(parsed)
    }

    /// Compare multiple players with a single AI call.
    pub async fn analyze_comparison(
        &self,
        players: &[Player],
        season: Option<&str>,
    ) -> Result<Value, String> {
        if players.len() < 2 || players.len() > 5 {
            return Err("Need 2-5 players for comparison".into());
        }

        let mut player_data: Vec<(Player, Vec<ice_data_core::models::SeasonStats>)> = Vec::new();
        for player in players {
            let stats = sqlx::query(
                "SELECT season, team_abbreviation, league, games_played, goals, assists, points,
                        plus_minus, shots, shooting_percentage, time_on_ice_per_game
                 FROM player_season_stats
                 WHERE nhl_player_id = $1
                   AND ($2::TEXT IS NULL OR season = $2)
                 ORDER BY season DESC LIMIT 5",
            )
            .bind(player.nhl_player_id)
            .bind(season)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

            let seasons: Vec<ice_data_core::models::SeasonStats> = stats
                .iter()
                .map(|r| {
                    use sqlx::Row;
                    ice_data_core::models::SeasonStats {
                        id: 0,
                        player_id: player.id,
                        season: r.get("season"),
                        team_abbreviation: r.get("team_abbreviation"),
                        league: r.get("league"),
                        games_played: r.get("games_played"),
                        goals: r.get("goals"),
                        assists: r.get("assists"),
                        points: r.get("points"),
                        plus_minus: r.get("plus_minus"),
                        penalty_minutes: r.get("penalty_minutes"),
                        shots: r.get("shots"),
                        shooting_percentage: r.get("shooting_percentage"),
                        time_on_ice_per_game: r.get("time_on_ice_per_game"),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                })
                .collect();

            player_data.push((player.clone(), seasons));
        }

        let prompt = prompts::build_comparison_prompt(&player_data);
        let (raw_response, _tokens) = self
            .client
            .chat_completion(prompts::SYSTEM_PROMPT, &prompt, 1024)
            .await
            .map_err(|e| format!("OpenAI error: {e}"))?;

        let mut parsed = parse_json_response(&raw_response)?;
        normalize_analysis(&mut parsed, "comparison");

        Ok(parsed)
    }

    /// Store analysis result in ai_analysis table.
    async fn store_analysis(
        &self,
        player: &Player,
        season: Option<&str>,
        analysis_type: &str,
        data: &Value,
    ) -> Result<(), String> {
        sqlx::query(
            r#"INSERT INTO ai_analysis
               (player_id, nhl_player_id, season, analysis_type, ai_model,
                strengths, weaknesses, potential, development_areas,
                statistical_insights, full_report, confidence_score, cache_key)
               VALUES ($1, $2, $3, $4, 'gpt-4',
                $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(player.id)
        .bind(player.nhl_player_id)
        .bind(season)
        .bind(analysis_type)
        .bind(data.get("strengths"))
        .bind(data.get("weaknesses"))
        .bind(data.get("potential"))
        .bind(data.get("development_areas"))
        .bind(data.get("statistical_insights"))
        .bind(data.get("full_report").and_then(|v| v.as_str()))
        .bind(data.get("confidence_score").and_then(|v| {
            v.as_f64().or_else(|| v.as_i64().map(|i| i as f64))
        }))
        .bind(format!("ai:{}-{}", analysis_type, player.nhl_player_id))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store analysis: {e}"))?;

        info!(
            player_id = player.nhl_player_id,
            analysis_type,
            "Stored AI analysis"
        );
        Ok(())
    }
}
