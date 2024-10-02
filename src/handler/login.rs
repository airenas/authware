use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{self, State},
    http::HeaderMap,
    Json,
};
use base64::{prelude::BASE64_URL_SAFE, Engine};
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::model::{data::SessionData, service};

use super::error::ApiError;

#[derive(Serialize, Clone, Deserialize)]
pub struct Request {
    user: Option<String>,
    pass: Option<String>,
}

// Define the response structure for returning a session ID
#[derive(Serialize)]
pub struct User {
    name: String,
    department: String,
    roles: Vec<String>,
}

#[derive(Serialize)]
pub struct Response {
    session_id: String,
    user: User,
}

#[debug_handler]
pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    Json(payload): Json<Request>,
) -> Result<extract::Json<Response>, ApiError> {
    let user = payload.user.as_deref().unwrap_or("");
    tracing::debug!(user = user, "starting login");
    if payload.user.is_none() || payload.pass.is_none() {
        return Err(ApiError::WrongUserPass());
    }
    let ip = extract_ip(&headers);

    let now = Utc::now();
    let cfg = &data.config;
    let store = &data.store;
    let auth = &data.auth_service;

    let pass = payload.pass.as_deref().unwrap_or("");

    tracing::debug!(user = user, ip = ip, "call auth service login");
    let res = auth.login(user, pass).await?;
    tracing::trace!(user = user, "got result");
    tracing::trace!(user = user, "creating session");
    let session_id = generate_session();
    tracing::trace!(user = user, "saving");
    store
        .add(
            &session_id,
            SessionData {
                user: res.name.clone(),
                ip,
                valid_till: now.timestamp_millis() + cfg.session_timeout,
                last_access: now.timestamp_millis(),
            },
        )
        .await?;
    tracing::trace!(user = user, "saved");
    let response = Response {
        session_id,
        user: User {
            name: res.name.clone(),
            department: res.department.clone(),
            roles: res.roles.clone(),
        },
    };
    Ok(Json(response))
}

fn generate_session() -> String {
    let mut rng = OsRng;
    let mut session_id_bytes = [0u8; 128];
    rng.fill_bytes(&mut session_id_bytes);
    BASE64_URL_SAFE.encode(session_id_bytes)
}

pub fn extract_ip(headers: &HeaderMap) -> String {
    let ips = headers
        .get("x-forwarded-for")
        .and_then(|header_value| header_value.to_str().ok());
    tracing::trace!(ips = ?ips, "ips");
    return ips
        .and_then(|ip_list| ip_list.split(',').next())
        .unwrap_or("")
        .to_string();
}
