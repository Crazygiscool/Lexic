PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  content TEXT NOT NULL DEFAULT '',
  subject TEXT NOT NULL DEFAULT 'General',
  tags TEXT NOT NULL DEFAULT '[]',
  source_url TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS timeline_events (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  start_date INTEGER NOT NULL,
  end_date INTEGER,
  subject TEXT NOT NULL DEFAULT 'General',
  note_id TEXT REFERENCES notes(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS grammar_rules (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  category TEXT NOT NULL DEFAULT 'General',
  explanation TEXT NOT NULL DEFAULT '',
  examples TEXT NOT NULL DEFAULT '[]',
  counter_examples TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sources (
  id TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT,
  author TEXT,
  imported_at INTEGER NOT NULL,
  browser_history_id TEXT
);

CREATE TABLE IF NOT EXISTS note_links (
  id TEXT PRIMARY KEY,
  source_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  target_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  link_type TEXT NOT NULL DEFAULT 'related',
  UNIQUE(source_note_id, target_note_id, link_type)
);

CREATE INDEX IF NOT EXISTS notes_subject_idx ON notes(subject);
CREATE INDEX IF NOT EXISTS notes_created_at_idx ON notes(created_at DESC);
CREATE INDEX IF NOT EXISTS timeline_events_subject_idx ON timeline_events(subject);
CREATE INDEX IF NOT EXISTS timeline_events_start_date_idx ON timeline_events(start_date);
CREATE INDEX IF NOT EXISTS grammar_rules_category_idx ON grammar_rules(category);
CREATE INDEX IF NOT EXISTS note_links_source_idx ON note_links(source_note_id);
CREATE INDEX IF NOT EXISTS note_links_target_idx ON note_links(target_note_id);
