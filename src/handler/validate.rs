use std::sync::Arc;

use axum::{extract::State, http::HeaderMap};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::Utc;

use crate::model::service;

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    tracing::debug!("start validate");
    let ip = data.ip_extractor.get(&headers);
    tracing::debug!(ip = ip.as_ref(), "caller");
    match bearer {
        None => Err(ApiError::NoSession()),
        Some(bearer) => {
            let session_id = bearer.token();
            tracing::debug!(session_id = session_id, "validate");
            let store = &data.store;
            let res = store.get(session_id).await?;

            res.check_ip(&ip)?;
            let now = Utc::now().timestamp_millis();
            res.check_expired(now)?;
            let config = &data.config;
            res.check_inactivity(now, config.inactivity)?;
            Ok(())
        }
    }
}
