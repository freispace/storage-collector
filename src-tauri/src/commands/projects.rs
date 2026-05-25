use std::sync::Arc;
use tauri::State;
use chrono::Utc;
use crate::{
    api::types::{FreispaceProject, FreispaceStorage, PaginatedResponse, StorageProjectItem},
    db::{models::CachedEntityName, queries},
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

/// Fetch all new/updated project and storage names and upsert them into the local cache.
/// Uses `updated_since` (stored in settings) so only changed entries are re-fetched.
pub(crate) async fn sync_names_inner(state: &Arc<AppState>) -> Result<(), AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    if api_key.is_empty() {
        return Ok(());
    }

    let today = Utc::now().date_naive().to_string();

    // ── Projects ──────────────────────────────────────────────────────────────
    let projects_since = queries::get_setting(&state.pool, "names_synced_projects").await.ok();
    let projects_since = projects_since.filter(|s| !s.is_empty());

    let mut page = 1u32;
    loop {
        let resp = state
            .api_client
            .fetch_projects_page_since(&api_key, page, projects_since.as_deref())
            .await?;
        for project in &resp.data {
            queries::upsert_entity_name(
                &state.pool,
                "project",
                &project.id,
                project.name.as_deref(),
            )
            .await?;
        }
        let has_next = resp
            .pagination_links
            .as_ref()
            .and_then(|l| l.next.as_ref())
            .is_some();
        if !has_next {
            break;
        }
        page += 1;
    }
    queries::set_setting(&state.pool, "names_synced_projects", &today).await?;

    // ── Storages ──────────────────────────────────────────────────────────────
    let storages_since = queries::get_setting(&state.pool, "names_synced_storages").await.ok();
    let storages_since = storages_since.filter(|s| !s.is_empty());

    let mut page = 1u32;
    loop {
        let resp = state
            .api_client
            .fetch_storages_page_since(&api_key, page, storages_since.as_deref())
            .await?;
        for storage in &resp.data {
            queries::upsert_entity_name(
                &state.pool,
                "storage",
                &storage.id,
                storage.name.as_deref(),
            )
            .await?;
        }
        let has_next = resp
            .pagination_links
            .as_ref()
            .and_then(|l| l.next.as_ref())
            .is_some();
        if !has_next {
            break;
        }
        page += 1;
    }
    queries::set_setting(&state.pool, "names_synced_storages", &today).await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn sync_entity_names(state: State<'_, Arc<AppState>>) -> Result<(), AppError> {
    sync_names_inner(&state).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_entity_names(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CachedEntityName>, AppError> {
    queries::get_entity_names(&state.pool).await
}
