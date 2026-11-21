use axum::Json;
use chrono::Utc;
use crate::models::HealthResponse;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "stackplane-backend".to_string(),
        version: "0.1.0".to_string(),
        timestamp: Utc::now().timestamp(),
    })
}
