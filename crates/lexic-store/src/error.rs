use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database lock poisoned: {0}")]
    Internal(String),
    #[error("database migration failed: {0}")]
    Migration(String),
    #[error("not found")]
    NotFound,
    #[error("constraint error: {0}")]
    Constraint(String),
}
