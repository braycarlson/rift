use rift::client::{
    JITTER_MS_MAX, RETRY_BACKOFF_MS_BASE, RETRY_BACKOFF_MS_MAX, backoff_duration, path_encode,
};
use rift::error::status_retriable;

#[test]
fn backoff_grows_and_caps() {
    let first = backoff_duration(1).as_millis();
    let second = backoff_duration(2).as_millis();
    let far = backoff_duration(12).as_millis();

    let base = u128::from(RETRY_BACKOFF_MS_BASE);
    let jitter = u128::from(JITTER_MS_MAX);

    assert!(first >= base, "first backoff floor: {first}");
    assert!(
        first < base + jitter,
        "first backoff ceiling with jitter: {first}"
    );
    assert!(second >= 2 * base, "second backoff floor: {second}");
    assert!(
        far < u128::from(RETRY_BACKOFF_MS_MAX + JITTER_MS_MAX),
        "backoff must cap: {far}"
    );
}

#[test]
fn retriable_statuses_only() {
    assert!(status_retriable(429), "429 retriable");
    assert!(status_retriable(503), "503 retriable");
    assert!(!status_retriable(400), "400 not retriable");
    assert!(!status_retriable(200), "200 not retriable");
}

#[test]
fn path_encode_escapes_reserved() {
    let encoded = path_encode("a b/c");

    assert!(encoded == "a%20b%2Fc", "unexpected encoding: {encoded}");
}
