use std::sync::Arc;
use tauri::{AppHandle, State};
use crate::{
    db::{models::{FolderConfig, FolderConfigInput}, queries},
    error::AppError,
    AppState,
    scheduler,
};

#[tauri::command]
#[specta::specta]
pub async fn list_folder_configs(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<FolderConfig>, AppError> {
    queries::list_folder_configs(&state.pool).await
}

#[tauri::command]
#[specta::specta]
pub async fn upsert_folder_config(
    state: State<'_, Arc<AppState>>,
    input: FolderConfigInput,
) -> Result<FolderConfig, AppError> {
    let config = queries::upsert_folder_config(&state.pool, &input).await?;
    rebuild_scheduler(&*state).await?;
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_folder_config(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), AppError> {
    queries::delete_folder_config(&state.pool, &id).await?;
    rebuild_scheduler(&*state).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn pick_folder(app_handle: AppHandle) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let path = app_handle
        .dialog()
        .file()
        .blocking_pick_folder();

    Ok(path.map(|p| p.to_string()))
}

async fn rebuild_scheduler(state: &Arc<AppState>) -> Result<(), AppError> {
    let mut sched_guard = state.scheduler.lock().await;
    if let Some(mut old) = sched_guard.take() {
        let _ = old.shutdown().await;
    }
    let new_sched = scheduler::setup_scheduler(Arc::clone(state)).await?;
    *sched_guard = Some(new_sched);
    Ok(())
}
