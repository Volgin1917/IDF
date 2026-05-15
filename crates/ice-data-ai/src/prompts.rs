use ice_data_core::models::{Player, SeasonStats};

pub const SYSTEM_PROMPT: &str = r#"You are an elite NHL analytics expert and professional scout.
Analyze hockey players using advanced metrics (xG, Corsi, Fenwick, PDO, per-60 rates)
combined with traditional stats. Provide data-driven insights in Russian or English as requested.
Always respond in valid JSON matching the requested schema exactly."#;

fn team_abbrev(stats: &SeasonStats) -> &str {
    stats.team_abbreviation.as_deref().unwrap_or("N/A")
}

fn format_stats(stats: &[SeasonStats]) -> String {
    if stats.is_empty() {
        return "No NHL stats available.".into();
    }
    stats
        .iter()
        .map(|s| {
            format!(
                "{} ({}): {} GP, {}G, {}A, {}P, {} +/-, {} shots, {} TOI/GP",
                s.season,
                team_abbrev(s),
                s.games_played,
                s.goals,
                s.assists,
                s.points,
                s.plus_minus,
                s.shots,
                s.time_on_ice_per_game.as_deref().unwrap_or("N/A"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_analysis_prompt(player: &Player, stats: &[SeasonStats]) -> String {
    format!(
        r#"Analyze this NHL player and return a JSON object with these fields:
- "strengths": array of 3-5 key strengths with explanations
- "weaknesses": array of 2-4 areas for improvement
- "player_comp": a comparable NHL player (past or present)
- "confidence_score": 0.0-1.0
- "summary": 2-3 sentence overall assessment
- "key_insight": one notable statistical observation

Player: {name} ({pos}) — {team}
Stats:
{stats}"#,
        name = player.full_name,
        pos = player.position,
        team = player.current_team_abbreviation.as_deref().unwrap_or("FA"),
        stats = format_stats(stats),
    )
}

pub fn build_comparison_prompt(players: &[(Player, Vec<SeasonStats>)]) -> String {
    let player_blocks: Vec<String> = players
        .iter()
        .map(|(p, s)| {
            format!(
                "---\n{} ({}) — {}\n{}",
                p.full_name,
                p.position,
                p.current_team_abbreviation.as_deref().unwrap_or("FA"),
                format_stats(s),
            )
        })
        .collect();

    format!(
        r#"Compare these NHL players and return a JSON object with:
- "comparison_table": array of objects with player name and key stats
- "best_offensive": which player has the best offensive upside
- "best_defensive": which player has the best defensive impact
- "recommendation": which player would you build a team around and why
- "confidence_score": 0.0-1.0

{players}"#,
        players = player_blocks.join("\n"),
    )
}

pub fn build_potential_prompt(player: &Player, stats: &[SeasonStats]) -> String {
    format!(
        r#"Project this NHL player's development and return JSON with:
- "ceiling": best-case scenario (1-2 sentences)
- "floor": worst-case scenario (1-2 sentences)
- "most_likely_outcome": realistic projection (1-2 sentences)
- "timeframe": expected timeline to peak (e.g., "2-3 seasons")
- "comparables": array of 2-3 similar players at same age
- "confidence_score": 0.0-1.0
- "key_development_areas": array of 2-3 specific skills to focus on

Player: {name} ({pos}) — {team}
Stats:
{stats}"#,
        name = player.full_name,
        pos = player.position,
        team = player.current_team_abbreviation.as_deref().unwrap_or("FA"),
        stats = format_stats(stats),
    )
}

pub fn build_strengths_prompt(player: &Player, stats: &[SeasonStats]) -> String {
    format!(
        r#"Analyze this player's strengths in depth and return JSON with:
- "elite_skills": array of skills where they rank among best in NHL
- "above_average_skills": array of above-average skills
- "signature_move_or_trait": what makes them unique
- "situational_strengths": power play, penalty kill, clutch, etc.
- "confidence_score": 0.0-1.0

Player: {name} ({pos}) — {team}
Stats:
{stats}"#,
        name = player.full_name,
        pos = player.position,
        team = player.current_team_abbreviation.as_deref().unwrap_or("FA"),
        stats = format_stats(stats),
    )
}
