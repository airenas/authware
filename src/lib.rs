pub mod auth;
pub mod model;
pub mod store;

use async_trait::async_trait;
use model::data::SessionData;
use tokio::signal;

pub mod handler;

// Define a trait for session store
#[async_trait]
pub trait SessionStore {
    async fn add(&self, session_id: &str, data: SessionData) -> Result<(), model::store::Error>;
    async fn get(&self, session_id: &str) -> Result<SessionData, model::store::Error>;
    async fn remove(&self, session_id: &str) -> Result<(), model::store::Error>;
    async fn mark_last_used(&self, session_id: &str, now: i64) -> Result<(), model::store::Error>;
}

#[async_trait]
pub trait AuthService {
    async fn login(&self, user: &str, pass: &str) -> Result<model::auth::User, model::auth::Error>;
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            log::info!("Ctrl-C received, shutting down");
        },
        _ = terminate => {
            log::info!("SIGTERM received, shutting down");
        },
    }
}
