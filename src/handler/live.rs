use axum::{extract, Json};
use serde::Serialize;

use super::error::ApiError;

#[derive(Serialize, Clone)]
pub struct LiveResult {
    success: bool,
    version: String,
}

pub async fn handler() -> Result<extract::Json<LiveResult>, ApiError> {
    let res = LiveResult {
        success: true,
        version: env!("CARGO_APP_VERSION").to_string(),
    };
    Ok(Json(res))
}
