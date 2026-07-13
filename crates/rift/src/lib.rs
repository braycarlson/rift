//! rift is a lean, generated, asynchronous client for the Riot Games API.
//!
//! The bulk of the surface, one method per Riot endpoint, is generated from the
//! official `OpenAPI` specification and hangs off [`RiotApi`]. Around that sit a
//! header-learned rate limiter, linear-with-jitter retries, a concurrency cap,
//! observe-only [`Hooks`], and optional response caching.
//!
//! # Quickstart
//!
//! ```no_run
//! use rift::routes::RegionalRoute;
//! use rift::{RiotApi, RiotApiConfig};
//!
//! # async fn run() -> Result<(), rift::Error> {
//! let api = RiotApi::new(RiotApiConfig::from_env().expect("RGAPI_KEY set"))?;
//!
//! let account = api
//!     .account_v1_get_by_riot_id(RegionalRoute::Americas, "Doublelift", "NA1")
//!     .await?;
//!
//! println!("{account:#?}");
//! # Ok(())
//! # }
//! ```
//!
//! # Feature flags
//!
//! - `rustls-tls` (default): use the rustls TLS backend.
//! - `native-tls`: use the platform native TLS backend instead.
//! - `tracing`: emit `tracing` events on the request path.
//! - `cache`: in-memory response cache for cacheable `GET`s.
//! - `rso`: Riot Sign-On OAuth URL and token helpers.
//! - `live-client`: the local Live Client Data API with a pinned certificate.
#![warn(missing_docs)]

pub mod generated;

#[cfg(feature = "cache")]
#[doc(hidden)]
pub mod cache;
#[doc(hidden)]
pub mod client;
mod config;
#[doc(hidden)]
pub mod error;
mod hooks;
/// The local Live Client Data API served by a running League client.
#[cfg(feature = "live-client")]
pub mod live_client;
#[doc(hidden)]
pub mod pagination;
#[doc(hidden)]
pub mod rate_limit;
#[doc(hidden)]
pub mod raw;
/// Riot Sign-On (RSO) OAuth authorization and token helpers.
#[cfg(feature = "rso")]
pub mod rso;

pub use bytes::Bytes;
#[cfg(feature = "cache")]
pub use cache::CacheConfig;
pub use client::RiotApi;
pub use config::RiotApiConfig;
pub use error::Error;
pub use generated::consts;
pub use generated::endpoints;
pub use generated::meta;
pub use generated::models;
pub use generated::routes;
pub use hooks::{Hooks, RequestInfo};
pub use rate_limit::{LimiterSettings, RateLimiter};
