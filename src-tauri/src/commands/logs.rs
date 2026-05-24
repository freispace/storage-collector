use std::sync::Arc;
use tauri::State;
use crate::{db::{models::LogEntry, queries}, error::AppError, AppState};

#[tauri::command]
#[specta::specta]
pub async fn list_log_entries(
    state: State<'_, Arc<AppState>>,
    level_filter: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<LogEntry>, AppError> {
    queries::list_log_entries(&state.pool, level_filter.as_deref(), limit, offset).await
}

#[tauri::command]
#[specta::specta]
pub async fn clear_log_entries(state: State<'_, Arc<AppState>>) -> Result<(), AppError> {
    queries::clear_log_entries(&state.pool).await
}
