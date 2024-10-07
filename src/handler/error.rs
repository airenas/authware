use std::borrow::Cow;

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
    #[error("No access`")]
    NoAccess(),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message): (StatusCode, Cow<'static, str>) = match self {
            ApiError::BadRequest(msg, details) => {
                tracing::warn!("{}: {}", msg, details);
                (StatusCode::BAD_REQUEST, Cow::Owned(msg))
            }
            ApiError::Server(msg) => {
                tracing::error!("{}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Cow::Borrowed("Internal Server Error"),
                )
            }
            ApiError::Other(err) => {
                tracing::error!("{}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Cow::Borrowed("Internal Server Error"),
                )
            }
            ApiError::WrongUserPass() => {
                tracing::warn!("Wrong user or password");
                (
                    StatusCode::UNAUTHORIZED,
                    Cow::Borrowed("Wrong user or password"),
                )
            }
            ApiError::ExpiredPass() => {
                tracing::warn!("Expired pass");
                (StatusCode::UNAUTHORIZED, Cow::Borrowed("Password expired"))
            }
            ApiError::ExpiredSession() => {
                tracing::warn!("Expired session");
                (StatusCode::UNAUTHORIZED, Cow::Borrowed("Session expired"))
            }
            ApiError::NoSession() => {
                tracing::warn!("No session");
                (StatusCode::UNAUTHORIZED, Cow::Borrowed("No session"))
            }
            ApiError::NoAccess() => {
                tracing::warn!("No access");
                (StatusCode::UNAUTHORIZED, Cow::Borrowed("No access"))
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
            auth::Error::NoAccess() => ApiError::NoAccess(),
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
