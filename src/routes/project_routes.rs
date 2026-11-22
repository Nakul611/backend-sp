use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{
    controllers::{
        cloud_controller::provision_servers,
        execution_controller::{dispatch_execution, list_project_executions},
        project_controller::{create_project, get_project, list_projects, update_project_status},
        server_controller::{
            add_server, batch_import_servers, list_servers, ping_all_servers,
            terminate_idle_servers,
        },
    },
    state::AppState,
};

pub fn project_routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/{id}", get(get_project))
        .route("/projects/{id}/status", patch(update_project_status))
        .route("/projects/{id}/servers", get(list_servers).post(add_server))
        .route("/projects/{id}/servers/batch", post(batch_import_servers))
        .route("/projects/{id}/provision", post(provision_servers))
        .route("/projects/{id}/servers/terminate-idle", post(terminate_idle_servers))
        .route("/projects/{id}/servers/ping-all", post(ping_all_servers))
        .route("/projects/{id}/execute", post(dispatch_execution))
        .route("/projects/{id}/executions", get(list_project_executions))
}
