#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("deserialization failed for {resource_id}: {source}")]
    Deserialize {
        resource_id: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("resource not found for {resource_id}")]
    NotFound { resource_id: &'static str },
    #[error("response for {resource_id} exceeds size limit: {len} bytes")]
    ResponseTooLarge {
        resource_id: &'static str,
        len: usize,
    },
    #[error("retries exhausted for {resource_id}")]
    RetriesExhausted { resource_id: &'static str },
    #[error("status {status} for {resource_id}")]
    Status {
        resource_id: &'static str,
        status: u16,
    },
    #[error("transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("version list is empty")]
    VersionsEmpty,
}

impl Error {
    #[cold]
    #[inline(never)]
    pub(crate) fn deserialize(resource_id: &'static str, source: serde_json::Error) -> Error {
        Error::Deserialize {
            resource_id,
            source,
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn not_found(resource_id: &'static str) -> Error {
        Error::NotFound { resource_id }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn response_too_large(resource_id: &'static str, len: usize) -> Error {
        Error::ResponseTooLarge { resource_id, len }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn retries_exhausted(resource_id: &'static str) -> Error {
        Error::RetriesExhausted { resource_id }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn status_unexpected(resource_id: &'static str, status: u16) -> Error {
        Error::Status {
            resource_id,
            status,
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn versions_empty() -> Error {
        Error::VersionsEmpty
    }
}
