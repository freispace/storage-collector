use serde::{Deserialize, Serialize};
use specta::Type;

/// A configured folder path for a storage-project pair.
#[derive(Debug, Clone, Serialize, Deserialize, Type, sqlx::FromRow)]
pub struct FolderConfig {
    pub id: String,
    pub storage_id: String,
    pub project_id: String,
    pub folder_path: String,
    /// HH:MM local time override, or None to use the global schedule.
    pub custom_schedule: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input DTO for creating or updating a folder config.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FolderConfigInput {
    pub storage_id: String,
    pub project_id: String,
    pub folder_path: String,
    /// HH:MM or None to inherit the global schedule.
    pub custom_schedule: Option<String>,
}

/// A submission that failed and is queued for retry.
#[derive(Debug, Clone, Serialize, Deserialize, Type, sqlx::FromRow)]
pub struct PendingSubmission {
    pub id: String,
    pub storage_id: String,
    pub project_id: String,
    /// YYYY-MM-DD of the original scan.
    pub date: String,
    pub size_bytes: i64,
    pub attempts: i64,
    pub last_attempted: Option<String>,
    pub created_at: String,
}

/// Log severity level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A cached storage or project name fetched from the freispace API.
#[derive(Debug, Clone, Serialize, Deserialize, Type, sqlx::FromRow)]
pub struct CachedEntityName {
    pub entity_type: String,
    pub entity_id: String,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub project_number: Option<String>,
    pub color: Option<String>,
}

/// Enabled/disabled state for a storage-project pair.
#[derive(Debug, Clone, Serialize, Deserialize, Type, sqlx::FromRow)]
pub struct StorageProjectSetting {
    pub storage_id: String,
    pub project_id: String,
    pub enabled: bool,
}

/// A single log entry.
#[derive(Debug, Clone, Serialize, Deserialize, Type, sqlx::FromRow)]
pub struct LogEntry {
    pub id: String,
    pub level: String,
    pub message: String,
    pub context: Option<String>,
    pub created_at: String,
}
