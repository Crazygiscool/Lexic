DROP TABLE IF EXISTS entries_fts;

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
  word, definitions, content='', contentless_delete=1, tokenize='unicode61 remove_diacritics 2'
);

INSERT INTO entries_fts (rowid, word, definitions)
SELECT
  e.rowid,
  e.word,
  COALESCE((
      SELECT GROUP_CONCAT(s.definition, ' ')
      FROM senses s
      WHERE s.entry_id = e.id
  ), '')
FROM entries e;