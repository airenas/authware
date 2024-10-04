use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};
use serde_xml_rs::from_str;
use urlencoding::encode;

use crate::{model::auth, AuthService};

pub struct Auth {
    ws_url: String,
    ws_user: String,
    ws_pass: String,
    app_code: String,
    client: reqwest::Client,
}

impl Auth {
    pub fn new(ws_url: &str, ws_user: &str, ws_pass: &str, app_code: &str) -> anyhow::Result<Self> {
        tracing::debug!(ws_url, ws_user, app_code, "init auth");
        if ws_url.is_empty() || ws_user.is_empty() || ws_pass.is_empty() || app_code.is_empty() {
            return Err(anyhow::anyhow!("Empty auth params"));
        }
        Ok(Auth {
            ws_url: ws_url.to_string(),
            ws_user: ws_user.to_string(),
            ws_pass: ws_pass.to_string(),
            app_code: app_code.to_string(),
            client: Client::builder().timeout(Duration::from_secs(5)).build()?,
        })
    }

    fn make_details_url(&self, user: &str, pass: &str) -> String {
        format!(
            "{}/authenticate_details/{}/{}/{}",
            self.ws_url,
            encode(&self.app_code),
            encode(user),
            encode(pass)
        )
    }

    fn make_roles_url(&self, user: &str) -> String {
        format!(
            "{}/get_roles/{}/{}",
            self.ws_url,
            encode(&self.app_code),
            encode(user)
        )
    }

    async fn make_call(&self, url: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(url)
            .basic_auth(&self.ws_user, Some(&self.ws_pass))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("url call error: {:?}", e))?;
        response
            .error_for_status_ref()
            .map_err(|e| anyhow::anyhow!("ws error: {:?}", e))?;
        let response_body = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("can't get body: {:?}", e))?;
        tracing::trace!(response = response_body, "response");
        Ok(response_body)
    }
}

#[async_trait]
impl AuthService for Auth {
    async fn login(&self, user: &str, pass: &str) -> Result<auth::User, auth::Error> {
        tracing::debug!(user = user, app = &self.app_code, "call auth details");
        let user_details = self.make_call(&self.make_details_url(user, pass)).await?;
        let user_data: User = process_body(&user_details)?;
        tracing::debug!("got user");

        tracing::debug!(user = user, app = &self.app_code, "call roles");
        let roles_details = self.make_call(&self.make_roles_url(user)).await?;
        let roles: Roles = process_body(&roles_details)?;
        tracing::debug!(len = roles.roles.len(), "got roles");
        if roles.roles.is_empty() {
            return Err(auth::Error::NoAccess());
        }
        map_res(user_data, roles)
    }
}

fn map_res(user_data: User, roles: Roles) -> Result<auth::User, auth::Error> {
    let roles_str: Vec<String> = roles.roles.iter().map(|r| r.name.clone()).collect();
    let res = auth::User {
        name: user_data.first_name + " " + &user_data.last_name,
        department: user_data.organization_unit.name,
        roles: roles_str,
    };
    Ok(res)
}

#[derive(Debug, Deserialize)]
struct User {
    first_name: String,
    last_name: String,
    organization_unit: Department,
}

#[derive(Debug, Deserialize)]
struct Department {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Role {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Roles {
    #[serde(rename = "role")]
    roles: Vec<Role>,
}

fn process_body<T>(response_body: &str) -> Result<T, auth::Error>
where
    T: DeserializeOwned,
{
    if response_body.starts_with("<") {
        let res: T =
            from_str(response_body).map_err(|e| anyhow::anyhow!("can't deserialize: {:?}", e))?;
        return Ok(res);
    }
    let res_code: i32 = response_body
        .parse::<i32>()
        .map_err(|e| anyhow::anyhow!("can't parse to int {}: {:?}", response_body, e))?;
    Err(map_err_codes(res_code))
}

fn map_err_codes(res_code: i32) -> auth::Error {
    match res_code {
        1 => auth::Error::WrongUserPass(),
        4 => auth::Error::WrongUserPass(),
        5 => auth::Error::WrongUserPass(),
        2 => auth::Error::ExpiredPass(),
        3 => auth::Error::ExpiredPass(),
        _ => auth::Error::Other(anyhow::anyhow!("Auth Service error: {}", res_code)),
    }
}
