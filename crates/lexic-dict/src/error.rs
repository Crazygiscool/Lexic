use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DictError {
    #[error("search query cannot be empty")]
    EmptyQuery,
    #[error("search query is too long")]
    TooLong,
    #[error("search query contains control characters")]
    InvalidCharacter,
    #[error("no definition found")]
    NotFound,
    #[error("dictionary rate limit reached")]
    RateLimited,
    #[error("dictionary returned HTTP status {0}")]
    Http(u16),
    #[error("network error: {0}")]
    Network(String),
    #[error("invalid dictionary response: {0}")]
    InvalidResponse(String),
}
