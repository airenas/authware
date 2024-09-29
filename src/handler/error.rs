use axum::response::IntoResponse;
use reqwest::StatusCode;
use thiserror::Error;

use crate::model::{auth, store};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}, details: {1}")]
    BadRequest(String, String),
    #[error("Server error: {0}`")]
    Server(String),
    #[error("Wrong password`")]
    WrongUserPass(),
    #[error("Expired password`")]
    ExpiredPass(),
    #[error("Expired session`")]
    ExpiredSession(),
    #[error("No session`")]
    NoSession(),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg, details) => {
                tracing::warn!("{}: {}", msg, details);
                (StatusCode::BAD_REQUEST, msg)
            }
            ApiError::Server(msg) => {
                tracing::error!("{}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            ApiError::Other(err) => {
                tracing::error!("{}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            ApiError::WrongUserPass() => {
                tracing::error!("Wrong user or password");
                (
                    StatusCode::UNAUTHORIZED,
                    "Wrong user or password".to_string(),
                )
            }
            ApiError::ExpiredPass() => {
                tracing::error!("Expired pass");
                (StatusCode::UNAUTHORIZED, "Expired password".to_string())
            }
            ApiError::ExpiredSession() => {
                tracing::error!("Expired session");
                (StatusCode::UNAUTHORIZED, "Session expired".to_string())
            }
            ApiError::NoSession() => {
                tracing::error!("No session");
                (StatusCode::UNAUTHORIZED, "No session".to_string())
            }
        };

        (status, message).into_response()
    }
}

impl From<auth::Error> for ApiError {
    fn from(error: auth::Error) -> Self {
        match error {
            auth::Error::WrongUserPass() => ApiError::WrongUserPass(),
            auth::Error::ExpiredPass() => ApiError::ExpiredPass(),
            auth::Error::Other(error) => ApiError::Other(error),
        }
    }
}

impl From<store::Error> for ApiError {
    fn from(error: store::Error) -> Self {
        match error {
            store::Error::NoSession() => ApiError::NoSession(),
            store::Error::Other(error) => ApiError::Other(error),
        }
    }
}
