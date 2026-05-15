use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: i32,
    pub nhl_player_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub position: String,
    pub shoot_catches: Option<String>,
    pub height_cm: Option<i32>,
    pub weight_lbs: Option<i32>,
    pub birth_date: Option<NaiveDate>,
    pub birth_city: Option<String>,
    pub birth_country: Option<String>,
    pub nationality: Option<String>,
    pub current_team_abbreviation: Option<String>,
    pub jersey_number: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeasonStats {
    pub id: i64,
    pub player_id: i32,
    pub season: String,
    pub team_abbreviation: Option<String>,
    pub league: String,
    pub games_played: i32,
    pub goals: i32,
    pub assists: i32,
    pub points: i32,
    pub plus_minus: i32,
    pub penalty_minutes: i32,
    pub shots: i32,
    pub shooting_percentage: Option<String>,
    pub time_on_ice_per_game: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedStats {
    pub xg: Option<f64>,
    pub xg_per_60: Option<f64>,
    pub corsi_for_percentage: Option<f64>,
    pub fenwick_for_percentage: Option<f64>,
    pub pdo: Option<f64>,
    pub offensive_zone_start_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAnalysis {
    pub id: i64,
    pub player_id: i32,
    pub season: Option<String>,
    pub analysis_data: serde_json::Value,
    pub model: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewsEvent {
    pub id: i64,
    pub title: String,
    pub source: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub category: Option<String>,
    pub sentiment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Serialize)]
pub struct PageMeta {
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}
