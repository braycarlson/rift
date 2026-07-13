use rift::consts::queue_type;
use rift::routes::PlatformRoute;
use rift::{RiotApi, RiotApiConfig};

const LADDER_TOP_SHOWN: usize = 5;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let api = RiotApi::new(RiotApiConfig::from_env()?)?;

    let ladder = api
        .league_v4_get_challenger_league(PlatformRoute::Na1, queue_type::RANKED_SOLO_5X5)
        .await?;

    println!(
        "{} challenger: {} entries",
        ladder.tier,
        ladder.entries.len()
    );

    let mut entries = ladder.entries;

    entries.sort_by_key(|entry| std::cmp::Reverse(entry.league_points));

    for entry in entries.iter().take(LADDER_TOP_SHOWN) {
        println!("  {} LP: {} wins", entry.league_points, entry.wins);
    }

    Ok(())
}
