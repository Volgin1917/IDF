use std::sync::Arc;

use chrono::Utc;
use ice_data_ai::{AiService, AnalysisType};
use ice_data_db::queries;
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::protocol::ToolDefinition;

pub struct McpContext {
    pub pool: PgPool,
    pub nhl_client: Arc<ice_data_nhl::client::NhlClient>,
    pub ai_service: AiService,
}

pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "get_player".into(),
            description: "Retrieve player information and statistics by name".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Player full name" },
                    "season": { "type": "string", "description": "Season in format YYYYYYYY" },
                    "include_advanced": { "type": "boolean", "default": false }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "search_players".into(),
            description: "Search for players by name with autocomplete".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query (min 2 chars)", "minLength": 2 },
                    "limit": { "type": "integer", "default": 10, "maximum": 50 }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "analyze_player".into(),
            description: "Run comprehensive AI analysis of a player".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "focus": { "type": "string", "enum": ["potential", "strengths", "comparison", "all"], "default": "all" },
                    "depth": { "type": "string", "enum": ["quick", "standard", "deep"], "default": "standard" }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "compare_players".into(),
            description: "Compare multiple players side by side".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "players": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 5 },
                    "metrics": { "type": "array", "items": { "type": "string" } },
                    "season": { "type": "string" }
                },
                "required": ["players"]
            }),
        },
        ToolDefinition {
            name: "get_team_roster".into(),
            description: "Get the full roster for an NHL team".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "team_abbreviation": { "type": "string", "description": "Team abbreviation (e.g. EDM, TOR)" },
                    "season": { "type": "string", "description": "Season in format YYYYYYYY" }
                },
                "required": ["team_abbreviation"]
            }),
        },
        ToolDefinition {
            name: "get_player_news".into(),
            description: "Get recent news about a player".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "player_name": { "type": "string", "description": "Player full name" },
                    "days_back": { "type": "integer", "default": 30 }
                },
                "required": ["player_name"]
            }),
        },
    ]
}

pub async fn handle_tool_call(
    ctx: &McpContext,
    name: &str,
    args: &Value,
) -> Result<Value, String> {
    match name {
        "get_player" => cmd_get_player(ctx, args).await,
        "search_players" => cmd_search_players(ctx, args).await,
        "analyze_player" => cmd_analyze_player(ctx, args).await,
        "compare_players" => cmd_compare_players(ctx, args).await,
        "get_team_roster" => cmd_get_team_roster(ctx, args).await,
        "get_player_news" => cmd_get_player_news(ctx, args).await,
        _ => Err(format!("Unknown tool: {name}")),
    }
}

async fn cmd_get_player(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: name")?;
    let season: Option<&str> = args.get("season").and_then(|v| v.as_str());

    let players = queries::search_players(&ctx.pool, name, 5)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    if let Some(player) = players.first() {
        let stats_rows = sqlx::query(
            "SELECT season, team_abbreviation, games_played, goals, assists, points,
                    plus_minus, shots, shooting_percentage, time_on_ice_per_game,
                    advanced_metrics
             FROM player_season_stats
             WHERE nhl_player_id = $1
               AND ($2::TEXT IS NULL OR season = $2)
             ORDER BY season DESC",
        )
        .bind(player.nhl_player_id)
        .bind(season)
        .fetch_all(&ctx.pool)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

        let seasons: Vec<Value> = stats_rows.iter().map(|r| {
            serde_json::json!({
                "season": r.get::<String, _>("season"),
                "games": r.get::<i32, _>("games_played"),
                "goals": r.get::<i32, _>("goals"),
                "assists": r.get::<i32, _>("assists"),
                "points": r.get::<i32, _>("points"),
                "plus_minus": r.get::<i32, _>("plus_minus"),
                "advanced_metrics": r.get::<Option<Value>, _>("advanced_metrics"),
            })
        }).collect();

        Ok(serde_json::json!({
            "player": player,
            "seasons": seasons
        }))
    } else {
        let results = ctx.nhl_client.search_players(name, 1).await
            .map_err(|e| format!("NHL API error: {e}"))?;
        if let Some(r) = results.first() {
            Ok(serde_json::json!({
                "player_id": r.player_id,
                "name": r.name,
                "team": r.team_abbrev,
                "position": r.position,
                "note": "Data not yet imported to local DB"
            }))
        } else {
            Err(format!("Player not found: {name}"))
        }
    }
}

async fn cmd_search_players(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: query")?;
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10).min(50) as i32;

    let players = queries::search_players(&ctx.pool, query, limit)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    Ok(serde_json::json!({ "results": players }))
}

async fn cmd_analyze_player(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: name")?;
    let focus = args.get("focus").and_then(|v| v.as_str()).unwrap_or("all");
    let depth = args.get("depth").and_then(|v| v.as_str()).unwrap_or("standard");

    let players = queries::search_players(&ctx.pool, name, 1)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    let player = players.first().ok_or_else(|| format!("Player not found: {name}"))?;

    let analysis_type = match focus {
        "potential" => AnalysisType::Potential,
        "strengths" => AnalysisType::Strengths,
        _ => AnalysisType::Full,
    };

    // Deep analysis requests multiple types and merges them
    let result = if depth == "deep" {
        let full = ctx.ai_service.get_or_create_analysis(player, None, &AnalysisType::Full).await?;
        let potential = ctx.ai_service.get_or_create_analysis(player, None, &AnalysisType::Potential).await?;
        let strengths = ctx.ai_service.get_or_create_analysis(player, None, &AnalysisType::Strengths).await?;

        let mut merged = serde_json::json!({
            "strengths": strengths.get("elite_skills").or_else(|| full.get("strengths")).cloned().unwrap_or(serde_json::Value::Array(vec![])),
            "weaknesses": full.get("weaknesses").cloned().unwrap_or(serde_json::Value::Array(vec![])),
            "potential": potential.get("most_likely_outcome").cloned(),
            "summary": full.get("summary").cloned().unwrap_or(serde_json::Value::Null),
            "key_insight": full.get("key_insight").cloned().unwrap_or(serde_json::Value::Null),
        });
        merged["model"] = serde_json::json!("gpt-4");
        merged["depth"] = serde_json::json!("deep");
        merged
    } else {
        let mut analysis = ctx.ai_service.get_or_create_analysis(player, None, &analysis_type).await?;
        analysis["depth"] = serde_json::json!("standard");
        analysis
    };

    Ok(serde_json::json!({
        "player": player,
        "analysis": result
    }))
}

async fn cmd_compare_players(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let names = args.get("players")
        .and_then(|v| v.as_array())
        .ok_or("Missing required parameter: players")?;

    if names.len() < 2 || names.len() > 5 {
        return Err("Must compare between 2 and 5 players".into());
    }

    let season: Option<&str> = args.get("season").and_then(|v| v.as_str());

    let mut results = Vec::new();
    for name_val in names {
        let name = name_val.as_str().ok_or("Player name must be a string")?;
        let players = queries::search_players(&ctx.pool, name, 1)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        if let Some(player) = players.first() {
            let stats = sqlx::query(
                "SELECT season, team_abbreviation, games_played, goals, assists, points,
                        plus_minus, shots, advanced_metrics
                 FROM player_season_stats
                 WHERE nhl_player_id = $1
                   AND ($2::TEXT IS NULL OR season = $2)
                 ORDER BY season DESC LIMIT 5",
            )
            .bind(player.nhl_player_id)
            .bind(season)
            .fetch_all(&ctx.pool)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

            let seasons: Vec<Value> = stats.iter().map(|r| {
                serde_json::json!({
                    "season": r.get::<String, _>("season"),
                    "games_played": r.get::<i32, _>("games_played"),
                    "goals": r.get::<i32, _>("goals"),
                    "assists": r.get::<i32, _>("assists"),
                    "points": r.get::<i32, _>("points"),
                })
            }).collect();

            results.push(serde_json::json!({
                "player": player,
                "seasons": seasons
            }));
        }
    }

    Ok(serde_json::json!({ "comparison": results }))
}

async fn cmd_get_team_roster(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let team = args.get("team_abbreviation")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: team_abbreviation")?;

    let season: Option<&str> = args.get("season").and_then(|v| v.as_str());
    let season_str = season.unwrap_or_else(|| {
        let y = Utc::now().format("%Y").to_string();
        Box::leak(format!("{}{}", y.parse::<i32>().unwrap_or(2025) - 1, y).into_boxed_str())
    });

    let rows = sqlx::query(
        "SELECT nhl_player_id, full_name, position, jersey_number
         FROM players
         WHERE current_team_abbreviation = $1 AND is_active = true
         ORDER BY position, last_name",
    )
    .bind(team.to_uppercase())
    .fetch_all(&ctx.pool)
    .await
    .map_err(|e| format!("DB error: {e}"))?;

    let roster: Vec<Value> = rows.iter().map(|r| {
        serde_json::json!({
            "player_id": r.get::<i32, _>("nhl_player_id"),
            "name": r.get::<String, _>("full_name"),
            "position": r.get::<String, _>("position"),
            "jersey_number": r.get::<Option<i32>, _>("jersey_number"),
        })
    }).collect();

    Ok(serde_json::json!({
        "team": team.to_uppercase(),
        "season": season_str,
        "roster": roster,
    }))
}

async fn cmd_get_player_news(ctx: &McpContext, args: &Value) -> Result<Value, String> {
    let player_name = args.get("player_name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: player_name")?;
    let days_back = args.get("days_back").and_then(|v| v.as_i64()).unwrap_or(30);

    let players = queries::search_players(&ctx.pool, player_name, 1)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    let pid = players.first().map(|p| p.nhl_player_id);

    let rows = sqlx::query(
        "SELECT id, title, source, url, published_at, category, sentiment, summary
         FROM news_events
         WHERE ($1::INT IS NULL OR player_ids @> ARRAY[$1]::INTEGER[])
           AND published_at >= NOW() - ($2 || ' days')::INTERVAL
         ORDER BY published_at DESC
         LIMIT 20",
    )
    .bind(pid)
    .bind(days_back.to_string())
    .fetch_all(&ctx.pool)
    .await
    .map_err(|e| format!("DB error: {e}"))?;

    let news: Vec<Value> = rows.iter().map(|r| {
        serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "title": r.get::<String, _>("title"),
            "source": r.get::<String, _>("source"),
            "url": r.get::<String, _>("url"),
            "published_at": r.get::<chrono::DateTime<chrono::Utc>, _>("published_at"),
            "category": r.get::<Option<String>, _>("category"),
            "sentiment": r.get::<Option<String>, _>("sentiment"),
        })
    }).collect();

    Ok(serde_json::json!({
        "player": player_name,
        "news": news
    }))
}
