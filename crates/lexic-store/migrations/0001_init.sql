PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS entries (
  id               TEXT PRIMARY KEY,
  word             TEXT NOT NULL,
  normalized_word  TEXT NOT NULL,
  phonetic         TEXT,
  audio_url        TEXT,
  origin           TEXT,
  source           TEXT NOT NULL,
  source_url       TEXT,
  license          TEXT,
  fetched_at       INTEGER,
  created_at       INTEGER NOT NULL,
  updated_at       INTEGER NOT NULL,
  UNIQUE(normalized_word, source)
);

CREATE TABLE IF NOT EXISTS senses (
  id             TEXT PRIMARY KEY,
  entry_id       TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
  part_of_speech TEXT,
  definition     TEXT NOT NULL,
  example        TEXT,
  synonyms_json  TEXT NOT NULL DEFAULT '[]',
  antonyms_json  TEXT NOT NULL DEFAULT '[]',
  sort_order     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS decks (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL UNIQUE COLLATE NOCASE,
  description   TEXT,
  new_per_day   INTEGER,
  reviews_per_day INTEGER,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  archived_at   INTEGER
);

CREATE TABLE IF NOT EXISTS cards (
  id                TEXT PRIMARY KEY,
  entry_id          TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
  deck_id           TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
  direction         TEXT NOT NULL DEFAULT 'recognition',
  front_note        TEXT,
  back_note         TEXT,
  state             TEXT NOT NULL DEFAULT 'new',
  due_at            INTEGER,
  interval_days     REAL NOT NULL DEFAULT 0,
  ease_factor       REAL NOT NULL DEFAULT 2.5,
  repetitions       INTEGER NOT NULL DEFAULT 0,
  lapses            INTEGER NOT NULL DEFAULT 0,
  first_reviewed_at INTEGER,
  last_reviewed_at  INTEGER,
  created_at        INTEGER NOT NULL,
  updated_at        INTEGER NOT NULL,
  UNIQUE(entry_id, deck_id, direction)
);

CREATE INDEX IF NOT EXISTS cards_due_idx ON cards(state, due_at);
CREATE INDEX IF NOT EXISTS cards_deck_idx ON cards(deck_id, state);

CREATE TABLE IF NOT EXISTS reviews (
  id                 TEXT PRIMARY KEY,
  card_id            TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  reviewed_at        INTEGER NOT NULL,
  day                TEXT NOT NULL,
  rating             INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 4),
  prev_state         TEXT NOT NULL,
  prev_interval_days REAL NOT NULL,
  prev_ease          REAL NOT NULL,
  new_state          TEXT NOT NULL,
  new_interval_days  REAL NOT NULL,
  new_ease           REAL NOT NULL,
  duration_ms        INTEGER
);

CREATE INDEX IF NOT EXISTS reviews_day_idx ON reviews(day);
CREATE INDEX IF NOT EXISTS reviews_card_idx ON reviews(card_id);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
  word, definitions, content='', contentless_delete=1, tokenize='unicode61 remove_diacritics 2'
);