use async_trait::async_trait;
use chrono::Utc;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::{model, SessionData, SessionStore};

struct DB {
    store: HashMap<String, SessionData>,
    expirations: BTreeSet<(i64, String)>,
}

pub struct InMemorySessionStore {
    store: Arc<Mutex<DB>>,
}

impl DB {
    fn new() -> Self {
        DB {
            store: HashMap::new(),
            expirations: BTreeSet::new(),
        }
    }
    fn insert(&mut self, session_id: &str, data: SessionData) {
        self.expirations
            .insert((data.valid_till, session_id.to_string()));
        self.store.insert(session_id.to_string(), data);
        self.remove_expired();
    }
    fn get(&mut self, session_id: &str) -> Option<&SessionData> {
        self.remove_expired();
        self.store.get(session_id)
    }
    fn get_mut(&mut self, session_id: &str) -> Option<&mut SessionData> {
        self.remove_expired();
        self.store.get_mut(session_id)
    }
    fn remove(&mut self, session_id: &str) -> Option<SessionData> {
        self.store.remove(session_id)
        // it leaves the expired entry in the expirations set, it will be removed after expiration
    }

    fn remove_expired_int(&mut self, now: i64) {
        let mut to_remove = Vec::new();
        for (expiry_time, key) in &self.expirations {
            if *expiry_time > now {
                break;
            }
            to_remove.push((*expiry_time, key.clone()));
        }
        for (expiry_time, key) in to_remove {
            self.store.remove(&key);
            self.expirations.remove(&(expiry_time, key));
        }
    }

    fn remove_expired(&mut self) {
        let now = Utc::now().timestamp_millis();
        self.remove_expired_int(now);
    }
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        InMemorySessionStore {
            store: Arc::new(Mutex::new(DB::new())),
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
        let mut store = self.store.lock().await;
        store.insert(session_id, data);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<SessionData, model::store::Error> {
        let mut store = self.store.lock().await;
        match store.get(session_id) {
            Some(data) => Ok(data.clone()),
            None => Err(model::store::Error::NoSession()),
        }
    }

    async fn remove(&self, session_id: &str) -> Result<(), model::store::Error> {
        let mut store = self.store.lock().await;
        match store.remove(session_id) {
            Some(_) => Ok(()),
            None => Err(model::store::Error::NoSession()),
        }
    }
    async fn mark_last_used(&self, session_id: &str, now: i64) -> Result<(), model::store::Error> {
        let mut store = self.store.lock().await;
        match store.get_mut(session_id) {
            Some(data) => {
                data.last_access = now;
                Ok(())
            }
            None => Err(model::store::Error::NoSession()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::auth::User;

    use super::*;

    fn _session_data(at: i64) -> SessionData {
        SessionData {
            user: User {
                id: "test_user".to_string(),
                name: "Test User".to_string(),
                department: "Test Department".to_string(),
                roles: vec!["admin".to_string()],
            },
            ip: "".to_string(),
            valid_till: at,
            last_access: 20,
        }
    }

    #[test]
    fn test_db_add() {
        let mut db = DB::new();
        let session_id = "test";
        let data = _session_data(Utc::now().timestamp_millis() + 1000);
        db.insert(session_id, data.clone());
        assert_eq!(db.store.len(), 1);
        assert_eq!(db.expirations.len(), 1);
        assert_eq!(db.store.get(session_id), Some(&data));
    }

    #[test]
    fn test_db_add_expired() {
        let mut db = DB::new();
        let session_id = "test";
        let data = _session_data(Utc::now().timestamp_millis() - 1000);
        db.insert(session_id, data.clone());
        assert_eq!(db.store.len(), 0);
        assert_eq!(db.expirations.len(), 0);
        assert_eq!(db.store.get(session_id), None);
    }

    #[test]
    fn test_db_get_expired() {
        let mut db = DB::new();
        let session_id = "test";
        let data = _session_data(Utc::now().timestamp_millis() + 1000);
        db.insert(session_id, data.clone());
        assert_eq!(db.store.len(), 1);
        db.remove_expired_int(Utc::now().timestamp_millis() + 1001);
        assert_eq!(db.store.get(session_id), None);
    }

    #[test]
    fn test_db_remove() {
        let mut db = DB::new();
        let session_id = "test";
        let data = _session_data(Utc::now().timestamp_millis() + 1000);
        db.insert(session_id, data.clone());
        assert_eq!(db.store.len(), 1);
        db.remove(session_id);
        assert_eq!(db.store.get(session_id), None);
    }
}
