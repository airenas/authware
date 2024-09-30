use serde::{Deserialize, Serialize};

use crate::handler::error::ApiError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionData {
    pub user: String,
    pub ip: String,
    pub valid_till: i64,  // Unix timestamp
    pub last_access: i64, // Unix timestamp
}

impl SessionData {
    pub fn check_expired(&self, now: i64) -> Result<(), ApiError> {
        if self.valid_till < now {
            return Err(ApiError::ExpiredSession());
        }
        Ok(())
    }
    pub fn check_inactivity(&self, now: i64, dur: i64) -> Result<(), ApiError> {
        if self.last_access + dur < now {
            return Err(ApiError::ExpiredSession());
        }
        Ok(())
    }
    pub fn ip(&self, ip: &str) -> Result<(), ApiError> {
        if self.ip != ip {
            return Err(ApiError::NoSession());
        }
        Ok(())
    }
}
