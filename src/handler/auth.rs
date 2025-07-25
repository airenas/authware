use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::Response,
};
use base64::Engine;
use chrono::Utc;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use reqwest::StatusCode;
use urlencoding::decode;

use crate::model::service;

use super::error::ApiError;

const OK_RESPONSE: &str = "OK";

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<Response<String>, ApiError> {
    let forwarded_uri = headers
        .get("X-Forwarded-Uri")
        .map(|h| h.to_str().unwrap_or(""));

    let ip = data.ip_extractor.get(&headers);
    tracing::info!(url = forwarded_uri, ip = ip.as_ref(), "auth");
    let session_id = match bearer.as_ref() {
        None => match forwarded_uri {
            Some(token) => parse_token_from_url(token).unwrap_or(Cow::Borrowed("")),
            None => {
                return Err(ApiError::NoSession());
            }
        },
        Some(val) => Cow::Borrowed(val.token()),
    };
    tracing::debug!(session_id = session_id.as_ref(), "auth");
    let store = &data.store;
    let res = store.get(&session_id).await?;
    tracing::debug!(
        session_id = session_id.as_ref(),
        user = res.user.id,
        ip = res.ip,
        "got"
    );
    res.check_ip(&ip)?;
    let now = Utc::now().timestamp_millis();
    res.check_expired(now)?;
    let config = &data.config;
    res.check_inactivity(now, config.inactivity)?;
    store.mark_last_used(session_id.as_ref(), now).await?;

    let header = serde_json::to_string(&res.user)
        .map_err(|e| ApiError::Server(format!("serialize session data: {e}")))?;
    let encoded_header = base64::prelude::BASE64_STANDARD.encode(header.as_bytes());

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            "X-User-Info",
            HeaderValue::from_str(&encoded_header)
                .map_err(|e| ApiError::Server(format!("build response: {e}")))?,
        )
        .body(OK_RESPONSE.to_string())
        .map_err(|e| ApiError::Server(format!("build response: {e}")))?;

    Ok(response)
}

fn parse_token_from_url(url: &str) -> Option<Cow<str>> {
    if let Some(pos) = url.find('?') {
        let query = &url[pos + 1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "token" {
                    return decode(value).ok();
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("",  None; "empty")]
    #[test_case("/olia", None; "no token")]
    #[test_case("/olia?token=aaaaa", Some(Cow::Borrowed("aaaaa")); "parsed")]
    #[test_case("/olia?vvv=aaaaa", None; "none")]
    #[test_case("/olia?aaaa=nnnnnn&token=aaaaa", Some(Cow::Borrowed("aaaaa")); "long")]
    #[test_case("/olia?token=aaaaa%3D", Some(Cow::Borrowed("aaaaa=")); "decode")]
    fn test_parse_token_from_url(input: &str, expected: Option<Cow<str>>) {
        let actual = parse_token_from_url(input);
        assert_eq!(expected, actual);
    }
}
