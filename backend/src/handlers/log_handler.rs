use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use sqlx::PgPool;

use crate::{
    models::log::{
        CreateLogRequest,
        LogResponse,
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
// GET /api/logs
// ======================================================

pub async fn get_logs(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<LogResponse>>, StatusCode> {
    log_service::get_logs(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}