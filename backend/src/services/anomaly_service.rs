use sqlx::PgPool;
use uuid::Uuid;

use crate::models::analytics::{Alert, AlertQuery, AlertSummary};

// ======================================================
// THRESHOLDS
// ======================================================

const ERROR_RATE_THRESHOLD: f64 = 10.0;
const HIGH_LATENCY_THRESHOLD: f64 = 2000.0;
const TRAFFIC_SPIKE_MULTIPLIER: f64 = 3.0;

// ======================================================
// CHECK ANOMALIES (runs every 30 seconds)
// ======================================================

pub async fn check_anomalies(pool: &PgPool) {
    if let Err(e) = check_error_rate(pool).await {
        eprintln!("Error checking error rate: {}", e);
    }

    if let Err(e) = check_high_latency(pool).await {
        eprintln!("Error checking latency: {}", e);
    }

    if let Err(e) = check_traffic_spike(pool).await {
        eprintln!("Error checking traffic: {}", e);
    }
}

// ======================================================
// CHECK: Error Rate > 10%
// ======================================================

async fn check_error_rate(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // Check error rate in the last 2 minutes
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            COUNT(*)::bigint,
            (COUNT(*) FILTER (WHERE status_code >= 500))::bigint
        FROM logs
        WHERE timestamp >= NOW() - INTERVAL '2 minutes'
        "#,
    )
    .fetch_one(pool)
    .await?;

    let (total, errors) = row;

    if total < 10 {
        return Ok(());
    }

    let error_rate = (errors as f64 / total as f64) * 100.0;

    if error_rate > ERROR_RATE_THRESHOLD {
        // Check if we already alerted recently (last 5 min)
        let recent: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM alerts
            WHERE alert_type = 'HIGH_ERROR_RATE'
              AND created_at >= NOW() - INTERVAL '5 minutes'
            "#,
        )
        .fetch_one(pool)
        .await?;

        if recent == 0 {
            create_alert(
                pool,
                "HIGH_ERROR_RATE",
                None,
                &format!(
                    "Error rate is {:.1}% (threshold: {}%)",
                    error_rate, ERROR_RATE_THRESHOLD
                ),
                if error_rate > 25.0 { "CRITICAL" } else { "HIGH" },
                ERROR_RATE_THRESHOLD,
                error_rate,
            )
            .await?;

            println!(
                "🚨 ALERT: High error rate {:.1}%",
                error_rate
            );
        }
    }

    Ok(())
}

// ======================================================
// CHECK: Average Latency > 2000ms
// ======================================================

async fn check_high_latency(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let avg_latency: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT AVG(response_time_ms)::float8
        FROM logs
        WHERE timestamp >= NOW() - INTERVAL '2 minutes'
        "#,
    )
    .fetch_one(pool)
    .await?;

    let avg = match avg_latency {
        Some(v) if v > 0.0 => v,
        _ => return Ok(()),
    };

    if avg > HIGH_LATENCY_THRESHOLD {
        let recent: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM alerts
            WHERE alert_type = 'HIGH_LATENCY'
              AND created_at >= NOW() - INTERVAL '5 minutes'
            "#,
        )
        .fetch_one(pool)
        .await?;

        if recent == 0 {
            create_alert(
                pool,
                "HIGH_LATENCY",
                None,
                &format!(
                    "Average latency is {:.0}ms (threshold: {}ms)",
                    avg, HIGH_LATENCY_THRESHOLD
                ),
                if avg > 4000.0 { "CRITICAL" } else { "HIGH" },
                HIGH_LATENCY_THRESHOLD,
                avg,
            )
            .await?;

            println!(
                "🚨 ALERT: High latency {:.0}ms",
                avg
            );
        }
    }

    Ok(())
}

// ======================================================
// CHECK: Traffic Spike (3x sudden increase)
// ======================================================

async fn check_traffic_spike(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // Compare last 1 min vs previous 5 min average
    #[derive(sqlx::FromRow)]
    struct TrafficRow {
        recent_count: i64,
        baseline_count: i64,
    }

    let row = sqlx::query_as::<_, TrafficRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM logs
             WHERE timestamp >= NOW() - INTERVAL '1 minute')::bigint AS recent_count,
            (SELECT COUNT(*) FROM logs
             WHERE timestamp >= NOW() - INTERVAL '6 minutes'
               AND timestamp < NOW() - INTERVAL '1 minute')::bigint AS baseline_count
        "#,
    )
    .fetch_one(pool)
    .await?;

    // Baseline is over 5 minutes, so per-minute average
    let baseline_per_min = row.baseline_count as f64 / 5.0;

    if baseline_per_min < 5.0 {
        return Ok(());
    }

    let spike_ratio =
        row.recent_count as f64 / baseline_per_min;

    if spike_ratio > TRAFFIC_SPIKE_MULTIPLIER {
        let recent: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM alerts
            WHERE alert_type = 'TRAFFIC_SPIKE'
              AND created_at >= NOW() - INTERVAL '5 minutes'
            "#,
        )
        .fetch_one(pool)
        .await?;

        if recent == 0 {
            create_alert(
                pool,
                "TRAFFIC_SPIKE",
                None,
                &format!(
                    "Traffic spiked {:.1}x (recent: {}/min, baseline: {:.0}/min)",
                    spike_ratio, row.recent_count, baseline_per_min
                ),
                "MEDIUM",
                baseline_per_min,
                row.recent_count as f64,
            )
            .await?;

            println!(
                "🚨 ALERT: Traffic spike {:.1}x",
                spike_ratio
            );
        }
    }

    Ok(())
}

// ======================================================
// CREATE ALERT
// ======================================================

async fn create_alert(
    pool: &PgPool,
    alert_type: &str,
    service_id: Option<Uuid>,
    message: &str,
    severity: &str,
    threshold: f64,
    actual: f64,
) -> Result<(), sqlx::Error> {
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
    .bind(threshold)
    .bind(actual)
    .execute(pool)
    .await?;

    Ok(())
}

// ======================================================
// GET ALERTS
// ======================================================

pub async fn get_alerts(
    pool: &PgPool,
    query: &AlertQuery,
) -> Result<AlertSummary, sqlx::Error> {
    use sqlx::{QueryBuilder, Postgres};

    let limit = query.limit.unwrap_or(50).min(200).max(1);

    // Get counts by severity
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts",
    )
    .fetch_one(pool)
    .await?;

    let critical: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity = 'CRITICAL'",
    )
    .fetch_one(pool)
    .await?;

    let high: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity = 'HIGH'",
    )
    .fetch_one(pool)
    .await?;

    let medium: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity = 'MEDIUM'",
    )
    .fetch_one(pool)
    .await?;

    let low: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity = 'LOW'",
    )
    .fetch_one(pool)
    .await?;

    // Get recent alerts with optional filters
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            id,
            alert_type,
            service_id,
            message,
            severity,
            threshold_value,
            actual_value,
            created_at::text AS created_at,
            resolved_at::text AS resolved_at
        FROM alerts"#
    );

    let mut has_where = false;

    if let Some(ref severity_filter) = query.severity {
        qb.push(" WHERE severity = ");
        qb.push_bind(severity_filter.clone());
        has_where = true;
    }

    if let Some(ref alert_type) = query.alert_type {
        if has_where {
            qb.push(" AND alert_type = ");
        } else {
            qb.push(" WHERE alert_type = ");
        }
        qb.push_bind(alert_type.clone());
    }

    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(limit);

    let recent_alerts = qb
        .build_query_as::<Alert>()
        .fetch_all(pool)
        .await?;

    Ok(AlertSummary {
        total_alerts: total,
        critical,
        high,
        medium,
        low,
        recent_alerts,
    })
}
