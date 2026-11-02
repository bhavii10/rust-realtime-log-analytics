use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

use std::time::Duration;

// ======================================================
// Phase 11 — Optimized Connection Pool
//
// Tuned for high-throughput log ingestion:
//   - max_connections: 50 (up from 20)
//   - min_connections: 5  (up from 2)
//   - acquire_timeout: 30s (up from 10s)
//   - idle_timeout: 10 min (new — reclaim idle connections)
//   - max_lifetime: 30 min (new — prevent stale connections)
// ======================================================

pub async fn create_pool(
    database_url: &str,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}
