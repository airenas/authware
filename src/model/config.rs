#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub inactivity: i64,      // Unix timestamp
    pub session_timeout: i64, // Unix timestamp
}
