use sqlx::PgPool;
use uuid::Uuid;

use crate::models::analytics::{Alert, AlertQuery, AlertSummary};

// ======================================================
// Phase 8 — Anomaly Detection Engine
//
// After each real-time metrics snapshot, this module checks
// for anomalies against configurable thresholds:
//
//   1. HIGH_ERROR_RATE  — error rate exceeds 10%
//   2. HIGH_LATENCY     — avg latency exceeds 2000ms
//   3. TRAFFIC_SPIKE    — requests/sec suddenly jumps
//                         (current > 3x the average of
//                          the previous 6 snapshots)
//
// When an anomaly is detected, an alert is stored in the
// `alerts` table. Duplicate alerts of the same type within
// the last 60 seconds are suppressed to avoid spam.
// ======================================================

// --- Thresholds ---
const ERROR_RATE_THRESHOLD: f64 = 10.0;       // 10%
const LATENCY_THRESHOLD: f64 = 2000.0;        // 2000 ms
const TRAFFIC_SPIKE_MULTIPLIER: f64 = 3.0;    // 3x average

/// Run all anomaly checks against the latest metrics snapshot.
/// Called by the background task right after storing a snapshot.
pub async fn check_anomalies(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Fetch the latest snapshot
    let latest = sqlx::query_as::<_, LatestSnapshot>(
        r#"
        SELECT
            error_rate,
            average_latency_ms,
            requests_per_second,
            request_count
        FROM analytics
        ORDER BY recorded_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    let snapshot = match latest {
        Some(s) => s,
        None => return Ok(()), // no data yet
    };

    // Skip anomaly checks if no requests in this window
    if snapshot.request_count == 0 {
        return Ok(());
    }

    // --- Check 1: High Error Rate ---
    if snapshot.error_rate > ERROR_RATE_THRESHOLD {
        create_alert_if_new(
            pool,
            "HIGH_ERROR_RATE",
            None,
            &format!(
                "🚨 Error rate spiked to {:.2}% (threshold: {}%)",
                snapshot.error_rate, ERROR_RATE_THRESHOLD
            ),
            severity_from_error_rate(snapshot.error_rate),
            ERROR_RATE_THRESHOLD,
            snapshot.error_rate,
        )
        .await?;
    }

    // --- Check 2: High Latency ---
    if snapshot.average_latency_ms > LATENCY_THRESHOLD {
        create_alert_if_new(
            pool,
            "HIGH_LATENCY",
            None,
            &format!(
                "🚨 Average latency reached {:.0}ms (threshold: {}ms)",
                snapshot.average_latency_ms, LATENCY_THRESHOLD
            ),
            severity_from_latency(snapshot.average_latency_ms),
            LATENCY_THRESHOLD,
            snapshot.average_latency_ms,
        )
        .await?;
    }

    // --- Check 3: Traffic Spike ---
    let avg_rps = get_historical_avg_rps(pool).await?;
    if avg_rps > 0.0 && snapshot.requests_per_second > avg_rps * TRAFFIC_SPIKE_MULTIPLIER {
        create_alert_if_new(
            pool,
            "TRAFFIC_SPIKE",
            None,
            &format!(
                "🚨 Traffic spike detected: {:.1} req/s (avg was {:.1} req/s, {}x threshold)",
                snapshot.requests_per_second,
                avg_rps,
                TRAFFIC_SPIKE_MULTIPLIER
            ),
            "HIGH",
            avg_rps * TRAFFIC_SPIKE_MULTIPLIER,
            snapshot.requests_per_second,
        )
        .await?;
    }

    Ok(())
}

/// Get the average requests_per_second from the 6 snapshots
/// before the latest one (used for traffic spike detection).
async fn get_historical_avg_rps(pool: &PgPool) -> Result<f64, sqlx::Error> {
    let row: (f64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(AVG(requests_per_second), 0)::float8
        FROM (
            SELECT requests_per_second
            FROM analytics
            ORDER BY recorded_at DESC
            OFFSET 1
            LIMIT 6
        ) recent
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Insert an alert only if no alert of the same type was
/// created in the last 60 seconds (deduplication).
async fn create_alert_if_new(
    pool: &PgPool,
    alert_type: &str,
    service_id: Option<Uuid>,
    message: &str,
    severity: &str,
    threshold_value: f64,
    actual_value: f64,
) -> Result<(), sqlx::Error> {
    // Check for recent duplicate
    let recent_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM alerts
        WHERE alert_type = $1
          AND created_at >= NOW() - INTERVAL '60 seconds'
        "#,
    )
    .bind(alert_type)
    .fetch_one(pool)
    .await?;

    if recent_count.0 > 0 {
        return Ok(()); // suppress duplicate
    }

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO alerts (
            id, alert_type, service_id, message,
            severity, threshold_value, actual_value
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(alert_type)
    .bind(service_id)
    .bind(message)
    .bind(severity)
    .bind(threshold_value)
    .bind(actual_value)
    .execute(pool)
    .await?;

    println!(
        "🚨 ALERT [{}] {} — severity: {} (actual: {:.2}, threshold: {:.2})",
        alert_type, message, severity, actual_value, threshold_value
    );

    Ok(())
}

/// Map error rate to severity level
fn severity_from_error_rate(rate: f64) -> &'static str {
    match rate {
        r if r >= 50.0 => "CRITICAL",
        r if r >= 25.0 => "HIGH",
        r if r >= 10.0 => "MEDIUM",
        _ => "LOW",
    }
}

/// Map latency to severity level
fn severity_from_latency(latency: f64) -> &'static str {
    match latency {
        l if l >= 5000.0 => "CRITICAL",
        l if l >= 3000.0 => "HIGH",
        l if l >= 2000.0 => "MEDIUM",
        _ => "LOW",
    }
}

// ======================================================
// ALERT RETRIEVAL
// ======================================================

/// Get recent alerts with optional filtering.
pub async fn get_alerts(
    pool: &PgPool,
    query: AlertQuery,
) -> Result<Vec<Alert>, sqlx::Error> {
    let limit = query.limit.unwrap_or(50).min(200).max(1);

    let mut sql = String::from(
        r#"
        SELECT
            id,
            alert_type,
            service_id,
            message,
            severity,
            threshold_value,
            actual_value,
            created_at::text AS created_at,
            resolved_at::text AS resolved_at
        FROM alerts
        WHERE 1=1
        "#,
    );

    if query.severity.is_some() {
        sql.push_str(" AND severity = $2");
    }
    if query.alert_type.is_some() {
        if query.severity.is_some() {
            sql.push_str(" AND alert_type = $3");
        } else {
            sql.push_str(" AND alert_type = $2");
        }
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT $1");

    // Build the query dynamically based on which filters exist
    match (&query.severity, &query.alert_type) {
        (Some(sev), Some(at)) => {
            sqlx::query_as::<_, Alert>(sqlx::AssertSqlSafe(sql))
                .bind(limit)
                .bind(sev)
                .bind(at)
                .fetch_all(pool)
                .await
        }
        (Some(sev), None) => {
            sqlx::query_as::<_, Alert>(sqlx::AssertSqlSafe(sql))
                .bind(limit)
                .bind(sev)
                .fetch_all(pool)
                .await
        }
        (None, Some(at)) => {
            sqlx::query_as::<_, Alert>(sqlx::AssertSqlSafe(sql))
                .bind(limit)
                .bind(at)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            sqlx::query_as::<_, Alert>(sqlx::AssertSqlSafe(sql))
                .bind(limit)
                .fetch_all(pool)
                .await
        }
    }
}

/// Get alert summary with counts by severity.
pub async fn get_alert_summary(
    pool: &PgPool,
) -> Result<AlertSummary, sqlx::Error> {
    let counts = sqlx::query_as::<_, AlertCounts>(
        r#"
        SELECT
            COUNT(*)                                          AS total_alerts,
            COUNT(*) FILTER (WHERE severity = 'CRITICAL')     AS critical,
            COUNT(*) FILTER (WHERE severity = 'HIGH')         AS high,
            COUNT(*) FILTER (WHERE severity = 'MEDIUM')       AS medium,
            COUNT(*) FILTER (WHERE severity = 'LOW')          AS low
        FROM alerts
        "#,
    )
    .fetch_one(pool)
    .await?;

    let recent = sqlx::query_as::<_, Alert>(
        r#"
        SELECT
            id,
            alert_type,
            service_id,
            message,
            severity,
            threshold_value,
            actual_value,
            created_at::text AS created_at,
            resolved_at::text AS resolved_at
        FROM alerts
        ORDER BY created_at DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(AlertSummary {
        total_alerts: counts.total_alerts,
        critical: counts.critical,
        high: counts.high,
        medium: counts.medium,
        low: counts.low,
        recent_alerts: recent,
    })
}

// Internal helper types
#[derive(sqlx::FromRow)]
struct LatestSnapshot {
    error_rate: f64,
    average_latency_ms: f64,
    requests_per_second: f64,
    request_count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertCounts {
    total_alerts: i64,
    critical: i64,
    high: i64,
    medium: i64,
    low: i64,
}
