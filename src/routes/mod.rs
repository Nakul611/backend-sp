pub mod auth_routes;
pub mod cloud_routes;
pub mod project_routes;
pub mod server_routes;
pub mod system_routes;
pub mod telemetry_routes;
pub mod vcs_routes;

use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .nest("/auth", auth_routes::auth_routes())
        .merge(cloud_routes::cloud_routes())
        .merge(project_routes::project_routes())
        .merge(server_routes::server_routes())
        .merge(vcs_routes::vcs_routes())
        .merge(telemetry_routes::telemetry_routes())
        .merge(system_routes::system_routes());

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
