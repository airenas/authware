use std::borrow::Cow;

use crate::IPExtractor;

pub struct Header {
    index: i16,
}

impl Header {
    pub fn new(index: i16) -> Self {
        tracing::debug!(index = index, "IP Extractor");
        Header { index }
    }
}

impl IPExtractor for Header {
    fn get<'a>(&self, headers: &'a axum::http::HeaderMap) -> Cow<'a, str> {
        let ips = headers
            .get("x-forwarded-for")
            .and_then(|header_value| header_value.to_str().ok());
        tracing::trace!(ips = ?ips, "ips");
        extract_ip(ips, self.index)
    }
}

fn extract_ip(input: Option<&str>, index: i16) -> Cow<'_, str> {
    if let Some(ip_list) = input {
        let ips: Vec<&str> = ip_list.split(',').collect();
        let len = ips.len() as i16;

        if index >= 0 && index < len {
            return Cow::Borrowed(ips[index as usize]);
        }
        if index < 0 && index.abs() <= len {
            return Cow::Borrowed(ips[(len + index) as usize]);
        }
    }
    Cow::Borrowed("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(None, -2, ""; "none")]
    #[test_case(Some(""), -2, ""; "empty")]
    #[test_case(Some("1,2"), -2, "1"; "extracts")]
    #[test_case(Some("1,2,3"), -2, "2"; "extracts 2")]
    #[test_case(Some("1,2,3"), -3, "1"; "extracts 1")]
    #[test_case(Some("1,2,3"), 0, "1"; "positive extracts 1")]
    #[test_case(Some("1,2,3"), 2, "3"; "positive extracts 3")]
    #[test_case(Some("1,2,3"), 10, ""; "positive extracts none")]
    #[test_case(Some("1.1.1.1,2"), -2, "1.1.1.1"; "extracts ip")]
    fn test_extract_ip(input: Option<&str>, index: i16, expected: &str) {
        let actual = extract_ip(input, index);
        assert_eq!(expected, actual);
    }
}
