#![cfg(feature = "cache")]

use std::time::Instant;

use bytes::Bytes;

use rift::cache::{CacheConfig, ENTRY_BYTES_MAX_DEFAULT, ResponseCache};

const ENTRIES_SMALL: usize = 16;
const ENTRY_BYTES_TINY: usize = 4;

fn cache(entries_max: usize) -> ResponseCache {
    ResponseCache::new(CacheConfig::new(entries_max, ENTRY_BYTES_MAX_DEFAULT))
}

#[test]
fn insert_then_get_hits() {
    let cache = cache(ENTRIES_SMALL);
    let body = Bytes::from_static(b"payload");

    cache.insert("https://x/match", "match-v5.getMatch", &body);

    assert!(
        cache.get("https://x/match").as_deref() == Some(b"payload".as_ref()),
        "cached body must be returned"
    );
}

#[test]
fn uncached_endpoint_not_stored() {
    let cache = cache(ENTRIES_SMALL);
    let body = Bytes::from_static(b"payload");

    cache.insert(
        "https://x/live",
        "spectator-v5.getCurrentGameInfoByPuuid",
        &body,
    );

    assert!(
        cache.get("https://x/live").is_none(),
        "ttl 0 must not cache"
    );
}

#[test]
fn oversized_body_not_stored() {
    let cache = ResponseCache::new(CacheConfig::new(ENTRIES_SMALL, ENTRY_BYTES_TINY));
    let body = Bytes::from_static(b"too-large");

    cache.insert("https://x/match", "match-v5.getMatch", &body);

    assert!(
        cache.get("https://x/match").is_none(),
        "oversized must skip"
    );
}

#[test]
fn fifo_eviction_drops_oldest() {
    let cache = cache(2);
    let body = Bytes::from_static(b"payload");

    cache.insert("https://x/1", "match-v5.getMatch", &body);
    cache.insert("https://x/2", "match-v5.getMatch", &body);
    cache.insert("https://x/3", "match-v5.getMatch", &body);

    assert!(cache.get("https://x/1").is_none(), "oldest must be evicted");
    assert!(cache.get("https://x/3").is_some(), "newest must remain");
}

#[test]
fn expired_entry_not_returned_and_keeps_invariant() {
    let cache = cache(4);
    let body = Bytes::from_static(b"payload");

    cache.insert("https://x/a", "match-v5.getMatch", &body);

    {
        let mut inner = cache.lock();

        for entry in inner.entries.values_mut() {
            entry.expires_at = Instant::now();
        }
    }

    assert!(
        cache.get("https://x/a").is_none(),
        "expired entry must not be returned"
    );

    cache.insert("https://x/b", "match-v5.getMatch", &body);

    assert!(
        cache.get("https://x/b").is_some(),
        "insert after an expired get must not desync the cache"
    );
}

#[test]
fn ttl_override_takes_precedence() {
    let mut config = CacheConfig::new(ENTRIES_SMALL, ENTRY_BYTES_MAX_DEFAULT);

    config.ttl_overrides.push(("league-v4.".to_string(), 120));

    assert!(
        config.ttl_seconds("league-v4.getLeagueById") == 120,
        "override must apply"
    );
    assert!(
        config.ttl_seconds("match-v5.getMatch") == 3_600,
        "table default must still apply"
    );
    assert!(
        config.ttl_seconds("champion-v3.getChampionInfo") == 0,
        "unlisted endpoint is uncached"
    );
}
