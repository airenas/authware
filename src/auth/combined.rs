use async_trait::async_trait;

use crate::{model::auth, utils::secret_str::SecretString, AuthService};

pub struct Auths {
    auths: Vec<Box<dyn AuthService + Send + Sync>>,
}

impl Auths {
    pub fn new(auths: Vec<Box<dyn AuthService + Send + Sync>>) -> anyhow::Result<Self> {
        if auths.is_empty() {
            return Err(anyhow::anyhow!("No auth services provided"));
        }
        Ok(Auths { auths })
    }
}

#[async_trait]
impl AuthService for Auths {
    async fn login(&self, user: &str, pass: &SecretString) -> Result<auth::User, auth::Error> {
        let mut last_err: Option<auth::Error> = None;
        for auth in &self.auths {
            match auth.login(user, pass).await {
                Ok(user) => return Ok(user),
                Err(err) => last_err = Some(err),
            }
        }
        last_err.map_or_else(|| Err(auth::Error::NoAccess()), Err)
    }
}
