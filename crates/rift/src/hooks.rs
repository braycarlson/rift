use std::time::Duration;

/// Immutable description of a request, passed to [`Hooks`] callbacks.
///
/// Deliberately omits the request path. Paths embed PUUIDs and other player
/// identifiers, so exposing them to observers would leak those identifiers into
/// logs and metrics. Only the stable operation id, HTTP method, and routing
/// value are exposed.
#[derive(Clone, Copy, Debug)]
pub struct RequestInfo<'a> {
    /// The Riot operation id, e.g. `"summoner-v4.getByPUUID"`.
    pub endpoint_id: &'a str,
    /// The HTTP method, e.g. `"GET"`.
    pub method: &'a str,
    /// The routing value token, e.g. `"na1"`.
    pub route: &'a str,
}

/// Observe-only lifecycle callbacks for a [`crate::RiotApi`] client.
///
/// Every method has an empty default, so an implementor overrides only the
/// events it cares about. Hooks observe; they cannot alter the request, the
/// response, or the retry decision. This is deliberately not a middleware
/// chain: there is exactly one hooks object per client and it never mutates.
///
/// Implementations must be cheap and non-blocking: they run inline on the
/// request path. Offload real work to a channel or background task.
pub trait Hooks: Send + Sync {
    /// Called immediately before a request attempt is sent.
    fn on_request(&self, info: &RequestInfo<'_>) {
        let _ = info;
    }

    /// Called after a response is received, with its status and elapsed time.
    fn on_response(&self, info: &RequestInfo<'_>, status: u16, elapsed: Duration) {
        let _ = (info, status, elapsed);
    }

    /// Called before sleeping between retry attempts.
    fn on_retry(&self, info: &RequestInfo<'_>, attempt: u32, wait: Duration) {
        let _ = (info, attempt, wait);
    }

    /// Called before sleeping to respect a rate-limit window.
    ///
    /// `scope` is `"app"` for the per-route application limit, or the operation
    /// id for a per-method limit.
    fn on_rate_limit_sleep(&self, route: &str, scope: &str, wait: Duration) {
        let _ = (route, scope, wait);
    }
}
