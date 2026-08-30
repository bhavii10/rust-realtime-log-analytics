use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use sqlx::PgPool;

use crate::{
    models::analytics::{
        AnalyticsQuery, FullAnalyticsResponse,
        RealtimeMetrics, RealtimeQuery,
        Alert, AlertQuery, AlertSummary,
    },
    services::{analytics_service, realtime_service, anomaly_service},
};

// ======================================================
// GET /api/analytics (Phase 6)
// ======================================================

pub async fn get_analytics(
    State(pool): State<PgPool>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<FullAnalyticsResponse>, StatusCode> {
    analytics_service::get_full_analytics(&pool, query)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("❌ Analytics error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ======================================================
// GET /api/analytics/realtime (Phase 7)
// ======================================================

pub async fn get_realtime_metrics(
    State(pool): State<PgPool>,
    Query(query): Query<RealtimeQuery>,
) -> Result<Json<Vec<RealtimeMetrics>>, StatusCode> {
    realtime_service::get_realtime_metrics(&pool, query)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("❌ Realtime metrics error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ======================================================
// GET /api/alerts (Phase 8)
// ======================================================

pub async fn get_alerts(
    State(pool): State<PgPool>,
    Query(query): Query<AlertQuery>,
) -> Result<Json<Vec<Alert>>, StatusCode> {
    anomaly_service::get_alerts(&pool, query)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("❌ Alerts error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ======================================================
// GET /api/alerts/summary (Phase 8)
// ======================================================

pub async fn get_alert_summary(
    State(pool): State<PgPool>,
) -> Result<Json<AlertSummary>, StatusCode> {
    anomaly_service::get_alert_summary(&pool)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("❌ Alert summary error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
