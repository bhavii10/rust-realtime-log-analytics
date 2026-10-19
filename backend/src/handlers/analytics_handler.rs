use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use sqlx::PgPool;

use crate::{
    models::analytics::{
        AnalyticsQuery,
        AlertQuery,
        AlertSummary,
        FullAnalyticsResponse,
    },
    services::{analytics_service, anomaly_service},
};

// ======================================================
// GET /api/analytics
// ======================================================

pub async fn get_analytics(
    State(pool): State<PgPool>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<FullAnalyticsResponse>, StatusCode> {
    analytics_service::get_analytics(&pool, &query)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Analytics error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ======================================================
// GET /api/alerts
// ======================================================

pub async fn get_alerts(
    State(pool): State<PgPool>,
    Query(query): Query<AlertQuery>,
) -> Result<Json<AlertSummary>, StatusCode> {
    anomaly_service::get_alerts(&pool, &query)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Alerts error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
