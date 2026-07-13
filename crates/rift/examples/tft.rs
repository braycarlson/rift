use rift::endpoints::TftMatchV1GetMatchIdsByPuuidQuery;
use rift::routes::RegionalRoute;
use rift::{RiotApi, RiotApiConfig};

const TFT_MATCH_IDS_COUNT: i32 = 5;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let api = RiotApi::new(RiotApiConfig::from_env()?)?;

    let account = api
        .account_v1_get_by_riot_id(RegionalRoute::Americas, "Doublelift", "NA1")
        .await?
        .ok_or("account not found")?;

    let query = TftMatchV1GetMatchIdsByPuuidQuery {
        count: Some(TFT_MATCH_IDS_COUNT),
        ..Default::default()
    };

    let match_ids = api
        .tft_match_v1_get_match_ids_by_puuid(RegionalRoute::Americas, &account.puuid, &query)
        .await?;

    println!("recent TFT matches: {}", match_ids.len());

    let Some(match_id) = match_ids.first() else {
        return Ok(());
    };

    let Some(match_data) = api
        .tft_match_v1_get_match(RegionalRoute::Americas, match_id)
        .await?
    else {
        return Ok(());
    };

    println!(
        "match {} had {} participants",
        match_id,
        match_data.info.participants.len()
    );

    Ok(())
}
