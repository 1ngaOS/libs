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
}
