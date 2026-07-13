use rift::endpoints::MatchV5GetMatchIdsByPuuidQuery;
use rift::pagination::{LEAGUE_PAGES_MAX, MATCH_IDS_ALL_MAX};
use rift::routes::{PlatformRoute, RegionalRoute};
use rift::{RiotApi, RiotApiConfig};

fn api() -> RiotApi {
    RiotApi::new(RiotApiConfig::new("test-key".to_string())).expect("client build")
}

#[tokio::test]
#[should_panic(expected = "ids_max exceeds")]
async fn match_ids_all_bounds_request() {
    let api = api();
    let query = MatchV5GetMatchIdsByPuuidQuery::default();

    let _ = api
        .match_v5_match_ids_all(
            RegionalRoute::Americas,
            "puuid",
            &query,
            MATCH_IDS_ALL_MAX + 1,
        )
        .await;
}

#[tokio::test]
#[should_panic(expected = "pages_max exceeds")]
async fn league_entries_all_bounds_pages() {
    let api = api();

    let _ = api
        .league_v4_entries_all(
            PlatformRoute::Na1,
            "RANKED_SOLO_5x5",
            "DIAMOND",
            "I",
            LEAGUE_PAGES_MAX + 1,
        )
        .await;
}
