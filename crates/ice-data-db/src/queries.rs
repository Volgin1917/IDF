use ice_data_core::models::Player;
use sqlx::PgPool;

pub async fn get_player_by_nhl_id(pool: &PgPool, nhl_id: i32) -> Result<Option<Player>, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "SELECT id, nhl_player_id, first_name, last_name, full_name, position,
                shoot_catches, height_cm, weight_lbs, birth_date, birth_city,
                birth_country, nationality, current_team_abbreviation, jersey_number,
                is_active, created_at, updated_at
         FROM players WHERE nhl_player_id = $1",
    )
    .bind(nhl_id)
    .fetch_optional(pool)
    .await
}

pub async fn search_players(
    pool: &PgPool,
    query: &str,
    limit: i32,
) -> Result<Vec<Player>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    let prefix = format!("{}%", query);

    sqlx::query_as::<_, Player>(
        "SELECT id, nhl_player_id, first_name, last_name, full_name, position,
                shoot_catches, height_cm, weight_lbs, birth_date, birth_city,
                birth_country, nationality, current_team_abbreviation, jersey_number,
                is_active, created_at, updated_at
         FROM players
         WHERE search_vector @@ plainto_tsquery('english', $1)
            OR full_name ILIKE $2
         ORDER BY CASE WHEN full_name ILIKE $3 THEN 0 ELSE 1 END
         LIMIT $4",
    )
    .bind(query)
    .bind(&pattern)
    .bind(&prefix)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
}

pub async fn upsert_player(pool: &PgPool, nhl_id: i32, first_name: &str, last_name: &str, full_name: &str, position: &str) -> Result<Player, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "INSERT INTO players (nhl_player_id, first_name, last_name, full_name, position)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (nhl_player_id) DO UPDATE SET
             first_name = EXCLUDED.first_name,
             last_name = EXCLUDED.last_name,
             full_name = EXCLUDED.full_name,
             position = EXCLUDED.position,
             updated_at = NOW()
         RETURNING id, nhl_player_id, first_name, last_name, full_name, position,
                   shoot_catches, height_cm, weight_lbs, birth_date, birth_city,
                   birth_country, nationality, current_team_abbreviation, jersey_number,
                   is_active, created_at, updated_at",
    )
    .bind(nhl_id)
    .bind(first_name)
    .bind(last_name)
    .bind(full_name)
    .bind(position)
    .fetch_one(pool)
    .await
}
