mod backup;
mod db;
mod error;

pub use backup::{export_backup, import_backup};
pub use db::Store;
pub use error::StoreError;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ImportMode {
    Merge,
    #[default]
    Replace,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackupSummary {
    pub decks: usize,
    pub entries: usize,
    pub cards: usize,
    pub reviews: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub decks: usize,
    pub entries: usize,
    pub cards: usize,
    pub reviews: usize,
}

pub const VERSION: &str = "0.1.0";
