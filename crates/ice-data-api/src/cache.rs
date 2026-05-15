use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheManager {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    data: String,
    expires_at: Instant,
}

impl CacheManager {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: String) {
        self.set_ttl(key, value, self.default_ttl).await;
    }

    pub async fn set_ttl(&self, key: String, value: String, ttl: Duration) {
        let mut store = self.store.write().await;
        store.insert(key, CacheEntry {
            data: value,
            expires_at: Instant::now() + ttl,
        });
    }

    pub async fn invalidate(&self, key: &str) {
        let mut store = self.store.write().await;
        store.remove(key);
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut store = self.store.write().await;
        store.retain(|k, _| !k.starts_with(prefix));
    }
}
