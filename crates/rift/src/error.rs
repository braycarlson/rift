use std::time::Duration;

#[doc(hidden)]
pub const STATUS_BODY_BYTES_MAX: usize = 512;

/// Everything that can go wrong when calling the Riot API.
///
/// The enum is `#[non_exhaustive]`: match with a trailing `_ =>` arm so new
/// variants do not break your build. Prefer the [`Error::is_retriable`],
/// [`Error::is_not_found`], and [`Error::status_code`] helpers over matching
/// individual variants where they suffice.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A request body failed to serialize to JSON before sending.
    #[error("body serialization failed: {0}")]
    BodySerialize(#[source] serde_json::Error),
    /// A 2xx response body failed to deserialize into the expected type.
    #[error("deserialization failed for {endpoint_id}: {source}")]
    Deserialize {
        /// The operation id whose response failed to deserialize.
        endpoint_id: &'static str,
        /// The underlying serde error.
        #[source]
        source: serde_json::Error,
    },
    /// The resource was absent (HTTP 404) on a non-nullable endpoint.
    #[error("resource not found for {endpoint_id}")]
    NotFound {
        /// The operation id that returned 404.
        endpoint_id: &'static str,
    },
    /// The response body exceeded the client's size limit and was rejected.
    #[error("response for {endpoint_id} exceeds size limit: {len} bytes")]
    ResponseTooLarge {
        /// The operation id whose response was too large.
        endpoint_id: &'static str,
        /// The observed body length in bytes.
        len: usize,
    },
    /// Retries were exhausted without a success or a terminal error.
    #[error("retries exhausted for {endpoint_id} (last status {status_last})")]
    RetriesExhausted {
        /// The operation id whose retries were exhausted.
        endpoint_id: &'static str,
        /// The HTTP status seen on the final attempt, or `0` when the final
        /// attempt failed in transport before any status was received.
        status_last: u16,
    },
    /// The server returned an unexpected, non-retriable HTTP status.
    #[error("status {status} for {endpoint_id}: {body}")]
    Status {
        /// A bounded, lossy UTF-8 snippet of the response body (up to 512 bytes).
        body: String,
        /// The operation id that returned the status.
        endpoint_id: &'static str,
        /// The `Retry-After` delay, if the server supplied one.
        retry_after: Option<Duration>,
        /// The HTTP status code.
        status: u16,
    },
    /// The underlying HTTP transport failed (connect, TLS, timeout, ...).
    #[error("transport failed: {0}")]
    Transport(#[from] reqwest::Error),
}

impl Error {
    /// Whether this is a not-found (HTTP 404) error on a non-nullable endpoint.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound { .. })
    }

    /// Whether retrying the request could plausibly succeed.
    ///
    /// True for rate-limit and transient server statuses (429, 500, 502, 503,
    /// 504), for transport timeouts or connection failures, and for exhausted
    /// retries whose final attempt was itself transient (`status_last` is a
    /// retriable status, or `0` for a transport fault) -- a later, fresh
    /// request could still succeed.
    #[must_use]
    pub fn is_retriable(&self) -> bool {
        match self {
            Error::Status { status, .. } => status_retriable(*status),
            Error::Transport(source) => source.is_timeout() || source.is_connect(),
            Error::RetriesExhausted { status_last, .. } => {
                status_retriable(*status_last) || *status_last == 0
            }
            _ => false,
        }
    }

    /// The HTTP status code carried by this error, if any.
    #[must_use]
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Error::Status { status, .. } => Some(*status),
            Error::RetriesExhausted { status_last, .. } => Some(*status_last),
            _ => None,
        }
    }

    /// The parsed Riot error message from a [`Error::Status`] body, if present.
    ///
    /// Riot returns `{"status":{"message":"...","status_code":...}}` on error;
    /// this best-effort parse extracts the message and returns `None` otherwise.
    #[must_use]
    pub fn status_message(&self) -> Option<String> {
        let Error::Status { body, .. } = self else {
            return None;
        };

        riot_status_message(body)
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn body_serialize(source: serde_json::Error) -> Error {
        Error::BodySerialize(source)
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn deserialize(endpoint_id: &'static str, source: serde_json::Error) -> Error {
        Error::Deserialize {
            endpoint_id,
            source,
        }
    }

    #[doc(hidden)]
    #[must_use]
    #[cold]
    #[inline(never)]
    pub fn not_found(endpoint_id: &'static str) -> Error {
        Error::NotFound { endpoint_id }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn response_too_large(endpoint_id: &'static str, len: usize) -> Error {
        Error::ResponseTooLarge { endpoint_id, len }
    }

    #[doc(hidden)]
    #[must_use]
    #[cold]
    #[inline(never)]
    pub fn retries_exhausted(endpoint_id: &'static str, status_last: u16) -> Error {
        Error::RetriesExhausted {
            endpoint_id,
            status_last,
        }
    }

    #[doc(hidden)]
    #[must_use]
    #[cold]
    #[inline(never)]
    pub fn status(
        endpoint_id: &'static str,
        status: u16,
        body_raw: &[u8],
        retry_after: Option<Duration>,
    ) -> Error {
        assert!(!endpoint_id.is_empty(), "endpoint id must not be empty");
        assert!(
            (100..600).contains(&status),
            "status must be a valid HTTP status: {status}"
        );

        let body = body_snippet(body_raw);

        Error::Status {
            body,
            endpoint_id,
            retry_after,
            status,
        }
    }
}

fn body_snippet(body_raw: &[u8]) -> String {
    let end = body_raw.len().min(STATUS_BODY_BYTES_MAX);
    let mut snippet = String::from_utf8_lossy(&body_raw[..end]).trim().to_string();
    let mut cut = STATUS_BODY_BYTES_MAX.min(snippet.len());
    let mut iterations: usize = 0;

    while !snippet.is_char_boundary(cut) {
        iterations += 1;

        assert!(iterations <= 4, "char boundary search must be bounded");

        cut -= 1;
    }

    snippet.truncate(cut);

    assert!(
        snippet.len() <= STATUS_BODY_BYTES_MAX,
        "status body snippet must not exceed {STATUS_BODY_BYTES_MAX} bytes"
    );

    snippet
}

fn riot_status_message(body: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Envelope {
        status: Inner,
    }

    #[derive(serde::Deserialize)]
    struct Inner {
        message: String,
    }

    let envelope = serde_json::from_str::<Envelope>(body).ok()?;

    Some(envelope.status.message)
}

/// HTTP statuses worth retrying: rate limit (429) and transient server errors.
///
/// Shared by [`Error::is_retriable`] and the client's request loop so the two
/// cannot drift apart.
#[doc(hidden)]
#[must_use]
pub fn status_retriable(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}
