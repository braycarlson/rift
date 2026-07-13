use rift::routes::ValPlatformRoute;
use rift::{RiotApi, RiotApiConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let api = RiotApi::new(RiotApiConfig::from_env()?)?;

    let content = api
        .val_content_v1_get_content(ValPlatformRoute::Na, None)
        .await?;

    println!("Valorant content version: {}", content.version);
    println!("  acts:       {}", content.acts.len());
    println!("  characters: {}", content.characters.len());
    println!("  maps:       {}", content.maps.len());

    Ok(())
}
