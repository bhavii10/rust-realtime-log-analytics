use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use sqlx::PgPool;

use crate::{
    models::log::{
        CreateLogRequest,
        LogFilter,
        LogResponse,
        PaginatedLogs,
    },
    services::log_service,
};

// ======================================================
// POST /api/logs
// ======================================================

pub async fn create_log(
    State(pool): State<PgPool>,
    Json(request): Json<CreateLogRequest>,
) -> Result<StatusCode, StatusCode> {
    log_service::create_log(&pool, request)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ======================================================
// GET /api/logs (basic - last 100)
// ======================================================

pub async fn get_logs(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<LogResponse>>, StatusCode> {
    log_service::get_logs(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ======================================================
// GET /api/logs/search?level=ERROR&service=payment-service&...
// Phase 5: Advanced Filtering & Pagination
// ======================================================

pub async fn search_logs(
    State(pool): State<PgPool>,
    Query(filter): Query<LogFilter>,
) -> Result<Json<PaginatedLogs>, StatusCode> {
    log_service::get_logs_filtered(&pool, filter)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
