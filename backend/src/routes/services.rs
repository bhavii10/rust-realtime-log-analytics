use axum::{
    routing::get,
    Router,
};

use sqlx::PgPool;

use crate::handlers::service_handler;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/api/services",
            get(service_handler::get_services),
        )
}