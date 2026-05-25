-- Cached storage and project names fetched from the freispace API.
-- Updated incrementally using updated_since to avoid re-fetching unchanged entries.
CREATE TABLE IF NOT EXISTS entity_names (
    entity_type TEXT NOT NULL CHECK (entity_type IN ('storage', 'project')),
    entity_id   TEXT NOT NULL,
    name        TEXT,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id)
);
