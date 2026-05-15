use serde::Deserialize;

// ── Search ──

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "playerId")]
    pub player_id: i32,
    pub name: String,
    #[serde(rename = "teamAbbrev")]
    pub team_abbrev: Option<String>,
    pub position: Option<String>,
}

// ── Player Landing ──

#[derive(Debug, Deserialize)]
pub struct PlayerLandingResponse {
    #[serde(rename = "playerId")]
    pub player_id: i32,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "birthDate")]
    pub birth_date: Option<String>,
    #[serde(rename = "birthCity")]
    pub birth_city: Option<String>,
    #[serde(rename = "birthCountry")]
    pub birth_country: Option<String>,
    #[serde(rename = "birthStateProvince")]
    pub birth_state_province: Option<String>,
    pub height: Option<f64>,
    pub weight: Option<f64>,
    #[serde(rename = "shootsCatches")]
    pub shoots_catches: Option<String>,
    pub position: Option<String>,
    #[serde(rename = "currentTeamAbbrev")]
    pub current_team_abbrev: Option<String>,
    #[serde(rename = "currentTeamId")]
    pub current_team_id: Option<i32>,
    #[serde(rename = "sweaterNumber")]
    pub sweater_number: Option<i32>,
    #[serde(rename = "last5Games")]
    pub last_5_games: Option<Vec<serde_json::Value>>,
    #[serde(rename = "seasonTotals")]
    pub season_totals: Option<Vec<SeasonTotal>>,
    #[serde(rename = "careerTotals")]
    pub career_totals: Option<CareerTotals>,
    #[serde(rename = "featuredStats")]
    pub featured_stats: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SeasonTotal {
    pub season: i32,
    #[serde(rename = "gameTypeId")]
    pub game_type_id: i32,
    #[serde(rename = "teamAbbrev")]
    pub team_abbrev: Option<String>,
    pub league: Option<String>,
    pub games: Option<i32>,
    pub goals: Option<i32>,
    pub assists: Option<i32>,
    pub points: Option<i32>,
    #[serde(rename = "plusMinus")]
    pub plus_minus: Option<i32>,
    pub pim: Option<i32>,
    pub shots: Option<i32>,
    #[serde(rename = "shootingPct")]
    pub shooting_pct: Option<f64>,
    #[serde(rename = "toiPerGame")]
    pub toi_per_game: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CareerTotals {
    #[serde(rename = "regularSeason")]
    pub regular_season: Option<Vec<SeasonTotal>>,
    pub playoffs: Option<Vec<SeasonTotal>>,
}

// ── Roster ──

#[derive(Debug, Deserialize)]
pub struct RosterResponse {
    pub forwards: Vec<RosterEntry>,
    pub defensemen: Vec<RosterEntry>,
    pub goalies: Vec<RosterEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RosterEntry {
    #[serde(rename = "playerId")]
    pub player_id: i32,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "sweaterNumber")]
    pub sweater_number: Option<i32>,
    pub position: Option<String>,
    #[serde(rename = "birthDate")]
    pub birth_date: Option<String>,
    pub height: Option<f64>,
    pub weight: Option<f64>,
    #[serde(rename = "shootsCatches")]
    pub shoots_catches: Option<String>,
}
