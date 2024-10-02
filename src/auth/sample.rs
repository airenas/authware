use std::collections::HashMap;

use async_trait::async_trait;

use crate::{model::auth, AuthService};

pub struct Sample {
    users: HashMap<String, auth::User>,
    passwords: HashMap<String, String>,
}

impl Sample {
    pub fn new(user_pass_pairs: &str) -> anyhow::Result<Self> {
        let passwords = parse_user_pass(user_pass_pairs)?;
        let users = passwords.keys().map(|un| to_user(un)).collect();
        passwords.keys().for_each(|un| {
            tracing::debug!(user = un, "Sample");
        });

        Ok(Sample { users, passwords })
    }
}

fn to_user(user_name: &str) -> (String, auth::User) {
    (user_name.to_string(), to_auth_user(user_name))
}

fn to_auth_user(user: &str) -> auth::User {
    auth::User {
        name: user.to_string(),
        department: "IT".to_string(),
        roles: vec!["USER".to_string()],
    }
}

fn parse_user_pass(user_pass_pairs: &str) -> anyhow::Result<HashMap<String, String>> {
    let mut users = HashMap::new();
    if user_pass_pairs.is_empty() {
        return Ok(users);
    }
    for pair in user_pass_pairs.split(';') {
        let credentials: Vec<&str> = pair.split(':').collect();
        if credentials.len() != 2 {
            return Err(anyhow::anyhow!("Invalid user:pass format in: {}", pair));
        }
        let user = credentials[0].to_string();
        let pass = credentials[1].to_string();
        if user.is_empty() || pass.is_empty() {
            return Err(anyhow::anyhow!(
                "User or password cannot be empty: '{}'",
                pair
            ));
        }
        users.insert(user, pass);
    }
    Ok(users)
}

#[async_trait]
impl AuthService for Sample {
    async fn login(&self, user: &str, pass: &str) -> Result<auth::User, auth::Error> {
        match self.passwords.get(user) {
            Some(stored_pass) if stored_pass == pass => {
                if let Some(auth_user) = self.users.get(user) {
                    Ok(auth_user.clone())
                } else {
                    Err(auth::Error::WrongUserPass())
                }
            }
            _ => Err(auth::Error::WrongUserPass()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("",  HashMap::new(); "empty")]
    #[test_case("olia:oo", {
        let mut map = HashMap::new();
        map.insert("olia".to_string(), "oo".to_string());
        map
    }; "single entry")]
    fn test_parse_user_pass(input: &str, expected: HashMap<String, String>) {
        let actual = parse_user_pass(input).unwrap();
        assert_eq!(expected, actual);
    }

    // Failure cases
    #[test_case("olia:"; "empty password")]
    #[test_case(":oo"; "empty user")]
    #[test_case("olia"; "invalid format")]

    fn test_parse_user_pass_failure(input: &str) {
        let result = parse_user_pass(input);
        assert!(result.is_err());
    }
}
