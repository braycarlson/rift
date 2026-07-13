use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bytes::Bytes;
use rustc_hash::{FxHashMap, FxHasher};

const ENTRIES_MAX_DEFAULT: usize = 4_096;
#[doc(hidden)]
pub const ENTRY_BYTES_MAX_DEFAULT: usize = 1024 * 1024;
const TTL_OVERRIDE_COUNT_MAX: usize = 64;

const TTL_TABLE: [(&str, u64); 3] = [
    ("match-v5.", 3_600),
    ("summoner-v4.", 300),
    ("-status-", 60),
];

/// Configuration for the optional in-memory response cache (feature `cache`).
///
/// Only `GET` responses are cached. Time-to-live is chosen per endpoint from a
/// built-in table (`match-v5.` for one hour, `summoner-v4.` for five minutes,
/// any status endpoint for one minute, everything else uncached), with
/// user-supplied [`CacheConfig::ttl_overrides`] taking precedence.
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Maximum number of cached entries before FIFO eviction begins.
    pub entries_max: usize,
    /// Maximum size of a single cached body; larger responses are not cached.
    pub entry_bytes_max: usize,
    /// User overrides as `(endpoint-id substring, ttl seconds)`, checked first.
    pub ttl_overrides: Vec<(String, u64)>,
}

impl CacheConfig {
    /// Builds a cache configuration with the given caps and no TTL overrides.
    #[must_use]
    pub fn new(entries_max: usize, entry_bytes_max: usize) -> CacheConfig {
        assert!(entries_max >= 1, "entries_max must be positive");
        assert!(
            entries_max <= ENTRIES_MAX_DEFAULT,
            "entries_max exceeds {ENTRIES_MAX_DEFAULT}"
        );
        assert!(entry_bytes_max >= 1, "entry_bytes_max must be positive");
        assert!(
            entry_bytes_max <= ENTRY_BYTES_MAX_DEFAULT,
            "entry_bytes_max exceeds {ENTRY_BYTES_MAX_DEFAULT}"
        );

        CacheConfig {
            entries_max,
            entry_bytes_max,
            ttl_overrides: Vec::new(),
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn ttl_seconds(&self, endpoint_id: &str) -> u64 {
        assert!(
            self.ttl_overrides.len() <= TTL_OVERRIDE_COUNT_MAX,
            "ttl override count exceeds {TTL_OVERRIDE_COUNT_MAX}"
        );

        for (pattern, ttl) in &self.ttl_overrides {
            if endpoint_id.contains(pattern.as_str()) {
                return *ttl;
            }
        }

        for (pattern, ttl) in TTL_TABLE {
            if endpoint_id.contains(pattern) {
                return ttl;
            }
        }

        0
    }
}

impl Default for CacheConfig {
    fn default() -> CacheConfig {
        CacheConfig::new(ENTRIES_MAX_DEFAULT, ENTRY_BYTES_MAX_DEFAULT)
    }
}

#[doc(hidden)]
pub struct Entry {
    bytes: Bytes,
    pub expires_at: Instant,
    url: Box<str>,
}

#[doc(hidden)]
pub struct Inner {
    pub entries: FxHashMap<u64, Entry>,
    order: VecDeque<u64>,
}

#[doc(hidden)]
pub struct ResponseCache {
    config: CacheConfig,
    inner: Mutex<Inner>,
}

impl ResponseCache {
    #[must_use]
    pub fn new(config: CacheConfig) -> ResponseCache {
        assert!(config.entries_max >= 1, "entries_max must be positive");
        assert!(
            config.entry_bytes_max >= 1,
            "entry_bytes_max must be positive"
        );

        ResponseCache {
            config,
            inner: Mutex::new(Inner {
                entries: FxHashMap::default(),
                order: VecDeque::new(),
            }),
        }
    }

    #[must_use]
    pub fn get(&self, url: &str) -> Option<Bytes> {
        assert!(!url.is_empty(), "url must not be empty");

        let key = url_hash(url);
        let now = Instant::now();
        let inner = self.lock();

        match inner.entries.get(&key) {
            Some(entry) if entry.expires_at > now && entry.url.as_ref() == url => {
                Some(entry.bytes.clone())
            }
            _ => None,
        }
    }

    pub fn insert(&self, url: &str, endpoint_id: &str, bytes: &Bytes) {
        assert!(!url.is_empty(), "url must not be empty");
        assert!(!endpoint_id.is_empty(), "endpoint id must not be empty");

        let ttl = self.config.ttl_seconds(endpoint_id);

        if ttl == 0 {
            return;
        }

        if bytes.len() > self.config.entry_bytes_max {
            return;
        }

        let key = url_hash(url);
        let expires_at = Instant::now() + Duration::from_secs(ttl);

        let entry = Entry {
            bytes: bytes.clone(),
            expires_at,
            url: Box::from(url),
        };

        let mut inner = self.lock();

        if !inner.entries.contains_key(&key) {
            evict_to_capacity(&mut inner, self.config.entries_max);
        }

        if inner.entries.insert(key, entry).is_none() {
            inner.order.push_back(key);
        }

        assert!(
            inner.order.len() == inner.entries.len(),
            "cache order and entries must stay in sync"
        );
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn evict_to_capacity(inner: &mut Inner, entries_max: usize) {
    assert!(entries_max >= 1, "entries_max must be positive");

    let mut iterations: usize = 0;

    while inner.entries.len() >= entries_max {
        iterations += 1;

        assert!(
            iterations <= entries_max + 1,
            "eviction exceeded {entries_max} iterations"
        );

        let Some(key) = inner.order.pop_front() else {
            break;
        };

        inner.entries.remove(&key);
    }
}

fn url_hash(url: &str) -> u64 {
    assert!(!url.is_empty(), "url must not be empty");

    let mut hasher = FxHasher::default();

    url.hash(&mut hasher);

    hasher.finish()
}
