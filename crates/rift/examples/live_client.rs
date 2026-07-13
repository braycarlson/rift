use rift::live_client::LiveClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LiveClient::new()?;

    match client.all_game_data().await {
        Ok(data) => {
            println!("game mode: {}", data.game_data.game_mode);
            println!("game time: {:.0}s", data.game_data.game_time);
            println!("players:   {}", data.all_players.len());
        }
        Err(error) => {
            println!("Live Client Data API unavailable (is a game in progress?):");
            println!("  {error}");
        }
    }

    Ok(())
}
