use rift::Error;
use rift::error::STATUS_BODY_BYTES_MAX;

#[test]
fn status_snippet_is_bounded() {
    let raw = vec![b'x'; STATUS_BODY_BYTES_MAX + 1];
    let error = Error::status("endpoint-a", 500, &raw, None);

    let Error::Status { body, .. } = &error else {
        panic!("expected status error");
    };

    assert!(body.len() <= STATUS_BODY_BYTES_MAX, "snippet not bounded");
}

#[test]
fn status_message_parses_riot_shape() {
    let raw = br#"{"status":{"message":"Forbidden","status_code":403}}"#;
    let error = Error::status("endpoint-a", 403, raw, None);

    assert!(
        error.status_message().as_deref() == Some("Forbidden"),
        "message must parse"
    );
}

#[test]
fn status_message_absent_on_plain_body() {
    let error = Error::status("endpoint-a", 500, b"internal error", None);

    assert!(error.status_message().is_none(), "no envelope to parse");
}

#[test]
fn is_retriable_covers_transient_statuses() {
    let retriable = Error::status("endpoint-a", 503, b"", None);
    let terminal = Error::status("endpoint-a", 400, b"", None);

    assert!(retriable.is_retriable(), "503 must be retriable");
    assert!(!terminal.is_retriable(), "400 must not be retriable");
}

#[test]
fn status_code_reads_variants() {
    let status = Error::status("endpoint-a", 429, b"", None);
    let exhausted = Error::retries_exhausted("endpoint-a", 500);

    assert!(status.status_code() == Some(429), "status code mismatch");
    assert!(exhausted.status_code() == Some(500), "last status mismatch");
    assert!(
        Error::not_found("endpoint-a").status_code().is_none(),
        "not found has no status code"
    );
}
