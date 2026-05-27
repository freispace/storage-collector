use serde::{Deserialize, Serialize};
use specta::Type;

// ── Freispace API response types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FreispaceProject {
    pub id: String,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub number: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FreispaceStorage {
    pub id: String,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StorageProjectItem {
    pub id: String,
    pub storage_id: Option<String>,
    pub project_id: Option<String>,
    #[specta(type = Option<f64>)]
    pub storage_size_estimated: Option<i64>,
    #[specta(type = Option<f64>)]
    pub storage_size_current: Option<i64>,
}

/// Pagination links returned by all freispace list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaginationLinks {
    pub first: Option<String>,
    pub last: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

/// Pagination meta returned by all freispace list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub from: Option<u32>,
    pub per_page: u32,
    pub to: Option<u32>,
}

/// Generic paginated response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaginatedResponse<T: specta::Type + Clone + std::fmt::Debug> {
    pub data: Vec<T>,
    pub pagination_links: Option<PaginationLinks>,
    pub pagination_meta: Option<PaginationMeta>,
}

/// Request body for upserting storage statistics.
#[derive(Debug, Serialize)]
pub struct StatisticsPayload {
    /// Storage size in bytes (field name is `size` per freispace API spec).
    pub size: u64,
}

/// API error response body shape.
#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub message: Option<String>,
}
