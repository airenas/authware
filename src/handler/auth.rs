use std::sync::Arc;

use axum::{extract::State, http::HeaderMap};
use chrono::Utc;
use url::Url;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{handler::login::extract_ip, model::service};

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    let forwarded_uri = headers
        .get("X-Forwarded-Uri")
        .map(|h| h.to_str().unwrap_or("").to_string());

    let ip = extract_ip(&headers);
    tracing::info!(url = forwarded_uri, ip = ip, "auth");
    let session_id = match bearer {
        None => match forwarded_uri {
            Some(token) => parse_token_from_url(&token).unwrap_or_default(),
            None => {
                return Err(ApiError::NoSession());
            }
        },
        Some(val) => val.token().to_string(),
    };
    tracing::debug!(session_id = session_id, "auth");
    let store = &data.store;
    let res = store.get(&session_id).await?;
    tracing::debug!(session_id = session_id, user = res.user, ip = res.ip, "got");
    res.check_ip(&ip)?;
    let now = Utc::now().timestamp_millis();
    res.check_expired(now)?;
    let config = &data.config;
    res.check_inactivity(now, config.inactivity)?;
    Ok(())
}

fn parse_token_from_url(url: &str) -> Option<String> {
    tracing::info!(url = url, "parse_token_from_url");
    let parsed_url = Url::parse(&format!("http://localhost{}", url)).ok()?;
    tracing::info!("parsed");
    parsed_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("",  None; "empty")]
    #[test_case("/olia", None; "no token")]
    #[test_case("/olia?token=aaaaa", Some("aaaaa".to_string()); "parsed")]
    #[test_case("/olia?vvv=aaaaa", None; "none")]
    #[test_case("/olia?aaaa=nnnnnn&token=aaaaa", Some("aaaaa".to_string()); "long")]
    fn test_parse_token_from_url(input: &str, expected: Option<String>) {
        let actual = parse_token_from_url(input);
        assert_eq!(expected, actual);
    }
}
