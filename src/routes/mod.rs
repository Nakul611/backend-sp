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

use crate::{
    middlewares::require_auth,
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 1. Public Unauthenticated Routes
    let public_routes = Router::new()
        .nest("/auth", auth_routes::public_auth_routes())
        .merge(system_routes::public_system_routes())
        .merge(vcs_routes::public_vcs_routes())
        .merge(telemetry_routes::telemetry_routes());

    // 2. Protected Routes (Requiring valid JWT Authorization header)
    let protected_routes = Router::new()
        .nest("/auth", auth_routes::protected_auth_routes())
        .merge(cloud_routes::cloud_routes())
        .merge(project_routes::project_routes())
        .merge(server_routes::server_routes())
        .merge(vcs_routes::protected_vcs_routes())
        .merge(system_routes::protected_system_routes())
        .layer(axum::middleware::from_fn_with_state(state.clone(), require_auth));

    // Combine both trees under `/api`
    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes);

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
