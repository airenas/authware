use crate::{AuthService, IPExtractor, SessionStore};

use super::config::SessionConfig;

#[derive()]
pub struct Data {
    pub config: SessionConfig,
    pub store: Box<dyn SessionStore + Send + Sync>,
    pub auth_service: Box<dyn AuthService + Send + Sync>,
    pub ip_extractor: Box<dyn IPExtractor + Send + Sync>,
    pub is_test_mode: bool,
}
