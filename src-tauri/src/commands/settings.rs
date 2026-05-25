use std::sync::Arc;
use tauri::State;
use tauri_plugin_autostart::ManagerExt;
use crate::{db::queries, error::AppError, AppState, scheduler};

#[tauri::command]
#[specta::specta]
pub async fn get_api_key(state: State<'_, Arc<AppState>>) -> Result<String, AppError> {
    queries::get_setting(&state.pool, "api_key").await
}

#[tauri::command]
#[specta::specta]
pub async fn set_api_key(state: State<'_, Arc<AppState>>, key: String) -> Result<(), AppError> {
    queries::set_setting(&state.pool, "api_key", &key).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_global_schedule(state: State<'_, Arc<AppState>>) -> Result<String, AppError> {
    queries::get_setting(&state.pool, "global_schedule_time").await
}

#[tauri::command]
#[specta::specta]
pub async fn set_global_schedule(
    state: State<'_, Arc<AppState>>,
    time: String,
) -> Result<(), AppError> {
    validate_hhmm(&time)?;
    queries::set_setting(&state.pool, "global_schedule_time", &time).await?;
    rebuild_scheduler(&state).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_scheduler_auto_run(state: State<'_, Arc<AppState>>) -> Result<bool, AppError> {
    let val = queries::get_setting(&state.pool, "scheduler_auto_run").await?;
    Ok(val.parse::<bool>().unwrap_or(true))
}

#[tauri::command]
#[specta::specta]
pub async fn set_scheduler_auto_run(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), AppError> {
    queries::set_setting(&state.pool, "scheduler_auto_run", &enabled.to_string()).await?;
    rebuild_scheduler(&state).await
}

fn validate_hhmm(time: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::Config(format!("Invalid time format: {time}")));
    }
    let h: u32 = parts[0].parse().map_err(|_| AppError::Config("Invalid hour".to_string()))?;
    let m: u32 = parts[1].parse().map_err(|_| AppError::Config("Invalid minute".to_string()))?;
    if h > 23 || m > 59 {
        return Err(AppError::Config(format!("Time out of range: {time}")));
    }
    Ok(())
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

#[tauri::command]
#[specta::specta]
pub fn get_launch_at_startup(app: tauri::AppHandle) -> Result<bool, AppError> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| AppError::Config(e.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn set_launch_at_startup(app: tauri::AppHandle, enabled: bool) -> Result<(), AppError> {
    if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|e| AppError::Config(e.to_string()))
}
