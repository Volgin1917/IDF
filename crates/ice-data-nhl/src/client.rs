use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use reqwest::StatusCode;
use tokio::sync::Semaphore;

use crate::types::*;

const SEARCH_BASE: &str = "https://search.d3.nhle.com/api/v1";
const API_BASE: &str = "https://api-web.nhle.com/v1";
const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

#[derive(Debug, thiserror::Error)]
pub enum NhlError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Rate limited, retry after {0}ms")]
    RateLimited(u64),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Max retries exceeded")]
    MaxRetries,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub struct NhlClient {
    client: reqwest::Client,
    semaphore: Semaphore,
    last_request: AtomicU64,
}

impl NhlClient {
    pub fn new(rate_per_minute: u64) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("IceDataForge/0.1")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            semaphore: Semaphore::new(rate_per_minute as usize),
            last_request: AtomicU64::new(0),
        }
    }

    pub async fn search_players(&self, query: &str, limit: i32) -> Result<Vec<SearchResult>, NhlError> {
        let url = format!("{SEARCH_BASE}/search/player");
        let resp = self
            .request_with_retry(|| {
                self.client
                    .get(&url)
                    .query(&[("culture", "en-us"), ("limit", &limit.to_string()), ("q", query)])
                    .send()
            })
            .await?;

        let data: SearchResponse = resp.json().await.map_err(NhlError::Http)?;
        Ok(data.results)
    }

    pub async fn player_landing(&self, player_id: i32) -> Result<PlayerLandingResponse, NhlError> {
        let url = format!("{API_BASE}/player/{player_id}/landing");
        let resp = self.request_with_retry(|| self.client.get(&url).send()).await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Err(NhlError::NotFound(format!("Player {player_id}")));
        }

        let data: PlayerLandingResponse = resp.json().await.map_err(NhlError::Http)?;
        Ok(data)
    }

    pub async fn team_roster(&self, team: &str, season: &str) -> Result<Vec<RosterEntry>, NhlError> {
        let url = format!("{API_BASE}/roster/{team}/{season}");
        let resp = self.request_with_retry(|| self.client.get(&url).send()).await?;

        let data: RosterResponse = resp.json().await.map_err(NhlError::Http)?;
        let mut players = data.forwards;
        players.extend(data.defensemen);
        players.extend(data.goalies);
        Ok(players)
    }

    async fn request_with_retry<F, Fut>(&self, request_fn: F) -> Result<reqwest::Response, NhlError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        self.throttle().await;

        for attempt in 0..MAX_RETRIES {
            self.record_request();
            let resp = request_fn().await.map_err(NhlError::Http)?;

            match resp.status() {
                StatusCode::OK => return Ok(resp),
                StatusCode::TOO_MANY_REQUESTS => {
                    let delay = BASE_DELAY_MS * 2u64.pow(attempt);
                    tracing::warn!("Rate limited, retrying in {delay}ms (attempt {})", attempt + 1);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                StatusCode::NOT_FOUND => {
                    return Err(NhlError::NotFound("Resource not found".to_string()));
                }
                StatusCode::SERVICE_UNAVAILABLE => {
                    let delay = BASE_DELAY_MS * 2u64.pow(attempt);
                    tracing::warn!("Service unavailable, retrying in {delay}ms (attempt {})", attempt + 1);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                _ => return Ok(resp),
            }
        }

        Err(NhlError::MaxRetries)
    }

    async fn throttle(&self) {
        let permit = self.semaphore.acquire().await;
        let now = Instant::now().elapsed().as_millis() as u64;
        let last = self.last_request.load(Ordering::Relaxed);
        if now - last < 1000 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        drop(permit);
    }

    fn record_request(&self) {
        self.last_request
            .store(Instant::now().elapsed().as_millis() as u64, Ordering::Relaxed);
    }
}
