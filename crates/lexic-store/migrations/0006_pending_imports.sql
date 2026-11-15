PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pending_imports (
  id TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT,
  suggested_subject TEXT NOT NULL DEFAULT 'General',
  suggested_tags TEXT NOT NULL DEFAULT '[]',
  llm_category TEXT,
  llm_confidence REAL,
  status TEXT NOT NULL DEFAULT 'pending',
  created_at INTEGER NOT NULL,
  reviewed_at INTEGER
);

CREATE INDEX IF NOT EXISTS pending_imports_status_idx ON pending_imports(status);
CREATE INDEX IF NOT EXISTS pending_imports_subject_idx ON pending_imports(suggested_subject);
CREATE INDEX IF NOT EXISTS pending_imports_created_at_idx ON pending_imports(created_at DESC);
