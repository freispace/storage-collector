use tauri::image::Image;

/// Load all 5 tray icon states from embedded PNG bytes at startup.
/// Each image is loaded once and cached for the lifetime of the app.
pub fn load_icons() -> TrayIcons {
    TrayIcons {
        idle: load_png(include_bytes!("../../icons/tray-idle.png")),
        ok: load_png(include_bytes!("../../icons/tray-ok.png")),
        warning: load_png(include_bytes!("../../icons/tray-warning.png")),
        error: load_png(include_bytes!("../../icons/tray-error.png")),
        active: load_png(include_bytes!("../../icons/tray-active.png")),
    }
}

fn load_png(bytes: &[u8]) -> Image<'static> {
    let img = image::load_from_memory(bytes)
        .expect("failed to load tray PNG")
        .into_rgba8();
    let (w, h) = img.dimensions();
    // `new_owned` takes ownership of the Vec<u8>, producing Image<'static>
    Image::new_owned(img.into_raw(), w, h)
}

pub struct TrayIcons {
    pub idle: Image<'static>,
    pub ok: Image<'static>,
    pub warning: Image<'static>,
    pub error: Image<'static>,
    pub active: Image<'static>,
}
