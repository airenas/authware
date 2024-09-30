use std::sync::Arc;

use axum::{extract::State, http::HeaderMap};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::Utc;

use crate::{handler::login::extract_ip, model::service};

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    tracing::debug!("start keep_alive");
    let ip = extract_ip(&headers);
    tracing::debug!(ip = ip, "caller");
    return match bearer {
        None => {
            return Err(ApiError::NoSession());
        }
        Some(bearer) => {
            let session_id = bearer.token();
            tracing::info!(session_id = session_id, "keep_alive");
            let store = &data.store;
            let res = store.get(session_id).await?;

            res.check_ip(&ip)?;
            let now = Utc::now().timestamp_millis();
            res.check_expired(now)?;
            let config = &data.config;
            res.check_inactivity(now, config.inactivity)?;
            store.mark_last_used(session_id, now).await?;
            Ok(())
        }
    };
}
