pub(super) const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS track (
  path TEXT PRIMARY KEY, mtime INTEGER NOT NULL, size INTEGER NOT NULL,
  duration_s REAL NOT NULL, sample_rate INTEGER NOT NULL, channels INTEGER NOT NULL,
  cue_start_s REAL NOT NULL DEFAULT 0, cue_end_s REAL,
  gain_db REAL NOT NULL DEFAULT 0, norm_enabled INTEGER NOT NULL DEFAULT 0,
  norm_gain_db REAL NOT NULL DEFAULT 0, measured_peak_db REAL, measured_lufs REAL,
  analyzed_at INTEGER, last_played INTEGER
);";

pub(super) const SCHEMA_V2: &str = "
CREATE TABLE IF NOT EXISTS library_root (
  id INTEGER PRIMARY KEY, path TEXT NOT NULL, path_key TEXT NOT NULL UNIQUE,
  collection TEXT NOT NULL CHECK(collection IN ('music', 'effects')),
  recursive INTEGER NOT NULL DEFAULT 1, enabled INTEGER NOT NULL DEFAULT 1,
  state TEXT NOT NULL DEFAULT 'pending', scan_generation INTEGER NOT NULL DEFAULT 0,
  last_scan_at INTEGER, created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS library_root_collection
  ON library_root(collection, enabled);";

pub(super) const SCHEMA_V3: &str = "
CREATE TABLE IF NOT EXISTS library_track (
  path_key TEXT PRIMARY KEY REFERENCES track(path) ON DELETE CASCADE,
  root_id INTEGER NOT NULL REFERENCES library_root(id) ON DELETE CASCADE,
  collection TEXT NOT NULL CHECK(collection IN ('music', 'effects')),
  relative_path TEXT NOT NULL, file_name TEXT NOT NULL, extension TEXT NOT NULL,
  title TEXT, artist TEXT, album TEXT, genre TEXT, year INTEGER, track_number INTEGER,
  metadata_state TEXT NOT NULL DEFAULT 'pending',
  tags_readable INTEGER NOT NULL DEFAULT 0,
  present INTEGER NOT NULL DEFAULT 1,
  scan_generation INTEGER NOT NULL DEFAULT 0, last_seen_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS library_track_root
  ON library_track(root_id, scan_generation);
CREATE INDEX IF NOT EXISTS library_track_collection
  ON library_track(collection, metadata_state);
CREATE VIRTUAL TABLE IF NOT EXISTS library_track_search USING fts5(
  path_key UNINDEXED, search_text, tokenize='trigram'
);";

pub(super) const SCHEMA_V4: &str = "
CREATE INDEX IF NOT EXISTS library_track_browse_all
  ON library_track(present,file_name COLLATE NOCASE,path_key);
CREATE INDEX IF NOT EXISTS library_track_browse_collection
  ON library_track(collection,present,file_name COLLATE NOCASE,path_key);";
