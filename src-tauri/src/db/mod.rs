pub mod models;
pub mod queries;

use sqlx::SqlitePool;
use crate::error::AppError;

pub async fn init_db(app_data_dir: &std::path::Path) -> Result<SqlitePool, AppError> {
    std::fs::create_dir_all(app_data_dir)?;

    let db_path = app_data_dir.join("storage-collector.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
