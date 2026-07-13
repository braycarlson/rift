use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::cache::DragonCache;
use crate::config::{self, DragonApiConfig};
use crate::error::Error;
use crate::models::{
    CDragonChampionSummary, ChampionDetailFile, ChampionsFile, ItemsFile, ProfileIconsFile, Realm,
    RuneTree, SummonerSpellsFile,
};
use crate::url;

const RESPONSE_BYTES_MAX: usize = 64 * 1024 * 1024;
const RETRY_AFTER_S_MAX: u64 = 120;
const RETRY_BACKOFF_MS_BASE: u64 = 1_000;
const RETRY_BACKOFF_MS_MAX: u64 = 32_000;

/// One attempt's outcome inside the retry loop: give up now, or retry.
enum Fetch {
    Retriable { retry_after: Option<Duration> },
    Terminal(Error),
}

pub struct DragonApi {
    cache: Option<DragonCache>,
    client: reqwest::Client,
    config: DragonApiConfig,
}

impl DragonApi {
    pub fn new(config: DragonApiConfig) -> Result<DragonApi, Error> {
        assert!(
            config.retry_count <= config::RETRY_COUNT_MAX,
            "retry_count exceeds {}",
            config::RETRY_COUNT_MAX,
        );
        assert!(config.timeout_ms > 0, "timeout_ms must be positive");
        assert!(
            config.timeout_ms <= config::TIMEOUT_MS_MAX,
            "timeout_ms exceeds {}",
            config::TIMEOUT_MS_MAX,
        );

        let client = reqwest::Client::builder()
            .user_agent(concat!("dragon/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()?;

        let cache = config.cache_enabled.then(DragonCache::new);

        Ok(DragonApi {
            cache,
            client,
            config,
        })
    }

    pub async fn cdragon_champion_summary_fetch(
        &self,
        patch: &str,
    ) -> Result<Vec<CDragonChampionSummary>, Error> {
        let url = url::cdragon_champion_summary_url(patch);

        self.execute("cdragon_champion_summary", &url).await
    }

    pub async fn champion_fetch(
        &self,
        version: &str,
        language: &str,
        key: &str,
    ) -> Result<ChampionDetailFile, Error> {
        let url = url::champion_detail_url(version, language, key);

        self.execute("champion", &url).await
    }

    pub async fn champions_fetch(
        &self,
        version: &str,
        language: &str,
    ) -> Result<ChampionsFile, Error> {
        let url = url::data_file_url(version, language, "champion.json");

        self.execute("champions", &url).await
    }

    pub async fn items_fetch(&self, version: &str, language: &str) -> Result<ItemsFile, Error> {
        let url = url::data_file_url(version, language, "item.json");

        self.execute("items", &url).await
    }

    pub async fn languages_fetch(&self) -> Result<Vec<String>, Error> {
        self.execute("languages", url::LANGUAGES_URL).await
    }

    pub async fn profile_icons_fetch(
        &self,
        version: &str,
        language: &str,
    ) -> Result<ProfileIconsFile, Error> {
        let url = url::data_file_url(version, language, "profileicon.json");

        self.execute("profile_icons", &url).await
    }

    pub async fn realms_fetch(&self, region: &str) -> Result<Realm, Error> {
        let url = url::realm_url(region);

        self.execute("realms", &url).await
    }

    pub async fn runes_fetch(&self, version: &str, language: &str) -> Result<Vec<RuneTree>, Error> {
        let url = url::data_file_url(version, language, "runesReforged.json");

        self.execute("runes", &url).await
    }

    pub async fn summoner_spells_fetch(
        &self,
        version: &str,
        language: &str,
    ) -> Result<SummonerSpellsFile, Error> {
        let url = url::data_file_url(version, language, "summoner.json");

        self.execute("summoner_spells", &url).await
    }

    pub async fn version_latest_fetch(&self) -> Result<String, Error> {
        let versions = self.versions_fetch().await?;

        let latest = versions
            .into_iter()
            .next()
            .ok_or_else(Error::versions_empty)?;

        assert!(!latest.is_empty(), "latest version must not be empty");

        Ok(latest)
    }

    pub async fn versions_fetch(&self) -> Result<Vec<String>, Error> {
        self.execute("versions", url::VERSIONS_URL).await
    }

    async fn execute<T: DeserializeOwned>(
        &self,
        resource_id: &'static str,
        url: &str,
    ) -> Result<T, Error> {
        let body = self.request_bytes(resource_id, url).await?;

        let value = serde_json::from_slice::<T>(&body)
            .map_err(|source| Error::deserialize(resource_id, source))?;

        Ok(value)
    }

    async fn request_bytes(
        &self,
        resource_id: &'static str,
        url: &str,
    ) -> Result<bytes::Bytes, Error> {
        assert!(!url.is_empty(), "url must not be empty");
        assert!(url.starts_with("https://"), "url must use https: {url}");

        if let Some(cache) = &self.cache
            && let Some(bytes) = cache.get(url)
        {
            return Ok(bytes);
        }

        let mut attempts: u32 = 0;

        loop {
            attempts += 1;

            assert!(
                attempts <= config::RETRY_COUNT_MAX + 1,
                "attempts exceeded {}",
                config::RETRY_COUNT_MAX + 1,
            );

            let retry_after = match self.attempt_fetch(resource_id, url).await {
                Ok(body) => {
                    if let Some(cache) = &self.cache {
                        cache.insert(url, &body);
                    }

                    return Ok(body);
                }
                Err(Fetch::Terminal(error)) => return Err(error),
                Err(Fetch::Retriable { retry_after }) => retry_after,
            };

            if attempts > self.config.retry_count {
                return Err(Error::retries_exhausted(resource_id));
            }

            let wait = retry_after.unwrap_or_else(|| backoff_duration(attempts));

            tokio::time::sleep(wait).await;
        }
    }

    async fn attempt_fetch(
        &self,
        resource_id: &'static str,
        url: &str,
    ) -> Result<bytes::Bytes, Fetch> {
        // Transport faults (timeouts, connection resets) participate in the
        // retry loop like retriable statuses instead of failing immediately.
        let response = match self.client.get(url).send().await {
            Ok(response) => response,
            Err(error) => return Err(fetch_transport(error)),
        };

        let status = response.status().as_u16();
        let retry_after = retry_after_duration(response.headers());

        if (200..300).contains(&status) {
            let body = match response.bytes().await {
                Ok(body) => body,
                Err(error) => return Err(fetch_transport(error)),
            };

            // The body size is network-controlled data, so an oversized asset
            // is an operating error rather than a crash.
            if body.len() > RESPONSE_BYTES_MAX {
                return Err(Fetch::Terminal(Error::response_too_large(
                    resource_id,
                    body.len(),
                )));
            }

            return Ok(body);
        }

        if matches!(status, 403 | 404) {
            return Err(Fetch::Terminal(Error::not_found(resource_id)));
        }

        if !matches!(status, 429 | 500 | 502 | 503 | 504) {
            return Err(Fetch::Terminal(Error::status_unexpected(
                resource_id,
                status,
            )));
        }

        Err(Fetch::Retriable { retry_after })
    }
}

fn fetch_transport(error: reqwest::Error) -> Fetch {
    if error.is_timeout() || error.is_connect() {
        return Fetch::Retriable { retry_after: None };
    }

    Fetch::Terminal(Error::Transport(error))
}

fn retry_after_duration(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let seconds = headers
        .get("retry-after")?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;

    Some(Duration::from_secs(seconds.min(RETRY_AFTER_S_MAX)))
}

fn backoff_duration(attempts: u32) -> Duration {
    assert!(attempts >= 1, "attempts must be positive");

    let backoff_ms = RETRY_BACKOFF_MS_BASE
        .checked_shl(attempts - 1)
        .unwrap_or(u64::MAX)
        .min(RETRY_BACKOFF_MS_MAX);

    Duration::from_millis(backoff_ms)
}
