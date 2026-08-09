use axum::{
    routing::post,
    Router,
};

use sqlx::PgPool;

use crate::handlers::log_handler;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/api/logs",
            post(log_handler::create_log)
                .get(log_handler::get_logs),
        )
}