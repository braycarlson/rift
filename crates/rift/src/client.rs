use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use bytes::Bytes;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use tokio::sync::Semaphore;

#[cfg(feature = "cache")]
use crate::cache::ResponseCache;
use crate::config::{self, ROUTE_PLACEHOLDER, RiotApiConfig};
use crate::error::{Error, STATUS_BODY_BYTES_MAX, status_retriable};
use crate::hooks::RequestInfo;
use crate::rate_limit::{RateHeaders, RateLimiter};

#[doc(hidden)]
pub const JITTER_MS_MAX: u64 = 500;
const PATH_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
const RATE_LIMIT_ITERATIONS_MAX: u32 = 256;
const RATE_LIMIT_WAIT_MS_MAX: u64 = 120_000;
const RESPONSE_BYTES_MAX: usize = 64 * 1024 * 1024;
const RETRY_AFTER_S_MAX: u64 = 120;
#[doc(hidden)]
pub const RETRY_BACKOFF_MS_BASE: u64 = 1_000;
#[doc(hidden)]
pub const RETRY_BACKOFF_MS_MAX: u64 = 32_000;

static PROCESS_START: LazyLock<Instant> = LazyLock::new(Instant::now);

/// An asynchronous Riot API client.
///
/// Build one from a [`RiotApiConfig`] with [`RiotApi::new`], then call the
/// generated endpoint methods (for example `account_v1_get_by_riot_id`). The
/// client owns its rate limiter, concurrency cap, and optional response cache;
/// share it behind an `Arc` to reuse across tasks.
pub struct RiotApi {
    #[cfg(feature = "cache")]
    cache: Option<ResponseCache>,
    client: reqwest::Client,
    config: RiotApiConfig,
    rate_limiter: Arc<RateLimiter>,
    semaphore: Semaphore,
}

/// How a single request authenticates with the Riot API.
pub(crate) enum Auth {
    /// Send the configured API key as `X-Riot-Token`.
    ApiKey,
    /// Send a per-call RSO access token as `Authorization: Bearer <token>`.
    Bearer(String),
}

pub(crate) struct RequestPlan {
    pub auth: Auth,
    pub body: Option<String>,
    pub endpoint_id: &'static str,
    pub method: &'static str,
    pub path: Cow<'static, str>,
    pub query: Vec<(&'static str, String)>,
    pub route: &'static str,
}

/// One attempt's outcome inside the retry loop: give up now, or retry.
///
/// `status_last` is `0` when the attempt failed in transport before any HTTP
/// status was received; [`Error::RetriesExhausted`] carries that value through.
enum AttemptError {
    Retriable {
        status_last: u16,
        retry_after: Option<Duration>,
    },
    Terminal(Error),
}

fn attempt_transport(error: Error) -> AttemptError {
    if error.is_retriable() {
        return AttemptError::Retriable {
            status_last: 0,
            retry_after: None,
        };
    }

    AttemptError::Terminal(error)
}

#[doc(hidden)]
#[must_use]
pub fn path_encode(value: &str) -> String {
    assert!(!value.is_empty(), "path parameter must not be empty");

    utf8_percent_encode(value, PATH_ENCODE_SET).to_string()
}

impl RiotApi {
    /// Builds a client, validating every configuration value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the underlying HTTP client fails to
    /// build (for example a TLS backend initialization error).
    pub fn new(config: RiotApiConfig) -> Result<RiotApi, Error> {
        assert!(!config.api_key.is_empty(), "api_key must not be empty");
        assert!(
            config.retry_count <= config::RETRY_COUNT_MAX,
            "retry_count exceeds {}",
            config::RETRY_COUNT_MAX,
        );
        assert!(config.timeout_ms > 0, "timeout_ms must be positive");
        assert!(
            config.timeout_ms <= config::TIMEOUT_MS_MAX,
            "timeout_ms exceeds {}",
            config::TIMEOUT_MS_MAX,
        );
        assert!(
            config.base_url.matches(ROUTE_PLACEHOLDER).count() == 1,
            "base_url must contain exactly one {ROUTE_PLACEHOLDER}"
        );
        assert!(
            config.concurrent_requests_max >= 1,
            "concurrent_requests_max must be positive"
        );
        assert!(
            config.concurrent_requests_max <= config::CONCURRENT_REQUESTS_MAX,
            "concurrent_requests_max exceeds {}",
            config::CONCURRENT_REQUESTS_MAX,
        );

        let builder = reqwest::Client::builder()
            .user_agent(concat!("rift/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_millis(config.timeout_ms));

        #[cfg(feature = "native-tls")]
        let builder = builder.use_native_tls();

        let client = builder.build()?;

        let rate_limiter = config
            .rate_limiter
            .clone()
            .unwrap_or_else(|| RateLimiter::shared(config.limiter_settings));

        let semaphore = Semaphore::new(config.concurrent_requests_max as usize);

        #[cfg(feature = "cache")]
        let cache = config.cache.clone().map(ResponseCache::new);

        Ok(RiotApi {
            #[cfg(feature = "cache")]
            cache,
            client,
            config,
            rate_limiter,
            semaphore,
        })
    }

    pub(crate) async fn execute<T: DeserializeOwned>(&self, plan: RequestPlan) -> Result<T, Error> {
        let endpoint_id = plan.endpoint_id;
        let body = self.request_bytes(plan).await?;

        let value = serde_json::from_slice::<T>(&body)
            .map_err(|source| Error::deserialize(endpoint_id, source))?;

        Ok(value)
    }

    pub(crate) async fn execute_unit(&self, plan: RequestPlan) -> Result<(), Error> {
        let _ = self.request_bytes(plan).await?;

        Ok(())
    }

    pub(crate) async fn execute_optional<T: DeserializeOwned>(
        &self,
        plan: RequestPlan,
    ) -> Result<Option<T>, Error> {
        let optional = self.execute_optional_raw::<T>(plan).await?;

        Ok(optional.map(|(value, _body)| value))
    }

    pub(crate) async fn execute_optional_raw<T: DeserializeOwned>(
        &self,
        plan: RequestPlan,
    ) -> Result<Option<(T, Bytes)>, Error> {
        let endpoint_id = plan.endpoint_id;

        match self.request_bytes(plan).await {
            Ok(body) => {
                let value = serde_json::from_slice::<T>(&body)
                    .map_err(|source| Error::deserialize(endpoint_id, source))?;

                Ok(Some((value, body)))
            }
            Err(Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    async fn request_bytes(&self, plan: RequestPlan) -> Result<Bytes, Error> {
        assert!(
            !plan.endpoint_id.is_empty(),
            "endpoint id must not be empty"
        );
        assert!(!plan.route.is_empty(), "route must not be empty");

        let url = self.url_build(&plan);

        if let Some(cached) = self.cache_lookup(&plan, &url) {
            return Ok(cached);
        }

        let permit = self.semaphore.acquire().await;
        let _permit = permit.expect("semaphore is never closed");

        let info = RequestInfo {
            endpoint_id: plan.endpoint_id,
            method: plan.method,
            route: plan.route,
        };

        let mut attempts: u32 = 0;

        loop {
            attempts += 1;

            assert!(
                attempts <= config::RETRY_COUNT_MAX + 1,
                "attempts exceeded {}",
                config::RETRY_COUNT_MAX + 1,
            );

            self.rate_limit_wait(&plan, &info).await;
            self.hooks_on_request(&info);

            let (status, retry_after) = match self.attempt_send(&url, &plan, &info).await {
                Ok(body) => {
                    self.cache_store(&plan, &url, &body);

                    return Ok(body);
                }
                Err(AttemptError::Terminal(error)) => return Err(error),
                Err(AttemptError::Retriable {
                    status_last,
                    retry_after,
                }) => (status_last, retry_after),
            };

            if attempts > self.config.retry_count {
                return Err(Error::retries_exhausted(plan.endpoint_id, status));
            }

            let wait = retry_after.unwrap_or_else(|| backoff_duration(attempts));

            self.hooks_on_retry(&info, attempts, wait);

            tokio::time::sleep(wait).await;
        }
    }

    async fn attempt_send(
        &self,
        url: &str,
        plan: &RequestPlan,
        info: &RequestInfo<'_>,
    ) -> Result<Bytes, AttemptError> {
        let sent_at = Instant::now();

        // Transport faults (timeouts, connection resets) participate in the
        // retry loop like retriable statuses instead of failing immediately.
        let response = match self.request_send(url, plan).await {
            Ok(response) => response,
            Err(error) => return Err(attempt_transport(error)),
        };

        let status = response.status().as_u16();
        let elapsed = sent_at.elapsed();

        self.hooks_on_response(info, status, elapsed);
        self.headers_apply(plan, response.headers()).await;

        let retry_after = retry_after_duration(response.headers());

        if (200..300).contains(&status) {
            let body = match response.bytes().await {
                Ok(body) => body,
                Err(error) => return Err(attempt_transport(Error::Transport(error))),
            };

            if body.len() > RESPONSE_BYTES_MAX {
                return Err(AttemptError::Terminal(Error::response_too_large(
                    plan.endpoint_id,
                    body.len(),
                )));
            }

            return Ok(body);
        }

        if status == 404 {
            return Err(AttemptError::Terminal(Error::not_found(plan.endpoint_id)));
        }

        if !status_retriable(status) {
            let snippet = body_snippet_read(response).await;

            return Err(AttemptError::Terminal(Error::status(
                plan.endpoint_id,
                status,
                &snippet,
                retry_after,
            )));
        }

        Err(AttemptError::Retriable {
            status_last: status,
            retry_after,
        })
    }

    async fn rate_limit_wait(&self, plan: &RequestPlan, info: &RequestInfo<'_>) {
        let mut iterations: u32 = 0;

        loop {
            iterations += 1;

            assert!(
                iterations <= RATE_LIMIT_ITERATIONS_MAX,
                "rate limit wait exceeded {RATE_LIMIT_ITERATIONS_MAX} iterations",
            );

            let (wait_ms, scope) = self.rate_limiter.probe(plan.route, plan.endpoint_id).await;

            if wait_ms == 0 {
                return;
            }

            let wait = Duration::from_millis(wait_ms.min(RATE_LIMIT_WAIT_MS_MAX));

            self.hooks_on_rate_limit_sleep(info.route, scope, wait);

            tokio::time::sleep(wait).await;
        }
    }

    async fn request_send(
        &self,
        url: &str,
        plan: &RequestPlan,
    ) -> Result<reqwest::Response, Error> {
        let method = match plan.method {
            "DELETE" => reqwest::Method::DELETE,
            "GET" => reqwest::Method::GET,
            "PATCH" => reqwest::Method::PATCH,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            _ => unreachable!("unsupported method: {}", plan.method),
        };

        let mut request = self.client.request(method, url).query(&plan.query);

        request = match &plan.auth {
            Auth::ApiKey => request.header("X-Riot-Token", self.config.api_key.as_str()),
            Auth::Bearer(token) => {
                request.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            }
        };

        if let Some(body) = &plan.body {
            request = request
                .header("Content-Type", "application/json")
                .body(body.clone());
        }

        let response = request.send().await?;

        Ok(response)
    }

    async fn headers_apply(&self, plan: &RequestPlan, headers: &HeaderMap) {
        let rate_headers = RateHeaders {
            app_count: header_str(headers, "x-app-rate-limit-count"),
            app_limit: header_str(headers, "x-app-rate-limit"),
            method_count: header_str(headers, "x-method-rate-limit-count"),
            method_limit: header_str(headers, "x-method-rate-limit"),
        };

        self.rate_limiter
            .update(plan.route, plan.endpoint_id, &rate_headers)
            .await;
    }

    fn url_build(&self, plan: &RequestPlan) -> String {
        let host = self.config.base_url.replace(ROUTE_PLACEHOLDER, plan.route);

        format!("{host}{}", plan.path)
    }

    fn hooks_on_request(&self, info: &RequestInfo<'_>) {
        trace_request(info);

        if let Some(hooks) = &self.config.hooks {
            hooks.on_request(info);
        }
    }

    fn hooks_on_response(&self, info: &RequestInfo<'_>, status: u16, elapsed: Duration) {
        trace_response(info, status, elapsed);

        if let Some(hooks) = &self.config.hooks {
            hooks.on_response(info, status, elapsed);
        }
    }

    fn hooks_on_retry(&self, info: &RequestInfo<'_>, attempt: u32, wait: Duration) {
        trace_retry(info, attempt, wait);

        if let Some(hooks) = &self.config.hooks {
            hooks.on_retry(info, attempt, wait);
        }
    }

    fn hooks_on_rate_limit_sleep(&self, route: &str, scope: &str, wait: Duration) {
        trace_rate_limit_sleep(route, scope, wait);

        if let Some(hooks) = &self.config.hooks {
            hooks.on_rate_limit_sleep(route, scope, wait);
        }
    }

    #[cfg(feature = "cache")]
    fn cache_lookup(&self, plan: &RequestPlan, url: &str) -> Option<Bytes> {
        if plan.method != "GET" {
            return None;
        }

        self.cache.as_ref().and_then(|cache| cache.get(url))
    }

    #[cfg(not(feature = "cache"))]
    #[expect(clippy::unused_self)]
    fn cache_lookup(&self, _plan: &RequestPlan, _url: &str) -> Option<Bytes> {
        None
    }

    #[cfg(feature = "cache")]
    fn cache_store(&self, plan: &RequestPlan, url: &str, body: &Bytes) {
        if plan.method != "GET" {
            return;
        }

        if let Some(cache) = &self.cache {
            cache.insert(url, plan.endpoint_id, body);
        }
    }

    #[cfg(not(feature = "cache"))]
    #[expect(clippy::unused_self)]
    fn cache_store(&self, _plan: &RequestPlan, _url: &str, _body: &Bytes) {}
}

#[cfg(feature = "tracing")]
fn trace_request(info: &RequestInfo<'_>) {
    tracing::debug!(
        endpoint = info.endpoint_id,
        method = info.method,
        route = info.route,
        "rift request",
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_request(_info: &RequestInfo<'_>) {}

#[cfg(feature = "tracing")]
fn trace_response(info: &RequestInfo<'_>, status: u16, elapsed: Duration) {
    tracing::debug!(
        endpoint = info.endpoint_id,
        route = info.route,
        status,
        elapsed_ms = elapsed.as_millis(),
        "rift response",
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_response(_info: &RequestInfo<'_>, _status: u16, _elapsed: Duration) {}

#[cfg(feature = "tracing")]
fn trace_retry(info: &RequestInfo<'_>, attempt: u32, wait: Duration) {
    tracing::warn!(
        endpoint = info.endpoint_id,
        route = info.route,
        attempt,
        wait_ms = wait.as_millis(),
        "rift retry",
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_retry(_info: &RequestInfo<'_>, _attempt: u32, _wait: Duration) {}

#[cfg(feature = "tracing")]
fn trace_rate_limit_sleep(route: &str, scope: &str, wait: Duration) {
    tracing::debug!(
        route,
        scope,
        wait_ms = wait.as_millis(),
        "rift rate limit sleep",
    );
}

#[cfg(not(feature = "tracing"))]
fn trace_rate_limit_sleep(_route: &str, _scope: &str, _wait: Duration) {}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn retry_after_duration(headers: &HeaderMap) -> Option<Duration> {
    let seconds = header_str(headers, "retry-after")?
        .trim()
        .parse::<u64>()
        .ok()?;

    Some(Duration::from_secs(seconds.min(RETRY_AFTER_S_MAX)))
}

#[doc(hidden)]
#[must_use]
pub fn backoff_duration(attempts: u32) -> Duration {
    assert!(attempts >= 1, "attempts must be positive");

    let base = RETRY_BACKOFF_MS_BASE
        .checked_shl(attempts - 1)
        .unwrap_or(u64::MAX)
        .min(RETRY_BACKOFF_MS_MAX);

    let jitter = jitter_ms();

    assert!(jitter < JITTER_MS_MAX, "jitter must be bounded");

    Duration::from_millis(base + jitter)
}

/// Splitmix64 over an atomic counter: integer-only, lock-free, and seeded from
/// process uptime plus a stack address so concurrent tasks decorrelate even
/// when they back off at the same instant.
static JITTER_STATE: LazyLock<AtomicU64> = LazyLock::new(|| {
    let stack_marker = 0_u8;
    let address = std::ptr::from_ref(&stack_marker).addr() as u64;
    let seed = u64::from(PROCESS_START.elapsed().subsec_nanos()) ^ address;

    AtomicU64::new(seed | 1)
});

fn jitter_ms() -> u64 {
    let state = JITTER_STATE.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
    let mut mixed = state;

    mixed ^= mixed >> 30;
    mixed = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^= mixed >> 31;

    mixed % JITTER_MS_MAX
}

async fn body_snippet_read(response: reqwest::Response) -> Vec<u8> {
    let Ok(bytes) = response.bytes().await else {
        return Vec::new();
    };

    let end = bytes.len().min(STATUS_BODY_BYTES_MAX);

    bytes[..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::{JITTER_MS_MAX, backoff_duration, jitter_ms};

    #[test]
    fn jitter_is_bounded_and_varied() {
        let mut seen = std::collections::HashSet::new();

        for _ in 0..1_000 {
            let jitter = jitter_ms();

            assert!(jitter < JITTER_MS_MAX);

            seen.insert(jitter);
        }

        assert!(
            seen.len() >= 100,
            "jitter must vary: {} distinct",
            seen.len()
        );
    }

    #[test]
    fn backoff_doubles_and_caps() {
        let first = backoff_duration(1).as_millis();
        let second = backoff_duration(2).as_millis();
        let late = backoff_duration(16).as_millis();

        assert!((1_000..1_500).contains(&first));
        assert!((2_000..2_500).contains(&second));
        assert!((32_000..32_500).contains(&late));
    }
}
