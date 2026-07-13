use rift::endpoints::MatchV5GetMatchIdsByPuuidQuery;
use rift::routes::RegionalRoute;
use rift::{RiotApi, RiotApiConfig};

const MATCH_IDS_MAX: u32 = 120;
const MATCH_IDS_SHOWN: usize = 5;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let api = RiotApi::new(RiotApiConfig::from_env()?)?;

    let account = api
        .account_v1_get_by_riot_id(RegionalRoute::Americas, "Doublelift", "NA1")
        .await?
        .ok_or("account not found")?;

    let query = MatchV5GetMatchIdsByPuuidQuery::default();

    let match_ids = api
        .match_v5_match_ids_all(
            RegionalRoute::Americas,
            &account.puuid,
            &query,
            MATCH_IDS_MAX,
        )
        .await?;

    println!("collected {} match ids", match_ids.len());

    for match_id in match_ids.iter().take(MATCH_IDS_SHOWN) {
        println!("  {match_id}");
    }

    Ok(())
}
