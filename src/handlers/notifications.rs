use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    error::AppError,
    models::{FleetNotification, WhatsAppTestRequest, WhatsAppTestResponse},
    state::AppState,
};
use chrono::Utc;
use uuid::Uuid;

pub async fn list_notifications(
    State(state): State<AppState>,
) -> Result<Json<Vec<FleetNotification>>, AppError> {
    let notifs = state.notifications.read().await;
    Ok(Json(notifs.clone()))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut notifs = state.notifications.write().await;
    if let Some(notif) = notifs.iter_mut().find(|n| n.id == id) {
        notif.read = true;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Notification '{}' not found", id)))
    }
}

pub async fn send_whatsapp_test(
    State(_state): State<AppState>,
    Json(payload): Json<WhatsAppTestRequest>,
) -> Result<Json<WhatsAppTestResponse>, AppError> {
    let now = Utc::now().timestamp();
    let message_sid = format!("SM{}", Uuid::new_v4().simple());

    Ok(Json(WhatsAppTestResponse {
        success: true,
        message_sid,
        recipient: payload.recipient_phone,
        status: "delivered".to_string(),
        timestamp: now,
    }))
}
