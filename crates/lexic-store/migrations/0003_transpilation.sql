PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL UNIQUE,
  source_language TEXT,
  target_language TEXT NOT NULL DEFAULT 'go',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS transpile_runs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  target_language TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  exit_code INTEGER,
  diagnostic_count INTEGER NOT NULL DEFAULT 0,
  summary TEXT
);

CREATE TABLE IF NOT EXISTS transpile_sections (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES transpile_runs(id) ON DELETE CASCADE,
  section_key TEXT NOT NULL,
  source_path TEXT,
  target_path TEXT,
  status TEXT NOT NULL,
  diagnostics_json TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS missing_functions (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES transpile_runs(id) ON DELETE CASCADE,
  section_id TEXT REFERENCES transpile_sections(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  source_path TEXT,
  line INTEGER,
  reason TEXT NOT NULL,
  resolved_at INTEGER,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS manual_edits (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  section_id TEXT REFERENCES transpile_sections(id) ON DELETE SET NULL,
  path TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS transpile_runs_project_idx ON transpile_runs(project_id, started_at);
CREATE INDEX IF NOT EXISTS transpile_sections_run_idx ON transpile_sections(run_id, section_key);
CREATE INDEX IF NOT EXISTS missing_functions_run_idx ON missing_functions(run_id, name);