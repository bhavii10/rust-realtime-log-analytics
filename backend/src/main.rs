mod config;
mod database;
mod handlers;
mod models;
mod routes;
mod services;

use axum::Router;

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
    println!("   POST /api/logs");

    axum::serve(listener, app)
        .await
        .expect("❌ Server failed");
}