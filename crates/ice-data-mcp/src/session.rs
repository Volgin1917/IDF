use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub type SessionId = String;

#[derive(Debug, Clone)]
pub enum SseEvent {
    Message(String),
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, broadcast::Sender<SseEvent>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self) -> (SessionId, broadcast::Receiver<SseEvent>) {
        let id = Uuid::new_v4().to_string();
        let (tx, rx) = broadcast::channel(256);
        self.sessions.write().await.insert(id.clone(), tx);
        (id, rx)
    }

    pub async fn send_to_session(&self, session_id: &str, event: SseEvent) -> bool {
        if let Some(tx) = self.sessions.read().await.get(session_id) {
            tx.send(event).is_ok()
        } else {
            false
        }
    }

    pub async fn remove_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }

    pub async fn subscribe(&self, session_id: &str) -> Option<broadcast::Receiver<SseEvent>> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|tx| tx.subscribe())
    }
}
