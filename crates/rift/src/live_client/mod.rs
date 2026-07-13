use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::error::Error;

pub mod models;

use models::{ActivePlayer, AllGameData, EventData, GameStats, Player};

const BASE_URL: &str = "https://127.0.0.1:2999";
const CERTIFICATE_PEM: &str = include_str!("riotgames.pem");
const TIMEOUT_MS: u64 = 5_000;

/// A client for the League of Legends Live Client Data API.
///
/// The API is served over HTTPS on `127.0.0.1:2999` by a running game client,
/// using Riot's self-signed root certificate. [`LiveClient::new`] pins that
/// certificate with [`reqwest::ClientBuilder::add_root_certificate`], so TLS is
/// still verified. [`LiveClient::new_insecure`] disables verification entirely
/// and exists only as a documented escape hatch.
pub struct LiveClient {
    client: reqwest::Client,
}

impl LiveClient {
    /// Builds a client that trusts only Riot's pinned self-signed certificate.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the pinned certificate cannot be parsed
    /// or the HTTP client cannot be built.
    pub fn new() -> Result<LiveClient, Error> {
        let certificate = reqwest::Certificate::from_pem(CERTIFICATE_PEM.as_bytes())?;

        let client = reqwest::Client::builder()
            .user_agent(concat!("rift/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .add_root_certificate(certificate)
            .build()?;

        Ok(LiveClient { client })
    }

    /// Builds a client that does not verify the server certificate.
    ///
    /// Prefer [`LiveClient::new`]. This exists only for environments where the
    /// pinned certificate cannot be used; it accepts any certificate and so
    /// offers no protection against a local man-in-the-middle.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the HTTP client cannot be built.
    pub fn new_insecure() -> Result<LiveClient, Error> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("rift/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(LiveClient { client })
    }

    /// Fetches the complete game snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the game client is unreachable or the response
    /// cannot be parsed.
    pub async fn all_game_data(&self) -> Result<AllGameData, Error> {
        self.get("live-client.allGameData", "/liveclientdata/allgamedata")
            .await
    }

    /// Fetches the local player's live state.
    ///
    /// # Errors
    ///
    /// Returns an error if the game client is unreachable or the response
    /// cannot be parsed.
    pub async fn active_player(&self) -> Result<ActivePlayer, Error> {
        self.get("live-client.activePlayer", "/liveclientdata/activeplayer")
            .await
    }

    /// Fetches every player in the game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game client is unreachable or the response
    /// cannot be parsed.
    pub async fn player_list(&self) -> Result<Vec<Player>, Error> {
        self.get("live-client.playerList", "/liveclientdata/playerlist")
            .await
    }

    /// Fetches the match event log.
    ///
    /// # Errors
    ///
    /// Returns an error if the game client is unreachable or the response
    /// cannot be parsed.
    pub async fn event_data(&self) -> Result<EventData, Error> {
        self.get("live-client.eventData", "/liveclientdata/eventdata")
            .await
    }

    /// Fetches match-wide statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if the game client is unreachable or the response
    /// cannot be parsed.
    pub async fn game_stats(&self) -> Result<GameStats, Error> {
        self.get("live-client.gameStats", "/liveclientdata/gamestats")
            .await
    }

    #[doc(hidden)]
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint_id: &'static str,
        path: &str,
    ) -> Result<T, Error> {
        assert!(!endpoint_id.is_empty(), "endpoint id must not be empty");
        assert!(path.starts_with('/'), "path must be absolute: {path}");

        let url = format!("{BASE_URL}{path}");
        let response = self.client.get(&url).send().await?;
        let status = response.status().as_u16();

        if !(200..300).contains(&status) {
            let body = response.bytes().await.unwrap_or_default();

            return Err(Error::status(endpoint_id, status, &body, None));
        }

        let body = response.bytes().await?;

        serde_json::from_slice::<T>(&body).map_err(|source| Error::deserialize(endpoint_id, source))
    }
}
