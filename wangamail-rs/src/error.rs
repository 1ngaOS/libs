use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Graph API error: {0}")]
    Graph(String),

    #[error("Invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
