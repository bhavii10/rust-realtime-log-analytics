use sqlx::PgPool;
use uuid::Uuid;

use crate::models::analytics::{RealtimeMetrics, RealtimeQuery};

// ======================================================
// Phase 7 — Real-Time Analytics Processing Engine
//
// This module provides two capabilities:
//
// 1. compute_and_store_snapshot() — called by the background
//    task every 10 seconds. It queries logs from the last
//    10 seconds, computes summary metrics, and inserts a
//    snapshot row into the `analytics` table.
//
// 2. get_realtime_metrics() — called by the HTTP handler
//    to return the most recent metric snapshots.
// ======================================================

/// Compute metrics from the last `window_seconds` of logs
/// and insert a snapshot into the `analytics` table.
pub async fn compute_and_store_snapshot(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();

    // Single query: compute all metrics from recent logs
    let interval_str = format!("{} seconds", window_seconds);

    let row = sqlx::query_as::<_, SnapshotRow>(
        r#"
        SELECT
            COUNT(*)                                           AS request_count,
            COUNT(*) FILTER (WHERE status_code >= 500)         AS error_count,
            ROUND(
                COUNT(*) FILTER (WHERE status_code >= 500)::numeric
                / NULLIF(COUNT(*), 0) * 100, 2
            )::float8                                          AS error_rate,
            COALESCE(AVG(response_time_ms), 0)::float8         AS average_latency_ms,
            ROUND(
                COUNT(*)::numeric
                / NULLIF($2::numeric, 0), 2
            )::float8                                          AS requests_per_second,
            COUNT(*) FILTER (WHERE response_time_ms > 2000)    AS slow_request_count
        FROM logs
        WHERE timestamp >= NOW() - $1::INTERVAL
        "#,
    )
    .bind(&interval_str)
    .bind(window_seconds)
    .fetch_one(pool)
    .await?;

    // Insert the computed snapshot into analytics table
    sqlx::query(
        r#"
        INSERT INTO analytics (
            id,
            recorded_at,
            request_count,
            error_count,
            error_rate,
            average_latency_ms,
            requests_per_second,
            slow_request_count
        )
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(row.request_count)
    .bind(row.error_count)
    .bind(row.error_rate)
    .bind(row.average_latency_ms)
    .bind(row.requests_per_second)
    .bind(row.slow_request_count)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch the most recent real-time metric snapshots.
pub async fn get_realtime_metrics(
    pool: &PgPool,
    query: RealtimeQuery,
) -> Result<Vec<RealtimeMetrics>, sqlx::Error> {
    let limit = query.limit.unwrap_or(30).min(200).max(1);

    let metrics = sqlx::query_as::<_, RealtimeMetrics>(
        r#"
        SELECT
            id,
            recorded_at::text AS recorded_at,
            request_count,
            error_count,
            error_rate,
            average_latency_ms,
            requests_per_second,
            slow_request_count
        FROM analytics
        ORDER BY recorded_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

// Internal row type for the aggregation query
#[derive(sqlx::FromRow)]
struct SnapshotRow {
    request_count: i64,
    error_count: i64,
    error_rate: f64,
    average_latency_ms: f64,
    requests_per_second: f64,
    slow_request_count: i64,
}
