use serde::{Deserialize, Serialize};
use uuid::Uuid;

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