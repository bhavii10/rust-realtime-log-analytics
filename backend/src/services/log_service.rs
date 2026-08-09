use sqlx::PgPool;
use uuid::Uuid;

use crate::models::log::{
    CreateLogRequest,
    LogResponse,
};

// ======================================================
// CREATE LOG
// ======================================================

pub async fn create_log(
    pool: &PgPool,
    request: CreateLogRequest,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO logs (
            id,
            timestamp,
            level,
            service_id,
            method,
            endpoint,
            status_code,
            response_time_ms,
            ip_address,
            message
        )
        VALUES (
            $1,
            $2::timestamptz,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9::inet,
            $10
        )
        "#,
    )
    .bind(id)
    .bind(&request.timestamp)
    .bind(&request.level)
    .bind(request.service_id)
    .bind(&request.method)
    .bind(&request.endpoint)
    .bind(request.status_code)
    .bind(request.response_time_ms)
    .bind(&request.ip_address)
    .bind(&request.message)
    .execute(pool)
    .await?;

    Ok(id)
}

// ======================================================
// GET LOGS
// ======================================================

pub async fn get_logs(
    pool: &PgPool,
) -> Result<Vec<LogResponse>, sqlx::Error> {
    let logs = sqlx::query_as::<_, LogResponse>(
        r#"
        SELECT
            id,
            timestamp::text AS timestamp,
            level,
            service_id,
            method,
            endpoint,
            status_code,
            response_time_ms,
            host(ip_address) AS ip_address,
            message
        FROM logs
        ORDER BY timestamp DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(logs)
}