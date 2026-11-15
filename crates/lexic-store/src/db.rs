#![allow(clippy::collapsible_if)]

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use lexic_core::model::*;
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;

const MIGRATIONS: &[(&str, usize)] = &[
    (include_str!("../migrations/0001_init.sql"), 1),
    (
        include_str!("../migrations/0002_fts_contentless_delete.sql"),
        2,
    ),
    (include_str!("../migrations/0004_drop_transpilation.sql"), 4),
    (include_str!("../migrations/0005_humanities.sql"), 5),
    (include_str!("../migrations/0006_pending_imports.sql"), 6),
];

#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| StoreError::Internal(error.to_string()))?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", 1)?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        for (sql, expected_version) in MIGRATIONS {
            let sql = *sql;
            let expected_version = *expected_version as i64;
            let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            if expected_version > current {
                conn.execute_batch(sql)
                    .map_err(|error| StoreError::Migration(error.to_string()))?;
                conn.pragma_update(None, "user_version", expected_version)?;
            }
        }
        seed_database(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", 1)?;
        for (sql, expected_version) in MIGRATIONS {
            conn.execute_batch(sql)
                .map_err(|error| StoreError::Migration(error.to_string()))?;
            conn.pragma_update(None, "user_version", *expected_version as i64)?;
        }
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn
            .lock()
            .map_err(|error| StoreError::Internal(error.to_string()))
    }

    pub fn upsert_entry(&self, entry: &Entry, senses: &[Sense]) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let transaction = conn.transaction()?;
        transaction.execute(
            "INSERT INTO entries (id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, fetched_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, ?11)
             ON CONFLICT(normalized_word, source) DO UPDATE SET
                word=excluded.word,
                phonetic=excluded.phonetic,
                audio_url=excluded.audio_url,
                origin=excluded.origin,
                source_url=excluded.source_url,
                license=excluded.license,
                updated_at=excluded.updated_at",
            params![
                entry.id.to_string(),
                entry.word,
                entry.normalized_word,
                entry.phonetic,
                entry.audio_url,
                entry.origin,
                entry.source,
                entry.source_url,
                entry.license,
                entry.created_at.timestamp(),
                entry.updated_at.timestamp()
            ],
        )?;
        let entry_row: String = transaction.query_row(
            "SELECT id FROM entries WHERE normalized_word = ?1 AND source = ?2",
            params![entry.normalized_word, entry.source],
            |row| row.get(0),
        )?;
        transaction.execute("DELETE FROM senses WHERE entry_id = ?1", params![entry_row])?;
        for sense in senses {
            transaction.execute(
                "INSERT INTO senses (id, entry_id, part_of_speech, definition, example, synonyms_json, antonyms_json, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    sense.id.to_string(),
                    entry_row,
                    sense.part_of_speech,
                    sense.definition,
                    sense.example,
                    serde_json::to_string(&sense.synonyms).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&sense.antonyms).unwrap_or_else(|_| "[]".to_string()),
                    sense.sort_order
                ],
            )?;
        }
        transaction.execute(
            "DELETE FROM entries_fts WHERE rowid = (SELECT rowid FROM entries WHERE id = ?1)",
            params![entry_row],
        )?;
        let mut definitions = String::new();
        for sense in senses {
            if !definitions.is_empty() {
                definitions.push(' ');
            }
            definitions.push_str(&sense.definition);
        }
        transaction.execute(
            "INSERT INTO entries_fts (rowid, word, definitions) VALUES ((SELECT rowid FROM entries WHERE id = ?1), ?2, ?3)",
            params![entry_row, entry.word, definitions],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn get_entry(&self, id: Id) -> Result<Option<Entry>, StoreError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, created_at, updated_at
             FROM entries WHERE id = ?1",
            params![id.to_string()],
            map_entry_row,
        ).optional()?;
        Ok(result)
    }

    pub fn get_senses(&self, entry_id: Id) -> Result<Vec<Sense>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, entry_id, part_of_speech, definition, example, synonyms_json, antonyms_json, sort_order
             FROM senses WHERE entry_id = ?1 ORDER BY sort_order",
        )?;
        let rows = statement.query_map(params![entry_id.to_string()], map_sense_row)?;
        let mut senses = Vec::new();
        for row in rows {
            senses.push(row?);
        }
        Ok(senses)
    }

    pub fn search_entries(&self, query: &str, limit: usize) -> Result<Vec<Entry>, StoreError> {
        let normalized = query.trim().to_lowercase();
        if normalized.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.lock()?;
        let mut result: Vec<Entry> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let exact = conn.query_row(
            "SELECT id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, created_at, updated_at
             FROM entries WHERE normalized_word = ?1 LIMIT 1",
            params![normalized],
            map_entry_row,
        ).optional()?;
        if let Some(entry) = exact {
            seen.insert(entry.id);
            result.push(entry);
        }

        let fts_query = format!("{}*", normalized);
        let mut statement = conn.prepare(
            "SELECT e.id, e.word, e.normalized_word, e.phonetic, e.audio_url, e.origin, e.source, e.source_url, e.license, e.created_at, e.updated_at
             FROM entries e JOIN entries_fts f ON f.rowid = e.rowid
             WHERE entries_fts MATCH ?1
             LIMIT ?2",
        )?;
        let mut rows = statement.query(params![fts_query, limit as i64])?;
        while let Some(row) = rows.next()? {
            let entry = map_entry_row(row)?;
            if seen.insert(entry.id) {
                result.push(entry);
            }
        }

        if result.len() < limit {
            let pattern = format!("{}%", normalized);
            let mut statement = conn.prepare(
                "SELECT id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, created_at, updated_at
                 FROM entries WHERE normalized_word LIKE ?1 ORDER BY normalized_word LIMIT ?2",
            )?;
            let mut rows = statement.query(params![pattern, (limit - result.len()) as i64])?;
            while let Some(row) = rows.next()? {
                let entry = map_entry_row(row)?;
                if seen.insert(entry.id) {
                    result.push(entry);
                }
            }
        }

        result.truncate(limit);
        Ok(result)
    }

    pub fn create_deck(&self, name: &str, description: Option<&str>) -> Result<Id, StoreError> {
        let id = unused_uuid();
        let now = Utc::now();
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO decks (id, name, description, new_per_day, reviews_per_day, created_at, updated_at, archived_at)
             VALUES (?1, ?2, ?3, NULL, NULL, ?4, ?5, NULL)",
            params![id.to_string(), name, description, now.timestamp(), now.timestamp()],
        )?;
        Ok(id)
    }

    pub fn list_decks(&self) -> Result<Vec<Deck>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, name, description, new_per_day, reviews_per_day, created_at, updated_at, archived_at
             FROM decks ORDER BY name COLLATE NOCASE",
        )?;
        let rows = statement.query_map([], map_deck_row)?;
        let mut decks = Vec::new();
        for row in rows {
            decks.push(row?);
        }
        Ok(decks)
    }

    pub fn add_card(&self, entry_id: Id, deck_id: Id) -> Result<Id, StoreError> {
        let id = unused_uuid();
        let now = Utc::now();
        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT OR IGNORE INTO cards (id, entry_id, deck_id, direction, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'recognition', 'new', ?4, ?5)",
            params![id.to_string(), entry_id.to_string(), deck_id.to_string(), now.timestamp(), now.timestamp()],
        )?;
        if inserted == 0 {
            let existing: String = conn.query_row(
                "SELECT id FROM cards WHERE entry_id = ?1 AND deck_id = ?2 AND direction = 'recognition'",
                params![entry_id.to_string(), deck_id.to_string()],
                |row| row.get(0),
            )?;
            return existing
                .parse()
                .map_err(|_| StoreError::Internal(existing.clone()));
        }
        Ok(id)
    }

    pub fn list_cards_by_deck(&self, deck_id: Id) -> Result<Vec<Card>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, entry_id, deck_id, direction, front_note, back_note, state, due_at, interval_days, ease_factor, repetitions, lapses, first_reviewed_at, last_reviewed_at, created_at, updated_at
             FROM cards WHERE deck_id = ?1",
        )?;
        let rows = statement.query_map(params![deck_id.to_string()], map_card_row)?;
        let mut cards = Vec::new();
        for row in rows {
            cards.push(row?);
        }
        Ok(cards)
    }

    pub fn list_cards_due(&self, now: DateTime<Utc>) -> Result<Vec<Card>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, entry_id, deck_id, direction, front_note, back_note, state, due_at, interval_days, ease_factor, repetitions, lapses, first_reviewed_at, last_reviewed_at, created_at, updated_at
             FROM cards
             WHERE state IN ('new', 'learning', 'relearning', 'review')
             ORDER BY due_at IS NULL, due_at",
        )?;
        let rows = statement.query_map(params![], map_card_row)?;
        let mut cards = Vec::new();
        for row in rows {
            let card = row?;
            if card.state == CardState::New || card.due_at.map(|due| due <= now).unwrap_or(true) {
                cards.push(card);
            }
        }
        let _ = now;
        Ok(cards)
    }

    pub fn list_cards_due_batch(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<Card>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, entry_id, deck_id, direction, front_note, back_note, state, due_at, interval_days, ease_factor, repetitions, lapses, first_reviewed_at, last_reviewed_at, created_at, updated_at
             FROM cards
             WHERE state IN ('new', 'learning', 'relearning', 'review')
               AND (state = 'new' OR due_at IS NULL OR due_at <= ?1)
             ORDER BY
               CASE WHEN state = 'new' THEN 0 ELSE 1 END,
               due_at IS NULL,
               due_at,
               CASE WHEN state = 'new' THEN 0 ELSE ease_factor END ASC,
               CASE WHEN state = 'new' THEN 0 ELSE lapses END DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![now.timestamp(), limit as i64], map_card_row)?;
        let mut cards = Vec::new();
        for row in rows {
            let card = row?;
            cards.push(card);
        }
        Ok(cards)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_card_after_review(
        &self,
        card_id: Id,
        outcome_state: CardState,
        due_at: DateTime<Utc>,
        interval_days: f64,
        ease_factor: f64,
        repetitions: i32,
        lapses: i32,
    ) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE cards SET state = ?1, due_at = ?2, interval_days = ?3, ease_factor = ?4, repetitions = ?5, lapses = ?6,
             last_reviewed_at = ?7, first_reviewed_at = COALESCE(first_reviewed_at, ?7), updated_at = ?7
             WHERE id = ?8",
            params![
                card_state_string(outcome_state),
                due_at.timestamp(),
                interval_days,
                ease_factor,
                repetitions,
                lapses,
                Utc::now().timestamp(),
                card_id.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn append_review(&self, review: &ReviewLog) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO reviews (id, card_id, reviewed_at, day, rating, prev_state, prev_interval_days, prev_ease, new_state, new_interval_days, new_ease, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                review.id.to_string(),
                review.card_id.to_string(),
                review.reviewed_at.timestamp(),
                review.day,
                match review.rating {
                    Rating::Again => 1,
                    Rating::Hard => 2,
                    Rating::Good => 3,
                    Rating::Easy => 4,
                },
                card_state_string(review.prev_state),
                review.prev_interval_days,
                review.prev_ease,
                card_state_string(review.new_state),
                review.new_interval_days,
                review.new_ease,
                review.duration_ms
            ],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        let conn = self.lock()?;
        let value = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn review_stats(&self) -> Result<(i64, i64), StoreError> {
        let conn = self.lock()?;
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM reviews", [], |row| row.get(0))?;
        let successful: i64 = conn.query_row(
            "SELECT COUNT(*) FROM reviews WHERE rating >= 3",
            [],
            |row| row.get(0),
        )?;
        Ok((successful, total))
    }

    pub fn rename_deck(&self, id: Id, name: &str) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE decks SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, Utc::now().timestamp(), id.to_string()],
        )?;
        Ok(())
    }

    pub fn update_deck_limits(
        &self,
        id: Id,
        new_per_day: Option<i32>,
        reviews_per_day: Option<i32>,
    ) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
        "UPDATE decks SET new_per_day = ?1, reviews_per_day = ?2, updated_at = ?3 WHERE id = ?4",
        params![
            new_per_day,
            reviews_per_day,
            Utc::now().timestamp(),
            id.to_string()
        ],
    )?;
        Ok(())
    }

    pub fn archive_deck(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE decks SET archived_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp(), id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_deck(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM decks WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn deck_counts(
        &self,
        deck_id: Id,
        now: DateTime<Utc>,
    ) -> Result<(i64, i64, i64), StoreError> {
        let conn = self.lock()?;
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1",
            params![deck_id.to_string()],
            |row| row.get(0),
        )?;
        let new_cards: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND state = 'new'",
            params![deck_id.to_string()],
            |row| row.get(0),
        )?;
        let due: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND state IN ('learning','review','relearning') AND due_at <= ?2",
            params![deck_id.to_string(), now.timestamp()],
            |row| row.get(0),
        )?;
        Ok((total, new_cards, due))
    }

    pub fn storage_stats(&self, now: DateTime<Utc>) -> Result<crate::StorageStats, StoreError> {
        let conn = self.lock()?;
        let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |r| r.get(0))?;
        let entry_count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
        let sense_count: i64 = conn.query_row("SELECT COUNT(*) FROM senses", [], |r| r.get(0))?;
        let deck_count: i64 = conn.query_row("SELECT COUNT(*) FROM decks", [], |r| r.get(0))?;
        let card_count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))?;
        let review_count: i64 = conn.query_row("SELECT COUNT(*) FROM reviews", [], |r| r.get(0))?;
        let due_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE state IN ('learning','review','relearning') AND due_at <= ?1",
            params![now.timestamp()],
            |r| r.get(0),
        )?;
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap_or_else(|_| String::from("unknown"));
        Ok(crate::StorageStats {
            sqlite_version: rusqlite::version().to_string(),
            journal_mode,
            foreign_keys: foreign_keys != 0,
            page_size,
            page_count,
            freelist_count,
            entry_count,
            sense_count,
            deck_count,
            card_count,
            review_count,
            due_count,
            integrity,
        })
    }

    pub fn vacuum(&self) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    pub fn remove_card(&self, card_id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM cards WHERE id = ?1",
            params![card_id.to_string()],
        )?;
        Ok(())
    }

    pub fn suspend_card(&self, card_id: Id, suspended: bool) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let state = if suspended { "suspended" } else { "new" };
        conn.execute(
            "UPDATE cards SET state = ?1 WHERE id = ?2",
            params![state, card_id.to_string()],
        )?;
        Ok(())
    }

    pub fn reset_card(&self, card_id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE cards SET state = 'new', due_at = NULL, interval_days = 0, ease_factor = 2.5, repetitions = 0, lapses = 0 WHERE id = ?1",
            params![card_id.to_string()],
        )?;
        Ok(())
    }
    pub fn restore_card(&self, card: &Card) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE cards SET entry_id = ?1, deck_id = ?2, direction = ?3, front_note = ?4, back_note = ?5, state = ?6, due_at = ?7, interval_days = ?8, ease_factor = ?9, repetitions = ?10, lapses = ?11, first_reviewed_at = ?12, last_reviewed_at = ?13, created_at = ?14, updated_at = ?15 WHERE id = ?16",
            params![
                card.entry_id.to_string(),
                card.deck_id.to_string(),
                card.direction,
                card.front_note,
                card.back_note,
                card.state.to_string(),
                card.due_at.map(|value| value.timestamp()),
                card.interval_days,
                card.ease_factor,
                card.repetitions,
                card.lapses,
                card.first_reviewed_at.map(|value| value.timestamp()),
                card.last_reviewed_at.map(|value| value.timestamp()),
                card.created_at.timestamp(),
                card.updated_at.timestamp(),
                card.id.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn delete_review(&self, review_id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM reviews WHERE id = ?1",
            params![review_id.to_string()],
        )?;
        Ok(())
    }

    pub fn review_history_rows(
        &self,
        since_day: &str,
    ) -> Result<Vec<(String, i64, Option<i64>)>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare("SELECT day, rating, duration_ms FROM reviews WHERE day >= ?1 ORDER BY day")?;
        let rows = statement.query_map(params![since_day], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })?;
        let mut values = Vec::new();
        for row in rows {
            values.push(row?);
        }
        Ok(values)
    }

    pub fn due_forecast_counts(
        &self,
        horizon_days: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT due_at FROM cards WHERE state IN ('learning','review','relearning') AND due_at IS NOT NULL",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, i64>(0))?;
        use std::collections::BTreeMap;
        let mut buckets: BTreeMap<String, i64> = BTreeMap::new();
        for row in rows {
            let due = DateTime::from_timestamp(row?, 0).unwrap_or_else(Utc::now);
            let local_day = due
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d")
                .to_string();
            if due <= now + chrono::Duration::days(horizon_days) {
                *buckets.entry(local_day).or_insert(0) += 1;
            }
        }
        Ok(buckets.into_iter().collect())
    }

    pub fn all_entries(&self) -> Result<Vec<Entry>, StoreError> {
        let conn = self.lock()?;
        // get ids then map_entry_row struct usage not borrowed across? Use query list
        let mut statement = conn.prepare(
            "SELECT id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, created_at, updated_at FROM entries",
        )?;
        let mut rows = statement.query([])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            entries.push(map_entry_row(row)?);
        }
        Ok(entries)
    }

    pub fn random_entries(&self, limit: usize) -> Result<Vec<Entry>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, word, normalized_word, phonetic, audio_url, origin, source, source_url, license, created_at, updated_at FROM entries ORDER BY RANDOM() LIMIT ?1",
        )?;
        let mut rows = statement.query([limit as i64])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            entries.push(map_entry_row(row)?);
        }
        Ok(entries)
    }

    pub fn due_cards_preview(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<crate::DueCardPreview>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT c.id, c.entry_id, c.deck_id, c.direction, c.front_note, c.back_note, c.state, c.due_at, c.interval_days, c.ease_factor, c.repetitions, c.lapses, c.first_reviewed_at, c.last_reviewed_at, c.created_at, c.updated_at, e.word, d.name FROM cards c JOIN entries e ON e.id = c.entry_id JOIN decks d ON d.id = c.deck_id WHERE c.state IN ('new','learning','relearning','review') AND (c.state = 'new' OR c.due_at IS NULL OR c.due_at <= ?1) ORDER BY CASE WHEN c.state = 'new' THEN 1 ELSE 0 END, c.due_at IS NULL, c.due_at LIMIT ?2",
        )?;
        let rows = statement.query_map(params![now.timestamp(), limit as i64], |row| {
            let card = map_card_row(row)?;
            let word: String = row.get(16)?;
            let deck_name: String = row.get(17)?;
            Ok(crate::DueCardPreview {
                card,
                word,
                deck_name,
            })
        })?;
        let mut values = Vec::new();
        for row in rows {
            values.push(row?);
        }
        Ok(values)
    }

    pub fn all_cards(&self) -> Result<Vec<Card>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, entry_id, deck_id, direction, front_note, back_note, state, due_at, interval_days, ease_factor, repetitions, lapses, first_reviewed_at, last_reviewed_at, created_at, updated_at FROM cards",
        )?;
        let mut rows = statement.query([])?;
        let mut cards = Vec::new();
        while let Some(row) = rows.next()? {
            cards.push(map_card_row(row)?);
        }
        Ok(cards)
    }

    pub fn all_reviews(&self) -> Result<Vec<ReviewLog>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, card_id, reviewed_at, day, rating, prev_state, prev_interval_days, prev_ease, new_state, new_interval_days, new_ease, duration_ms FROM reviews",
        )?;
        let rows = statement.query_map([], map_review_row)?;
        let mut reviews = Vec::new();
        for row in rows {
            reviews.push(row?);
        }
        Ok(reviews)
    }
    pub fn clear_database(&self) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let transaction = conn.transaction()?;
        transaction.execute("DELETE FROM reviews", [])?;
        transaction.execute("DELETE FROM cards", [])?;
        transaction.execute("DELETE FROM senses", [])?;
        transaction.execute("DELETE FROM entries", [])?;
        transaction.execute("DELETE FROM decks", [])?;
        transaction.execute("DELETE FROM entries_fts", [])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn add_deck_record(&self, deck: &Deck) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT OR IGNORE INTO decks (id, name, description, new_per_day, reviews_per_day, created_at, updated_at, archived_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![deck.id.to_string(), deck.name, deck.description, deck.new_per_day, deck.reviews_per_day, deck.created_at.timestamp(), deck.updated_at.timestamp(), deck.archived_at.map(|value| value.timestamp())],
        )?;
        Ok(())
    }

    pub fn add_card_record(&self, card: &Card) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT OR REPLACE INTO cards (id, entry_id, deck_id, direction, front_note, back_note, state, due_at, interval_days, ease_factor, repetitions, lapses, first_reviewed_at, last_reviewed_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                card.id.to_string(),
                card.entry_id.to_string(),
                card.deck_id.to_string(),
                card.direction,
                card.front_note,
                card.back_note,
                card.state.to_string(),
                card.due_at.map(|value| value.timestamp()),
                card.interval_days,
                card.ease_factor,
                card.repetitions,
                card.lapses,
                card.first_reviewed_at.map(|value| value.timestamp()),
                card.last_reviewed_at.map(|value| value.timestamp()),
                card.created_at.timestamp(),
                card.updated_at.timestamp()
            ],
        )?;
        Ok(())
    }
    // === Notes ===

    pub fn create_note(&self, note: &Note) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO notes (id, title, content, subject, tags, source_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                note.id.to_string(),
                note.title,
                note.content,
                note.subject,
                serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string()),
                note.source_url,
                note.created_at.timestamp(),
                note.updated_at.timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn update_note(&self, note: &Note) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, subject = ?3, tags = ?4, source_url = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                note.title,
                note.content,
                note.subject,
                serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string()),
                note.source_url,
                note.updated_at.timestamp(),
                note.id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_note(&self, id: Id) -> Result<Option<Note>, StoreError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT id, title, content, subject, tags, source_url, created_at, updated_at FROM notes WHERE id = ?1",
            params![id.to_string()],
            map_note_row,
        ).optional()?;
        Ok(result)
    }

    pub fn list_notes(
        &self,
        subject: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Note>, StoreError> {
        let conn = self.lock()?;
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match subject {
            Some(s) if !s.is_empty() => match limit {
                Some(l) => (
                    "SELECT id, title, content, subject, tags, source_url, created_at, updated_at FROM notes WHERE subject = ?1 ORDER BY updated_at DESC LIMIT ?2".to_string(),
                    vec![Box::new(s.to_string()) as Box<dyn rusqlite::ToSql>, Box::new(l as i64) as Box<dyn rusqlite::ToSql>],
                ),
                None => (
                    "SELECT id, title, content, subject, tags, source_url, created_at, updated_at FROM notes WHERE subject = ?1 ORDER BY updated_at DESC".to_string(),
                    vec![Box::new(s.to_string()) as Box<dyn rusqlite::ToSql>],
                ),
            },
            _ => match limit {
                Some(l) => (
                    "SELECT id, title, content, subject, tags, source_url, created_at, updated_at FROM notes ORDER BY updated_at DESC LIMIT ?1".to_string(),
                    vec![Box::new(l as i64) as Box<dyn rusqlite::ToSql>],
                ),
                None => (
                    "SELECT id, title, content, subject, tags, source_url, created_at, updated_at FROM notes ORDER BY updated_at DESC".to_string(),
                    vec![],
                ),
            },
        };
        let mut statement = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = statement.query_map(params_refs.as_slice(), map_note_row)?;
        let mut notes = Vec::new();
        for row in rows {
            notes.push(row?);
        }
        Ok(notes)
    }

    pub fn delete_note(&self, id: Id) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM note_links WHERE source_note_id = ?1 OR target_note_id = ?1",
            params![id.to_string()],
        )?;
        tx.execute(
            "UPDATE timeline_events SET note_id = NULL WHERE note_id = ?1",
            params![id.to_string()],
        )?;
        tx.execute("DELETE FROM notes WHERE id = ?1", params![id.to_string()])?;
        tx.commit()?;
        Ok(())
    }

    pub fn search_notes(&self, query: &str, limit: usize) -> Result<Vec<Note>, StoreError> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.lock()?;
        let pattern = format!("%{}%", normalized);
        let mut statement = conn.prepare(
            "SELECT id, title, content, subject, tags, source_url, created_at, updated_at
             FROM notes
             WHERE title LIKE ?1 OR content LIKE ?1 OR subject LIKE ?1
             ORDER BY updated_at DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![pattern, limit as i64], map_note_row)?;
        let mut notes = Vec::new();
        for row in rows {
            notes.push(row?);
        }
        Ok(notes)
    }

    pub fn list_subjects(&self) -> Result<Vec<String>, StoreError> {
        let conn = self.lock()?;
        let mut statement =
            conn.prepare("SELECT DISTINCT subject FROM notes ORDER BY subject COLLATE NOCASE")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut subjects = Vec::new();
        for row in rows {
            subjects.push(row?);
        }
        Ok(subjects)
    }

    // === Timeline Events ===

    pub fn create_timeline_event(&self, event: &TimelineEvent) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO timeline_events (id, title, description, start_date, end_date, subject, note_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id.to_string(),
                event.title,
                event.description,
                event.start_date.timestamp(),
                event.end_date.map(|d| d.timestamp()),
                event.subject,
                event.note_id.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn update_timeline_event(&self, event: &TimelineEvent) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE timeline_events SET title = ?1, description = ?2, start_date = ?3, end_date = ?4, subject = ?5, note_id = ?6 WHERE id = ?7",
            params![
                event.title,
                event.description,
                event.start_date.timestamp(),
                event.end_date.map(|d| d.timestamp()),
                event.subject,
                event.note_id.map(|id| id.to_string()),
                event.id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn list_timeline_events(
        &self,
        subject: Option<&str>,
    ) -> Result<Vec<TimelineEvent>, StoreError> {
        let conn = self.lock()?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match subject {
            Some(s) if !s.is_empty() => (
                "SELECT id, title, description, start_date, end_date, subject, note_id FROM timeline_events WHERE subject = ?1 ORDER BY start_date",
                vec![Box::new(s.to_string()) as Box<dyn rusqlite::ToSql>],
            ),
            _ => (
                "SELECT id, title, description, start_date, end_date, subject, note_id FROM timeline_events ORDER BY start_date",
                vec![],
            ),
        };
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map(params_refs.as_slice(), map_timeline_event_row)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    pub fn get_timeline_event(&self, id: Id) -> Result<Option<TimelineEvent>, StoreError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT id, title, description, start_date, end_date, subject, note_id FROM timeline_events WHERE id = ?1",
            params![id.to_string()],
            map_timeline_event_row,
        ).optional()?;
        Ok(result)
    }

    pub fn delete_timeline_event(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM timeline_events WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // === Grammar Rules ===

    pub fn create_grammar_rule(&self, rule: &GrammarRule) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO grammar_rules (id, title, category, explanation, examples, counter_examples, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                rule.id.to_string(),
                rule.title,
                rule.category,
                rule.explanation,
                serde_json::to_string(&rule.examples).unwrap_or_else(|_| "[]".to_string()),
                serde_json::to_string(&rule.counter_examples).unwrap_or_else(|_| "[]".to_string()),
                rule.created_at.timestamp(),
                rule.updated_at.timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn update_grammar_rule(&self, rule: &GrammarRule) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE grammar_rules SET title = ?1, category = ?2, explanation = ?3, examples = ?4, counter_examples = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                rule.title,
                rule.category,
                rule.explanation,
                serde_json::to_string(&rule.examples).unwrap_or_else(|_| "[]".to_string()),
                serde_json::to_string(&rule.counter_examples).unwrap_or_else(|_| "[]".to_string()),
                rule.updated_at.timestamp(),
                rule.id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_grammar_rule(&self, id: Id) -> Result<Option<GrammarRule>, StoreError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT id, title, category, explanation, examples, counter_examples, created_at, updated_at FROM grammar_rules WHERE id = ?1",
            params![id.to_string()],
            map_grammar_rule_row,
        ).optional()?;
        Ok(result)
    }

    pub fn list_grammar_rules(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<GrammarRule>, StoreError> {
        let conn = self.lock()?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match category {
            Some(c) if !c.is_empty() => (
                "SELECT id, title, category, explanation, examples, counter_examples, created_at, updated_at FROM grammar_rules WHERE category = ?1 ORDER BY title COLLATE NOCASE",
                vec![Box::new(c.to_string()) as Box<dyn rusqlite::ToSql>],
            ),
            _ => (
                "SELECT id, title, category, explanation, examples, counter_examples, created_at, updated_at FROM grammar_rules ORDER BY title COLLATE NOCASE",
                vec![],
            ),
        };
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map(params_refs.as_slice(), map_grammar_rule_row)?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    pub fn delete_grammar_rule(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM grammar_rules WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    pub fn list_grammar_categories(&self) -> Result<Vec<String>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT DISTINCT category FROM grammar_rules ORDER BY category COLLATE NOCASE",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut categories = Vec::new();
        for row in rows {
            categories.push(row?);
        }
        Ok(categories)
    }

    // === Sources ===

    pub fn create_source(&self, source: &Source) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO sources (id, url, title, author, imported_at, browser_history_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                source.id.to_string(),
                source.url,
                source.title,
                source.author,
                source.imported_at.timestamp(),
                source.browser_history_id,
            ],
        )?;
        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<Source>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare("SELECT id, url, title, author, imported_at, browser_history_id FROM sources ORDER BY imported_at DESC")?;
        let rows = statement.query_map([], map_source_row)?;
        let mut sources = Vec::new();
        for row in rows {
            sources.push(row?);
        }
        Ok(sources)
    }

    // === Note Links ===

    pub fn create_note_link(&self, link: &NoteLink) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT OR IGNORE INTO note_links (id, source_note_id, target_note_id, link_type)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                link.id.to_string(),
                link.source_note_id.to_string(),
                link.target_note_id.to_string(),
                link.link_type,
            ],
        )?;
        Ok(())
    }

    pub fn list_note_links_for_note(&self, note_id: Id) -> Result<Vec<NoteLink>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT id, source_note_id, target_note_id, link_type FROM note_links WHERE source_note_id = ?1 OR target_note_id = ?1",
        )?;
        let rows = statement.query_map(params![note_id.to_string()], map_note_link_row)?;
        let mut links = Vec::new();
        for row in rows {
            links.push(row?);
        }
        Ok(links)
    }

    pub fn delete_note_link(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM note_links WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // === Link management ===

    pub fn find_note_by_title(&self, title: &str) -> Result<Option<Note>, StoreError> {
        let conn = self.lock()?;
        let result = conn
            .query_row(
                "SELECT id, title, content, subject, tags, source_url, created_at, updated_at
             FROM notes WHERE title = ?1 LIMIT 1",
                params![title],
                map_note_row,
            )
            .optional()?;
        Ok(result)
    }

    pub fn parse_and_create_links(&self, note_id: Id, content: &str) -> Result<(), StoreError> {
        // Delete existing outgoing links from this note first
        self.delete_links_for_source(note_id)?;

        // Find all [[Title]] patterns
        let re =
            Regex::new(r"\[\[([^\]]+)\]\]").map_err(|e| StoreError::Internal(e.to_string()))?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;

        for cap in re.captures_iter(content) {
            let link_title = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let link_title = link_title.trim();
            if link_title.is_empty() {
                continue;
            }

            // Look up target note by title
            if let Some(target) = tx
                .query_row(
                    "SELECT id, title, content, subject, tags, source_url, created_at, updated_at
                 FROM notes WHERE title = ?1 LIMIT 1",
                    params![link_title],
                    map_note_row,
                )
                .optional()?
            {
                if target.id != note_id {
                    let link = NoteLink {
                        id: unused_uuid(),
                        source_note_id: note_id,
                        target_note_id: target.id,
                        link_type: String::from("wiki"),
                    };
                    tx.execute(
                        "INSERT OR IGNORE INTO note_links (id, source_note_id, target_note_id, link_type)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![
                            link.id.to_string(),
                            link.source_note_id.to_string(),
                            link.target_note_id.to_string(),
                            link.link_type,
                        ],
                    )?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_outgoing_links(&self, note_id: Id) -> Result<Vec<(NoteLink, Note)>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT nl.id, nl.source_note_id, nl.target_note_id, nl.link_type,
                    n2.id, n2.title, n2.content, n2.subject, n2.tags, n2.source_url, n2.created_at, n2.updated_at
             FROM note_links nl
             JOIN notes n2 ON n2.id = nl.target_note_id
             WHERE nl.source_note_id = ?1",
        )?;
        let rows = statement.query_map(params![note_id.to_string()], |row| {
            let link = NoteLink {
                id: row.get::<_, String>(0)?.parse().unwrap(),
                source_note_id: row.get::<_, String>(1)?.parse().unwrap(),
                target_note_id: row.get::<_, String>(2)?.parse().unwrap(),
                link_type: row.get(3)?,
            };
            let target = Note {
                id: row.get::<_, String>(4)?.parse().unwrap(),
                title: row.get(5)?,
                content: row.get(6)?,
                subject: row.get(7)?,
                tags: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                source_url: row.get(9)?,
                created_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap_or_else(Utc::now),
                updated_at: DateTime::from_timestamp(row.get(11)?, 0).unwrap_or_else(Utc::now),
            };
            Ok((link, target))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_incoming_links(&self, note_id: Id) -> Result<Vec<(NoteLink, Note)>, StoreError> {
        let conn = self.lock()?;
        let mut statement = conn.prepare(
            "SELECT nl.id, nl.source_note_id, nl.target_note_id, nl.link_type,
                    n2.id, n2.title, n2.content, n2.subject, n2.tags, n2.source_url, n2.created_at, n2.updated_at
             FROM note_links nl
             JOIN notes n2 ON n2.id = nl.source_note_id
             WHERE nl.target_note_id = ?1",
        )?;
        let rows = statement.query_map(params![note_id.to_string()], |row| {
            let link = NoteLink {
                id: row.get::<_, String>(0)?.parse().unwrap(),
                source_note_id: row.get::<_, String>(1)?.parse().unwrap(),
                target_note_id: row.get::<_, String>(2)?.parse().unwrap(),
                link_type: row.get(3)?,
            };
            let source = Note {
                id: row.get::<_, String>(4)?.parse().unwrap(),
                title: row.get(5)?,
                content: row.get(6)?,
                subject: row.get(7)?,
                tags: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                source_url: row.get(9)?,
                created_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap_or_else(Utc::now),
                updated_at: DateTime::from_timestamp(row.get(11)?, 0).unwrap_or_else(Utc::now),
            };
            Ok((link, source))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete_links_for_source(&self, note_id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM note_links WHERE source_note_id = ?1",
            params![note_id.to_string()],
        )?;
        Ok(())
    }

    pub fn all_notes_for_graph(&self) -> Result<Vec<(Note, Vec<Id>)>, StoreError> {
        let notes = self.list_notes(None, None)?;
        let mut result = Vec::new();
        for note in &notes {
            let links = self.get_outgoing_links(note.id)?;
            let targets: Vec<Id> = links.into_iter().map(|(_, target)| target.id).collect();
            result.push((note.clone(), targets));
        }
        Ok(result)
    }

    // === Pending Imports ===

    pub fn create_pending_import(&self, import: &PendingImport) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO pending_imports (id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                import.id.to_string(),
                import.url,
                import.title,
                import.suggested_subject,
                serde_json::to_string(&import.suggested_tags).unwrap_or_else(|_| "[]".to_string()),
                import.llm_category,
                import.llm_confidence,
                match import.status {
                    ImportStatus::Pending => "pending",
                    ImportStatus::Approved => "approved",
                    ImportStatus::Rejected => "rejected",
                },
                import.created_at.timestamp(),
                import.reviewed_at.map(|d| d.timestamp()),
            ],
        )?;
        Ok(())
    }

    pub fn list_pending_imports(
        &self,
        status: Option<ImportStatus>,
    ) -> Result<Vec<PendingImport>, StoreError> {
        let conn = self.lock()?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match status {
            Some(ImportStatus::Pending) => (
                "SELECT id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at FROM pending_imports WHERE status = 'pending' ORDER BY created_at DESC",
                vec![],
            ),
            Some(ImportStatus::Approved) => (
                "SELECT id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at FROM pending_imports WHERE status = 'approved' ORDER BY created_at DESC",
                vec![],
            ),
            Some(ImportStatus::Rejected) => (
                "SELECT id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at FROM pending_imports WHERE status = 'rejected' ORDER BY created_at DESC",
                vec![],
            ),
            None => (
                "SELECT id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at FROM pending_imports ORDER BY created_at DESC",
                vec![],
            ),
        };
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map(params_refs.as_slice(), map_pending_import_row)?;
        let mut imports = Vec::new();
        for row in rows {
            imports.push(row?);
        }
        Ok(imports)
    }

    pub fn get_pending_import(&self, id: Id) -> Result<Option<PendingImport>, StoreError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT id, url, title, suggested_subject, suggested_tags, llm_category, llm_confidence, status, created_at, reviewed_at FROM pending_imports WHERE id = ?1",
            params![id.to_string()],
            map_pending_import_row,
        ).optional()?;
        Ok(result)
    }

    pub fn approve_pending_import(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE pending_imports SET status = 'approved', reviewed_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp(), id.to_string()],
        )?;
        Ok(())
    }

    pub fn reject_pending_import(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE pending_imports SET status = 'rejected', reviewed_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp(), id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_pending_import(&self, id: Id) -> Result<(), StoreError> {
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM pending_imports WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}

fn map_entry_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        word: row.get(1)?,
        normalized_word: row.get(2)?,
        phonetic: row.get(3)?,
        audio_url: row.get(4)?,
        origin: row.get(5)?,
        source: row.get(6)?,
        source_url: row.get(7)?,
        license: row.get(8)?,
        created_at: DateTime::from_timestamp(row.get(9)?, 0).unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap_or_else(Utc::now),
    })
}

fn map_sense_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Sense> {
    let synonyms: String = row.get(5)?;
    let antonyms: String = row.get(6)?;
    Ok(Sense {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        entry_id: row.get::<_, String>(1)?.parse().unwrap(),
        part_of_speech: row.get(2)?,
        definition: row.get(3)?,
        example: row.get(4)?,
        synonyms: serde_json::from_str(&synonyms).unwrap_or_default(),
        antonyms: serde_json::from_str(&antonyms).unwrap_or_default(),
        sort_order: row.get(7)?,
    })
}

fn map_deck_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deck> {
    Ok(Deck {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        name: row.get(1)?,
        description: row.get(2)?,
        new_per_day: row.get(3)?,
        reviews_per_day: row.get(4)?,
        created_at: DateTime::from_timestamp(row.get(5)?, 0).unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp(row.get(6)?, 0).unwrap_or_else(Utc::now),
        archived_at: row
            .get::<_, Option<i64>>(7)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
    })
}

fn map_card_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        entry_id: row.get::<_, String>(1)?.parse().unwrap(),
        deck_id: row.get::<_, String>(2)?.parse().unwrap(),
        direction: row.get(3)?,
        front_note: row.get(4)?,
        back_note: row.get(5)?,
        state: parse_card_state(row.get::<_, String>(6)?.as_str()),
        due_at: row
            .get::<_, Option<i64>>(7)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
        interval_days: row.get(8)?,
        ease_factor: row.get(9)?,
        repetitions: row.get(10)?,
        lapses: row.get(11)?,
        first_reviewed_at: row
            .get::<_, Option<i64>>(12)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
        last_reviewed_at: row
            .get::<_, Option<i64>>(13)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
        created_at: DateTime::from_timestamp(row.get(14)?, 0).unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp(row.get(15)?, 0).unwrap_or_else(Utc::now),
    })
}

fn card_state_string(state: CardState) -> &'static str {
    match state {
        CardState::New => "new",
        CardState::Learning => "learning",
        CardState::Review => "review",
        CardState::Relearning => "relearning",
        CardState::Suspended => "suspended",
    }
}

fn parse_card_state(value: &str) -> CardState {
    match value {
        "learning" => CardState::Learning,
        "review" => CardState::Review,
        "relearning" => CardState::Relearning,
        "suspended" => CardState::Suspended,
        _ => CardState::New,
    }
}

fn map_review_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReviewLog> {
    Ok(ReviewLog {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        card_id: row.get::<_, String>(1)?.parse().unwrap(),
        reviewed_at: DateTime::from_timestamp(row.get(2)?, 0).unwrap_or_else(Utc::now),
        day: row.get(3)?,
        rating: match row.get::<_, i64>(4)? {
            1 => Rating::Again,
            2 => Rating::Hard,
            3 => Rating::Good,
            _ => Rating::Easy,
        },
        prev_state: parse_card_state(row.get::<_, String>(5)?.as_str()),
        prev_interval_days: row.get(6)?,
        prev_ease: row.get(7)?,
        new_state: parse_card_state(row.get::<_, String>(8)?.as_str()),
        new_interval_days: row.get(9)?,
        new_ease: row.get(10)?,
        duration_ms: row.get(11)?,
    })
}

fn unused_uuid() -> Id {
    uuid::Uuid::new_v4()
}

fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    let tags: String = row.get(4)?;
    Ok(Note {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        title: row.get(1)?,
        content: row.get(2)?,
        subject: row.get(3)?,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        source_url: row.get(5)?,
        created_at: DateTime::from_timestamp(row.get(6)?, 0).unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp(row.get(7)?, 0).unwrap_or_else(Utc::now),
    })
}

fn map_timeline_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimelineEvent> {
    Ok(TimelineEvent {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        title: row.get(1)?,
        description: row.get(2)?,
        start_date: DateTime::from_timestamp(row.get(3)?, 0).unwrap_or_else(Utc::now),
        end_date: row
            .get::<_, Option<i64>>(4)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
        subject: row.get(5)?,
        note_id: row
            .get::<_, Option<String>>(6)?
            .map(|v| v.parse().unwrap_or_else(|_| Id::nil())),
    })
}

fn map_grammar_rule_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GrammarRule> {
    let examples: String = row.get(4)?;
    let counter_examples: String = row.get(5)?;
    Ok(GrammarRule {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        title: row.get(1)?,
        category: row.get(2)?,
        explanation: row.get(3)?,
        examples: serde_json::from_str(&examples).unwrap_or_default(),
        counter_examples: serde_json::from_str(&counter_examples).unwrap_or_default(),
        created_at: DateTime::from_timestamp(row.get(6)?, 0).unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp(row.get(7)?, 0).unwrap_or_else(Utc::now),
    })
}

fn map_source_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    Ok(Source {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        url: row.get(1)?,
        title: row.get(2)?,
        author: row.get(3)?,
        imported_at: DateTime::from_timestamp(row.get(4)?, 0).unwrap_or_else(Utc::now),
        browser_history_id: row.get(5)?,
    })
}

fn map_note_link_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteLink> {
    Ok(NoteLink {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        source_note_id: row.get::<_, String>(1)?.parse().unwrap(),
        target_note_id: row.get::<_, String>(2)?.parse().unwrap(),
        link_type: row.get(3)?,
    })
}

fn seed_database(conn: &Connection) -> Result<(), StoreError> {
    // Only seed if the database is empty (no notes exist yet)
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let now = Utc::now().timestamp();

    // === Stock Notes ===
    let stock_notes = [
        (
            "How to Write a Thesis Statement",
            "A thesis statement is the central argument of your essay. It should be:\n- Specific and focused\n- Arguable (not just a fact)\n- Supportable with evidence\n- Placed at the end of your introduction\n\nExample: 'The use of symbolism in The Great Gatsby reveals the hollowness of the American Dream.'",
            "English",
            vec!["writing", "thesis", "essay"],
        ),
        (
            "Primary vs Secondary Sources",
            "Primary sources are original materials from the time period being studied (diaries, letters, photographs, official documents).\nSecondary sources interpret or analyze primary sources (textbooks, academic articles, documentaries).\n\nAlways prioritize primary sources when available, and use secondary sources for context and interpretation.",
            "History",
            vec!["research", "sources", "methodology"],
        ),
        (
            "The Five-Paragraph Essay Structure",
            "1. Introduction (hook, context, thesis)\n2. Body Paragraph 1 (topic sentence, evidence, analysis)\n3. Body Paragraph 2 (topic sentence, evidence, analysis)\n4. Body Paragraph 3 (topic sentence, evidence, analysis)\n5. Conclusion (restate thesis, summarize, final thought)",
            "English",
            vec!["writing", "structure", "essay"],
        ),
        (
            "How to Cite Sources in MLA Format",
            "In-text citation: (Author PageNumber)\nWorks Cited: Author Last, First. \"Title of Article.\" Title of Container, Publisher, Year, pp. pages.\n\nExample: (Smith 42)",
            "English",
            vec!["citation", "mla", "research"],
        ),
        (
            "The Scientific Method in Historical Research",
            "1. Ask a question about the past\n2. Conduct background research\n3. Form a hypothesis\n4. Gather and analyze primary sources\n5. Draw conclusions\n6. Communicate results",
            "History",
            vec!["methodology", "research", "history"],
        ),
    ];

    for (title, content, subject, tags) in stock_notes.iter() {
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO notes (id, title, content, subject, tags, source_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                title,
                content,
                subject,
                tags_json,
                Option::<String>::None,
                now,
                now,
            ],
        )?;
    }

    // === Stock Timeline Events ===
    let stock_events = [
        (
            "Fall of the Western Roman Empire",
            "The traditional date for the fall of Rome, marking the end of antiquity and the beginning of the Middle Ages in Europe.",
            "476-09-04",
            None,
            "History",
        ),
        (
            "Magna Carta Signed",
            "King John of England agrees to the Magna Carta, establishing the principle that everybody, including the king, is subject to the law.",
            "1215-06-15",
            None,
            "History",
        ),
        (
            "Christopher Columbus Reaches the Americas",
            "Columbus lands in the Bahamas, initiating sustained European contact with the Americas.",
            "1492-10-12",
            None,
            "History",
        ),
        (
            "Declaration of Independence",
            "The Thirteen American Colonies declare independence from Great Britain.",
            "1776-07-04",
            None,
            "History",
        ),
        (
            "French Revolution Begins",
            "The Storming of the Bastille marks the start of the French Revolution.",
            "1789-07-14",
            None,
            "History",
        ),
        (
            "World War I Ends",
            "The Armistice of November 11, 1918 ends fighting on the Western Front.",
            "1918-11-11",
            None,
            "History",
        ),
        (
            "World War II Ends",
            "Germany surrenders, ending World War II in Europe (V-E Day).",
            "1945-05-08",
            None,
            "History",
        ),
        (
            "First Moon Landing",
            "Apollo 11 lands on the Moon; Neil Armstrong becomes the first person to walk on the lunar surface.",
            "1969-07-20",
            None,
            "History",
        ),
        (
            "Fall of the Berlin Wall",
            "The Berlin Wall falls, symbolizing the end of the Cold War division of Europe.",
            "1989-11-09",
            None,
            "History",
        ),
    ];

    for (title, description, start_date, end_date, subject) in stock_events.iter() {
        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let end = end_date.map(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc()
                .timestamp()
        });
        conn.execute(
            "INSERT INTO timeline_events (id, title, description, start_date, end_date, subject, note_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                title,
                description,
                start,
                end,
                subject,
                Option::<String>::None,
            ],
        )?;
    }

    // === Stock Grammar Rules ===
    let stock_rules = [
        (
            "Subject-Verb Agreement",
            "Grammar",
            "The subject and verb must agree in number. Singular subjects take singular verbs; plural subjects take plural verbs.\n\nExample: 'The cat runs' (singular) vs. 'The cats run' (plural).",
            vec![
                "The dog barks loudly.",
                "The dogs bark loudly.",
                "She writes every day.",
                "They write every day.",
            ],
            vec![
                "The dog bark loudly.",
                "The dogs barks loudly.",
                "She write every day.",
                "They writes every day.",
            ],
        ),
        (
            "Comma Splice",
            "Punctuation",
            "A comma splice occurs when two independent clauses are joined by only a comma. Use a period, semicolon, or comma + coordinating conjunction instead.\n\nIncorrect: 'I went to the store, I bought milk.'\nCorrect: 'I went to the store. I bought milk.'",
            vec![
                "I finished my homework, and then I went to bed.",
                "She studied hard; she passed the exam.",
            ],
            vec![
                "I finished my homework, I went to bed.",
                "She studied hard, she passed the exam.",
            ],
        ),
        (
            "Its vs. It's",
            "Spelling",
            "'Its' is possessive (belonging to it). 'It's' is a contraction of 'it is' or 'it has'.\n\nExample: 'The cat licked its paws.' vs. 'It's raining outside.'",
            vec![
                "The company changed its policy.",
                "It's important to study regularly.",
            ],
            vec![
                "The company changed it's policy.",
                "Its important to study regularly.",
            ],
        ),
        (
            "There vs. Their vs. They're",
            "Spelling",
            "'There' indicates location. 'Their' is possessive. 'They're' is a contraction of 'they are'.\n\nExample: 'They're going to their house over there.'",
            vec![
                "There is a book on the table.",
                "Their car is blue.",
                "They're coming to the party.",
            ],
            vec![
                "Their is a book on the table.",
                "There car is blue.",
                "Theyre coming to the party.",
            ],
        ),
        (
            "Affect vs. Effect",
            "Spelling",
            "'Affect' is usually a verb (to influence). 'Effect' is usually a noun (a result).\n\nExample: 'The weather affected our plans.' vs. 'The effect was immediate.'",
            vec![
                "How will this affect your grade?",
                "The effect was immediate.",
            ],
            vec![
                "How will this effect your grade?",
                "The affect was immediate.",
            ],
        ),
        (
            "Semicolon Usage",
            "Punctuation",
            "Use a semicolon to join two closely related independent clauses without a coordinating conjunction, or to separate items in a complex list.\n\nExample: 'I have a big test tomorrow; I can't go out tonight.'",
            vec![
                "I love reading; it relaxes me.",
                "She visited Paris, France; London, England; and Rome, Italy.",
            ],
            vec![
                "I love reading; it relaxes me.",
                "She visited Paris France London England and Rome Italy.",
            ],
        ),
        (
            "Active vs. Passive Voice",
            "Style",
            "Active voice: the subject performs the action. Passive voice: the subject receives the action. Active voice is generally clearer and more direct.\n\nPassive: 'The ball was thrown by John.'\nActive: 'John threw the ball.'",
            vec![
                "The committee approved the proposal.",
                "Scientists discovered a new species.",
            ],
            vec![
                "The proposal was approved by the committee.",
                "A new species was discovered by scientists.",
            ],
        ),
        (
            "Dangling Modifiers",
            "Style",
            "A dangling modifier is a word or phrase that modifies a word not clearly stated in the sentence.\n\nIncorrect: 'Walking to school, the bus arrived.'\nCorrect: 'Walking to school, I saw the bus arrive.'",
            vec![
                "After finishing the assignment, I went to sleep.",
                "While studying for the exam, she drank coffee.",
            ],
            vec![
                "After finishing the assignment, the sleep came.",
                "While studying for the exam, the coffee was drunk.",
            ],
        ),
    ];

    for (title, category, explanation, examples, counter_examples) in stock_rules.iter() {
        let examples_json = serde_json::to_string(examples).unwrap_or_else(|_| "[]".to_string());
        let counter_examples_json =
            serde_json::to_string(counter_examples).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO grammar_rules (id, title, category, explanation, examples, counter_examples, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                title,
                category,
                explanation,
                examples_json,
                counter_examples_json,
                now,
                now,
            ],
        )?;
    }

    Ok(())
}

fn map_pending_import_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PendingImport> {
    let tags: String = row.get(4)?;
    let status_str: String = row.get(7)?;
    Ok(PendingImport {
        id: row.get::<_, String>(0)?.parse().unwrap(),
        url: row.get(1)?,
        title: row.get(2)?,
        suggested_subject: row.get(3)?,
        suggested_tags: serde_json::from_str(&tags).unwrap_or_default(),
        llm_category: row.get(5)?,
        llm_confidence: row.get(6)?,
        status: match status_str.as_str() {
            "approved" => ImportStatus::Approved,
            "rejected" => ImportStatus::Rejected,
            _ => ImportStatus::Pending,
        },
        created_at: DateTime::from_timestamp(row.get(8)?, 0).unwrap_or_else(Utc::now),
        reviewed_at: row
            .get::<_, Option<i64>>(9)?
            .map(|value| DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now)),
    })
}
