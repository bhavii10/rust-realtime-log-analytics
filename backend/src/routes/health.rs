use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use sqlx::PgPool;

pub fn routes() -> Router<PgPool> {
    Router::new().route("/api/health", get(health_check))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "rust-log-analytics"
    }))
}
