use thiserror::Error;

/// Result type used throughout this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur when working with PayFast ITN data.
#[derive(Debug, Error)]
pub enum Error {
    /// The raw ITN body could not be parsed as
    /// `application/x-www-form-urlencoded`.
    #[error("failed to parse ITN body as form data: {0}")]
    ParseBody(#[from] serde_urlencoded::de::Error),

    /// The ITN payload did not contain a `signature` field.
    #[error("missing `signature` field in ITN payload")]
    MissingSignature,

    /// HTTP error while calling PayFast (API / post-back validation).
    #[cfg(feature = "api")]
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error.
    #[cfg(feature = "api")]
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Non-2xx response from PayFast API.
    #[cfg(feature = "api")]
    #[error("api http error: status={status} body={body}")]
    ApiHttp {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },

    /// Other error.
    #[error("{0}")]
    Other(String),
}
