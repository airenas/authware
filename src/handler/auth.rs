use std::{borrow::Cow, sync::Arc};

use axum::{extract::State, http::HeaderMap};
use chrono::Utc;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::model::service;

use super::error::ApiError;

pub async fn handler(
    State(data): State<Arc<service::Data>>,
    headers: HeaderMap,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), ApiError> {
    let forwarded_uri = headers
        .get("X-Forwarded-Uri")
        .map(|h| h.to_str().unwrap_or(""));

    let ip = data.ip_extractor.get(&headers);
    tracing::info!(url = forwarded_uri, ip = ip.as_ref(), "auth");
    let session_id = match bearer.as_ref() {
        None => match forwarded_uri {
            Some(token) => Cow::Borrowed(parse_token_from_url(token).unwrap_or_default()),
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
        user = res.user,
        ip = res.ip,
        "got"
    );
    res.check_ip(&ip)?;
    let now = Utc::now().timestamp_millis();
    res.check_expired(now)?;
    let config = &data.config;
    res.check_inactivity(now, config.inactivity)?;
    store.mark_last_used(session_id.as_ref(), now).await?;
    Ok(())
}

fn parse_token_from_url(url: &str) -> Option<&str> {
    if let Some(pos) = url.find('?') {
        let query = &url[pos + 1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "token" {
                    return Some(value);
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
    #[test_case("/olia?token=aaaaa", Some("aaaaa"); "parsed")]
    #[test_case("/olia?vvv=aaaaa", None; "none")]
    #[test_case("/olia?aaaa=nnnnnn&token=aaaaa", Some("aaaaa"); "long")]
    fn test_parse_token_from_url(input: &str, expected: Option<&str>) {
        let actual = parse_token_from_url(input);
        assert_eq!(expected, actual);
    }
}
