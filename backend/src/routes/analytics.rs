use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::handlers::analytics_handler;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/api/analytics", get(analytics_handler::get_analytics))
        .route("/api/analytics/realtime", get(analytics_handler::get_realtime_metrics))
        .route("/api/alerts", get(analytics_handler::get_alerts))
        .route("/api/alerts/summary", get(analytics_handler::get_alert_summary))
}
