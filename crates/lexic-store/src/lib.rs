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

#[derive(Debug, Clone)]
pub struct Project {
    pub id: lexic_core::model::Id,
    pub name: String,
    pub root_path: String,
    pub source_language: Option<String>,
    pub target_language: String,
    pub created_at: lexic_core::model::Timestamp,
    pub updated_at: lexic_core::model::Timestamp,
}

#[derive(Debug, Clone)]
pub struct TranspileRun {
    pub id: lexic_core::model::Id,
    pub project_id: lexic_core::model::Id,
    pub target_language: String,
    pub status: String,
    pub started_at: lexic_core::model::Timestamp,
    pub finished_at: Option<lexic_core::model::Timestamp>,
    pub exit_code: Option<i32>,
    pub diagnostic_count: i32,
    pub summary: Option<String>,
}

pub const VERSION: &str = "1.0.0";
