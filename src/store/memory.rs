use async_trait::async_trait;
use chrono::Utc;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{model, SessionData, SessionStore};

pub struct InMemorySessionStore {
    store: Arc<Mutex<HashMap<String, SessionData>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        InMemorySessionStore {
            store: Arc::new(Mutex::new(HashMap::new())),
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
        tracing::info!("Adding session: {}", session_id);
        let mut store = self
            .store
            .lock()
            .map_err(|e| anyhow::anyhow!("Locking error: {:?}", e))?;
        check_remove_sessions(&mut store);
        store.insert(session_id.to_string(), data);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<SessionData, model::store::Error> {
        let store = self
            .store
            .lock()
            .map_err(|e| anyhow::anyhow!("Locking error: {:?}", e))?;
        match store.get(session_id) {
            Some(data) => Ok(data.clone()),
            None => Err(model::store::Error::NoSession()),
        }
    }

    async fn remove(&self, session_id: &str) -> Result<(), model::store::Error> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| anyhow::anyhow!("Locking error: {:?}", e))?;
        match store.remove(session_id) {
            Some(_) => Ok(()),
            None => Err(model::store::Error::NoSession()),
        }
    }
    async fn mark_last_used(&self, session_id: &str, now: i64) -> Result<(), model::store::Error> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| anyhow::anyhow!("Locking error: {:?}", e))?;
        match store.get_mut(session_id) {
            Some(data) => {
                data.last_access = now;
                Ok(())
            }
            None => Err(model::store::Error::NoSession()),
        }
    }
}

fn check_remove_sessions(store: &mut HashMap<String, SessionData>) -> () {
    let now = Utc::now().timestamp_millis();

    store.retain(|key, session_data| {
        if session_data.valid_till <= now {
            tracing::info!("remove session: {}", key);
            false
        } else {
            true
        }
    });
}
