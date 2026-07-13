use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rift::endpoints::MatchV5GetMatchIdsByPuuidQuery;
use rift::routes::RegionalRoute;
use rift::{Hooks, RequestInfo, RiotApi, RiotApiConfig};

const ACCOUNT_GAME_NAME: &str = "Doublelift";
const ACCOUNT_TAG_LINE: &str = "NA1";
const MATCH_IDS_ALL_COUNT: u32 = 120;
const MATCH_IDS_COUNT_MAX: i32 = 5;

#[derive(Default)]
struct CountingHooks {
    requests: AtomicU32,
    responses: AtomicU32,
}

impl Hooks for CountingHooks {
    fn on_request(&self, _info: &RequestInfo<'_>) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    fn on_response(&self, _info: &RequestInfo<'_>, _status: u16, _elapsed: Duration) {
        self.responses.fetch_add(1, Ordering::Relaxed);
    }
}

fn api_from_env() -> RiotApi {
    let _ = dotenvy::dotenv();

    let api_key = std::env::var("RGAPI_KEY").expect("RGAPI_KEY must be set via .env or shell");

    assert!(!api_key.is_empty(), "RGAPI_KEY must not be empty");

    RiotApi::new(RiotApiConfig::new(api_key)).expect("client construction must succeed")
}

async fn account_fetch(api: &RiotApi) -> rift::models::account_v1::AccountDto {
    api.account_v1_get_by_riot_id(RegionalRoute::Americas, ACCOUNT_GAME_NAME, ACCOUNT_TAG_LINE)
        .await
        .expect("account lookup must succeed")
        .expect("account must be present")
}

#[tokio::test]
async fn account_by_riot_id_returns_account() {
    let api = api_from_env();
    let account = account_fetch(&api).await;

    assert!(!account.puuid.is_empty(), "puuid must not be empty");

    let game_name = account.game_name.expect("game_name must be present");
    let tag_line = account.tag_line.expect("tag_line must be present");

    assert!(!game_name.is_empty(), "game_name must not be empty");
    assert!(
        tag_line.eq_ignore_ascii_case(ACCOUNT_TAG_LINE),
        "tag line must match"
    );
}

#[tokio::test]
async fn account_by_riot_id_unknown_returns_none() {
    let api = api_from_env();

    let result = api
        .account_v1_get_by_riot_id(RegionalRoute::Americas, "NoSuchNameZz9Zz9", "ZZ99")
        .await
        .expect("nullable endpoint must not error on 404");

    assert!(
        result.is_none(),
        "unknown account must be None, got {result:?}"
    );
}

#[tokio::test]
async fn api_key_invalid_returns_status() {
    let api = RiotApi::new(RiotApiConfig::new("RGAPI-invalid-key".to_string()))
        .expect("client construction must succeed");

    let result = api
        .account_v1_get_by_riot_id(RegionalRoute::Americas, ACCOUNT_GAME_NAME, ACCOUNT_TAG_LINE)
        .await;

    match result {
        Err(error) => {
            let status = error.status_code().expect("auth failure carries a status");

            assert!(
                status == 401 || status == 403,
                "expected auth failure, got status {status}"
            );
        }
        other => panic!("expected status error, got {other:?}"),
    }
}

#[tokio::test]
async fn match_ids_by_puuid_returns_bounded_list() {
    let api = api_from_env();
    let account = account_fetch(&api).await;

    let query = MatchV5GetMatchIdsByPuuidQuery {
        count: Some(MATCH_IDS_COUNT_MAX),
        ..Default::default()
    };

    let match_ids = api
        .match_v5_get_match_ids_by_puuid(RegionalRoute::Americas, &account.puuid, &query)
        .await
        .expect("match id lookup must succeed");

    assert!(
        match_ids.len() <= MATCH_IDS_COUNT_MAX as usize,
        "match id count must be bounded"
    );
    assert!(
        match_ids.iter().all(|id| id.contains('_')),
        "match ids must be platform-prefixed, got {match_ids:?}"
    );
}

#[tokio::test]
async fn match_ids_all_paginates_past_one_page() {
    let api = api_from_env();
    let account = account_fetch(&api).await;

    let query = MatchV5GetMatchIdsByPuuidQuery::default();

    let match_ids = api
        .match_v5_match_ids_all(
            RegionalRoute::Americas,
            &account.puuid,
            &query,
            MATCH_IDS_ALL_COUNT,
        )
        .await
        .expect("paginated match id lookup must succeed");

    assert!(
        match_ids.len() <= MATCH_IDS_ALL_COUNT as usize,
        "must not exceed the requested cap"
    );
    assert!(
        match_ids.iter().all(|id| id.contains('_')),
        "match ids must be platform-prefixed"
    );
}

#[tokio::test]
async fn match_by_id_deserializes_full_model() {
    let api = api_from_env();
    let account = account_fetch(&api).await;

    let query = MatchV5GetMatchIdsByPuuidQuery {
        count: Some(1),
        ..Default::default()
    };

    let match_ids = api
        .match_v5_get_match_ids_by_puuid(RegionalRoute::Americas, &account.puuid, &query)
        .await
        .expect("match id lookup must succeed");

    let Some(match_id) = match_ids.first() else {
        println!("no recent matches for {ACCOUNT_GAME_NAME}#{ACCOUNT_TAG_LINE}, skipping");
        return;
    };

    let match_data = api
        .match_v5_get_match(RegionalRoute::Americas, match_id)
        .await
        .expect("match fetch must succeed")
        .expect("recent match must be present");

    assert_eq!(&match_data.metadata.match_id, match_id);
    assert!(
        !match_data.info.participants.is_empty(),
        "match must have participants"
    );
}

#[tokio::test]
async fn hooks_observe_every_request() {
    let _ = dotenvy::dotenv();

    let api_key = std::env::var("RGAPI_KEY").expect("RGAPI_KEY must be set via .env or shell");
    let hooks = Arc::new(CountingHooks::default());
    let mut config = RiotApiConfig::new(api_key);

    config.hooks = Some(hooks.clone());

    let api = RiotApi::new(config).expect("client construction must succeed");

    let _ = account_fetch(&api).await;

    assert!(
        hooks.requests.load(Ordering::Relaxed) >= 1,
        "on_request must fire"
    );
    assert!(
        hooks.responses.load(Ordering::Relaxed) >= 1,
        "on_response must fire"
    );
}

#[cfg(feature = "cache")]
#[tokio::test]
async fn cache_serves_repeated_get_without_network() {
    let _ = dotenvy::dotenv();

    let api_key = std::env::var("RGAPI_KEY").expect("RGAPI_KEY must be set via .env or shell");
    let hooks = Arc::new(CountingHooks::default());
    let mut config = RiotApiConfig::new(api_key);

    config.hooks = Some(hooks.clone());
    config.cache = Some(rift::CacheConfig::default());

    let api = RiotApi::new(config).expect("client construction must succeed");
    let account = account_fetch(&api).await;

    let first = api
        .summoner_v4_get_by_puuid(rift::routes::PlatformRoute::Na1, &account.puuid)
        .await
        .expect("first summoner lookup must succeed");

    let before = hooks.responses.load(Ordering::Relaxed);

    let second = api
        .summoner_v4_get_by_puuid(rift::routes::PlatformRoute::Na1, &account.puuid)
        .await
        .expect("second summoner lookup must succeed");

    let after = hooks.responses.load(Ordering::Relaxed);

    assert_eq!(first, second, "cached response must match the original");
    assert_eq!(
        before, after,
        "second identical GET must be served from cache, not the network"
    );
}
