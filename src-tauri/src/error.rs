use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("freispace API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("freispace API conflict (inactive storage or project): {0}")]
    ApiConflict(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// Tauri commands must return serializable errors
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

// specta must know the TypeScript shape — errors are represented as strings
impl specta::Type for AppError {
    fn definition(types: &mut specta::Types) -> specta::datatype::DataType {
        String::definition(types)
    }
}
