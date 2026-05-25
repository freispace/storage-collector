use std::sync::Arc;
use tauri::State;
use crate::{
    error::AppError,
    AppState,
    scheduler::job::{run_tick, RunScope},
};

#[tauri::command]
#[specta::specta]
pub async fn trigger_all(state: State<'_, Arc<AppState>>) -> Result<(), AppError> {
    run_tick(&state, RunScope::All).await
}

#[tauri::command]
#[specta::specta]
pub async fn trigger_storage_project(
    state: State<'_, Arc<AppState>>,
    storage_id: String,
    project_id: String,
) -> Result<(), AppError> {
    run_tick(
        &state,
        RunScope::Specific { storage_id, project_id },
    )
    .await
}
