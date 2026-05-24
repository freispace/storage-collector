use std::sync::Arc;
use tauri::State;
use crate::{
    api::types::{FreispaceProject, FreispaceStorage, PaginatedResponse, StorageProjectItem},
    db::queries,
    error::AppError,
    AppState,
};

#[tauri::command]
#[specta::specta]
pub async fn fetch_projects_page(
    state: State<'_, Arc<AppState>>,
    page: u32,
) -> Result<PaginatedResponse<FreispaceProject>, AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    state.api_client.fetch_projects_page(&api_key, page).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_storages_page(
    state: State<'_, Arc<AppState>>,
    page: u32,
) -> Result<PaginatedResponse<FreispaceStorage>, AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    state.api_client.fetch_storages_page(&api_key, page).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_storage_projects_page(
    state: State<'_, Arc<AppState>>,
    page: u32,
) -> Result<PaginatedResponse<StorageProjectItem>, AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    state.api_client.fetch_storage_projects_page(&api_key, page).await
}
