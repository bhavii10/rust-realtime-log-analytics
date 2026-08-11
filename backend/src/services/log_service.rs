use sqlx::{PgPool, QueryBuilder, Postgres};
use uuid::Uuid;

use crate::models::log::{
    CreateLogRequest,
    LogFilter,
    LogResponse,
    PaginatedLogs,
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
// GET LOGS (basic - no filters)
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

// ======================================================
// GET LOGS WITH FILTERS & PAGINATION (Phase 5)
// ======================================================

pub async fn get_logs_filtered(
    pool: &PgPool,
    filter: LogFilter,
) -> Result<PaginatedLogs, sqlx::Error> {
    let page = filter.page.unwrap_or(1).max(1);
    let limit = filter.limit.unwrap_or(50).min(200).max(1);
    let offset = (page - 1) * limit;

    // ---- COUNT QUERY ----
    let mut count_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COUNT(*) FROM logs l LEFT JOIN services s ON l.service_id = s.id"
    );

    append_filters(&mut count_builder, &filter);

    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(pool)
        .await?;

    // ---- DATA QUERY ----
    let mut data_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT
            l.id,
            l.timestamp::text AS timestamp,
            l.level,
            l.service_id,
            l.method,
            l.endpoint,
            l.status_code,
            l.response_time_ms,
            host(l.ip_address) AS ip_address,
            l.message
        FROM logs l
        LEFT JOIN services s ON l.service_id = s.id"#
    );

    append_filters(&mut data_builder, &filter);

    data_builder.push(" ORDER BY l.timestamp DESC");
    data_builder.push(" LIMIT ");
    data_builder.push_bind(limit);
    data_builder.push(" OFFSET ");
    data_builder.push_bind(offset);

    let logs = data_builder
        .build_query_as::<LogResponse>()
        .fetch_all(pool)
        .await?;

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(PaginatedLogs {
        logs,
        total,
        page,
        limit,
        total_pages,
    })
}

// ======================================================
// HELPER: Append WHERE filters to QueryBuilder
// ======================================================

fn append_filters(
    builder: &mut QueryBuilder<Postgres>,
    filter: &LogFilter,
) {
    let mut has_condition = false;

    macro_rules! add_condition {
        ($condition:expr, $value:expr) => {
            if has_condition {
                builder.push(" AND ");
            } else {
                builder.push(" WHERE ");
                has_condition = true;
            }
            builder.push($condition);
            builder.push_bind($value);
        };
    }

    if let Some(ref level) = filter.level {
        add_condition!("l.level = ", level.clone());
    }
    if let Some(ref service) = filter.service {
        add_condition!("s.name = ", service.clone());
    }
    if let Some(ref method) = filter.method {
        add_condition!("l.method = ", method.clone());
    }
    if let Some(ref endpoint) = filter.endpoint {
        let pattern = format!("%{}%", endpoint);
        add_condition!("l.endpoint ILIKE ", pattern);
    }
    if let Some(status) = filter.status {
        add_condition!("l.status_code = ", status);
    }
    if let Some(min_latency) = filter.min_latency {
        add_condition!("l.response_time_ms >= ", min_latency);
    }
    if let Some(max_latency) = filter.max_latency {
        add_condition!("l.response_time_ms <= ", max_latency);
    }
}
