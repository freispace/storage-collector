mod api;
mod commands;
mod db;
mod error;
mod events;
mod scheduler;
mod tray;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;

use api::client::FreispaceClient;
#[cfg(debug_assertions)]
use events::{LogEntryEvent, SchedulerTickEvent};
use tray::{TrayManager, TrayState};

/// Shared application state. Managed as `Arc<AppState>` via Tauri.
/// Commands receive `State<'_, Arc<AppState>>`.
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub api_client: Arc<FreispaceClient>,
    pub tray: Arc<Mutex<TrayManager>>,
    /// Guarded so the scheduler can be rebuilt at runtime.
    pub scheduler: Mutex<Option<JobScheduler>>,
    pub app_handle: tauri::AppHandle,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Export TypeScript bindings in debug builds only
    #[cfg(debug_assertions)]
    export_bindings();

    // DSN is baked in at compile time: SENTRY_DSN=https://...@sentry.io/... cargo build
    // If the env var is absent the guard is a no-op and Sentry stays disabled.
    let _sentry_guard = sentry::init(sentry::ClientOptions {
        dsn: option_env!("SENTRY_DSN").and_then(|s| s.parse().ok()),
        release: sentry::release_name!(),
        ..Default::default()
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            for label in ["settings", "logs"] {
                if let Some(window) = app.get_webview_window(label) {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Hide from macOS Dock — run as a menu-bar-only app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let app_handle = app.handle().clone();

            // Resolve app data directory and initialise SQLite
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let pool = tauri::async_runtime::block_on(db::init_db(&app_data_dir))
                .expect("failed to initialise database");

            let api_client = Arc::new(FreispaceClient::new().expect("failed to build HTTP client"));
            let icons = tray::icons::load_icons();
            let tray_manager = tray::build_tray(&app_handle, icons)
                .expect("failed to build system tray");

            let state = Arc::new(AppState {
                pool,
                api_client,
                tray: tray_manager,
                scheduler: Mutex::new(None),
                app_handle: app_handle.clone(),
            });

            // Start the scheduler
            let sched = tauri::async_runtime::block_on(
                scheduler::setup_scheduler(Arc::clone(&state))
            )
            .expect("failed to start scheduler");

            tauri::async_runtime::block_on(async {
                *state.scheduler.lock().await = Some(sched);
            });

            // Determine initial tray state
            tauri::async_runtime::block_on(async {
                let api_key = db::queries::get_setting(&state.pool, "api_key")
                    .await
                    .unwrap_or_default();
                let has_configs = !db::queries::list_folder_configs(&state.pool)
                    .await
                    .unwrap_or_default()
                    .is_empty();
                let initial = if api_key.is_empty() || !has_configs {
                    TrayState::Idle
                } else {
                    TrayState::Ok
                };
                let _ = state.tray.lock().await.set_state(initial);
            });

            app_handle.manage(state);

            // On Linux/GTK `visible: false` in tauri.conf.json is not always
            // honoured; hide both windows explicitly so they don't open on launch.
            #[cfg(target_os = "linux")]
            for label in ["settings", "logs"] {
                if let Some(window) = app.get_webview_window(label) {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_api_key,
            commands::settings::set_api_key,
            commands::settings::get_global_schedule,
            commands::settings::set_global_schedule,
            commands::settings::get_scheduler_auto_run,
            commands::settings::set_scheduler_auto_run,
            commands::settings::get_launch_at_startup,
            commands::settings::set_launch_at_startup,
            commands::projects::fetch_projects_page,
            commands::projects::fetch_storages_page,
            commands::projects::fetch_storage_projects_page,
            commands::projects::sync_entity_names,
            commands::projects::sync_entity_names_full,
            commands::projects::get_entity_names,
            commands::folders::list_folder_configs,
            commands::folders::upsert_folder_config,
            commands::folders::delete_folder_config,
            commands::folders::pick_folder,
            commands::folders::list_storage_project_settings,
            commands::folders::set_storage_project_enabled,
            commands::folders::remove_storage_project,
            commands::scheduler::trigger_all,
            commands::scheduler::trigger_storage_project,
            commands::logs::list_log_entries,
            commands::logs::clear_log_entries,
            commands::logs::save_log_file,
            commands::settings::show_logs_window,
        ])
        .on_window_event(|window, event| {
            // Hide to tray instead of quitting when window close button is clicked
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error building Tauri application");

    app.run(|_app_handle, _event| {});
}

#[cfg(debug_assertions)]
fn export_bindings() {
    use tauri_specta::{collect_commands, collect_events, Builder};
    use specta_typescript::Typescript;

    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::settings::get_api_key,
            commands::settings::set_api_key,
            commands::settings::get_global_schedule,
            commands::settings::set_global_schedule,
            commands::settings::get_scheduler_auto_run,
            commands::settings::set_scheduler_auto_run,
            commands::settings::get_launch_at_startup,
            commands::settings::set_launch_at_startup,
            commands::projects::fetch_projects_page,
            commands::projects::fetch_storages_page,
            commands::projects::fetch_storage_projects_page,
            commands::projects::sync_entity_names,
            commands::projects::sync_entity_names_full,
            commands::projects::get_entity_names,
            commands::folders::list_folder_configs,
            commands::folders::upsert_folder_config,
            commands::folders::delete_folder_config,
            commands::folders::pick_folder,
            commands::folders::list_storage_project_settings,
            commands::folders::set_storage_project_enabled,
            commands::folders::remove_storage_project,
            commands::scheduler::trigger_all,
            commands::scheduler::trigger_storage_project,
            commands::logs::list_log_entries,
            commands::logs::clear_log_entries,
            commands::logs::save_log_file,
            commands::settings::show_logs_window,
        ])
        .events(collect_events![LogEntryEvent, SchedulerTickEvent])
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("failed to export TypeScript bindings");
}
