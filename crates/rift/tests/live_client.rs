#![cfg(feature = "live-client")]

use rift::Error;
use rift::live_client::LiveClient;
use rift::live_client::models::AllGameData;

#[tokio::test]
#[should_panic(expected = "path must be absolute")]
async fn get_rejects_relative_path() {
    let client = LiveClient::new().expect("client build");

    let _: Result<AllGameData, Error> = client.get("live-client.test", "liveclientdata").await;
}
