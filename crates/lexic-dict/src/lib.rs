pub mod cache;
pub mod client;
pub mod error;
pub mod normalize;

pub use cache::is_fresh;
pub use client::{DictionaryClient, LookupResult};
pub use error::DictError;
pub use normalize::normalize;

pub const VERSION: &str = "0.1.0";
