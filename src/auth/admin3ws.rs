use std::time::Duration;

use again::RetryPolicy;
use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};
use serde_xml_rs::from_str;
use urlencoding::encode;

use crate::{model::auth, utils::secret_str::SecretString, AuthService};
pub struct Auth {
    ws_url: String,
    ws_user: String,
    ws_pass: SecretString,
    app_code: String,
    client: reqwest::Client,
}

impl Auth {
    pub fn new(
        ws_url: &str,
        ws_user: &str,
        ws_pass: SecretString,
        app_code: &str,
    ) -> anyhow::Result<Self> {
        tracing::debug!(ws_url, ws_user, app_code, "init auth");
        if ws_url.is_empty()
            || ws_user.is_empty()
            || ws_pass.reveal_secret().is_empty()
            || app_code.is_empty()
        {
            return Err(anyhow::anyhow!("Empty auth params"));
        }
        Ok(Auth {
            ws_url: ws_url.to_string(),
            ws_user: ws_user.to_string(),
            ws_pass,
            app_code: app_code.to_string(),
            client: Client::builder().timeout(Duration::from_secs(5)).build()?,
        })
    }

    fn make_details_url(&self, user: &str, pass: &SecretString) -> String {
        format!(
            "{}/authenticate_details/{}/{}/{}",
            self.ws_url,
            encode(&self.app_code),
            encode(user),
            encode(pass.reveal_secret())
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

    async fn make_call_int(&self, url: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(url)
            .basic_auth(&self.ws_user, Some(self.ws_pass.reveal_secret()))
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

    async fn make_call(&self, url: &str) -> anyhow::Result<String> {
        let policy = RetryPolicy::exponential(Duration::from_millis(200))
            .with_max_retries(3)
            .with_jitter(true);
        policy
            .retry(|| self.make_call_int(url))
            .await
            .map_err(|e| anyhow::anyhow!("Failed after retries: {:?}", e))
    }
}

#[async_trait]
impl AuthService for Auth {
    async fn login(&self, user: &str, pass: &SecretString) -> Result<auth::User, auth::Error> {
        tracing::trace!(
            url = self.make_details_url(user, &"----".into()),
            "call auth details"
        );
        let user_details = self.make_call(&self.make_details_url(user, pass)).await?;
        let user_data: User = process_body(&user_details)?;
        tracing::trace!("got user");

        let r_url = self.make_roles_url(user);
        tracing::trace!(url = r_url, "call roles");
        let roles_details = self.make_call(&r_url).await?;
        let roles: Roles = process_body(&roles_details)?;
        tracing::trace!(
            len = roles.roles.as_ref().map_or(0, |vec| vec.len()),
            "got roles"
        );
        if roles.roles.as_ref().is_none_or(|vec| vec.is_empty()) {
            return Err(auth::Error::NoAccess());
        }
        map_res(user, user_data, roles)
    }
}

fn map_res(id: &str, user_data: User, roles: Roles) -> Result<auth::User, auth::Error> {
    let roles_str: Vec<String> = match roles.roles {
        Some(roles) => roles.into_iter().map(|r| r.name).collect(),
        None => Vec::new(), // or vec![] to create an empty Vec<String>
    };
    let dep = user_data
        .organization_unit
        .as_ref() // Convert Option<Department> to Option<&Department>
        .map_or_else(|| "", |dept| &dept.name);
    let res = auth::User {
        id: id.to_string(),
        name: user_data.first_name + " " + &user_data.last_name,
        department: dep.to_string(),
        roles: roles_str,
    };
    Ok(res)
}

#[derive(Debug, Deserialize, PartialEq)]
struct User {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    #[serde(rename = "organizationUnit")]
    organization_unit: Option<Department>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Department {
    name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Role {
    name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Roles {
    #[serde(rename = "role")]
    roles: Option<Vec<Role>>,
}

fn process_body<T>(response_body: &str) -> Result<T, auth::Error>
where
    T: DeserializeOwned,
{
    if response_body.starts_with("<") {
        let res = from_str::<T>(response_body)
            .map_err(|e| anyhow::anyhow!("can't deserialize: {:?}", e))?;
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
        6 => auth::Error::OtherAuth(format!("Code {res_code}")),
        7 => auth::Error::OtherAuth(format!("Code {res_code}")),
        8 => auth::Error::OtherAuth(format!("Code {res_code}")),
        9 => auth::Error::OtherAuth(format!("Code {res_code}")),
        _ => auth::Error::ServiceError(anyhow::anyhow!("Auth Service error: {res_code}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<user>
  <firstName>oooo</firstName>
  <lastName>aaaa</lastName>
  <phone>+37000000000</phone>
</user>"#,
  User { first_name: "oooo".to_string(), last_name: "aaaa".to_string(), organization_unit: None}; "no dep")]
    #[test_case(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<user>
  <firstName>oooo</firstName>
  <lastName>aaaa</lastName>
  <phone>+37000000000</phone>
    <organizationUnit>
        <name>dep</name>
        <other>dep</other>
    </organizationUnit>
</user>"#,
  User { first_name: "oooo".to_string(), last_name: "aaaa".to_string(), organization_unit: Some(Department { name: "dep".to_string() })}; "dep")]
    fn test_parse_user(input: &str, wanted: User) {
        let result: User = process_body(input).unwrap();
        assert_eq!(result, wanted);
    }

    #[test_case("aaa", auth::Error::ServiceError(anyhow::anyhow!("can't parse to int aaa: ParseIntError {{ kind: InvalidDigit }}")); "not int")]
    #[test_case("1", auth::Error::WrongUserPass(); "wrong user")]
    #[test_case("2", auth::Error::ExpiredPass(); "expired")]
    #[test_case("8", auth::Error::OtherAuth("Code 8".to_string()); "code 8")]
    #[test_case("12", auth::Error::ServiceError(anyhow::anyhow!("Auth Service error: 12")); "other")]
    fn test_parse_user_err(input: &str, wanted: auth::Error) {
        let result = process_body::<User>(input);
        assert_eq!(result.err().unwrap().to_string(), wanted.to_string());
    }

    #[test_case(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<roles user="dev" application="dev">
</roles>
"#,
  Roles { roles: None}; "empty")]
    #[test_case(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<roles user="dev" application="dev">
<role>
  <name>R1</name>
</role>
</roles>
"#,
Roles { roles: Some(vec![Role{name: "R1".to_string()}]) }; "one")]
    #[test_case(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<roles user="dev" application="dev">
<role>
  <name>R1</name>
</role>
<role>
  <name>R2</name>
</role>
<role>
  <name>R3</name>
</role>
</roles>
"#,
Roles { roles: Some(vec![Role { name: "R1".to_string() }, Role { name: "R2".to_string() }, Role { name: "R3".to_string() }]) }; "several")]
    fn test_parse_roles(input: &str, wanted: Roles) {
        let result: Roles = process_body(input).unwrap();
        assert_eq!(result, wanted);
    }
}
