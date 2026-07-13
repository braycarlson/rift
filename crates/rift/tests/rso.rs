#![cfg(feature = "rso")]

use rift::rso::{SCOPE_COUNT_MAX, authorize_url};

#[test]
fn authorize_url_encodes_parameters() {
    let url = authorize_url(
        "rift-client",
        "https://app.example/callback",
        &["openid", "cpid"],
    );

    assert!(
        url.contains("client_id=rift-client"),
        "client id must appear: {url}"
    );
    assert!(
        url.contains("redirect_uri=https%3A%2F%2Fapp.example%2Fcallback"),
        "redirect uri must be encoded: {url}"
    );
    assert!(
        url.contains("scope=openid%20cpid"),
        "scopes must be space-joined and encoded: {url}"
    );
    assert!(
        url.contains("response_type=code"),
        "must request the code grant: {url}"
    );
}

#[test]
#[should_panic(expected = "client_id must not be empty")]
fn authorize_url_rejects_empty_client_id() {
    let _ = authorize_url("", "https://app.example/callback", &["openid"]);
}

#[test]
#[should_panic(expected = "scopes must not be empty")]
fn authorize_url_rejects_empty_scopes() {
    let _ = authorize_url("rift-client", "https://app.example/callback", &[]);
}

#[test]
#[should_panic(expected = "scope count exceeds")]
fn authorize_url_rejects_excess_scopes() {
    let scopes = ["openid"; SCOPE_COUNT_MAX + 1];

    let _ = authorize_url("rift-client", "https://app.example/callback", &scopes);
}
