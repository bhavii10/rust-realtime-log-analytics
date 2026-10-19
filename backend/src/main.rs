mod config;
mod database;
mod errors;
mod handlers;
mod models;
mod routes;
mod services;

use axum::Router;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

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

    // CORS middleware for React frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Start anomaly detection background task
    let anomaly_pool = pool.clone();
    tokio::spawn(async move {
        println!("🔍 Anomaly detection started (every 30s)");
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            services::anomaly_service::check_anomalies(
                &anomaly_pool,
            )
            .await;
        }
    });

    // Create application routes
    let app = Router::new()
        .merge(routes::health::routes())
        .merge(routes::logs::routes())
        .merge(routes::services::routes())
        .merge(routes::analytics::routes())
        .layer(cors)
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
    println!("   GET  /api/health");
    println!("   GET  /api/services");
    println!("   GET  /api/logs");
    println!("   GET  /api/logs/search");
    println!("   POST /api/logs");
    println!("   GET  /api/analytics");
    println!("   GET  /api/alerts");

    axum::serve(listener, app)
        .await
        .expect("❌ Server failed");
}
