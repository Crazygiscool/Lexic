use crate::{BackupSummary, ImportMode, ImportSummary, Store, StoreError};
use chrono::Utc;
use lexic_core::model::{Entry, ReviewLog, Sense};
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
struct BackupFile {
    format: String,
    version: u32,
    exported_at: String,
    app_version: String,
    decks: Vec<lexic_core::model::Deck>,
    entries: Vec<EntryBackup>,
    cards: Vec<lexic_core::model::Card>,
    reviews: Option<Vec<ReviewLog>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EntryBackup {
    entry: Entry,
    senses: Vec<Sense>,
}

pub fn export_backup(
    store: &Store,
    path: &Path,
    include_reviews: bool,
) -> Result<BackupSummary, StoreError> {
    let entries = store.all_entries()?;
    let mut backup_entries = Vec::new();
    for entry in entries {
        backup_entries.push(EntryBackup {
            senses: store.get_senses(entry.id)?,
            entry,
        });
    }
    let decks = store.list_decks()?;
    let cards = store.all_cards()?;
    let reviews = if include_reviews {
        Some(store.all_reviews()?)
    } else {
        None
    };
    let backup = BackupFile {
        format: String::from("lexic.backup"),
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        app_version: String::from(env!("CARGO_PKG_VERSION")),
        decks,
        entries: backup_entries,
        cards,
        reviews,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| StoreError::Internal(error.to_string()))?;
    }
    let temporary = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(&backup)
        .map_err(|error| StoreError::Internal(error.to_string()))?;
    std::fs::write(&temporary, bytes).map_err(|error| StoreError::Internal(error.to_string()))?;
    std::fs::rename(&temporary, path).map_err(|error| StoreError::Internal(error.to_string()))?;

    Ok(BackupSummary {
        decks: backup.decks.len(),
        entries: backup.entries.len(),
        cards: backup.cards.len(),
        reviews: backup
            .reviews
            .as_ref()
            .map(|value| value.len())
            .unwrap_or(0),
    })
}

pub fn import_backup(
    store: &Store,
    path: &Path,
    mode: ImportMode,
) -> Result<ImportSummary, StoreError> {
    let bytes = std::fs::read(path).map_err(|error| StoreError::Internal(error.to_string()))?;
    let backup: BackupFile =
        serde_json::from_slice(&bytes).map_err(|error| StoreError::Internal(error.to_string()))?;
    if backup.format != "lexic.backup" {
        return Err(StoreError::Internal(String::from("invalid backup format")));
    }
    if backup.version != 1 {
        return Err(StoreError::Internal(String::from(
            "unsupported backup version",
        )));
    }

    if mode == ImportMode::Replace {
        store.clear_database()?;
    }

    let mut summary = ImportSummary::default();
    for deck in &backup.decks {
        store.add_deck_record(deck)?;
        summary.decks += 1;
    }
    for record in &backup.entries {
        store.upsert_entry(&record.entry, &record.senses)?;
        summary.entries += 1;
    }
    for card in &backup.cards {
        store.add_card_record(card)?;
        summary.cards += 1;
    }
    if let Some(reviews) = &backup.reviews {
        for review in reviews {
            store.append_review(review)?;
            summary.reviews += 1;
        }
    }
    Ok(summary)
}
