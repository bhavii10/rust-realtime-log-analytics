use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ServiceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

pub async fn get_services(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ServiceResponse>>, StatusCode> {
    let services = sqlx::query_as!(
        ServiceResponse,
        r#"
        SELECT
            id,
            name,
            description
        FROM services
        ORDER BY name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(services))
}