mod backup;
mod db;
mod error;

pub use backup::{export_backup, import_backup};
pub use db::Store;
pub use error::StoreError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageStats {
    pub sqlite_version: String,
    pub journal_mode: String,
    pub foreign_keys: bool,
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub entry_count: i64,
    pub sense_count: i64,
    pub deck_count: i64,
    pub card_count: i64,
    pub review_count: i64,
    pub due_count: i64,
    pub integrity: String,
}

#[derive(Debug, Clone)]
pub struct DueCardPreview {
    pub card: lexic_core::model::Card,
    pub word: String,
    pub deck_name: String,
}

pub const VERSION: &str = "1.0.0";
