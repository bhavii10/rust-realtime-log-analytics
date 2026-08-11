use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ======================================================
// CREATE LOG REQUEST
// ======================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLogRequest {
    pub timestamp: String,
    pub level: String,
    pub service_id: Uuid,
    pub method: String,
    pub endpoint: String,
    pub status_code: i32,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub message: Option<String>,
}

// ======================================================
// LOG RESPONSE
// ======================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogResponse {
    pub id: Uuid,
    pub timestamp: String,
    pub level: String,
    pub service_id: Uuid,
    pub method: String,
    pub endpoint: String,
    pub status_code: i32,
    pub response_time_ms: i64,
    pub ip_address: Option<String>,
    pub message: Option<String>,
}

// ======================================================
// LOG FILTER (Phase 5)
// ======================================================

#[derive(Debug, Deserialize)]
pub struct LogFilter {
    pub level: Option<String>,
    pub service: Option<String>,
    pub method: Option<String>,
    pub endpoint: Option<String>,
    pub status: Option<i32>,
    pub min_latency: Option<i64>,
    pub max_latency: Option<i64>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ======================================================
// PAGINATED RESPONSE (Phase 5)
// ======================================================

#[derive(Debug, Serialize)]
pub struct PaginatedLogs {
    pub logs: Vec<LogResponse>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}
