use std::sync::Arc;

use axum::{extract::State, http::HeaderMap};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{handler::login::extract_ip, model::service};

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    let session_id = bearer.token();
    let ip = extract_ip(&headers);
    tracing::debug!(session_id = session_id, ip = ip, "logout");
    data.store.remove(session_id).await?;
    tracing::debug!(session_id = session_id, "logout done");
    Ok(())
}
