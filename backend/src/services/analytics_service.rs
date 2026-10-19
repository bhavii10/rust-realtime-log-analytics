use sqlx::{PgPool, QueryBuilder, Postgres};

use crate::models::analytics::{
    AnalyticsQuery,
    AnalyticsResponse,
    FullAnalyticsResponse,
    ServiceTraffic,
    StatusCodeDistribution,
    TopEndpoint,
};

// ======================================================
// GET FULL ANALYTICS
// ======================================================

pub async fn get_analytics(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<FullAnalyticsResponse, sqlx::Error> {
    let summary = get_summary(pool, query).await?;
    let status_distribution =
        get_status_distribution(pool, query).await?;
    let service_traffic =
        get_service_traffic(pool, query).await?;
    let top_endpoints =
        get_top_endpoints(pool, query).await?;

    Ok(FullAnalyticsResponse {
        summary,
        status_distribution,
        service_traffic,
        top_endpoints,
    })
}

// ======================================================
// SUMMARY STATS
// ======================================================

async fn get_summary(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<AnalyticsResponse, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            COUNT(*)::bigint AS total_requests,
            (COUNT(*) FILTER (WHERE l.status_code >= 500))::bigint AS total_errors,
            COALESCE(AVG(l.response_time_ms)::float8, 0) AS avg_latency,
            COALESCE(MAX(l.response_time_ms), 0)::bigint AS max_latency,
            COALESCE(MIN(l.response_time_ms), 0)::bigint AS min_latency,
            (COUNT(*) FILTER (WHERE l.response_time_ms > 2000))::bigint AS slow_requests,
            EXTRACT(EPOCH FROM (MAX(l.timestamp) - MIN(l.timestamp)))::float8 AS time_span_secs
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id"#
    );

    append_time_and_service_filter(&mut qb, query);

    #[derive(sqlx::FromRow)]
    struct Row {
        total_requests: i64,
        total_errors: i64,
        avg_latency: f64,
        max_latency: i64,
        min_latency: i64,
        slow_requests: i64,
        time_span_secs: Option<f64>,
    }

    let row = qb
        .build_query_as::<Row>()
        .fetch_one(pool)
        .await?;

    let error_rate = if row.total_requests > 0 {
        (row.total_errors as f64 / row.total_requests as f64) * 100.0
    } else {
        0.0
    };

    let rps = match row.time_span_secs {
        Some(secs) if secs > 0.0 => {
            row.total_requests as f64 / secs
        }
        _ => row.total_requests as f64,
    };

    Ok(AnalyticsResponse {
        total_requests: row.total_requests,
        total_errors: row.total_errors,
        error_rate: (error_rate * 100.0).round() / 100.0,
        average_latency_ms: (row.avg_latency * 100.0).round() / 100.0,
        max_latency_ms: row.max_latency,
        min_latency_ms: row.min_latency,
        requests_per_second: (rps * 100.0).round() / 100.0,
        slow_requests: row.slow_requests,
    })
}

// ======================================================
// STATUS CODE DISTRIBUTION
// ======================================================

async fn get_status_distribution(
    pool: &PgPool,
    query: &AnalyticsQuery,
) -> Result<Vec<StatusCodeDistribution>, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            l.status_code,
            COUNT(*)::bigint AS count,
            COALESCE(COUNT(*)::float8 * 100.0 / NULLIF(SUM(COUNT(*)) OVER(), 0)::float8, 0) AS percentage
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id"#
    );

    append_time_and_service_filter(&mut qb, query);
    qb.push(" GROUP BY l.status_code ORDER BY count DESC");

    qb.build_query_as::<StatusCodeDistribution>()
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
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            s.name AS service_name,
            COUNT(*)::bigint AS request_count,
            (COUNT(*) FILTER (WHERE l.status_code >= 500))::bigint AS error_count,
            COALESCE((COUNT(*) FILTER (WHERE l.status_code >= 500))::float8 * 100.0 / NULLIF(COUNT(*), 0)::float8, 0) AS error_rate,
            COALESCE(AVG(l.response_time_ms)::float8, 0) AS avg_latency_ms
        FROM logs l
        JOIN services s ON l.service_id = s.id"#
    );

    append_time_and_service_filter(&mut qb, query);
    qb.push(" GROUP BY s.name ORDER BY request_count DESC");

    qb.build_query_as::<ServiceTraffic>()
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
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            l.endpoint,
            l.method,
            COUNT(*)::bigint AS request_count,
            COALESCE(AVG(l.response_time_ms)::float8, 0) AS avg_latency_ms
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id"#
    );

    append_time_and_service_filter(&mut qb, query);
    qb.push(" GROUP BY l.endpoint, l.method ORDER BY request_count DESC LIMIT 10");

    qb.build_query_as::<TopEndpoint>()
        .fetch_all(pool)
        .await
}

// ======================================================
// HELPER: Append time + service filter
// ======================================================

fn append_time_and_service_filter(
    qb: &mut QueryBuilder<Postgres>,
    query: &AnalyticsQuery,
) {
    let mut has_where = false;

    if let Some(mins) = query.minutes {
        qb.push(" WHERE l.timestamp >= NOW() - make_interval(mins => ");
        qb.push_bind(mins as i32);
        qb.push(")");
        has_where = true;
    }

    if let Some(ref service) = query.service {
        if has_where {
            qb.push(" AND s.name = ");
        } else {
            qb.push(" WHERE s.name = ");
        }
        qb.push_bind(service.clone());
    }
}
