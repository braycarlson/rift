use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;

use rift::rate_limit::{
    APP_SCOPE_ID, Bucket, LimiterSettings, RateHeaders, RateLimiter, bucket_update,
    header_pairs_parse,
};

const ACQUIRE_ITERATIONS_MAX: u32 = 256;
const WAIT_MS_MAX: u64 = 120_000;

async fn acquire(limiter: &RateLimiter, route: &'static str, endpoint_id: &'static str) {
    let mut iterations: u32 = 0;

    loop {
        iterations += 1;

        assert!(
            iterations <= ACQUIRE_ITERATIONS_MAX,
            "acquire exceeded {ACQUIRE_ITERATIONS_MAX} iterations"
        );

        let (wait_ms, _scope) = limiter.probe(route, endpoint_id).await;

        if wait_ms == 0 {
            return;
        }

        tokio::time::sleep(Duration::from_millis(wait_ms.min(WAIT_MS_MAX))).await;
    }
}

fn headers_app(limit: &'static str, count: &'static str) -> RateHeaders<'static> {
    RateHeaders {
        app_count: Some(count),
        app_limit: Some(limit),
        method_count: None,
        method_limit: None,
    }
}

fn settings_plain() -> LimiterSettings {
    LimiterSettings::new(1_000, 0, 1_000)
}

#[test]
fn header_pairs_parse_valid() {
    let pairs = header_pairs_parse("20:1,100:120");

    assert!(pairs.len() == 2, "expected two pairs: {pairs:?}");
    assert!(pairs[0] == (20, 1_000), "first pair mismatch: {pairs:?}");
    assert!(
        pairs[1] == (100, 120_000),
        "second pair mismatch: {pairs:?}"
    );
}

#[test]
fn header_pairs_parse_whitespace() {
    let pairs = header_pairs_parse(" 20 : 1 ");

    assert!(pairs.len() == 1, "expected one pair: {pairs:?}");
    assert!(pairs[0] == (20, 1_000), "pair mismatch: {pairs:?}");
}

#[test]
fn header_pairs_parse_empty() {
    let pairs = header_pairs_parse("");

    assert!(pairs.is_empty(), "expected no pairs: {pairs:?}");
}

#[test]
fn header_pairs_parse_malformed_sections_skipped() {
    let pairs = header_pairs_parse("abc,20,:1,5:,x:y,20:1");

    assert!(pairs.len() == 1, "expected one pair: {pairs:?}");
    assert!(pairs[0] == (20, 1_000), "pair mismatch: {pairs:?}");
}

#[test]
fn header_pairs_parse_zero_duration_skipped() {
    let pairs = header_pairs_parse("5:0,3:2");

    assert!(pairs.len() == 1, "expected one pair: {pairs:?}");
    assert!(pairs[0] == (3, 2_000), "pair mismatch: {pairs:?}");
}

#[test]
fn header_pairs_parse_duration_bounded() {
    let pairs = header_pairs_parse("5:86401,3:86400");

    assert!(pairs.len() == 1, "expected one pair: {pairs:?}");
    assert!(pairs[0] == (3, 86_400_000), "pair mismatch: {pairs:?}");
}

#[test]
fn header_pairs_parse_overflow_skipped() {
    let pairs = header_pairs_parse("4294967296:1,1:18446744073709551616,7:9");

    assert!(pairs.len() == 1, "expected one pair: {pairs:?}");
    assert!(pairs[0] == (7, 9_000), "pair mismatch: {pairs:?}");
}

#[test]
fn header_pairs_parse_section_count_bounded() {
    let pairs = header_pairs_parse("1:1,1:2,1:3,1:4,1:5,1:6,1:7,1:8,1:9,1:10,1:11,1:12");

    assert!(
        pairs.len() == 8,
        "expected pair count capped at eight: {pairs:?}"
    );
    assert!(pairs[7] == (1, 8_000), "eighth pair mismatch: {pairs:?}");
}

#[tokio::test(start_paused = true)]
async fn bucket_update_creates_windows() {
    let mut bucket = Bucket::default();

    bucket_update(&mut bucket, "2:1,100:120", "1:1,3:120", &settings_plain());

    assert!(bucket.windows.len() == 2, "expected two windows");
    assert!(bucket.windows[0].limit == 2, "first window limit mismatch");
    assert!(bucket.windows[0].count == 1, "first window count mismatch");
    assert!(
        bucket.windows[0].duration_ms == 1_000,
        "first window duration mismatch"
    );
    assert!(
        bucket.windows[1].limit == 100,
        "second window limit mismatch"
    );
    assert!(bucket.windows[1].count == 3, "second window count mismatch");
    assert!(
        bucket.windows[1].duration_ms == 120_000,
        "second window duration mismatch"
    );
}

#[tokio::test(start_paused = true)]
async fn bucket_update_missing_counts_default_zero() {
    let mut bucket = Bucket::default();

    bucket_update(&mut bucket, "2:1", "", &settings_plain());

    assert!(bucket.windows.len() == 1, "expected one window");
    assert!(bucket.windows[0].count == 0, "count must default to zero");
}

#[tokio::test(start_paused = true)]
async fn bucket_update_preserves_existing_window() {
    let mut bucket = Bucket::default();

    bucket_update(&mut bucket, "2:1", "0:1", &settings_plain());

    bucket.windows[0].count = 2;

    let started_before = bucket.windows[0].started_at;

    bucket_update(&mut bucket, "2:1", "1:1", &settings_plain());

    assert!(bucket.windows.len() == 1, "expected one window");
    assert!(
        bucket.windows[0].count == 2,
        "count must keep local maximum"
    );
    assert!(
        bucket.windows[0].started_at == started_before,
        "window start must be preserved"
    );
}

#[tokio::test(start_paused = true)]
async fn bucket_update_zero_limit_skipped() {
    let mut bucket = Bucket::default();

    bucket_update(&mut bucket, "0:1,3:10", "", &settings_plain());

    assert!(
        bucket.windows.len() == 1,
        "zero limit window must be skipped"
    );
    assert!(bucket.windows[0].limit == 3, "window limit mismatch");
}

#[tokio::test(start_paused = true)]
async fn bucket_update_malformed_leaves_windows() {
    let mut bucket = Bucket::default();

    bucket_update(&mut bucket, "2:1", "1:1", &settings_plain());
    bucket_update(&mut bucket, "garbage", "", &settings_plain());

    assert!(
        bucket.windows.len() == 1,
        "windows must survive malformed header"
    );
    assert!(
        bucket.windows[0].count == 1,
        "count must survive malformed header"
    );
}

#[tokio::test(start_paused = true)]
async fn bucket_update_usage_permille_scales_limit() {
    let mut bucket = Bucket::default();
    let settings = LimiterSettings::new(1_000, 0, 500);

    bucket_update(&mut bucket, "10:10", "", &settings);

    assert!(bucket.windows.len() == 1, "expected one window");
    assert!(
        bucket.windows[0].limit == 5,
        "usage 500 permille must halve limit: {}",
        bucket.windows[0].limit
    );
}

#[tokio::test(start_paused = true)]
async fn bucket_update_burst_adds_sub_window() {
    let mut bucket = Bucket::default();
    let settings = LimiterSettings::new(250, 0, 1_000);

    bucket_update(&mut bucket, "20:1", "", &settings);

    assert!(bucket.windows.len() == 2, "burst must add a sub-window");

    let sub = bucket
        .windows
        .iter()
        .find(|window| window.duration_ms == 250)
        .expect("sub-window must exist");

    assert!(sub.limit == 5, "sub-window limit mismatch: {}", sub.limit);
}

#[tokio::test(start_paused = true)]
async fn acquire_unknown_bucket_immediate() {
    let limiter = RateLimiter::new(settings_plain());
    let before = Instant::now();

    acquire(&limiter, "na1", "endpoint-a").await;

    assert!(
        before.elapsed() == Duration::ZERO,
        "acquire must not wait without limits"
    );

    let buckets = limiter.buckets.lock().await;

    assert!(buckets.is_empty(), "acquire must not create buckets");
}

#[tokio::test(start_paused = true)]
async fn acquire_blocks_on_exhausted_window() {
    let limiter = RateLimiter::new(settings_plain());

    limiter
        .update("na1", "endpoint-a", &headers_app("2:1", "0:1"))
        .await;

    acquire(&limiter, "na1", "endpoint-a").await;
    acquire(&limiter, "na1", "endpoint-a").await;

    let before = Instant::now();

    acquire(&limiter, "na1", "endpoint-a").await;

    let elapsed = before.elapsed();

    assert!(
        elapsed >= Duration::from_secs(1),
        "third acquire must wait: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(1_100),
        "wait must not exceed window: {elapsed:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn acquire_after_expiry_immediate() {
    let limiter = RateLimiter::new(settings_plain());

    limiter
        .update("na1", "endpoint-a", &headers_app("2:1", "0:1"))
        .await;

    acquire(&limiter, "na1", "endpoint-a").await;
    acquire(&limiter, "na1", "endpoint-a").await;
    acquire(&limiter, "na1", "endpoint-a").await;

    let before = Instant::now();

    acquire(&limiter, "na1", "endpoint-a").await;

    assert!(
        before.elapsed() == Duration::ZERO,
        "acquire after reset must not wait"
    );

    let buckets = limiter.buckets.lock().await;
    let Some(bucket) = buckets.get(&("na1", APP_SCOPE_ID)) else {
        panic!("app bucket must exist");
    };

    assert!(
        bucket.windows[0].count == 2,
        "window count mismatch after reset"
    );
}

#[tokio::test(start_paused = true)]
async fn acquire_increments_all_scopes() {
    let limiter = RateLimiter::new(settings_plain());
    let headers = RateHeaders {
        app_count: Some("0:10"),
        app_limit: Some("10:10"),
        method_count: Some("0:10"),
        method_limit: Some("5:10"),
    };

    limiter.update("na1", "endpoint-a", &headers).await;
    acquire(&limiter, "na1", "endpoint-a").await;

    let buckets = limiter.buckets.lock().await;
    let Some(app_bucket) = buckets.get(&("na1", APP_SCOPE_ID)) else {
        panic!("app bucket must exist");
    };
    let Some(method_bucket) = buckets.get(&("na1", "endpoint-a")) else {
        panic!("method bucket must exist");
    };

    assert!(
        app_bucket.windows[0].count == 1,
        "app count must be incremented"
    );
    assert!(
        method_bucket.windows[0].count == 1,
        "method count must be incremented"
    );
}

#[tokio::test(start_paused = true)]
async fn acquire_method_buckets_independent() {
    let limiter = RateLimiter::new(settings_plain());
    let headers = RateHeaders {
        app_count: Some("1:1"),
        app_limit: Some("100:1"),
        method_count: Some("1:1"),
        method_limit: Some("1:1"),
    };

    limiter.update("na1", "endpoint-a", &headers).await;

    let before_b = Instant::now();

    acquire(&limiter, "na1", "endpoint-b").await;

    assert!(
        before_b.elapsed() == Duration::ZERO,
        "endpoint-b must not inherit endpoint-a limit"
    );

    let before_a = Instant::now();

    acquire(&limiter, "na1", "endpoint-a").await;

    assert!(
        before_a.elapsed() >= Duration::from_millis(1),
        "endpoint-a must wait on its own limit",
    );
}

#[tokio::test(start_paused = true)]
async fn update_adopts_server_count() {
    let limiter = RateLimiter::new(settings_plain());

    limiter
        .update("na1", "endpoint-a", &headers_app("10:120", "7:120"))
        .await;

    let buckets = limiter.buckets.lock().await;
    let Some(bucket) = buckets.get(&("na1", APP_SCOPE_ID)) else {
        panic!("app bucket must exist");
    };

    assert!(bucket.windows[0].count == 7, "server count must be adopted");
    assert!(bucket.windows[0].limit == 10, "window limit mismatch");
}

#[tokio::test(start_paused = true)]
async fn acquire_spread_paces_requests() {
    let limiter = RateLimiter::new(LimiterSettings::new(250, 0, 1_000));

    limiter
        .update("na1", "endpoint-a", &headers_app("20:1", "0:1"))
        .await;

    for _ in 0..5 {
        acquire(&limiter, "na1", "endpoint-a").await;
    }

    let before = Instant::now();

    acquire(&limiter, "na1", "endpoint-a").await;

    let elapsed = before.elapsed();

    assert!(
        elapsed >= Duration::from_millis(250),
        "sixth acquire must wait for the burst sub-window: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(300),
        "wait must not exceed the sub-window: {elapsed:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn acquire_shared_limiter_counts_combine() {
    let limiter = RateLimiter::shared(settings_plain());
    let handle_a = Arc::clone(&limiter);
    let handle_b = Arc::clone(&limiter);

    handle_a
        .update("na1", "endpoint-a", &headers_app("2:1", "0:1"))
        .await;

    acquire(&handle_a, "na1", "endpoint-a").await;
    acquire(&handle_b, "na1", "endpoint-a").await;

    let before = Instant::now();

    acquire(&handle_b, "na1", "endpoint-a").await;

    let elapsed = before.elapsed();

    assert!(
        elapsed >= Duration::from_secs(1),
        "third acquire across shared handles must wait: {elapsed:?}"
    );
}
