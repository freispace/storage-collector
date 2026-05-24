pub mod job;

use std::sync::Arc;
use chrono::{Local, TimeZone, Timelike};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    db::queries,
    error::AppError,
    AppState,
};
use self::job::{run_tick, retry_pending, RunScope};

/// Initialise (or reinitialise) the scheduler.
/// Must be called on app start and whenever settings/configs change.
pub async fn setup_scheduler(state: Arc<AppState>) -> Result<JobScheduler, AppError> {
    let sched = JobScheduler::new()
        .await
        .map_err(|e| AppError::Scheduler(e.to_string()))?;

    let auto_run: bool = queries::get_setting(&state.pool, "scheduler_auto_run")
        .await
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);

    if auto_run {
        let global_time = queries::get_setting(&state.pool, "global_schedule_time")
            .await
            .unwrap_or_else(|_| "17:55".to_string());

        let configs = queries::list_folder_configs(&state.pool).await?;
        let mut unique_times: std::collections::HashSet<String> = std::collections::HashSet::new();
        for cfg in &configs {
            match &cfg.custom_schedule {
                Some(t) => { unique_times.insert(t.clone()); }
                None => { unique_times.insert(global_time.clone()); }
            }
        }

        for local_time in unique_times {
            let cron_expr = local_hhmm_to_utc_cron(&local_time)
                .unwrap_or_else(|_| "0 55 15 * * *".to_string());

            let state_clone = Arc::clone(&state);
            let local_time_clone = local_time.clone();

            let job = Job::new_async(cron_expr.as_str(), move |_, _| {
                let s = Arc::clone(&state_clone);
                let t = local_time_clone.clone();
                Box::pin(async move {
                    if let Err(e) = run_tick(&s, RunScope::AllAtTime(t)).await {
                        tracing::error!("Scheduler tick error: {e}");
                    }
                })
            })
            .map_err(|e| AppError::Scheduler(e.to_string()))?;

            sched.add(job).await.map_err(|e| AppError::Scheduler(e.to_string()))?;
        }
    }

    // Always register the pending-retry job (every 5 min), regardless of auto_run
    {
        let state_clone = Arc::clone(&state);
        let retry_job = Job::new_async("0 */5 * * * *", move |_, _| {
            let s = Arc::clone(&state_clone);
            Box::pin(async move {
                if let Err(e) = retry_pending(&s).await {
                    tracing::error!("Retry pending error: {e}");
                }
            })
        })
        .map_err(|e| AppError::Scheduler(e.to_string()))?;

        sched.add(retry_job).await.map_err(|e| AppError::Scheduler(e.to_string()))?;
    }

    sched.start().await.map_err(|e| AppError::Scheduler(e.to_string()))?;

    Ok(sched)
}

/// Convert a local HH:MM string to a UTC 6-field cron expression.
/// e.g. "17:55" local (UTC+2) → "0 55 15 * * *"
fn local_hhmm_to_utc_cron(local_hhmm: &str) -> Result<String, AppError> {
    let parts: Vec<&str> = local_hhmm.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::Config(format!("Invalid HH:MM: {local_hhmm}")));
    }
    let hour: u32 = parts[0].parse().map_err(|_| AppError::Config(format!("Invalid hour: {}", parts[0])))?;
    let minute: u32 = parts[1].parse().map_err(|_| AppError::Config(format!("Invalid minute: {}", parts[1])))?;

    let today = Local::now().date_naive();
    let naive = today.and_hms_opt(hour, minute, 0)
        .ok_or_else(|| AppError::Config(format!("Invalid time: {local_hhmm}")))?;

    let local_dt = Local.from_local_datetime(&naive)
        .single()
        .ok_or_else(|| AppError::Config("Ambiguous local time".to_string()))?;
    let utc_dt = local_dt.with_timezone(&chrono::Utc);

    Ok(format!(
        "0 {} {} * * *",
        utc_dt.time().minute(),
        utc_dt.time().hour()
    ))
}
