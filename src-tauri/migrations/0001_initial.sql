-- Settings key-value store
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

-- Default settings inserted on first run
INSERT OR IGNORE INTO settings VALUES ('api_key', '');
INSERT OR IGNORE INTO settings VALUES ('global_schedule_time', '17:55');
INSERT OR IGNORE INTO settings VALUES ('scheduler_auto_run', 'true');

-- Folder configurations: one row per (storage_id, project_id, folder_path) triple.
-- Multiple rows for the same (storage_id, project_id) means multiple folders;
-- their sizes are summed before submission.
-- custom_schedule NULL means inherit global_schedule_time.
CREATE TABLE IF NOT EXISTS folder_configs (
    id              TEXT NOT NULL PRIMARY KEY,
    storage_id      TEXT NOT NULL,
    project_id      TEXT NOT NULL,
    folder_path     TEXT NOT NULL,
    custom_schedule TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE (storage_id, project_id, folder_path)
);

-- Queued submissions that failed (API unavailable, no internet).
-- Retried every 5 minutes. Abandoned after 10 attempts.
CREATE TABLE IF NOT EXISTS pending_submissions (
    id             TEXT NOT NULL PRIMARY KEY,
    storage_id     TEXT NOT NULL,
    project_id     TEXT NOT NULL,
    date           TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL,
    attempts       INTEGER NOT NULL DEFAULT 0,
    last_attempted TEXT,
    created_at     TEXT NOT NULL
);

-- Append-only audit log. Pruned to the last 10,000 rows after each scheduler tick.
CREATE TABLE IF NOT EXISTS log_entries (
    id         TEXT NOT NULL PRIMARY KEY,
    level      TEXT NOT NULL CHECK (level IN ('info', 'warning', 'error')),
    message    TEXT NOT NULL,
    context    TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_folder_configs_sp ON folder_configs (storage_id, project_id);
CREATE INDEX IF NOT EXISTS idx_pending_created   ON pending_submissions (created_at);
CREATE INDEX IF NOT EXISTS idx_log_level         ON log_entries (level);
CREATE INDEX IF NOT EXISTS idx_log_created       ON log_entries (created_at DESC);

-- Cached storage and project names fetched from the freispace API.
-- Updated incrementally using updated_since to avoid re-fetching unchanged entries.
CREATE TABLE IF NOT EXISTS entity_names (
    entity_type TEXT NOT NULL CHECK (entity_type IN ('storage', 'project')),
    entity_id   TEXT NOT NULL,
    name        TEXT,
    parent_id   TEXT,
    project_number TEXT,
    color       TEXT,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id)
);

CREATE TABLE IF NOT EXISTS storage_project_settings (
    storage_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    enabled    INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (storage_id, project_id)
);
