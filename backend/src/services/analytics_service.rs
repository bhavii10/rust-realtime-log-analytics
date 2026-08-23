use sqlx::PgPool;

use crate::models::analytics::{
    AnalyticsQuery, AnalyticsResponse, FullAnalyticsResponse,
    ServiceTraffic, StatusCodeDistribution, TopEndpoint,
};

pub async fn get_full_analytics(
    pool: &PgPool,
    query: AnalyticsQuery,
) -> Result<FullAnalyticsResponse, sqlx::Error> {
    let summary = get_summary(pool, &query).await?;
    let status_distribution = get_status_distribution(pool, &query).await?;
    let service_traffic = get_service_traffic(pool, &query).await?;
    let top_endpoints = get_top_endpoints(pool, &query).await?;

    Ok(FullAnalyticsResponse {
        summary,
        status_distribution,
        service_traffic,
        top_endpoints,
    })
}

// ======================================================
// SUMMARY METRICS
// ======================================================

async fn get_summary(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<AnalyticsResponse, sqlx::Error> {
    let time_filter = time_condition(query.minutes);
    let service_filter = service_condition(query.service.as_deref());

    let sql = format!(
        r#"
        SELECT
            COUNT(*)                                            AS total_requests,
            COUNT(*) FILTER (WHERE l.status_code >= 500)       AS total_errors,
            ROUND(
                COUNT(*) FILTER (WHERE l.status_code >= 500)::numeric
                / NULLIF(COUNT(*), 0) * 100, 2
            )::float8                                          AS error_rate,
            COALESCE(AVG(l.response_time_ms), 0)::float8       AS average_latency_ms,
            COALESCE(MAX(l.response_time_ms), 0)               AS max_latency_ms,
            COALESCE(MIN(l.response_time_ms), 0)               AS min_latency_ms,
            ROUND(
                COUNT(*)::numeric
                / NULLIF(
                    EXTRACT(EPOCH FROM (MAX(l.timestamp) - MIN(l.timestamp))),
                    0
                ), 2
            )::float8                                          AS requests_per_second,
            COUNT(*) FILTER (WHERE l.response_time_ms > 2000)  AS slow_requests
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id
        WHERE 1=1 {time_filter} {service_filter}
        "#
    );

    let row = sqlx::query_as::<_, SummaryRow>(sqlx::AssertSqlSafe(sql))
        .fetch_optional(pool)
        .await?
        .unwrap_or(SummaryRow {
            total_requests: 0,
            total_errors: 0,
            error_rate: 0.0,
            average_latency_ms: 0.0,
            max_latency_ms: 0,
            min_latency_ms: 0,
            requests_per_second: 0.0,
            slow_requests: 0,
        });

    Ok(AnalyticsResponse {
        total_requests: row.total_requests,
        total_errors: row.total_errors,
        error_rate: row.error_rate,
        average_latency_ms: row.average_latency_ms,
        max_latency_ms: row.max_latency_ms,
        min_latency_ms: row.min_latency_ms,
        requests_per_second: row.requests_per_second,
        slow_requests: row.slow_requests,
    })
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    total_requests: i64,
    total_errors: i64,
    error_rate: f64,
    average_latency_ms: f64,
    max_latency_ms: i64,
    min_latency_ms: i64,
    requests_per_second: f64,
    slow_requests: i64,
}

// ======================================================
// STATUS CODE DISTRIBUTION
// ======================================================

async fn get_status_distribution(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<Vec<StatusCodeDistribution>, sqlx::Error> {
    let time_filter = time_condition(query.minutes);
    let service_filter = service_condition(query.service.as_deref());

    let time_filter2 = time_condition_alias(query.minutes, "l2");
    let service_filter2 = service_condition_alias(query.service.as_deref(), "s2");

    let sql = format!(
        r#"
        SELECT
            l.status_code,
            COUNT(*)                                                        AS count,
            ROUND(COUNT(*)::numeric / NULLIF(total.cnt, 0) * 100, 2)::float8 AS percentage
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id
        CROSS JOIN (
            SELECT COUNT(*) AS cnt
            FROM logs l2
            LEFT JOIN services s2 ON l2.service_id = s2.id
            WHERE 1=1 {time_filter2} {service_filter2}
        ) total
        WHERE 1=1 {time_filter} {service_filter}
        GROUP BY l.status_code, total.cnt
        ORDER BY count DESC
        "#
    );

    sqlx::query_as::<_, StatusCodeDistribution>(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
}

// ======================================================
// SERVICE TRAFFIC
// ======================================================

async fn get_service_traffic(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<Vec<ServiceTraffic>, sqlx::Error> {
    let time_filter = time_condition(query.minutes);
    let service_filter = service_condition(query.service.as_deref());

    let sql = format!(
        r#"
        SELECT
            s.name                                                      AS service_name,
            COUNT(*)                                                    AS request_count,
            COUNT(*) FILTER (WHERE l.status_code >= 500)               AS error_count,
            ROUND(
                COUNT(*) FILTER (WHERE l.status_code >= 500)::numeric
                / NULLIF(COUNT(*), 0) * 100, 2
            )::float8                                                   AS error_rate,
            ROUND(AVG(l.response_time_ms)::numeric, 2)::float8         AS avg_latency_ms
        FROM logs l
        JOIN services s ON l.service_id = s.id
        WHERE 1=1 {time_filter} {service_filter}
        GROUP BY s.name
        ORDER BY request_count DESC
        "#
    );

    sqlx::query_as::<_, ServiceTraffic>(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
}

// ======================================================
// TOP ENDPOINTS
// ======================================================

async fn get_top_endpoints(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<Vec<TopEndpoint>, sqlx::Error> {
    let time_filter = time_condition(query.minutes);
    let service_filter = service_condition(query.service.as_deref());

    let sql = format!(
        r#"
        SELECT
            l.endpoint,
            l.method,
            COUNT(*)                                            AS request_count,
            ROUND(AVG(l.response_time_ms)::numeric, 2)::float8 AS avg_latency_ms
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id
        WHERE 1=1 {time_filter} {service_filter}
        GROUP BY l.endpoint, l.method
        ORDER BY request_count DESC
        LIMIT 10
        "#
    );

    sqlx::query_as::<_, TopEndpoint>(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
}

// ======================================================
// HELPERS
// ======================================================

fn time_condition(minutes: Option<i64>) -> String {
    match minutes {
        Some(m) => format!("AND l.timestamp >= NOW() - INTERVAL '{m} minutes'"),
        None => String::new(),
    }
}

fn time_condition_alias(minutes: Option<i64>, alias: &str) -> String {
    match minutes {
        Some(m) => format!("AND {alias}.timestamp >= NOW() - INTERVAL '{m} minutes'"),
        None => String::new(),
    }
}

fn service_condition(service: Option<&str>) -> String {
    match service {
        Some(s) => format!("AND s.name = '{s}'"),
        None => String::new(),
    }
}

fn service_condition_alias(service: Option<&str>, alias: &str) -> String {
    match service {
        Some(s) => format!("AND {alias}.name = '{s}'"),
        None => String::new(),
    }
}
