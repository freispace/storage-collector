use std::{collections::HashMap, sync::Arc};
use chrono::Local;
use walkdir::WalkDir;
use tauri::Emitter;

use crate::{
    AppState,
    db::{models::LogLevel, queries},
    error::AppError,
    events::{LogEntryEvent, SchedulerTickEvent},
    tray::TrayState,
};

/// Determines which folder configs to process in a scheduler run.
pub enum RunScope {
    /// All configs whose effective schedule time matches the given local HH:MM.
    AllAtTime(String),
    /// Only configs for a specific storage-project pair (manual trigger).
    Specific { storage_id: String, project_id: String },
    /// All configs, regardless of schedule (used by trigger_all command).
    All,
}

/// Main collection job: scan folders, submit sizes, handle errors.
pub async fn run_tick(state: &Arc<AppState>, scope: RunScope) -> Result<(), AppError> {
    // Set tray to Active immediately
    state.tray.lock().await.set_state(TrayState::Active)?;

    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    if api_key.is_empty() {
        log_and_emit(state, LogLevel::Warning, "API key not configured — skipping collection", None).await?;
        state.tray.lock().await.set_state(TrayState::Idle)?;
        return Ok(());
    }

    // Load the relevant folder configs
    let (configs, is_specific) = match &scope {
        RunScope::AllAtTime(time) => {
            let global = queries::get_setting(&state.pool, "global_schedule_time").await?;
            let cfgs = queries::folder_configs_for_schedule(&state.pool, time, &global).await?;
            (cfgs, false)
        }
        RunScope::Specific { storage_id, project_id } => {
            let cfgs = queries::folder_configs_for_storage_project(&state.pool, storage_id, project_id).await?;
            (cfgs, true)
        }
        RunScope::All => {
            let cfgs = queries::list_folder_configs(&state.pool).await?;
            (cfgs, false)
        }
    };

    // For non-manual runs, skip storage-project pairs that have been disabled
    let configs = if is_specific {
        configs
    } else {
        let disabled: std::collections::HashSet<(String, String)> =
            queries::list_storage_project_settings(&state.pool)
                .await?
                .into_iter()
                .filter(|s| !s.enabled)
                .map(|s| (s.storage_id.clone(), s.project_id.clone()))
                .collect();
        configs
            .into_iter()
            .filter(|c| !disabled.contains(&(c.storage_id.clone(), c.project_id.clone())))
            .collect()
    };

    if configs.is_empty() {
        log_and_emit(state, LogLevel::Info, "No folder configs to process", None).await?;
        state.tray.lock().await.set_state(TrayState::Idle)?;
        return Ok(());
    }

    // Group by (storage_id, project_id)
    let mut groups: HashMap<(String, String), Vec<String>> = HashMap::new();
    for cfg in configs {
        groups
            .entry((cfg.storage_id.clone(), cfg.project_id.clone()))
            .or_default()
            .push(cfg.folder_path.clone());
    }

    let mut had_error = false;
    let mut had_warning = false;

    for ((storage_id, project_id), paths) in &groups {
        // (a) Accessibility check — if any folder is offline, skip this group entirely
        let mut all_accessible = true;
        for path in paths {
            if let Err(e) = std::fs::metadata(path) {
                log_and_emit(
                    state,
                    LogLevel::Error,
                    &format!("Folder inaccessible: {path}: {e}"),
                    Some(&format!(r#"{{"storage_id":"{storage_id}","project_id":"{project_id}","path":"{path}"}}"#)),
                )
                .await?;
                all_accessible = false;
                had_error = true;
            }
        }
        if !all_accessible {
            continue;
        }

        // (b) Size calculation — sum all files across all configured folders
        let total_bytes: u64 = paths.iter().map(|path| folder_size(path)).sum();

        // (c) Submission (rate-limited)
        let date = Local::now().date_naive().to_string();
        state.api_client.rate_limiter.acquire().await;

        match state
            .api_client
            .submit_statistics(&api_key, storage_id, project_id, &date, total_bytes)
            .await
        {
            Ok(_) => {
                let msg = format!(
                    "Submitted {} bytes for storage {storage_id} / project {project_id}",
                    total_bytes
                );
                log_and_emit(state, LogLevel::Info, &msg, None).await?;
                // Clear any previous pending submission for this date
                let _ = queries::delete_pending_for_date(&state.pool, storage_id, project_id, &date).await;
            }
            Err(AppError::ApiConflict(_)) => {
                // 409 — inactive storage or project, do not retry
                log_and_emit(
                    state,
                    LogLevel::Warning,
                    &format!("Skipped inactive storage {storage_id} / project {project_id}"),
                    None,
                )
                .await?;
                had_warning = true;
            }
            Err(e) => {
                log_and_emit(
                    state,
                    LogLevel::Warning,
                    &format!("Submission failed, queued for retry: {e}"),
                    Some(&format!(r#"{{"storage_id":"{storage_id}","project_id":"{project_id}","date":"{date}"}}"#)),
                )
                .await?;
                queries::insert_pending_submission(
                    &state.pool,
                    storage_id,
                    project_id,
                    &date,
                    total_bytes as i64,
                )
                .await?;
                had_warning = true;
            }
        }

        // Emit per-group tick event
        let _ = state.app_handle.emit(
            "scheduler_tick",
            SchedulerTickEvent {
                storage_id: storage_id.clone(),
                project_id: project_id.clone(),
                status: if had_error { "error" } else if had_warning { "warning" } else { "ok" }.to_string(),
            },
        );
    }

    // Final tray state based on results
    let final_state = if had_error {
        TrayState::Error
    } else if had_warning {
        TrayState::Warning
    } else {
        TrayState::Ok
    };
    state.tray.lock().await.set_state(final_state)?;

    // Prune log entries to last 10,000
    let _ = queries::prune_log_entries(&state.pool).await;

    Ok(())
}

/// Retry all pending submissions. Fires every 5 minutes regardless of auto_run.
pub async fn retry_pending(state: &Arc<AppState>) -> Result<(), AppError> {
    let api_key = queries::get_setting(&state.pool, "api_key").await?;
    if api_key.is_empty() {
        return Ok(());
    }

    let pending = queries::list_pending_submissions(&state.pool).await?;
    if pending.is_empty() {
        return Ok(());
    }

    log_and_emit(
        state,
        LogLevel::Info,
        &format!("Retrying {} pending submission(s)", pending.len()),
        None,
    )
    .await?;

    for sub in pending {
        state.api_client.rate_limiter.acquire().await;

        match state
            .api_client
            .submit_statistics(
                &api_key,
                &sub.storage_id,
                &sub.project_id,
                &sub.date,
                sub.size_bytes as u64,
            )
            .await
        {
            Ok(_) => {
                queries::delete_pending_submission(&state.pool, &sub.id).await?;
                log_and_emit(
                    state,
                    LogLevel::Info,
                    &format!(
                        "Retry succeeded for storage {} / project {} / date {}",
                        sub.storage_id, sub.project_id, sub.date
                    ),
                    None,
                )
                .await?;
            }
            Err(AppError::ApiConflict(_)) => {
                // 409 — delete without retrying further
                queries::delete_pending_submission(&state.pool, &sub.id).await?;
                log_and_emit(
                    state,
                    LogLevel::Warning,
                    &format!(
                        "Abandoned pending submission (inactive storage/project): {} / {}",
                        sub.storage_id, sub.project_id
                    ),
                    None,
                )
                .await?;
            }
            Err(e) => {
                let new_attempts = sub.attempts + 1;
                if new_attempts >= 10 {
                    queries::delete_pending_submission(&state.pool, &sub.id).await?;
                    log_and_emit(
                        state,
                        LogLevel::Error,
                        &format!(
                            "Giving up on pending submission after 10 attempts \
                             (storage {}, project {}, date {}): {e}",
                            sub.storage_id, sub.project_id, sub.date
                        ),
                        None,
                    )
                    .await?;
                    // Update tray to error state
                    let _ = state.tray.lock().await.set_state(TrayState::Error);
                } else {
                    queries::increment_pending_attempts(&state.pool, &sub.id).await?;
                    log_and_emit(
                        state,
                        LogLevel::Warning,
                        &format!(
                            "Retry {new_attempts}/10 failed for storage {} / project {}: {e}",
                            sub.storage_id, sub.project_id
                        ),
                        None,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Recursively sum the size of all files under `path`. Returns 0 on any error.
fn folder_size(path: &str) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Insert a log entry into the DB and emit a `LogEntryEvent` to the frontend.
pub async fn log_and_emit(
    state: &Arc<AppState>,
    level: LogLevel,
    message: &str,
    context: Option<&str>,
) -> Result<(), AppError> {
    tracing::info!("[{}] {}", level, message);

    let entry = queries::insert_log_entry(&state.pool, &level, message, context).await?;

    let _ = state.app_handle.emit("log_entry", LogEntryEvent(entry));

    Ok(())
}
