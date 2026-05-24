use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::db::models::LogEntry;

/// Emitted after every log entry is inserted.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct LogEntryEvent(pub LogEntry);

/// Emitted after each storage-project is processed in a scheduler tick.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct SchedulerTickEvent {
    pub storage_id: String,
    pub project_id: String,
    /// "ok" | "warning" | "error"
    pub status: String,
}
