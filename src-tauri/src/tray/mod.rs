pub mod icons;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tokio::sync::Mutex;

use icons::TrayIcons;
use crate::error::AppError;

/// The five visual states the tray icon can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayState {
    /// No API key or no folder configs configured.
    Idle,
    /// Last run completed with no issues.
    Ok,
    /// Last run had warnings (e.g. API retry queued) but no hard errors.
    Warning,
    /// Last run had errors (e.g. inaccessible folder).
    Error,
    /// A collection/submission job is currently running.
    Active,
}

pub struct TrayManager {
    icons: TrayIcons,
    current_state: TrayState,
    tray: tauri::tray::TrayIcon,
}

impl TrayManager {
    /// Switch to a new state, updating the tray icon and tooltip.
    /// Skips the syscall if the state is already set.
    pub fn set_state(&mut self, state: TrayState) -> Result<(), AppError> {
        if self.current_state == state {
            return Ok(());
        }
        self.current_state = state.clone();

        let icon = match &state {
            TrayState::Idle => &self.icons.idle,
            TrayState::Ok => &self.icons.ok,
            TrayState::Warning => &self.icons.warning,
            TrayState::Error => &self.icons.error,
            TrayState::Active => &self.icons.active,
        };

        let tooltip = match &state {
            TrayState::Idle => "freispace Storage Collector",
            TrayState::Ok => "freispace Storage Collector — OK",
            TrayState::Warning => "freispace Storage Collector — warnings",
            TrayState::Error => "freispace Storage Collector — errors",
            TrayState::Active => "freispace Storage Collector — running…",
        };

        self.tray.set_icon(Some(icon.clone())).map_err(|e| AppError::Config(e.to_string()))?;
        self.tray.set_tooltip(Some(tooltip)).map_err(|e| AppError::Config(e.to_string()))?;

        Ok(())
    }

    pub fn current_state(&self) -> &TrayState {
        &self.current_state
    }
}

/// Build the system tray and return a `TrayManager` wrapped in `Arc<Mutex<>>`.
pub fn build_tray(
    app: &AppHandle,
    icons: TrayIcons,
) -> Result<Arc<Mutex<TrayManager>>, AppError> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| AppError::Config(e.to_string()))?;
    let logs_item = MenuItem::with_id(app, "logs", "Logs", true, None::<&str>)
        .map_err(|e| AppError::Config(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Config(e.to_string()))?;

    let menu = Menu::with_items(app, &[&settings_item, &logs_item, &quit_item])
        .map_err(|e| AppError::Config(e.to_string()))?;

    let idle_icon = icons.idle.clone();

    let tray = TrayIconBuilder::new()
        .icon(idle_icon)
        .tooltip("freispace Storage Collector")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event({
            let app = app.clone();
            move |_, event| match event.id().as_ref() {
                "settings" => show_named_window(&app, "settings"),
                "logs" => show_named_window(&app, "logs"),
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event({
            let app = app.clone();
            move |_, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    show_named_window(&app, "settings");
                }
            }
        })
        .build(app)
        .map_err(|e| AppError::Config(e.to_string()))?;

    let manager = TrayManager {
        icons,
        current_state: TrayState::Idle,
        tray,
    };

    Ok(Arc::new(Mutex::new(manager)))
}

fn show_named_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
}
