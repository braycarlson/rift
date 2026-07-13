use std::sync::Arc;

use rustc_hash::FxHashMap;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[doc(hidden)]
pub const APP_SCOPE_ID: &str = "app";
const BUCKET_COUNT_MAX: usize = 4_096;
const DURATION_OVERHEAD_MS_MAX: u64 = 10_000;
const DURATION_S_MAX: u64 = 86_400;
const HEADER_PAIR_COUNT_MAX: u32 = 8;
const PERMILLE_MAX: u16 = 1_000;
const PERMILLE_MIN: u16 = 1;
const WINDOW_COUNT_MAX: usize = 16;

type BucketKey = (&'static str, &'static str);

/// Tuning knobs for a [`RateLimiter`], expressed as integer permille (1..=1000).
///
/// All arithmetic is integer: floats never enter the rate-limit hot path.
#[derive(Clone, Copy, Debug)]
pub struct LimiterSettings {
    /// Burst spreading. Below 1000, a tighter derived sub-window paces requests
    /// evenly across each learned window instead of allowing a full burst.
    pub burst_permille: u16,
    /// Milliseconds added to each learned window to absorb clock skew and
    /// in-flight latency before the server's window rolls over.
    pub duration_overhead_ms: u64,
    /// Fraction of each learned limit the client will actually use, leaving
    /// headroom below the server's ceiling.
    pub usage_permille: u16,
}

impl LimiterSettings {
    /// Builds settings, asserting each knob is within range.
    #[must_use]
    pub fn new(
        burst_permille: u16,
        duration_overhead_ms: u64,
        usage_permille: u16,
    ) -> LimiterSettings {
        assert!(
            (PERMILLE_MIN..=PERMILLE_MAX).contains(&burst_permille),
            "burst_permille must be 1..=1000"
        );
        assert!(
            (PERMILLE_MIN..=PERMILLE_MAX).contains(&usage_permille),
            "usage_permille must be 1..=1000"
        );
        assert!(
            duration_overhead_ms <= DURATION_OVERHEAD_MS_MAX,
            "duration_overhead_ms exceeds {DURATION_OVERHEAD_MS_MAX}"
        );

        LimiterSettings {
            burst_permille,
            duration_overhead_ms,
            usage_permille,
        }
    }
}

impl Default for LimiterSettings {
    fn default() -> LimiterSettings {
        LimiterSettings::new(PERMILLE_MAX, 250, PERMILLE_MAX)
    }
}

#[doc(hidden)]
pub struct Window {
    pub count: u32,
    pub duration_ms: u64,
    pub limit: u32,
    pub started_at: Instant,
}

#[doc(hidden)]
#[derive(Default)]
pub struct Bucket {
    pub windows: Vec<Window>,
}

#[doc(hidden)]
pub struct RateHeaders<'a> {
    pub app_count: Option<&'a str>,
    pub app_limit: Option<&'a str>,
    pub method_count: Option<&'a str>,
    pub method_limit: Option<&'a str>,
}

/// A header-learned, per-instance rate limiter with app and method scopes.
///
/// The limiter learns Riot's advertised limits from response headers, then
/// proactively sleeps before a window would be exceeded. Construct one with
/// [`RateLimiter::new`] and share it across clients by placing it in an
/// [`Arc`] (see [`crate::RiotApiConfig::rate_limiter`]).
pub struct RateLimiter {
    #[doc(hidden)]
    pub buckets: Mutex<FxHashMap<BucketKey, Bucket>>,
    settings: LimiterSettings,
}

impl RateLimiter {
    /// Creates an empty limiter with the given [`LimiterSettings`].
    #[must_use]
    pub fn new(settings: LimiterSettings) -> RateLimiter {
        RateLimiter {
            buckets: Mutex::new(FxHashMap::default()),
            settings,
        }
    }

    /// Wraps a fresh limiter with default settings in an [`Arc`] for sharing.
    #[must_use]
    pub fn shared(settings: LimiterSettings) -> Arc<RateLimiter> {
        Arc::new(RateLimiter::new(settings))
    }

    #[doc(hidden)]
    pub async fn probe(
        &self,
        route: &'static str,
        endpoint_id: &'static str,
    ) -> (u64, &'static str) {
        assert!(!route.is_empty(), "route must not be empty");
        assert!(!endpoint_id.is_empty(), "endpoint id must not be empty");

        let keys: [BucketKey; 2] = [(route, APP_SCOPE_ID), (route, endpoint_id)];

        self.acquire_try(&keys).await
    }

    async fn acquire_try(&self, keys: &[BucketKey; 2]) -> (u64, &'static str) {
        debug_assert!(
            keys.iter()
                .all(|(route, scope)| !route.is_empty() && !scope.is_empty()),
            "bucket keys must be populated"
        );

        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let mut wait_ms: u64 = 0;
        let mut scope: &'static str = "";

        for key in keys {
            let Some(bucket) = buckets.get_mut(key) else {
                continue;
            };

            debug_assert!(
                bucket.windows.len() <= WINDOW_COUNT_MAX,
                "window count exceeds {WINDOW_COUNT_MAX}"
            );

            for window in &mut bucket.windows {
                let elapsed_ms = u64::try_from(now.duration_since(window.started_at).as_millis())
                    .unwrap_or(u64::MAX);

                if elapsed_ms >= window.duration_ms {
                    window.count = 0;
                    window.started_at = now;
                }

                if window.count >= window.limit {
                    let elapsed_current_ms =
                        u64::try_from(now.duration_since(window.started_at).as_millis())
                            .unwrap_or(u64::MAX);

                    let remaining_ms = window.duration_ms.saturating_sub(elapsed_current_ms).max(1);

                    if remaining_ms > wait_ms {
                        wait_ms = remaining_ms;
                        scope = key.1;
                    }
                }
            }
        }

        if wait_ms == 0 {
            for key in keys {
                let Some(bucket) = buckets.get_mut(key) else {
                    continue;
                };

                for window in &mut bucket.windows {
                    window.count += 1;
                }
            }
        }

        debug_assert!(
            (wait_ms == 0) == scope.is_empty(),
            "wait_ms and scope must agree"
        );

        (wait_ms, scope)
    }

    #[doc(hidden)]
    pub async fn update(
        &self,
        route: &'static str,
        endpoint_id: &'static str,
        headers: &RateHeaders<'_>,
    ) {
        assert!(!route.is_empty(), "route must not be empty");
        assert!(!endpoint_id.is_empty(), "endpoint id must not be empty");

        let mut buckets = self.buckets.lock().await;

        assert!(
            buckets.len() <= BUCKET_COUNT_MAX,
            "bucket count exceeds {BUCKET_COUNT_MAX}"
        );

        if let Some(limits) = headers.app_limit {
            let counts = headers.app_count.unwrap_or("");

            bucket_update(
                buckets.entry((route, APP_SCOPE_ID)).or_default(),
                limits,
                counts,
                &self.settings,
            );
        }

        if let Some(limits) = headers.method_limit {
            let counts = headers.method_count.unwrap_or("");

            bucket_update(
                buckets.entry((route, endpoint_id)).or_default(),
                limits,
                counts,
                &self.settings,
            );
        }
    }
}

#[doc(hidden)]
pub fn bucket_update(
    bucket: &mut Bucket,
    limits_header: &str,
    counts_header: &str,
    settings: &LimiterSettings,
) {
    let limits = header_pairs_parse(limits_header);
    let counts = header_pairs_parse(counts_header);

    if limits.is_empty() {
        return;
    }

    let now = Instant::now();
    let mut windows = Vec::with_capacity(limits.len() * 2);

    for (limit, duration_ms) in &limits {
        if *limit == 0 {
            continue;
        }

        let count_server = counts
            .iter()
            .find(|(_, count_duration_ms)| count_duration_ms == duration_ms)
            .map_or(0, |(count, _)| *count);

        let limit_effective = permille_scale(*limit, settings.usage_permille);
        let duration_effective = duration_ms.saturating_add(settings.duration_overhead_ms);

        window_carry(
            &mut windows,
            &bucket.windows,
            now,
            limit_effective,
            duration_effective,
            count_server,
        );

        if settings.burst_permille < PERMILLE_MAX {
            let limit_burst = permille_scale(limit_effective, settings.burst_permille);
            let duration_burst = duration_scale(duration_effective, settings.burst_permille);

            window_carry(
                &mut windows,
                &bucket.windows,
                now,
                limit_burst,
                duration_burst,
                0,
            );
        }
    }

    assert!(
        windows.len() <= WINDOW_COUNT_MAX,
        "window count exceeds {WINDOW_COUNT_MAX}"
    );

    bucket.windows = windows;
}

fn window_carry(
    windows: &mut Vec<Window>,
    previous: &[Window],
    now: Instant,
    limit: u32,
    duration_ms: u64,
    count_server: u32,
) {
    assert!(limit >= 1, "effective limit must be positive");
    assert!(duration_ms >= 1, "effective duration must be positive");

    let (count, started_at) = match previous
        .iter()
        .find(|window| window.duration_ms == duration_ms)
    {
        Some(window) => (window.count.max(count_server), window.started_at),
        None => (count_server, now),
    };

    windows.push(Window {
        count,
        duration_ms,
        limit,
        started_at,
    });
}

fn permille_scale(value: u32, permille: u16) -> u32 {
    assert!(permille >= PERMILLE_MIN, "permille must be positive");
    assert!(
        permille <= PERMILLE_MAX,
        "permille must not exceed {PERMILLE_MAX}"
    );

    let scaled = u64::from(value) * u64::from(permille) / u64::from(PERMILLE_MAX);
    let clamped = scaled.min(u64::from(u32::MAX));
    let result = u32::try_from(clamped).unwrap_or(u32::MAX).max(1);

    assert!(
        result <= value.max(1),
        "scaled value must not exceed the input"
    );

    result
}

fn duration_scale(duration_ms: u64, permille: u16) -> u64 {
    assert!(permille >= PERMILLE_MIN, "permille must be positive");
    assert!(
        permille <= PERMILLE_MAX,
        "permille must not exceed {PERMILLE_MAX}"
    );

    let scaled = duration_ms.saturating_mul(u64::from(permille)) / u64::from(PERMILLE_MAX);
    let result = scaled.max(1);

    assert!(
        result <= duration_ms.max(1),
        "scaled duration must not exceed the input"
    );

    result
}

#[doc(hidden)]
#[must_use]
pub fn header_pairs_parse(header: &str) -> Vec<(u32, u64)> {
    let mut pairs = Vec::with_capacity(4);
    let mut iterations: u32 = 0;

    for section in header.split(',') {
        iterations += 1;

        if iterations > HEADER_PAIR_COUNT_MAX {
            break;
        }

        let Some((value_raw, duration_raw)) = section.split_once(':') else {
            continue;
        };

        let Ok(value) = value_raw.trim().parse::<u32>() else {
            continue;
        };

        let Ok(duration_s) = duration_raw.trim().parse::<u64>() else {
            continue;
        };

        if duration_s == 0 {
            continue;
        }

        if duration_s > DURATION_S_MAX {
            continue;
        }

        pairs.push((value, duration_s * 1_000));
    }

    assert!(
        pairs.len() <= HEADER_PAIR_COUNT_MAX as usize,
        "pair count exceeds {HEADER_PAIR_COUNT_MAX}"
    );

    pairs
}
