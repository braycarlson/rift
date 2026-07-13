use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use bytes::Bytes;
use rustc_hash::{FxHashMap, FxHasher};

const ENTRIES_MAX: usize = 512;

struct Entry {
    bytes: Bytes,
    url: Box<str>,
}

struct Inner {
    entries: FxHashMap<u64, Entry>,
    order: VecDeque<u64>,
}

#[doc(hidden)]
pub struct DragonCache {
    inner: Mutex<Inner>,
}

#[allow(clippy::new_without_default)]
impl DragonCache {
    #[must_use]
    pub fn new() -> DragonCache {
        DragonCache {
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
        let inner = self.lock();

        inner
            .entries
            .get(&key)
            .filter(|entry| entry.url.as_ref() == url)
            .map(|entry| entry.bytes.clone())
    }

    pub fn insert(&self, url: &str, bytes: &Bytes) {
        assert!(!url.is_empty(), "url must not be empty");

        let key = url_hash(url);
        let mut inner = self.lock();

        if inner.entries.contains_key(&key) {
            return;
        }

        let mut iterations: usize = 0;

        while inner.entries.len() >= ENTRIES_MAX {
            iterations += 1;

            assert!(
                iterations <= ENTRIES_MAX + 1,
                "eviction exceeded {ENTRIES_MAX} iterations"
            );

            let Some(evicted) = inner.order.pop_front() else {
                break;
            };

            inner.entries.remove(&evicted);
        }

        inner.entries.insert(
            key,
            Entry {
                bytes: bytes.clone(),
                url: Box::from(url),
            },
        );

        inner.order.push_back(key);

        assert!(
            inner.order.len() == inner.entries.len(),
            "cache order and entries must stay in sync"
        );
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn url_hash(url: &str) -> u64 {
    assert!(!url.is_empty(), "url must not be empty");

    let mut hasher = FxHasher::default();

    url.hash(&mut hasher);

    hasher.finish()
}
