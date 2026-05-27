use std::sync::Arc;
use std::collections::HashSet;
use tauri::State;
use chrono::Utc;
use crate::{
    api::types::{FreispaceProject, FreispaceStorage, PaginatedResponse, StorageProjectItem},
    db::{models::CachedEntityName, queries},
    error::AppError,
    AppState,
};

async fn sync_projects_names(
    state: &Arc<AppState>,
    api_key: &str,
    since: Option<&str>,
) -> Result<(), AppError> {
    let mut known_project_ids = HashSet::<String>::new();
    let mut pending_parent_ids = HashSet::<String>::new();

    let mut page = 1u32;
    loop {
        let resp = state
            .api_client
            .fetch_projects_page_since(api_key, page, since)
            .await?;
        for project in &resp.data {
            known_project_ids.insert(project.id.clone());
            if let Some(parent_id) = project.parent_id.as_ref() {
                if parent_id != &project.id {
                    pending_parent_ids.insert(parent_id.clone());
                }
            }
            queries::upsert_entity_name(
                &state.pool,
                "project",
                &project.id,
                project.name.as_deref(),
                project.parent_id.as_deref(),
                project.number.as_deref(),
                project.color.as_deref(),
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

    // Ensure parent projects that are referenced but not present in the list response
    // are also fetched and cached (including parent chains).
    let mut parent_queue: Vec<String> = pending_parent_ids
        .into_iter()
        .filter(|id| !known_project_ids.contains(id))
        .collect();
    let mut idx = 0usize;
    while idx < parent_queue.len() {
        let parent_id = parent_queue[idx].clone();
        idx += 1;

        if known_project_ids.contains(&parent_id) {
            continue;
        }

        match state.api_client.fetch_project(api_key, &parent_id).await {
            Ok(project) => {
                if let Some(next_parent_id) = project.parent_id.as_ref() {
                    if next_parent_id != &project.id && !known_project_ids.contains(next_parent_id)
                    {
                        parent_queue.push(next_parent_id.clone());
                    }
                }

                known_project_ids.insert(project.id.clone());
                queries::upsert_entity_name(
                    &state.pool,
                    "project",
                    &project.id,
                    project.name.as_deref(),
                    project.parent_id.as_deref(),
                    project.number.as_deref(),
                    project.color.as_deref(),
                )
                .await?;
            }
            // A referenced parent may be inaccessible or gone; skip and continue.
            Err(AppError::Api { status, .. }) if status == 403 || status == 404 => {
                known_project_ids.insert(parent_id);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

async fn sync_storages_names(
    state: &Arc<AppState>,
    api_key: &str,
    since: Option<&str>,
) -> Result<(), AppError> {
    let mut page = 1u32;
    loop {
        let resp = state
            .api_client
            .fetch_storages_page_since(api_key, page, since)
            .await?;
        for storage in &resp.data {
            queries::upsert_entity_name(
                &state.pool,
                "storage",
                &storage.id,
                storage.name.as_deref(),
                None,
                None,
                None,
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

    Ok(())
}

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
    sync_projects_names(state, &api_key, projects_since.as_deref()).await?;
    queries::set_setting(&state.pool, "names_synced_projects", &today).await?;

    // ── Storages ──────────────────────────────────────────────────────────────
    let storages_since = queries::get_setting(&state.pool, "names_synced_storages").await.ok();
    let storages_since = storages_since.filter(|s| !s.is_empty());
    sync_storages_names(state, &api_key, storages_since.as_deref()).await?;
    queries::set_setting(&state.pool, "names_synced_storages", &today).await?;

    Ok(())
}

/// Force-refresh all project and storage names and metadata, updating existing entries.
pub(crate) async fn sync_names_full_inner(state: &Arc<AppState>) -> Result<(), AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    if api_key.is_empty() {
        return Ok(());
    }

    let today = Utc::now().date_naive().to_string();

    sync_projects_names(state, &api_key, None).await?;
    sync_storages_names(state, &api_key, None).await?;

    // Bump incremental sync markers after a full refresh.
    queries::set_setting(&state.pool, "names_synced_projects", &today).await?;
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
pub async fn sync_entity_names_full(state: State<'_, Arc<AppState>>) -> Result<(), AppError> {
    sync_names_full_inner(&state).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_entity_names(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CachedEntityName>, AppError> {
    queries::get_entity_names(&state.pool).await
}
