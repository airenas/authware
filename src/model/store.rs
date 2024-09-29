use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No session`")]
    NoSession(),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
