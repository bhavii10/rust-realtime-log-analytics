use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::handlers::analytics_handler;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/api/analytics",
            get(analytics_handler::get_analytics),
        )
        .route(
            "/api/alerts",
            get(analytics_handler::get_alerts),
        )
}
