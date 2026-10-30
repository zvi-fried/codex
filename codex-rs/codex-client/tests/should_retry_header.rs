use codex_client::RetryOn;
use codex_client::X_SHOULD_RETRY;
use codex_http_client::TransportError;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;

fn http_error(status: StatusCode, should_retry: Option<&'static str>) -> TransportError {
    let mut headers = HeaderMap::new();
    if let Some(value) = should_retry {
        headers.insert(X_SHOULD_RETRY, HeaderValue::from_static(value));
    }
    TransportError::Http {
        status,
        url: None,
        headers: Some(headers),
        body: None,
        retry_after: None,
    }
}

fn retry_on() -> RetryOn {
    RetryOn {
        retry_429: true,
        retry_5xx: true,
        retry_transport: true,
    }
}

#[test]
fn x_should_retry_false_stops_a_retryable_status() {
    let err = http_error(StatusCode::SERVICE_UNAVAILABLE, Some("false"));
    assert!(!retry_on().should_retry(&err, 0, 5));
    let err = http_error(StatusCode::TOO_MANY_REQUESTS, Some("false"));
    assert!(!retry_on().should_retry(&err, 0, 5));
}

#[test]
fn x_should_retry_true_retries_a_terminal_status() {
    let err = http_error(StatusCode::FORBIDDEN, Some("true"));
    assert!(retry_on().should_retry(&err, 0, 5));
}

#[test]
fn without_the_header_the_status_rule_decides() {
    assert!(retry_on().should_retry(&http_error(StatusCode::SERVICE_UNAVAILABLE, None), 0, 5));
    assert!(!retry_on().should_retry(&http_error(StatusCode::FORBIDDEN, None), 0, 5));
    assert!(retry_on().should_retry(
        &http_error(StatusCode::SERVICE_UNAVAILABLE, Some("maybe")),
        0,
        5
    ));
}
