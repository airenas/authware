use std::collections::HashMap;

use async_trait::async_trait;

use crate::{model::auth, utils::secret_str::SecretString, AuthService};

pub struct Sample {
    users: HashMap<String, auth::User>,
    passwords: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ParsedUser {
    user: String,
    pass: String,
    department: String,
    roles: Vec<String>,
}

impl ParsedUser {
    fn new(user: &str, pass: &str, department: &str, roles: &[String]) -> Self {
        Self {
            user: user.to_string(),
            pass: pass.to_string(),
            department: department.to_string(),
            roles: roles.to_vec(),
        }
    }

    fn to_auth_user(&self) -> auth::User {
        auth::User {
            id: self.user.clone(),
            name: self.user.clone(),
            department: self.department.clone(),
            roles: self.roles.clone(),
        }
    }
}

impl Sample {
    // Create a new Sample auth service from a string of user:pass pairs separated by semicolons.
    // Example input: "user1:pass1:department:role1,role2;user2:pass2:department2:HR"
    pub fn new(user_pass_pairs: &str) -> anyhow::Result<Self> {
        let data = parse_users(user_pass_pairs)?;
        let users: HashMap<String, auth::User> = data
            .iter()
            .map(|parsed| (parsed.user.clone(), parsed.to_auth_user()))
            .collect();
        let passwords = data
            .iter()
            .map(|parsed| (parsed.user.clone(), parsed.pass.clone()))
            .collect();

        users.iter().for_each(|(_, user)| {
            tracing::debug!(user = user.name, dep = user.department, roles = ?user.roles, "Sample");
        });

        Ok(Sample { users, passwords })
    }
}

// Parse a string of user:pass pairs separated by semicolons into a HashMap, value (pass, department, roles).
fn parse_users(user_pass_pairs: &str) -> anyhow::Result<Vec<ParsedUser>> {
    let mut users = vec![];
    if user_pass_pairs.is_empty() {
        return Ok(users);
    }
    for pair in user_pass_pairs.split(';') {
        let parsed: Vec<&str> = pair.split(':').collect();
        if parsed.len() < 2 || parsed.len() > 4 {
            return Err(anyhow::anyhow!("Invalid user:pass format in: {}", pair));
        }
        let user = parsed[0].to_string();
        let pass = parsed[1].to_string();
        if user.is_empty() || pass.is_empty() {
            return Err(anyhow::anyhow!(
                "User or password cannot be empty: '{}'",
                pair
            ));
        }
        let department = if parsed.len() >= 3 {
            parsed[2].to_string()
        } else {
            "IT dep of ".to_string() + &user
        };
        let roles = if parsed.len() == 4 {
            parsed[3]
                .split(',')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect()
        } else {
            vec!["USER".to_string()]
        };
        users.push(ParsedUser::new(&user, &pass, &department, &roles));
    }
    Ok(users)
}

#[async_trait]
impl AuthService for Sample {
    async fn login(&self, user: &str, pass: &SecretString) -> Result<auth::User, auth::Error> {
        match self.passwords.get(user) {
            Some(stored_pass) if stored_pass == pass.reveal_secret() => {
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

    #[test_case("", &[]; "empty")]
    #[test_case("olia:oo", 
        &[ParsedUser::new("olia", "oo", "IT dep of olia", &["USER".to_string()])]; "single entry")]
    #[test_case("olia:oo;IT:HR", 
    &[
        ParsedUser::new("olia", "oo", "IT dep of olia", &["USER".to_string()]),
        ParsedUser::new("IT", "HR", "IT dep of IT", &["USER".to_string()]),
    ]; "multiple entries")]
    #[test_case("olia:oo:dep:r1,r2;IT:HR:dep2:r1,r4", 
    &[
        ParsedUser::new("olia", "oo", "dep", &["r1".to_string(), "r2".to_string()]),
        ParsedUser::new("IT", "HR", "dep2", &["r1".to_string(), "r4".to_string()]),
    ]; "multiple entries with roles")]
    fn test_parse_users(input: &str, expected: &[ParsedUser]) {
        let actual = parse_users(input).unwrap();
        assert_eq!(expected, actual);
    }

    // Failure cases
    #[test_case("olia:"; "empty password")]
    #[test_case(":oo"; "empty user")]
    #[test_case("olia"; "invalid format")]

    fn test_parse_users_failure(input: &str) {
        let result = parse_users(input);
        assert!(result.is_err());
    }
}
