use bytes::Bytes;

use dragon::DragonCache;

#[test]
fn insert_then_get_hits() {
    let cache = DragonCache::new();
    let body = Bytes::from_static(b"data");

    cache.insert("https://x/champion.json", &body);

    assert!(
        cache.get("https://x/champion.json").as_deref() == Some(b"data".as_ref()),
        "cached body must be returned"
    );
}

#[test]
fn miss_returns_none() {
    let cache = DragonCache::new();

    assert!(cache.get("https://x/none").is_none(), "miss must be none");
}

#[test]
fn eviction_drops_oldest_and_keeps_recent() {
    let cache = DragonCache::new();
    let inserts: u32 = 600;
    let body = Bytes::from_static(b"data");
    let mut index: u32 = 0;

    while index < inserts {
        index += 1;

        cache.insert(&format!("https://x/{index}.json"), &body);
    }

    assert!(
        cache.get("https://x/1.json").is_none(),
        "oldest entry must be evicted"
    );
    assert!(
        cache.get(&format!("https://x/{inserts}.json")).is_some(),
        "most recent entry must be retained"
    );
}

#[test]
fn reinsert_same_url_keeps_first_body() {
    let cache = DragonCache::new();
    let first = Bytes::from_static(b"first");
    let second = Bytes::from_static(b"second");

    cache.insert("https://x/champion.json", &first);
    cache.insert("https://x/champion.json", &second);

    assert!(
        cache.get("https://x/champion.json").as_deref() == Some(b"first".as_ref()),
        "reinsert must not overwrite the cached body"
    );
}
