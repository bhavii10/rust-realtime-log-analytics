use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ======================================================
// ANALYTICS RESPONSE (Phase 6)
// ======================================================

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub total_requests: i64,
    pub total_errors: i64,
    pub error_rate: f64,
    pub average_latency_ms: f64,
    pub max_latency_ms: i64,
    pub min_latency_ms: i64,
    pub requests_per_second: f64,
    pub slow_requests: i64,
}

// ======================================================
// STATUS CODE DISTRIBUTION
// ======================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StatusCodeDistribution {
    pub status_code: i32,
    pub count: i64,
    pub percentage: f64,
}

// ======================================================
// SERVICE TRAFFIC
// ======================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ServiceTraffic {
    pub service_name: String,
    pub request_count: i64,
    pub error_count: i64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
}

// ======================================================
// TOP ENDPOINTS
// ======================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TopEndpoint {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub avg_latency_ms: f64,
}

// ======================================================
// FULL ANALYTICS (combined response)
// ======================================================

#[derive(Debug, Serialize)]
pub struct FullAnalyticsResponse {
    pub summary: AnalyticsResponse,
    pub status_distribution: Vec<StatusCodeDistribution>,
    pub service_traffic: Vec<ServiceTraffic>,
    pub top_endpoints: Vec<TopEndpoint>,
}

// ======================================================
// ANALYTICS QUERY PARAMS
// ======================================================

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Time window in minutes (default: all time)
    pub minutes: Option<i64>,
}

// ======================================================
// REAL-TIME METRICS SNAPSHOT (Phase 7)
// ======================================================

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct RealtimeMetrics {
    pub id: Uuid,
    pub recorded_at: String,
    pub request_count: i64,
    pub error_count: i64,
    pub error_rate: f64,
    pub average_latency_ms: f64,
    pub requests_per_second: f64,
    pub slow_request_count: i64,
}

// ======================================================
// REAL-TIME METRICS QUERY PARAMS (Phase 7)
// ======================================================

#[derive(Debug, Deserialize)]
pub struct RealtimeQuery {
    /// Number of recent snapshots to return (default: 30)
    pub limit: Option<i64>,
}

// ======================================================
// ALERT MODEL (Phase 8)
// ======================================================

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub alert_type: String,
    pub service_id: Option<Uuid>,
    pub message: String,
    pub severity: String,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

// ======================================================
// ALERT QUERY PARAMS (Phase 8)
// ======================================================

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    /// Filter by severity: LOW, MEDIUM, HIGH, CRITICAL
    pub severity: Option<String>,
    /// Filter by alert type: HIGH_ERROR_RATE, HIGH_LATENCY, TRAFFIC_SPIKE
    pub alert_type: Option<String>,
    /// Number of alerts to return (default: 50)
    pub limit: Option<i64>,
}

// ======================================================
// ALERT SUMMARY (Phase 8)
// ======================================================

#[derive(Debug, Serialize)]
pub struct AlertSummary {
    pub total_alerts: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
    pub recent_alerts: Vec<Alert>,
}
