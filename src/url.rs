pub(crate) fn redact_endpoint(value: &str) -> String {
    let Ok(url) = reqwest::Url::parse(value) else {
        return "<redacted-endpoint>".to_string();
    };
    let Some(host) = url.host_str() else {
        return "<redacted-endpoint>".to_string();
    };
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    format!("{}://{host}{port}/", url.scheme())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_keeps_only_scheme_host_and_port() {
        assert_eq!(
            redact_endpoint("https://user:pass@example.com:8443/private/token?api-key=secret"),
            "https://example.com:8443/"
        );
        assert_eq!(
            redact_endpoint("not a url with secret"),
            "<redacted-endpoint>"
        );
    }
}
