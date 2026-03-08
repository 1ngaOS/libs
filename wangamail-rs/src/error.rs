//! Error types for wangamail-rs.

use thiserror::Error;

/// Errors that can occur when using the Graph mail client or MCP server.
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP or network failure (e.g. reqwest error).
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// OAuth2 or token endpoint error (invalid credentials, bad response, etc.).
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Microsoft Graph API error (e.g. sendMail returned non-202 or error body).
    #[error("Graph API error: {0}")]
    Graph(String),

    /// Invalid configuration (missing required field, poisoned mutex, etc.).
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Result type alias using this crate’s [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
