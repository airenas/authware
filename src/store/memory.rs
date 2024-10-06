use async_trait::async_trait;
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{model, SessionData, SessionStore};

pub struct InMemorySessionStore {
    store: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        InMemorySessionStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn add(&self, session_id: &str, data: SessionData) -> Result<(), model::store::Error> {
        tracing::trace!("Adding session: {}", session_id);
        let mut store = self
            .store
            .write().await;
        check_remove_sessions(&mut store);
        store.insert(session_id.to_string(), data);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<SessionData, model::store::Error> {
        let store = self
            .store
            .read().await;
        match store.get(session_id) {
            Some(data) => Ok(data.clone()),
            None => Err(model::store::Error::NoSession()),
        }
    }

    async fn remove(&self, session_id: &str) -> Result<(), model::store::Error> {
        let mut store = self
            .store
            .write().await;
        match store.remove(session_id) {
            Some(_) => Ok(()),
            None => Err(model::store::Error::NoSession()),
        }
    }
    async fn mark_last_used(&self, session_id: &str, now: i64) -> Result<(), model::store::Error> {
        let mut store = self
            .store
            .write().await;
        match store.get_mut(session_id) {
            Some(data) => {
                data.last_access = now;
                Ok(())
            }
            None => Err(model::store::Error::NoSession()),
        }
    }
}

fn check_remove_sessions(store: &mut HashMap<String, SessionData>) {
    let now = Utc::now().timestamp_millis();

    store.retain(|key, session_data| {
        if session_data.valid_till <= now {
            tracing::trace!("remove session: {}", key);
            false
        } else {
            true
        }
    });
}
