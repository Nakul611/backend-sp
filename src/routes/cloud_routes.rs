use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    controllers::cloud_controller::{
        get_cloud_catalog, list_cloud_providers, test_cloud_provider, update_cloud_provider,
    },
    state::AppState,
};

pub fn cloud_routes() -> Router<AppState> {
    Router::new()
        .route("/cloud-providers", get(list_cloud_providers))
        .route("/cloud-providers/{provider}", post(update_cloud_provider))
        .route("/cloud-providers/test", post(test_cloud_provider))
        .route("/cloud-catalog", get(get_cloud_catalog))
}
