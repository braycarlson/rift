use std::time::Duration;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::error::Error;

const AUTHORIZE_URL_BASE: &str = "https://auth.riotgames.com/authorize";
const RSO_QUERY_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
#[doc(hidden)]
pub const SCOPE_COUNT_MAX: usize = 32;
const TIMEOUT_MS: u64 = 10_000;
const TOKEN_ENDPOINT_ID: &str = "rso.token";
const TOKEN_URL: &str = "https://auth.riotgames.com/token";

/// Tokens returned by the Riot Sign-On token endpoint.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RsoTokens {
    /// The bearer access token for RSO-authenticated endpoints.
    pub access_token: String,
    /// Lifetime of the access token in seconds.
    pub expires_in: i64,
    /// The `OpenID` Connect identity token, when the `openid` scope was granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    /// The refresh token, used to obtain a new access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// The space-delimited scopes actually granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The token type, typically `"Bearer"`.
    pub token_type: String,
}

/// A minimal client for the Riot Sign-On (RSO) OAuth token endpoints.
pub struct RsoClient {
    client: reqwest::Client,
}

impl RsoClient {
    /// Builds an RSO client with a dedicated HTTP client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the HTTP client cannot be built.
    pub fn new() -> Result<RsoClient, Error> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("rift/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .build()?;

        Ok(RsoClient { client })
    }

    /// Exchanges an authorization code for a set of [`RsoTokens`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] on a non-2xx response and [`Error::Deserialize`]
    /// if the token response cannot be parsed.
    pub async fn token_exchange(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<RsoTokens, Error> {
        assert!(!client_id.is_empty(), "client_id must not be empty");
        assert!(!code.is_empty(), "code must not be empty");
        assert!(!redirect_uri.is_empty(), "redirect_uri must not be empty");

        let form = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        self.token_request(client_id, client_secret, &form).await
    }

    /// Exchanges a refresh token for a fresh set of [`RsoTokens`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Status`] on a non-2xx response and [`Error::Deserialize`]
    /// if the token response cannot be parsed.
    pub async fn token_refresh(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<RsoTokens, Error> {
        assert!(!client_id.is_empty(), "client_id must not be empty");
        assert!(!refresh_token.is_empty(), "refresh_token must not be empty");

        let form = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        self.token_request(client_id, client_secret, &form).await
    }

    async fn token_request(
        &self,
        client_id: &str,
        client_secret: &str,
        form: &[(&str, &str)],
    ) -> Result<RsoTokens, Error> {
        assert!(!client_id.is_empty(), "client_id must not be empty");
        assert!(!form.is_empty(), "form must not be empty");

        let response = self
            .client
            .post(TOKEN_URL)
            .basic_auth(client_id, Some(client_secret))
            .form(form)
            .send()
            .await?;

        let status = response.status().as_u16();

        if !(200..300).contains(&status) {
            let body = response.bytes().await.unwrap_or_default();

            return Err(Error::status(TOKEN_ENDPOINT_ID, status, &body, None));
        }

        let body = response.bytes().await?;
        let tokens = serde_json::from_slice::<RsoTokens>(&body)
            .map_err(|source| Error::deserialize(TOKEN_ENDPOINT_ID, source))?;

        Ok(tokens)
    }
}

/// Builds the RSO authorization URL to which a user is redirected to sign in.
///
/// `scopes` are sent space-delimited; `openid` is required for an identity
/// token. The returned URL requests the authorization-code grant.
#[must_use]
pub fn authorize_url(client_id: &str, redirect_uri: &str, scopes: &[&str]) -> String {
    assert!(!client_id.is_empty(), "client_id must not be empty");
    assert!(!redirect_uri.is_empty(), "redirect_uri must not be empty");
    assert!(!scopes.is_empty(), "scopes must not be empty");
    assert!(
        scopes.len() <= SCOPE_COUNT_MAX,
        "scope count exceeds {SCOPE_COUNT_MAX}"
    );

    let scope = scopes.join(" ");

    format!(
        "{AUTHORIZE_URL_BASE}?redirect_uri={}&client_id={}&response_type=code&scope={}",
        query_encode(redirect_uri),
        query_encode(client_id),
        query_encode(&scope),
    )
}

fn query_encode(value: &str) -> String {
    utf8_percent_encode(value, RSO_QUERY_SET).to_string()
}
