use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Clone)]
#[allow(dead_code)]
pub struct RateLimiter {
    state: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    #[allow(dead_code)]
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    #[allow(dead_code)]
    pub async fn check(&self, key: &str) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let timestamps = state.entry(key.to_string()).or_default();

        timestamps.retain(|t| now - *t < self.window);
        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

#[allow(dead_code)]
pub async fn rate_limit_middleware(
    limiter: axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !limiter.check(client_ip).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": {"code": 429, "message": "Rate limit exceeded. Try again later."}})),
        ));
    }

    Ok(next.run(req).await)
}
