CREATE TABLE IF NOT EXISTS storage_project_settings (
    storage_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    enabled    INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (storage_id, project_id)
);
