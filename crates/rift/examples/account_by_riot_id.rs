use rift::routes::RegionalRoute;
use rift::{RiotApi, RiotApiConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let api_key = std::env::var("RGAPI_KEY")?;
    let api = RiotApi::new(RiotApiConfig::new(api_key))?;

    let account = api
        .account_v1_get_by_riot_id(RegionalRoute::Americas, "Doublelift", "NA1")
        .await?;

    println!("{account:#?}");

    Ok(())
}
