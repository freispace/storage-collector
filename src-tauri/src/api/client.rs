use std::time::Duration;
use reqwest::{Client, StatusCode};
use tokio::sync::Mutex;
use tokio::time::Instant;

use super::types::{
    ApiErrorBody, FreispaceProject, FreispaceStorage, PaginatedResponse, StatisticsPayload,
    StorageProjectItem,
};
use crate::error::AppError;

const BASE_URL: &str = "https://api.freispace.com/v1";

/// Rate limiter that enforces a minimum interval between calls.
/// Used exclusively for submission API calls (1 req/sec = 60/min).
pub struct SubmissionRateLimiter {
    last_call: Mutex<Option<Instant>>,
    min_interval: Duration,
}

impl SubmissionRateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_call: Mutex::new(None),
            min_interval,
        }
    }

    /// Waits until `min_interval` has elapsed since the last `acquire()` call.
    pub async fn acquire(&self) {
        let mut last = self.last_call.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }
}

/// HTTP client for the freispace REST API.
pub struct FreispaceClient {
    http: Client,
    pub rate_limiter: SubmissionRateLimiter,
}

impl FreispaceClient {
    pub fn new() -> Result<Self, AppError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .https_only(true)
            .build()?;

        Ok(Self {
            http,
            // Enforce 1 request per second for submissions (= 60/min limit)
            rate_limiter: SubmissionRateLimiter::new(Duration::from_secs(1)),
        })
    }

    // ── Paginated list endpoints (no rate limiting — called from UI) ──────────

    pub async fn fetch_projects_page(
        &self,
        api_key: &str,
        page: u32,
    ) -> Result<PaginatedResponse<FreispaceProject>, AppError> {
        let url = format!("{BASE_URL}/projects?page={page}&limit=25");
        self.get_paginated(&url, api_key).await
    }

    pub async fn fetch_storages_page(
        &self,
        api_key: &str,
        page: u32,
    ) -> Result<PaginatedResponse<FreispaceStorage>, AppError> {
        let url = format!("{BASE_URL}/storages?page={page}&limit=25");
        self.get_paginated(&url, api_key).await
    }

    pub async fn fetch_storage_projects_page(
        &self,
        api_key: &str,
        page: u32,
    ) -> Result<PaginatedResponse<StorageProjectItem>, AppError> {
        let url = format!("{BASE_URL}/storage-projects?page={page}");
        self.get_paginated(&url, api_key).await
    }

    async fn get_paginated<T>(&self, url: &str, api_key: &str) -> Result<PaginatedResponse<T>, AppError>
    where
        T: serde::de::DeserializeOwned + specta::Type + Clone + std::fmt::Debug,
    {
        let resp = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::Api {
                status: status.as_u16(),
                message: "Rate limit exceeded".to_string(),
            });
        }
        if !status.is_success() {
            let body: ApiErrorBody = resp.json().await.unwrap_or(ApiErrorBody { message: None });
            return Err(AppError::Api {
                status: status.as_u16(),
                message: body.message.unwrap_or_else(|| status.to_string()),
            });
        }

        Ok(resp.json::<PaginatedResponse<T>>().await?)
    }

    // ── Statistics submission (rate-limited, called from scheduler) ───────────

    /// Submit folder size statistics for a storage-project on a given date.
    /// The caller must call `rate_limiter.acquire().await` before this method.
    pub async fn submit_statistics(
        &self,
        api_key: &str,
        storage_id: &str,
        project_id: &str,
        date: &str,
        size_bytes: u64,
    ) -> Result<StatusCode, AppError> {
        let url = format!(
            "{BASE_URL}/storages/{storage_id}/projects/{project_id}/statistics/{date}"
        );
        let payload = StatisticsPayload { size: size_bytes };

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status = resp.status();

        match status {
            StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(status),
            StatusCode::CONFLICT => {
                // 409 = inactive storage or project — do not retry
                Err(AppError::ApiConflict(
                    "Inactive storage or project".to_string(),
                ))
            }
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::Api {
                status: 429,
                message: "Rate limit exceeded".to_string(),
            }),
            _ => {
                let body: ApiErrorBody = resp.json().await.unwrap_or(ApiErrorBody { message: None });
                Err(AppError::Api {
                    status: status.as_u16(),
                    message: body.message.unwrap_or_else(|| status.to_string()),
                })
            }
        }
    }
}

impl Default for FreispaceClient {
    fn default() -> Self {
        Self::new().expect("failed to build HTTP client")
    }
}
