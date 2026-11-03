use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("invalid rating value {0}")]
    InvalidRating(i32),
    #[error("invalid review state `{0}`")]
    InvalidState(String),
    #[error("empty search query")]
    EmptyQuery,
}
