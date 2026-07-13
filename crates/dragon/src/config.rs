const RETRY_COUNT_DEFAULT: u32 = 4;
const TIMEOUT_MS_DEFAULT: u64 = 10_000;

pub(crate) const RETRY_COUNT_MAX: u32 = 16;
pub(crate) const TIMEOUT_MS_MAX: u64 = 300_000;

const _: () = assert!(RETRY_COUNT_DEFAULT <= RETRY_COUNT_MAX);
const _: () = assert!(TIMEOUT_MS_DEFAULT <= TIMEOUT_MS_MAX);

/// Configuration for a [`crate::DragonApi`] client.
pub struct DragonApiConfig {
    /// Whether to cache fetched files in memory. Data Dragon assets are
    /// immutable per version and locale, so caching is enabled by default.
    pub cache_enabled: bool,
    /// Number of retries after the first attempt for retriable failures.
    pub retry_count: u32,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
}

impl DragonApiConfig {
    /// Builds a configuration with defaults: retries on, caching on.
    #[must_use]
    pub fn new() -> DragonApiConfig {
        DragonApiConfig {
            cache_enabled: true,
            retry_count: RETRY_COUNT_DEFAULT,
            timeout_ms: TIMEOUT_MS_DEFAULT,
        }
    }
}

impl Default for DragonApiConfig {
    fn default() -> DragonApiConfig {
        DragonApiConfig::new()
    }
}
