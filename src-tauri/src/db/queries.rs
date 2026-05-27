use sqlx::{SqlitePool, Row};
use chrono::Utc;
use uuid::Uuid;

use super::models::{CachedEntityName, FolderConfig, FolderConfigInput, LogEntry, LogLevel, PendingSubmission, StorageProjectSetting};
use crate::error::AppError;

// ── Settings ─────────────────────────────────────────────────────────────────

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<String, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("value")?)
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Folder configs ────────────────────────────────────────────────────────────

pub async fn list_folder_configs(pool: &SqlitePool) -> Result<Vec<FolderConfig>, AppError> {
    let rows = sqlx::query_as::<_, FolderConfig>(
        "SELECT id, storage_id, project_id, folder_path, custom_schedule, created_at, updated_at \
         FROM folder_configs ORDER BY storage_id, project_id, folder_path",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn upsert_folder_config(
    pool: &SqlitePool,
    input: &FolderConfigInput,
) -> Result<FolderConfig, AppError> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO folder_configs (id, storage_id, project_id, folder_path, custom_schedule, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(storage_id, project_id, folder_path) \
         DO UPDATE SET custom_schedule = excluded.custom_schedule, updated_at = excluded.updated_at",
    )
    .bind(&id)
    .bind(&input.storage_id)
    .bind(&input.project_id)
    .bind(&input.folder_path)
    .bind(&input.custom_schedule)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    // Return the actual row (id may differ if this was an update)
    let row = sqlx::query_as::<_, FolderConfig>(
        "SELECT id, storage_id, project_id, folder_path, custom_schedule, created_at, updated_at \
         FROM folder_configs WHERE storage_id = ? AND project_id = ? AND folder_path = ?",
    )
    .bind(&input.storage_id)
    .bind(&input.project_id)
    .bind(&input.folder_path)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_folder_config(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM folder_configs WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn folder_configs_for_schedule(
    pool: &SqlitePool,
    local_time: &str,
    global_schedule: &str,
) -> Result<Vec<FolderConfig>, AppError> {
    // Configs that explicitly set this time, OR have no custom schedule and the global matches
    let rows = sqlx::query_as::<_, FolderConfig>(
        "SELECT id, storage_id, project_id, folder_path, custom_schedule, created_at, updated_at \
         FROM folder_configs \
         WHERE custom_schedule = ? OR (custom_schedule IS NULL AND ? = ?)",
    )
    .bind(local_time)
    .bind(global_schedule)
    .bind(local_time)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn folder_configs_for_storage_project(
    pool: &SqlitePool,
    storage_id: &str,
    project_id: &str,
) -> Result<Vec<FolderConfig>, AppError> {
    let rows = sqlx::query_as::<_, FolderConfig>(
        "SELECT id, storage_id, project_id, folder_path, custom_schedule, created_at, updated_at \
         FROM folder_configs WHERE storage_id = ? AND project_id = ?",
    )
    .bind(storage_id)
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── Storage project settings ──────────────────────────────────────────────────

pub async fn list_storage_project_settings(pool: &SqlitePool) -> Result<Vec<StorageProjectSetting>, AppError> {
    let rows = sqlx::query_as::<_, StorageProjectSetting>(
        "SELECT storage_id, project_id, enabled FROM storage_project_settings",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn set_storage_project_enabled(
    pool: &SqlitePool,
    storage_id: &str,
    project_id: &str,
    enabled: bool,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO storage_project_settings (storage_id, project_id, enabled) VALUES (?, ?, ?) \
         ON CONFLICT(storage_id, project_id) DO UPDATE SET enabled = excluded.enabled",
    )
    .bind(storage_id)
    .bind(project_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_storage_project(
    pool: &SqlitePool,
    storage_id: &str,
    project_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM folder_configs WHERE storage_id = ? AND project_id = ?")
        .bind(storage_id)
        .bind(project_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM storage_project_settings WHERE storage_id = ? AND project_id = ?")
        .bind(storage_id)
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Pending submissions ───────────────────────────────────────────────────────

pub async fn insert_pending_submission(
    pool: &SqlitePool,
    storage_id: &str,
    project_id: &str,
    date: &str,
    size_bytes: i64,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT OR REPLACE INTO pending_submissions \
         (id, storage_id, project_id, date, size_bytes, attempts, last_attempted, created_at) \
         VALUES (?, ?, ?, ?, ?, 0, NULL, ?)",
    )
    .bind(&id)
    .bind(storage_id)
    .bind(project_id)
    .bind(date)
    .bind(size_bytes)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_pending_submissions(pool: &SqlitePool) -> Result<Vec<PendingSubmission>, AppError> {
    let rows = sqlx::query_as::<_, PendingSubmission>(
        "SELECT id, storage_id, project_id, date, size_bytes, attempts, last_attempted, created_at \
         FROM pending_submissions ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete_pending_submission(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM pending_submissions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_pending_for_date(
    pool: &SqlitePool,
    storage_id: &str,
    project_id: &str,
    date: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM pending_submissions WHERE storage_id = ? AND project_id = ? AND date = ?",
    )
    .bind(storage_id)
    .bind(project_id)
    .bind(date)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn increment_pending_attempts(
    pool: &SqlitePool,
    id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE pending_submissions SET attempts = attempts + 1, last_attempted = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Log entries ───────────────────────────────────────────────────────────────

pub async fn insert_log_entry(
    pool: &SqlitePool,
    level: &LogLevel,
    message: &str,
    context: Option<&str>,
) -> Result<LogEntry, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let level_str = level.as_str();

    sqlx::query(
        "INSERT INTO log_entries (id, level, message, context, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(level_str)
    .bind(message)
    .bind(context)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(LogEntry {
        id,
        level: level_str.to_string(),
        message: message.to_string(),
        context: context.map(str::to_string),
        created_at: now,
    })
}

pub async fn list_log_entries(
    pool: &SqlitePool,
    level_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<LogEntry>, AppError> {
    let rows = match level_filter {
        Some(level) => {
            sqlx::query_as::<_, LogEntry>(
                "SELECT id, level, message, context, created_at FROM log_entries \
                 WHERE level = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(level)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, LogEntry>(
                "SELECT id, level, message, context, created_at FROM log_entries \
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows)
}

pub async fn clear_log_entries(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM log_entries").execute(pool).await?;
    Ok(())
}

pub async fn prune_log_entries(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM log_entries WHERE id NOT IN \
         (SELECT id FROM log_entries ORDER BY created_at DESC LIMIT 10000)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── Entity name cache ─────────────────────────────────────────────────────────

pub async fn upsert_entity_name(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
    name: Option<&str>,
    parent_id: Option<&str>,
    project_number: Option<&str>,
    color: Option<&str>,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO entity_names (entity_type, entity_id, name, parent_id, project_number, color, fetched_at) VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(entity_type, entity_id) DO UPDATE SET \
         name = excluded.name, \
         parent_id = excluded.parent_id, \
         project_number = excluded.project_number, \
         color = excluded.color, \
         fetched_at = excluded.fetched_at",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(name)
    .bind(parent_id)
    .bind(project_number)
    .bind(color)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_entity_names(pool: &SqlitePool) -> Result<Vec<CachedEntityName>, AppError> {
    let rows = sqlx::query_as::<_, CachedEntityName>(
        "SELECT entity_type, entity_id, name, parent_id, project_number, color FROM entity_names",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
