use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{
    controllers::server_controller::{
        delete_server, get_server_telemetry, reboot_server, start_server, stop_server,
        terminal_exec_server,
    },
    state::AppState,
};

pub fn server_routes() -> Router<AppState> {
    Router::new()
        .route("/servers/{id}", delete(delete_server))
        .route("/servers/{id}/start", post(start_server))
        .route("/servers/{id}/stop", post(stop_server))
        .route("/servers/{id}/reboot", post(reboot_server))
        .route("/servers/{id}/telemetry", get(get_server_telemetry))
        .route("/servers/{id}/terminal-exec", post(terminal_exec_server))
}
