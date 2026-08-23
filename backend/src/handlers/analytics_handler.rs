use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use sqlx::PgPool;

use crate::{
    models::analytics::{AnalyticsQuery, FullAnalyticsResponse},
    services::analytics_service,
};

// ======================================================
// GET /api/analytics
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
