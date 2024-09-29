use std::sync::Arc;

use axum::{extract::State, http::HeaderMap};
use chrono::Utc;
use url::Url;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::model::service;

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    let forwarded_uri = headers
        .get("X-Forwarded-Uri")
        .map(|h| h.to_str().unwrap_or("").to_string());

    tracing::info!(url = forwarded_uri, "auth");
    let session_id = match bearer {
        None => match forwarded_uri {
            Some(token) => parse_token_from_url(&token).unwrap_or_default(),
            None => {
                return Err(ApiError::NoSession());
            }
        },
        Some(val) => val.token().to_string(),
    };
    tracing::info!(session_id = session_id, "auth");
    let store = &data.store;
    let res = store.get(&session_id).await?;
    tracing::info!(session_id = session_id, user = res.user, "auth");
    let now = Utc::now().timestamp_millis();
    res.check_expired(now)?;
    let config = &data.config;
    res.check_inactivity(now, config.inactivity)?;
    Ok(())
}

fn parse_token_from_url(url: &str) -> Option<String> {
    tracing::info!(url = url, "parse_token_from_url");
    let parsed_url = Url::parse(&format!("http://localhost{}", url)).ok()?;
    tracing::info!("parsed");
    parsed_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string())
}
