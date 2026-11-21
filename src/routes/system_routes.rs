use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{
    controllers::{
        credential_controller::{create_credential, list_credentials},
        notification_controller::{
            list_notifications, mark_notification_read, send_whatsapp_test,
        },
        system_controller::{health_check, ws_handler},
        user_controller::{create_user, list_users},
    },
    state::AppState,
};

pub fn system_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", get(list_users).post(create_user))
        .route("/credentials", get(list_credentials).post(create_credential))
        .route("/notifications", get(list_notifications))
        .route("/notifications/{id}/read", patch(mark_notification_read))
        .route("/notifications/whatsapp-test", post(send_whatsapp_test))
        .route("/ws", get(ws_handler))
}
