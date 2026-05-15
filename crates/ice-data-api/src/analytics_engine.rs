use serde_json::Value;

/// Parses "MM:SS" TOI string into total seconds.
pub fn toi_to_seconds(toi: &str) -> Option<f64> {
    let parts: Vec<&str> = toi.split(':').collect();
    if parts.len() == 2 {
        let minutes: f64 = parts[0].parse().ok()?;
        let seconds: f64 = parts[1].parse().ok()?;
        Some(minutes * 60.0 + seconds)
    } else {
        None
    }
}

/// Points per game
pub fn points_per_game(games: i32, points: i32) -> Option<f64> {
    if games > 0 {
        Some((points as f64) / (games as f64))
    } else {
        None
    }
}

/// Per-60 rate: (value / toi_seconds) * 3600
pub fn per_60(value: i32, toi_seconds: f64) -> Option<f64> {
    if toi_seconds > 0.0 {
        Some((value as f64) / toi_seconds * 3600.0)
    } else {
        None
    }
}

/// Shooting percentage
pub fn shooting_pct(goals: i32, shots: i32) -> Option<f64> {
    if shots > 0 {
        Some((goals as f64) / (shots as f64) * 100.0)
    } else {
        None
    }
}

/// Corsi For % = CF / (CF + CA) * 100
pub fn corsi_pct(corsi_for: f64, corsi_against: f64) -> Option<f64> {
    let total = corsi_for + corsi_against;
    if total > 0.0 {
        Some((corsi_for / total) * 100.0)
    } else {
        None
    }
}

/// Fenwick For % = FF / (FF + FA) * 100
pub fn fenwick_pct(fenwick_for: f64, fenwick_against: f64) -> Option<f64> {
    let total = fenwick_for + fenwick_against;
    if total > 0.0 {
        Some((fenwick_for / total) * 100.0)
    } else {
        None
    }
}

/// Simplified xG model using position-based coefficients.
/// In production this would use shot-location data from the NHL API.
pub fn expected_goals(_goals: i32, shots: i32, position: &str) -> f64 {
    let league_avg_sh_pct = match position {
        "C" => 0.102,
        "LW" | "RW" => 0.108,
        "D" => 0.065,
        "G" => 0.0,
        _ => 0.095,
    };
    (shots as f64) * league_avg_sh_pct
}

/// Build the advanced_metrics JSONB payload from raw stats.
#[allow(clippy::too_many_arguments)]
pub fn compute_advanced_metrics(
    games: i32,
    goals: i32,
    assists: i32,
    points: i32,
    shots: i32,
    plus_minus: i32,
    toi_per_game: Option<&str>,
    position: &str,
    corsi_for: Option<f64>,
    corsi_against: Option<f64>,
    fenwick_for: Option<f64>,
    fenwick_against: Option<f64>,
) -> Value {
    let ppg = points_per_game(games, points);
    let sh_pct = shooting_pct(goals, shots);
    let xg = expected_goals(goals, shots, position);

    let toi_sec = toi_per_game.and_then(toi_to_seconds);
    let g_per_60 = toi_sec.and_then(|s| per_60(goals, s * games as f64));
    let a_per_60 = toi_sec.and_then(|s| per_60(assists, s * games as f64));
    let p_per_60 = toi_sec.and_then(|s| per_60(points, s * games as f64));

    let cf_pct = match (corsi_for, corsi_against) {
        (Some(cf), Some(ca)) => corsi_pct(cf, ca),
        _ => None,
    };
    let ff_pct = match (fenwick_for, fenwick_against) {
        (Some(ff), Some(fa)) => fenwick_pct(ff, fa),
        _ => None,
    };

    let mut map = serde_json::Map::new();
    map.insert("xG".into(), Value::from((xg * 100.0).round() / 100.0));
    map.insert("shootingPercentage".into(), sh_pct.map_or(Value::Null, |v| Value::from((v * 100.0).round() / 100.0)));
    map.insert("pointsPerGame".into(), ppg.map_or(Value::Null, |v| Value::from((v * 100.0).round() / 100.0)));
    if let Some(v) = g_per_60 { map.insert("goalsPer60".into(), Value::from((v * 100.0).round() / 100.0)); }
    if let Some(v) = a_per_60 { map.insert("assistsPer60".into(), Value::from((v * 100.0).round() / 100.0)); }
    if let Some(v) = p_per_60 { map.insert("pointsPer60".into(), Value::from((v * 100.0).round() / 100.0)); }
    if let Some(v) = cf_pct { map.insert("corsiForPercentage".into(), Value::from((v * 100.0).round() / 100.0)); }
    if let Some(v) = ff_pct { map.insert("fenwickForPercentage".into(), Value::from((v * 100.0).round() / 100.0)); }
    map.insert("plusMinus".into(), Value::from(plus_minus));
    if let Some(s) = toi_per_game { map.insert("timeOnIcePerGame".into(), Value::from(s)); }

    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toi_parsing() {
        assert_eq!(toi_to_seconds("22:18"), Some(22.0 * 60.0 + 18.0));
        assert_eq!(toi_to_seconds("00:30"), Some(30.0));
        assert!(toi_to_seconds("invalid").is_none());
    }

    #[test]
    fn test_ppg() {
        assert_eq!(points_per_game(10, 5), Some(0.5));
        assert!(points_per_game(0, 0).is_none());
    }

    #[test]
    fn test_shooting_pct() {
        let pct = shooting_pct(10, 100).unwrap();
        assert!((pct - 10.0).abs() < 0.01);
        assert!(shooting_pct(0, 0).is_none());
    }

    #[test]
    fn test_corsi() {
        let pct = corsi_pct(500.0, 400.0).unwrap();
        assert!((pct - 55.56).abs() < 0.01);
    }

    #[test]
    fn test_xg() {
        let xg = expected_goals(10, 100, "C");
        assert!((xg - 10.2).abs() < 0.01);
    }

    #[test]
    fn test_per_60() {
        let rate = per_60(2, 1200.0);
        assert_eq!(rate, Some(6.0));
    }

    #[test]
    fn test_advanced_metrics_full() {
        let metrics = compute_advanced_metrics(
            82, 30, 50, 80, 200, 15, Some("20:00"), "C",
            Some(1200.0), Some(1000.0), Some(900.0), Some(800.0),
        );
        assert!(metrics.get("xG").is_some());
        assert!(metrics.get("corsiForPercentage").is_some());
        assert!(metrics.get("goalsPer60").is_some());
    }
}
