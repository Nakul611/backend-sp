use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    controllers::telemetry_controller::{
        get_agent_install_script, get_project_bootstrap_script, ingest_heartbeat,
    },
    state::AppState,
};

pub fn telemetry_routes() -> Router<AppState> {
    Router::new()
        .route("/telemetry/heartbeat", post(ingest_heartbeat))
        .route("/agent/install.sh", get(get_agent_install_script))
        .route("/projects/{id}/bootstrap-script", get(get_project_bootstrap_script))
}
