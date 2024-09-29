use async_trait::async_trait;

use crate::{model::auth, AuthService};

pub struct Sample {
    user: auth::User,
    pass: String,
}

impl Sample {
    pub fn new(user: &str, pass: &str) -> anyhow::Result<Self> {
        if user.is_empty() || pass.is_empty() {
            return Err(anyhow::anyhow!("User or pass is empty"));
        }
        tracing::info!("Creating Sample auth service with user: {}", user);
        Ok(Sample {
            user: auth::User {
                name: user.to_string(),
                department: "IT".to_string(),
                roles: vec!["USER".to_string()],
            },
            pass: pass.to_string(),
        })
    }
}

#[async_trait]
impl AuthService for Sample {
    async fn login(&self, user: &str, pass: &str) -> Result<auth::User, auth::Error> {
        if user == self.user.name && pass == self.pass {
            Ok(self.user.clone())
        } else {
            Err(auth::Error::WrongUserPass())
        }
    }
}
