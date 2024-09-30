use async_trait::async_trait;
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Connection, Pool};
use std::cmp::max;

use crate::{model, SessionData, SessionStore};

pub struct RedisSessionStore {
    pool: Pool,
}

impl RedisSessionStore {
    pub fn new(pool: Pool) -> Self {
        RedisSessionStore { pool }
    }
    async fn get_conn(&self) -> Result<Connection, model::store::Error> {
        self.pool
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("Connection error: {:?}", e).into())
    }

    async fn add_int(
        &self,
        conn: &mut Connection,
        session_id: &str,
        data: SessionData,
    ) -> Result<(), model::store::Error> {
        let serialized_data = serde_json::to_string(&data)
            .map_err(|e| anyhow::anyhow!("Serialization error: {:?}", e))?;
        let now = Utc::now();
        let secs = (max(data.valid_till - now.timestamp_millis(), 0) / 1000) as u64;
        tracing::debug!("Session valid for: {} secs", secs);
        let _: () = conn
            .set_ex(session_id, serialized_data, secs)
            .await
            .map_err(|e| anyhow::anyhow!("Redis set error: {:?}", e))?;
        Ok(())
    }

    async fn get_int(
        &self,
        conn: &mut Connection,
        session_id: &str,
    ) -> Result<SessionData, model::store::Error> {
        let data: Option<String> = conn
            .get(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("Redis get error: {:?}", e))?;

        match data {
            Some(serialized_data) => {
                let session_data: SessionData = serde_json::from_str(&serialized_data)
                    .map_err(|e| anyhow::anyhow!("Deserialization error: {:?}", e))?;
                Ok(session_data)
            }
            None => Err(model::store::Error::NoSession()),
        }
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn add(&self, session_id: &str, data: SessionData) -> Result<(), model::store::Error> {
        tracing::info!("Adding session: {}", session_id);
        let mut conn = self.get_conn().await?;
        self.add_int(&mut conn, session_id, data).await
    }

    async fn get(&self, session_id: &str) -> Result<SessionData, model::store::Error> {
        let mut conn = self.get_conn().await?;
        self.get_int(&mut conn, session_id).await
    }

    async fn remove(&self, session_id: &str) -> Result<(), model::store::Error> {
        let mut conn = self.get_conn().await?;
        let result: usize = conn
            .del(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("Redis delete error: {:?}", e))?;
        if result == 0 {
            Err(model::store::Error::NoSession())
        } else {
            Ok(())
        }
    }

    async fn mark_last_used(&self, session_id: &str, now: i64) -> Result<(), model::store::Error> {
        let mut conn = self.get_conn().await?;
        let mut session_data = self.get_int(&mut conn, session_id).await?;
        session_data.last_access = now;
        self.add_int(&mut conn, session_id, session_data).await
    }
}
