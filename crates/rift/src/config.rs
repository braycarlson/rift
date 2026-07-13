use std::sync::Arc;

#[cfg(feature = "cache")]
use crate::cache::CacheConfig;
use crate::hooks::Hooks;
use crate::rate_limit::{LimiterSettings, RateLimiter};

const API_KEY_LENGTH_MAX: usize = 256;
const CONCURRENT_REQUESTS_MAX_DEFAULT: u32 = 256;
const RETRY_COUNT_DEFAULT: u32 = 4;
const TIMEOUT_MS_DEFAULT: u64 = 10_000;

pub(crate) const BASE_URL_DEFAULT: &str = "https://{route}.api.riotgames.com";
pub(crate) const CONCURRENT_REQUESTS_MAX: u32 = 4_096;
pub(crate) const RETRY_COUNT_MAX: u32 = 16;
pub(crate) const ROUTE_PLACEHOLDER: &str = "{route}";
pub(crate) const TIMEOUT_MS_MAX: u64 = 300_000;

/// Configuration for a [`crate::RiotApi`] client.
///
/// Construct with [`RiotApiConfig::new`] (or [`RiotApiConfig::from_env`]) and
/// then set public fields as needed:
///
/// ```no_run
/// use rift::RiotApiConfig;
///
/// let mut config = RiotApiConfig::new("RGAPI-...".to_string());
/// config.retry_count = 2;
/// config.concurrent_requests_max = 64;
/// ```
///
/// All values are validated when the [`crate::RiotApi`] client is built.
pub struct RiotApiConfig {
    /// The Riot API key sent as `X-Riot-Token` on non-bearer endpoints.
    pub api_key: String,
    /// Base URL template containing exactly one `{route}` placeholder.
    ///
    /// Defaults to `https://{route}.api.riotgames.com`. Override to route
    /// requests through a proxy such as Kernel.
    pub base_url: String,
    /// Optional response-cache configuration (feature `cache`).
    #[cfg(feature = "cache")]
    pub cache: Option<CacheConfig>,
    /// Upper bound on in-flight requests from this client.
    pub concurrent_requests_max: u32,
    /// Optional observe-only lifecycle hooks.
    pub hooks: Option<Arc<dyn Hooks>>,
    /// Settings for the rate limiter created when [`Self::rate_limiter`] is
    /// `None`.
    pub limiter_settings: LimiterSettings,
    /// A shared rate limiter; `None` builds a fresh per-client limiter.
    pub rate_limiter: Option<Arc<RateLimiter>>,
    /// Number of retries after the first attempt for retriable failures.
    pub retry_count: u32,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
}

impl RiotApiConfig {
    /// Builds a configuration from an API key, filling every other field with
    /// its default.
    #[must_use]
    pub fn new(api_key: String) -> RiotApiConfig {
        assert!(!api_key.is_empty(), "api_key must not be empty");
        assert!(
            api_key.len() <= API_KEY_LENGTH_MAX,
            "api_key exceeds {API_KEY_LENGTH_MAX} bytes"
        );
        assert!(
            api_key
                .chars()
                .all(|character| character.is_ascii_graphic()),
            "api_key must be printable ascii",
        );
        assert!(
            BASE_URL_DEFAULT.matches(ROUTE_PLACEHOLDER).count() == 1,
            "base url must contain exactly one {ROUTE_PLACEHOLDER}"
        );

        RiotApiConfig {
            api_key,
            base_url: BASE_URL_DEFAULT.to_string(),
            #[cfg(feature = "cache")]
            cache: None,
            concurrent_requests_max: CONCURRENT_REQUESTS_MAX_DEFAULT,
            hooks: None,
            limiter_settings: LimiterSettings::default(),
            rate_limiter: None,
            retry_count: RETRY_COUNT_DEFAULT,
            timeout_ms: TIMEOUT_MS_DEFAULT,
        }
    }

    /// Builds a configuration from the environment.
    ///
    /// Reads `RGAPI_KEY` first, then falls back to `RIOT_API_KEY`. Returns the
    /// lookup error if neither variable is set.
    ///
    /// # Errors
    ///
    /// Returns [`std::env::VarError`] when no key variable is present.
    pub fn from_env() -> Result<RiotApiConfig, std::env::VarError> {
        let api_key = match std::env::var("RGAPI_KEY") {
            Ok(key) => key,
            Err(_) => std::env::var("RIOT_API_KEY")?,
        };

        Ok(RiotApiConfig::new(api_key))
    }
}
