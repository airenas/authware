use serde::Serialize;

use crate::model;

// Define the response structure for returning User info
#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub department: String,
    pub roles: Vec<String>,
}

impl From<model::auth::User> for User {
    fn from(user: model::auth::User) -> Self {
        User {
            id: user.id,
            name: user.name,
            department: user.department,
            roles: user.roles,
        }
    }
}
