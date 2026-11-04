use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use lexic_core::model::*;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;

const MIGRATIONS: &[(&str, usize)] = &[
    (include_str!("../migrations/0001_init.sql"), 1),
    (
        include_str!("../migrations/0002_fts_contentless_delete.sql"),
        2,
    ),
    (include_str!("../migrations/0004_drop_transpilation.sql"), 4),
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
