use std::sync::Arc;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
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

#[tauri::command]
#[specta::specta]
pub async fn save_log_file(
    app: tauri::AppHandle,
    content: String,
    default_name: String,
) -> Result<bool, AppError> {
    let (tx, rx) = oneshot::channel();

    app.dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("Text Files", &["txt"])
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let path = rx.await.map_err(|_| AppError::Config("dialog channel closed".into()))?;

    match path {
        Some(file_path) => {
            let path_buf = file_path
                .into_path()
                .map_err(|_| AppError::Config("unexpected file URL from dialog".into()))?;
            std::fs::write(&path_buf, content)?;
            Ok(true)
        }
        None => Ok(false),
    }
}
