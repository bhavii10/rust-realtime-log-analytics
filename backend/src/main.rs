mod config;
mod database;
mod errors;
mod handlers;
mod models;
mod routes;
mod services;

use axum::Router;
use sqlx::PgPool;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Get PostgreSQL connection URL
    let database_url = config::database_url();

    println!("🔌 Connecting to PostgreSQL...");

    // Create database connection pool
    let pool = database::postgres::create_pool(&database_url)
        .await
        .expect("❌ Failed to connect to PostgreSQL");

    println!("✅ PostgreSQL connected!");

    // ======================================================
    // Phase 7 & 8 — Spawn background processing engine
    // ======================================================
    spawn_realtime_engine(pool.clone());

    // Create application routes
    let app = Router::new()
        .merge(routes::logs::routes())
        .merge(routes::services::routes())
        .merge(routes::analytics::routes())
        .with_state(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind(
        "127.0.0.1:8080",
    )
    .await
    .expect("❌ Failed to bind server");

    println!("🚀 Server running at:");
    println!("   http://127.0.0.1:8080");

    println!("\nAvailable endpoints:");
    println!("   GET  /api/services");
    println!("   GET  /api/logs");
    println!("   GET  /api/logs/search?level=ERROR&service=payment-service&status=500&min_latency=2000&page=1&limit=50");
    println!("   GET  /api/analytics");
    println!("   GET  /api/analytics?service=payment-service&minutes=60");
    println!("   GET  /api/analytics/realtime          (Phase 7 — real-time metrics)");
    println!("   GET  /api/analytics/realtime?limit=60");
    println!("   GET  /api/alerts                      (Phase 8 — anomaly alerts)");
    println!("   GET  /api/alerts?severity=HIGH&alert_type=HIGH_ERROR_RATE");
    println!("   GET  /api/alerts/summary              (Phase 8 — alert summary)");
    println!("   POST /api/logs");

    axum::serve(listener, app)
        .await
        .expect("❌ Server failed");
}

// ======================================================
// Background Real-Time Processing Engine
//
// Every 10 seconds:
//   1. Compute metrics from logs in the last 10s window
//   2. Store snapshot in `analytics` table
//   3. Run anomaly detection checks
//   4. Store any triggered alerts in `alerts` table
// ======================================================

fn spawn_realtime_engine(pool: PgPool) {
    tokio::spawn(async move {
        println!("⚡ Real-time processing engine started (10s interval)");
        println!("🔍 Anomaly detection active — thresholds:");
        println!("   • Error rate  > 10%");
        println!("   • Avg latency > 2000ms");
        println!("   • Traffic     > 3x historical average");

        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            // Phase 7 — compute and store real-time metrics
            match services::realtime_service::compute_and_store_snapshot(&pool, 10).await {
                Ok(()) => {
                    // Phase 8 — run anomaly checks on the fresh snapshot
                    if let Err(e) = services::anomaly_service::check_anomalies(&pool).await {
                        eprintln!("❌ Anomaly detection error: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Real-time snapshot error: {e}");
                }
            }
        }
    });
}
