use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    controllers::vcs_controller::{
        handle_git_webhook, list_vcs_providers, test_vcs_provider, update_vcs_provider,
    },
    state::AppState,
};

pub fn vcs_routes() -> Router<AppState> {
    Router::new()
        .route("/vcs-providers", get(list_vcs_providers))
        .route("/vcs-providers/{provider}", post(update_vcs_provider))
        .route("/vcs-providers/test", post(test_vcs_provider))
        .route("/vcs-providers/webhook/{provider}", post(handle_git_webhook))
}
