use std::collections::HashMap;

use async_trait::async_trait;

use crate::{model::auth, utils::secret_str::SecretString, AuthService};

pub struct Sample {
    users: HashMap<String, auth::User>,
    passwords: HashMap<String, String>,
}

impl Sample {
    // Create a new Sample auth service from a string of user:pass pairs separated by semicolons.
    // Example input: "user1:pass1:department:role1,role2;user2:pass2:department2:HR"
    pub fn new(user_pass_pairs: &str) -> anyhow::Result<Self> {
        let data = parse_users(user_pass_pairs)?;
        let users: HashMap<String, auth::User> = data
            .iter()
            .map(|(un, (_, dep, roles))| to_user(un, dep, roles))
            .collect();
        let passwords = data
            .iter()
            .map(|(un, (pass, _, _))| (un.clone(), pass.clone()))
            .collect();

        users.iter().for_each(|(_, user)| {
            tracing::debug!(user = user.name, dep = user.department, roles = ?user.roles, "Sample");
        });

        Ok(Sample { users, passwords })
    }
}

fn to_user(user_name: &str, department: &str, roles: &Vec<String>) -> (String, auth::User) {
    (
        user_name.to_string(),
        to_auth_user(user_name, department, roles),
    )
}

fn to_auth_user(user: &str, department: &str, roles: &Vec<String>) -> auth::User {
    auth::User {
        id: user.to_string(),
        name: user.to_string(),
        department: department.to_string(),
        roles: roles.to_vec(),
    }
}
// Parse a string of user:pass pairs separated by semicolons into a HashMap, value (pass, department, roles).
fn parse_users(
    user_pass_pairs: &str,
) -> anyhow::Result<HashMap<String, (String, String, Vec<String>)>> {
    let mut users = HashMap::new();
    if user_pass_pairs.is_empty() {
        return Ok(users);
    }
    for pair in user_pass_pairs.split(';') {
        let credentials: Vec<&str> = pair.split(':').collect();
        if credentials.len() < 2 || credentials.len() > 4 {
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
        let department = if credentials.len() >= 3 {
            credentials[2].to_string()
        } else {
            "IT dep of ".to_string() + &user
        };
        let roles = if credentials.len() == 4 {
            credentials[3]
                .split(',')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect()
        } else {
            vec!["USER".to_string()]
        };
        users.insert(user, (pass, department, roles));
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

    #[test_case("",  HashMap::new(); "empty")]
    #[test_case("olia:oo", {
        let mut map = HashMap::new();
        map.insert("olia".to_string(), ("oo".to_string(), "IT dep of olia".to_string(), vec!["USER".to_string()]));
        map
    }; "single entry")]
    #[test_case("olia:oo;IT:HR", {
        let mut map = HashMap::new();
        map.insert("olia".to_string(), ("oo".to_string(), "IT dep of olia".to_string(), vec!["USER".to_string()]));
        map.insert("IT".to_string(), ("HR".to_string(), "IT dep of IT".to_string(), vec!["USER".to_string()]));
        map
    }; "multiple entries")]
    #[test_case("olia:oo:dep:r1,r2;IT:HR:dep2:r1,r4", {
        let mut map = HashMap::new();
        map.insert("olia".to_string(), ("oo".to_string(), "dep".to_string(), vec!["r1".to_string(), "r2".to_string()]));
        map.insert("IT".to_string(), ("HR".to_string(), "dep2".to_string(), vec!["r1".to_string(), "r4".to_string()]));
        map
    }; "multiple entries with roles")]
    fn test_parse_users(input: &str, expected: HashMap<String, (String, String, Vec<String>)>) {
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
