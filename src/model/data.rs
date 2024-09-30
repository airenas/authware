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
    pub fn check_ip(&self, ip: &str) -> Result<(), ApiError> {
        if self.ip != ip {
            return Err(ApiError::NoSession());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    // Failure cases
    #[test_case("2.2.2.2", true; "ok")]
    #[test_case("2.2.2.222", false; "fail")]
    #[test_case("", false; "fail empty")]

    fn test_ip(ip: &str, ok: bool) {
        let to = session_data();
        let res = to.check_ip(ip);
        assert_eq!(ok, res.is_ok());
    }

    #[test_case(800, 299, false; "ok")]
    #[test_case(800, 301, true; "ok 2")]
    #[test_case(800, 100, false; "fail expired")]

    fn test_check_inactivity(now: i64, dur: i64, ok: bool) {
        let to = session_data();
        let res = to.check_inactivity(now, dur);
        assert_eq!(ok, res.is_ok());
    }

    fn session_data() -> SessionData {
        SessionData {
            user: "admin".to_string(),
            ip: "2.2.2.2".to_string(),
            valid_till: 1000,
            last_access: 500,
        }
    }
}
