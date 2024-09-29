use std::sync::Arc;

use axum::extract::State;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::model::service;

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    let session_id = bearer.token();
    tracing::debug!(session_id = session_id, "logout");
    data.store.remove(session_id).await?;
    tracing::debug!(session_id = session_id, "logout done");
    Ok(())
}
