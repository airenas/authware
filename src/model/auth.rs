use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub department: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Wrong password")]
    WrongUserPass(),
    #[error("Expired password")]
    ExpiredPass(),
    #[error("No access")]
    NoAccess(),
    #[error("Other Auth error")]
    OtherAuth(String),
    #[error(transparent)]
    ServiceError(#[from] anyhow::Error),
}
